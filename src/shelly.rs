use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ShellyClient {
    client: Client,
    base_url: String,
    auth: Option<(String, String)>,
    pub generation: ShellyGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellyGeneration {
    Gen1,
    Gen2,
}

// Gen2 Status structures
#[derive(Debug, Deserialize, Serialize)]
pub struct ShellyGen2Status {
    #[serde(rename = "switch:0", default)]
    pub switch_0: Option<SwitchStatus>,
    #[serde(rename = "switch:1", default)]
    pub switch_1: Option<SwitchStatus>,
    #[serde(rename = "switch:2", default)]
    pub switch_2: Option<SwitchStatus>,
    #[serde(rename = "switch:3", default)]
    pub switch_3: Option<SwitchStatus>,
    pub sys: Option<SystemStatus>,
    pub wifi: Option<WifiStatus>,
}

// Gen1 Status structures
#[derive(Debug, Deserialize, Serialize)]
pub struct ShellyGen1Status {
    pub relays: Option<Vec<RelayStatus>>,
    pub meters: Option<Vec<MeterStatus>>,
    pub temperature: Option<f64>,
    pub overtemperature: Option<bool>,
    pub wifi_sta: Option<WifiGen1Status>,
    pub update: Option<UpdateStatus>,
    pub ram_total: Option<i64>,
    pub ram_free: Option<i64>,
    pub fs_size: Option<i64>,
    pub fs_free: Option<i64>,
    pub uptime: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RelayStatus {
    pub ison: bool,
    pub has_timer: bool,
    pub timer_started: Option<i64>,
    pub timer_duration: Option<i64>,
    pub timer_remaining: Option<i64>,
    pub overpower: Option<bool>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeterStatus {
    pub power: f64,
    pub is_valid: bool,
    pub timestamp: i64,
    pub counters: Vec<f64>,
    pub total: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WifiGen1Status {
    pub connected: bool,
    pub ssid: Option<String>,
    pub ip: Option<String>,
    pub rssi: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStatus {
    pub status: String,
    pub has_update: bool,
    pub new_version: Option<String>,
    pub old_version: String,
}

// Unified status enum
#[derive(Debug)]
pub enum ShellyStatus {
    Gen1(ShellyGen1Status),
    Gen2(ShellyGen2Status),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwitchStatus {
    pub id: i32,
    pub source: Option<String>,
    pub output: bool,
    pub apower: Option<f64>,
    pub voltage: Option<f64>,
    pub current: Option<f64>,
    pub freq: Option<f64>,
    pub pf: Option<f64>,
    pub aenergy: Option<EnergyCounter>,
    pub ret_aenergy: Option<EnergyCounter>,
    pub temperature: Option<Temperature>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnergyCounter {
    pub total: f64,
    pub by_minute: Vec<f64>,
    pub minute_ts: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Temperature {
    #[serde(rename = "tC")]
    pub t_c: Option<f64>,
    #[serde(rename = "tF")]
    pub t_f: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SystemStatus {
    pub mac: String,
    pub restart_required: bool,
    pub time: Option<String>,
    pub unixtime: Option<i64>,
    pub uptime: i64,
    pub ram_size: i64,
    pub ram_free: i64,
    pub fs_size: i64,
    pub fs_free: i64,
    pub cfg_rev: i32,
    pub available_updates: Option<AvailableUpdates>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvailableUpdates {
    pub stable: Option<UpdateInfo>,
    pub beta: Option<UpdateInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateInfo {
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WifiStatus {
    pub sta_ip: Option<String>,
    pub status: String,
    pub ssid: Option<String>,
    pub rssi: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub id: String,
    pub mac: String,
    pub model: String,
    #[serde(rename = "gen")]
    pub generation: i32,
    pub fw_id: String,
    pub ver: String,
    pub app: String,
    pub auth_en: bool,
    pub auth_domain: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcRequest {
    id: i32,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcResponse<T> {
    id: i32,
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl ShellyClient {
    pub fn new(base_url: String, timeout: Duration, auth: Option<(String, String)>, generation: ShellyGeneration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url,
            auth,
            generation,
        })
    }

    pub async fn detect_generation(base_url: &str, timeout: Duration, auth: Option<(String, String)>) -> Result<ShellyGeneration> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Try Gen2 endpoint first
        let gen2_url = format!("{}/rpc/Shelly.GetDeviceInfo", base_url);
        let mut request = client.get(&gen2_url);
        
        if let Some((username, password)) = &auth {
            request = request.basic_auth(username, Some(password));
        }

        if let Ok(response) = request.send().await {
            if response.status().is_success() {
                info!("Detected Gen2 device at {}", base_url);
                return Ok(ShellyGeneration::Gen2);
            }
        }

        // Try Gen1 endpoint
        let gen1_url = format!("{}/settings", base_url);
        let mut request = client.get(&gen1_url);
        
        if let Some((username, password)) = &auth {
            request = request.basic_auth(username, Some(password));
        }

        if let Ok(response) = request.send().await {
            if response.status().is_success() {
                info!("Detected Gen1 device at {}", base_url);
                return Ok(ShellyGeneration::Gen1);
            }
        }

        Err(anyhow!("Failed to detect Shelly generation for {}", base_url))
    }

    pub async fn get_device_info(&self) -> Result<DeviceInfo> {
        let url = format!("{}/rpc/Shelly.GetDeviceInfo", self.base_url);
        debug!("Fetching device info from: {}", url);

        let mut request = self.client.get(&url);
        
        if let Some((username, password)) = &self.auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch device info: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch device info: HTTP {}",
                response.status()
            ));
        }

        let device_info = response
            .json::<DeviceInfo>()
            .await
            .map_err(|e| anyhow!("Failed to parse device info: {}", e))?;

        info!("Device info: {} ({})", device_info.name, device_info.model);
        Ok(device_info)
    }

    pub async fn get_status(&self) -> Result<ShellyStatus> {
        match self.generation {
            ShellyGeneration::Gen2 => self.get_gen2_status().await,
            ShellyGeneration::Gen1 => self.get_gen1_status().await,
        }
    }

    async fn get_gen2_status(&self) -> Result<ShellyStatus> {
        let url = format!("{}/rpc/Shelly.GetStatus", self.base_url);
        debug!("Fetching Gen2 status from: {}", url);

        let mut request = self.client.get(&url);
        
        if let Some((username, password)) = &self.auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch Gen2 status: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch Gen2 status: HTTP {}",
                response.status()
            ));
        }

        let status = response
            .json::<ShellyGen2Status>()
            .await
            .map_err(|e| anyhow!("Failed to parse Gen2 status: {}", e))?;

        debug!("Gen2 status fetched successfully");
        Ok(ShellyStatus::Gen2(status))
    }

    async fn get_gen1_status(&self) -> Result<ShellyStatus> {
        let url = format!("{}/status", self.base_url);
        debug!("Fetching Gen1 status from: {}", url);

        let mut request = self.client.get(&url);
        
        if let Some((username, password)) = &self.auth {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch Gen1 status: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch Gen1 status: HTTP {}",
                response.status()
            ));
        }

        let status = response
            .json::<ShellyGen1Status>()
            .await
            .map_err(|e| anyhow!("Failed to parse Gen1 status: {}", e))?;

        debug!("Gen1 status fetched successfully");
        Ok(ShellyStatus::Gen1(status))
    }

    pub async fn discover_devices(_timeout: Duration) -> Result<Vec<String>> {
        info!("Starting mDNS discovery for Shelly devices...");
        let devices = Vec::new();
        
        // Note: mDNS discovery would be implemented here
        // For now, we'll return an empty list and rely on manually configured devices
        
        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_device_info() {
        let mock_server = MockServer::start().await;
        
        let device_info_response = r#"{
            "name": "Test Shelly",
            "id": "shelly1-123456",
            "mac": "AA:BB:CC:DD:EE:FF",
            "model": "SNSW-001X16EU",
            "gen": 2,
            "fw_id": "20230913-123456/v1.14.0",
            "ver": "1.14.0",
            "app": "S1",
            "auth_en": false,
            "auth_domain": null
        }"#;

        Mock::given(method("GET"))
            .and(path("/rpc/Shelly.GetDeviceInfo"))
            .respond_with(ResponseTemplate::new(200).set_body_string(device_info_response))
            .mount(&mock_server)
            .await;

        let client = ShellyClient::new(
            mock_server.uri(),
            Duration::from_secs(5),
            None,
            ShellyGeneration::Gen2,
        ).unwrap();

        let info = client.get_device_info().await.unwrap();
        assert_eq!(info.name, "Test Shelly");
        assert_eq!(info.model, "SNSW-001X16EU");
        assert_eq!(info.generation, 2);
    }

    #[tokio::test]
    async fn test_get_status() {
        let mock_server = MockServer::start().await;
        
        let status_response = r#"{
            "switch:0": {
                "id": 0,
                "source": "manual",
                "output": true,
                "apower": 15.5,
                "voltage": 230.1,
                "current": 0.067,
                "freq": 50.0,
                "pf": 0.99,
                "aenergy": {
                    "total": 1234.567,
                    "by_minute": [250.0, 251.0, 249.0],
                    "minute_ts": 1234567890
                },
                "temperature": {
                    "tC": 25.5,
                    "tF": 77.9
                }
            },
            "sys": {
                "mac": "AA:BB:CC:DD:EE:FF",
                "restart_required": false,
                "time": "12:34:56",
                "unixtime": 1234567890,
                "uptime": 3600,
                "ram_size": 262144,
                "ram_free": 131072,
                "fs_size": 524288,
                "fs_free": 262144,
                "cfg_rev": 10,
                "available_updates": null
            },
            "wifi": {
                "sta_ip": "192.168.1.100",
                "status": "got ip",
                "ssid": "TestNetwork",
                "rssi": -65
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/rpc/Shelly.GetStatus"))
            .respond_with(ResponseTemplate::new(200).set_body_string(status_response))
            .mount(&mock_server)
            .await;

        let client = ShellyClient::new(
            mock_server.uri(),
            Duration::from_secs(5),
            None,
            ShellyGeneration::Gen2,
        ).unwrap();

        let status = client.get_status().await.unwrap();
        
        match status {
            ShellyStatus::Gen2(gen2_status) => {
                assert!(gen2_status.switch_0.is_some());
                let switch = gen2_status.switch_0.unwrap();
                assert_eq!(switch.output, true);
                assert_eq!(switch.apower, Some(15.5));
                assert_eq!(switch.voltage, Some(230.1));
                
                assert!(gen2_status.sys.is_some());
                let sys = gen2_status.sys.unwrap();
                assert_eq!(sys.uptime, 3600);
                
                assert!(gen2_status.wifi.is_some());
                let wifi = gen2_status.wifi.unwrap();
                assert_eq!(wifi.sta_ip, Some("192.168.1.100".to_string()));
                assert_eq!(wifi.rssi, Some(-65));
            }
            ShellyStatus::Gen1(_) => panic!("Expected Gen2 status"),
        }
    }

    #[tokio::test]
    async fn test_authentication() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/rpc/Shelly.GetDeviceInfo"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = ShellyClient::new(
            mock_server.uri(),
            Duration::from_secs(5),
            None,
            ShellyGeneration::Gen2,
        ).unwrap();

        let result = client.get_device_info().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTP 401"));
    }

    #[tokio::test]
    async fn test_get_gen1_status() {
        let mock_server = MockServer::start().await;
        
        let status_response = r#"{
            "relays": [{
                "ison": true,
                "has_timer": false,
                "timer_started": 0,
                "timer_duration": 0,
                "timer_remaining": 0,
                "source": "input"
            }],
            "meters": [{
                "power": 23.45,
                "is_valid": true,
                "timestamp": 1234567890,
                "counters": [1234.56, 0.0, 0.0],
                "total": 1234.56
            }],
            "temperature": 25.5,
            "overtemperature": false,
            "wifi_sta": {
                "connected": true,
                "ssid": "TestNetwork",
                "ip": "192.168.1.101",
                "rssi": -60
            },
            "uptime": 7200
        }"#;

        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(ResponseTemplate::new(200).set_body_string(status_response))
            .mount(&mock_server)
            .await;

        let client = ShellyClient::new(
            mock_server.uri(),
            Duration::from_secs(5),
            None,
            ShellyGeneration::Gen1,
        ).unwrap();

        let status = client.get_status().await.unwrap();
        
        match status {
            ShellyStatus::Gen1(gen1_status) => {
                assert!(gen1_status.relays.is_some());
                let relays = gen1_status.relays.unwrap();
                assert_eq!(relays.len(), 1);
                assert_eq!(relays[0].ison, true);
                
                assert!(gen1_status.meters.is_some());
                let meters = gen1_status.meters.unwrap();
                assert_eq!(meters.len(), 1);
                assert_eq!(meters[0].power, 23.45);
                assert_eq!(meters[0].total, 1234.56);
                
                assert_eq!(gen1_status.temperature, Some(25.5));
                assert_eq!(gen1_status.uptime, Some(7200));
                
                assert!(gen1_status.wifi_sta.is_some());
                let wifi = gen1_status.wifi_sta.unwrap();
                assert_eq!(wifi.connected, true);
                assert_eq!(wifi.ip, Some("192.168.1.101".to_string()));
                assert_eq!(wifi.rssi, -60);
            }
            ShellyStatus::Gen2(_) => panic!("Expected Gen1 status"),
        }
    }

    #[tokio::test]
    async fn test_detect_generation() {
        let mock_server = MockServer::start().await;
        
        // Mock Gen2 device
        Mock::given(method("GET"))
            .and(path("/rpc/Shelly.GetDeviceInfo"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let generation = ShellyClient::detect_generation(
            &mock_server.uri(),
            Duration::from_secs(5),
            None,
        ).await.unwrap();
        
        assert_eq!(generation, ShellyGeneration::Gen2);
    }
}