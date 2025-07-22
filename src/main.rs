mod config;
mod metrics;
mod shelly;

use anyhow::Result;
use axum::{Router, routing::get};
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use tracing::{error, info, warn, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::metrics::Metrics;
use crate::shelly::{ShellyClient, ShellyGeneration};

type SharedMetrics = Arc<RwLock<String>>;
type DeviceClients = Arc<Mutex<HashMap<String, (ShellyClient, String, String)>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Shelly Prometheus Exporter");
    info!("Monitoring {} devices", config.hosts.len());
    info!("Metrics port: {}", config.port);
    info!("Poll interval: {}s", config.poll_interval);

    // Initialize metrics
    let metrics = Arc::new(Metrics::new()?);
    let shared_metrics: SharedMetrics = Arc::new(RwLock::new(String::new()));

    // Initialize device clients
    let device_clients: DeviceClients = Arc::new(Mutex::new(HashMap::new()));
    
    // Setup initial devices
    for (host, name) in config.get_device_names() {
        match setup_device_client(&host, &config).await {
            Ok((client, model)) => {
                info!("Added device: {} ({}) at {}", name, model, host);
                let mut clients = device_clients.lock().await;
                clients.insert(host, (client, name, model));
            }
            Err(e) => {
                warn!("Failed to setup device at {}: {}", host, e);
            }
        }
    }

    // Start polling task
    let poll_metrics = metrics.clone();
    let poll_shared_metrics = shared_metrics.clone();
    let poll_interval = config.poll_interval_duration();
    let poll_clients = device_clients.clone();

    tokio::spawn(async move {
        let mut interval = interval(poll_interval);
        interval.tick().await; // First tick completes immediately

        loop {
            interval.tick().await;
            
            let clients = poll_clients.lock().await;
            for (host, (client, device_name, model)) in clients.iter() {
                let generation = match client.generation {
                    ShellyGeneration::Gen1 => "gen1",
                    ShellyGeneration::Gen2 => "gen2",
                };

                match client.get_status().await {
                    Ok(status) => {
                        debug!("Successfully fetched status from {} ({})", device_name, host);
                        
                        if let Err(e) = poll_metrics.update_device(
                            device_name,
                            host,
                            model,
                            generation,
                            &status,
                        ) {
                            error!("Failed to update metrics for {}: {}", device_name, e);
                            continue;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch status from {} ({}): {}", device_name, host, e);
                        poll_metrics.mark_device_down(device_name, host, model, generation);
                    }
                }
            }
            
            drop(clients);

            // Gather all metrics
            match poll_metrics.gather() {
                Ok(metrics_text) => {
                    let mut metrics_guard = poll_shared_metrics.write().await;
                    *metrics_guard = metrics_text;
                }
                Err(e) => {
                    error!("Failed to gather metrics: {}", e);
                }
            }
        }
    });

    // Start discovery task if enabled
    if config.enable_discovery {
        let discovery_interval = config.discovery_interval_duration();
        let discovery_clients = device_clients.clone();
        let discovery_config = config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(discovery_interval);
            
            loop {
                interval.tick().await;
                info!("Running device discovery...");
                
                match ShellyClient::discover_devices(discovery_config.http_timeout_duration()).await {
                    Ok(discovered) => {
                        info!("Discovered {} devices", discovered.len());
                        for device_url in discovered {
                            let mut clients = discovery_clients.lock().await;
                            if !clients.contains_key(&device_url) {
                                match setup_device_client(&device_url, &discovery_config).await {
                                    Ok((client, model)) => {
                                        let name = device_url
                                            .trim_start_matches("http://")
                                            .trim_start_matches("https://")
                                            .split(':')
                                            .next()
                                            .unwrap_or("unknown")
                                            .to_string();
                                        info!("Added discovered device: {} ({}) at {}", name, model, device_url);
                                        clients.insert(device_url, (client, name, model));
                                    }
                                    Err(e) => {
                                        warn!("Failed to setup discovered device at {}: {}", device_url, e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Device discovery failed: {}", e);
                    }
                }
            }
        });
    }

    // Initialize HTTP server
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/", get(root_handler))
        .with_state(shared_metrics);

    let addr = config.metrics_bind_address();
    info!("Starting metrics server on {}", &addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_device_client(host: &str, config: &Config) -> Result<(ShellyClient, String)> {
    let timeout = config.http_timeout_duration();
    let auth = config.auth();
    
    // Detect device generation
    let generation = ShellyClient::detect_generation(host, timeout, auth.clone()).await?;
    
    // Create client
    let client = ShellyClient::new(host.to_string(), timeout, auth, generation)?;
    
    // Get device info for model
    let model = if generation == ShellyGeneration::Gen2 {
        match client.get_device_info().await {
            Ok(info) => info.model,
            Err(_) => "Unknown".to_string(),
        }
    } else {
        // Gen1 devices don't have a unified device info endpoint
        "Shelly Gen1".to_string()
    };
    
    Ok((client, model))
}

async fn metrics_handler(
    axum::extract::State(metrics): axum::extract::State<SharedMetrics>,
) -> String {
    let metrics_guard = metrics.read().await;
    metrics_guard.clone()
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn root_handler() -> &'static str {
    "Shelly Prometheus Exporter\n\nEndpoints:\n  /metrics - Prometheus metrics\n  /health  - Health check\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let shared_metrics: SharedMetrics = Arc::new(RwLock::new(
            "# HELP shelly_device_up Whether device is up\n# TYPE shelly_device_up gauge\nshelly_device_up{device=\"test\"} 1\n"
                .to_string(),
        ));

        Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .route("/", get(root_handler))
            .with_state(shared_metrics)
    }

    #[tokio::test]
    async fn test_health_handler() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "OK");
    }

    #[tokio::test]
    async fn test_root_handler() {
        let app = create_test_app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Shelly Prometheus Exporter"));
        assert!(body_str.contains("/metrics"));
        assert!(body_str.contains("/health"));
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("shelly_device_up"));
        assert!(body_str.contains("test"));
    }
}