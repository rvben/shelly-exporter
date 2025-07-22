use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Comma-separated list of Shelly device URLs (e.g., http://192.168.1.100,http://192.168.1.101)
    #[arg(long, env = "SHELLY_HOSTS", value_delimiter = ',', required = true)]
    pub hosts: Vec<String>,

    /// Optional comma-separated list of device names (same order as hosts)
    #[arg(long, env = "SHELLY_NAMES", value_delimiter = ',')]
    pub names: Option<Vec<String>>,

    /// Authentication username (default: admin for Gen2)
    #[arg(long, env = "SHELLY_USERNAME", default_value = "admin")]
    pub username: String,

    /// Authentication password
    #[arg(long, env = "SHELLY_PASSWORD")]
    pub password: Option<String>,

    /// Port to expose metrics on
    #[arg(short, long, env = "SHELLY_EXPORTER_PORT", default_value = "9925")]
    pub port: u16,

    /// Bind address for metrics server
    #[arg(long, env = "SHELLY_EXPORTER_BIND", default_value = "0.0.0.0")]
    pub bind: String,

    /// Poll interval in seconds
    #[arg(long, env = "SHELLY_POLL_INTERVAL", default_value = "30")]
    pub poll_interval: u64,

    /// HTTP timeout in seconds
    #[arg(long, env = "SHELLY_HTTP_TIMEOUT", default_value = "10")]
    pub http_timeout: u64,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "SHELLY_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Enable auto-discovery of devices via mDNS
    #[arg(long, env = "SHELLY_DISCOVERY", default_value = "false")]
    pub enable_discovery: bool,

    /// Discovery interval in seconds (when discovery is enabled)
    #[arg(long, env = "SHELLY_DISCOVERY_INTERVAL", default_value = "300")]
    pub discovery_interval: u64,
}

impl Config {
    pub fn metrics_bind_address(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }

    pub fn poll_interval_duration(&self) -> Duration {
        Duration::from_secs(self.poll_interval)
    }

    pub fn http_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.http_timeout)
    }

    pub fn discovery_interval_duration(&self) -> Duration {
        Duration::from_secs(self.discovery_interval)
    }

    pub fn auth(&self) -> Option<(String, String)> {
        self.password
            .as_ref()
            .map(|pass| (self.username.clone(), pass.clone()))
    }

    pub fn get_device_names(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();
        
        for (idx, host) in self.hosts.iter().enumerate() {
            let name = if let Some(names) = &self.names {
                names.get(idx).cloned().unwrap_or_else(|| {
                    // Extract IP or hostname from URL
                    host.trim_start_matches("http://")
                        .trim_start_matches("https://")
                        .split(':')
                        .next()
                        .unwrap_or("unknown")
                        .to_string()
                })
            } else {
                // Extract IP or hostname from URL
                host.trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .split(':')
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            };
            
            result.push((host.clone(), name));
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_bind_address() {
        let config = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        assert_eq!(config.metrics_bind_address(), "0.0.0.0:9925");
    }

    #[test]
    fn test_durations() {
        let config = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 45,
            http_timeout: 15,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 600,
        };

        assert_eq!(config.poll_interval_duration(), Duration::from_secs(45));
        assert_eq!(config.http_timeout_duration(), Duration::from_secs(15));
        assert_eq!(config.discovery_interval_duration(), Duration::from_secs(600));
    }

    #[test]
    fn test_auth() {
        let config_without_password = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        assert!(config_without_password.auth().is_none());

        let config_with_password = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            username: "admin".to_string(),
            password: Some("secret".to_string()),
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        assert_eq!(
            config_with_password.auth(),
            Some(("admin".to_string(), "secret".to_string()))
        );
    }

    #[test]
    fn test_get_device_names() {
        let config_with_names = Config {
            hosts: vec![
                "http://192.168.1.100".to_string(),
                "http://192.168.1.101:8080".to_string(),
            ],
            names: Some(vec!["Living Room".to_string(), "Kitchen".to_string()]),
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        let names = config_with_names.get_device_names();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], ("http://192.168.1.100".to_string(), "Living Room".to_string()));
        assert_eq!(names[1], ("http://192.168.1.101:8080".to_string(), "Kitchen".to_string()));

        let config_without_names = Config {
            hosts: vec![
                "http://192.168.1.100".to_string(),
                "https://shelly.local".to_string(),
            ],
            names: None,
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        let names = config_without_names.get_device_names();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], ("http://192.168.1.100".to_string(), "192.168.1.100".to_string()));
        assert_eq!(names[1], ("https://shelly.local".to_string(), "shelly.local".to_string()));
    }

    #[test]
    fn test_partial_device_names() {
        let config = Config {
            hosts: vec![
                "http://192.168.1.100".to_string(),
                "http://192.168.1.101".to_string(),
                "http://192.168.1.102".to_string(),
            ],
            names: Some(vec!["Living Room".to_string(), "Kitchen".to_string()]),
            username: "admin".to_string(),
            password: None,
            port: 9925,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
            enable_discovery: false,
            discovery_interval: 300,
        };

        let names = config.get_device_names();
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], ("http://192.168.1.100".to_string(), "Living Room".to_string()));
        assert_eq!(names[1], ("http://192.168.1.101".to_string(), "Kitchen".to_string()));
        assert_eq!(names[2], ("http://192.168.1.102".to_string(), "192.168.1.102".to_string()));
    }
}