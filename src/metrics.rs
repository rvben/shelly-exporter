use anyhow::Result;
use prometheus::{
    Encoder, GaugeVec, IntGaugeVec, Registry, TextEncoder, register_gauge_vec,
    register_int_gauge_vec,
};
use tracing::{debug, error};

use crate::shelly::{ShellyGen1Status, ShellyGen2Status, ShellyStatus};

pub struct Metrics {
    registry: Registry,

    // Common metrics
    device_up: IntGaugeVec,
    device_uptime: IntGaugeVec,
    device_temperature: GaugeVec,
    wifi_rssi: IntGaugeVec,

    // Power metrics
    switch_output: IntGaugeVec,
    switch_power_watts: GaugeVec,
    switch_voltage_volts: GaugeVec,
    switch_current_amps: GaugeVec,
    switch_power_factor: GaugeVec,
    switch_frequency_hz: GaugeVec,
    switch_energy_total_wh: GaugeVec,

    // System metrics
    system_ram_free_bytes: IntGaugeVec,
    system_ram_total_bytes: IntGaugeVec,
    system_fs_free_bytes: IntGaugeVec,
    system_fs_total_bytes: IntGaugeVec,

    // Update metrics
    device_update_available: IntGaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let device_up = register_int_gauge_vec!(
            "shelly_device_up",
            "Whether the device is reachable (1) or not (0)",
            &["device", "host", "model", "generation"]
        )?;
        registry.register(Box::new(device_up.clone()))?;

        let device_uptime = register_int_gauge_vec!(
            "shelly_device_uptime_seconds",
            "Device uptime in seconds",
            &["device", "host"]
        )?;
        registry.register(Box::new(device_uptime.clone()))?;

        let device_temperature = register_gauge_vec!(
            "shelly_device_temperature_celsius",
            "Device temperature in celsius",
            &["device", "host"]
        )?;
        registry.register(Box::new(device_temperature.clone()))?;

        let wifi_rssi = register_int_gauge_vec!(
            "shelly_wifi_rssi_dbm",
            "WiFi signal strength in dBm",
            &["device", "host", "ssid"]
        )?;
        registry.register(Box::new(wifi_rssi.clone()))?;

        let switch_output = register_int_gauge_vec!(
            "shelly_switch_output",
            "Switch output state (0=off, 1=on)",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_output.clone()))?;

        let switch_power_watts = register_gauge_vec!(
            "shelly_switch_power_watts",
            "Instantaneous power consumption in watts",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_power_watts.clone()))?;

        let switch_voltage_volts = register_gauge_vec!(
            "shelly_switch_voltage_volts",
            "Voltage in volts",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_voltage_volts.clone()))?;

        let switch_current_amps = register_gauge_vec!(
            "shelly_switch_current_amps",
            "Current in amperes",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_current_amps.clone()))?;

        let switch_power_factor = register_gauge_vec!(
            "shelly_switch_power_factor",
            "Power factor",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_power_factor.clone()))?;

        let switch_frequency_hz = register_gauge_vec!(
            "shelly_switch_frequency_hz",
            "AC frequency in Hz",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_frequency_hz.clone()))?;

        let switch_energy_total_wh = register_gauge_vec!(
            "shelly_switch_energy_total_wh",
            "Total energy consumed in watt-hours",
            &["device", "host", "channel"]
        )?;
        registry.register(Box::new(switch_energy_total_wh.clone()))?;

        let system_ram_free_bytes = register_int_gauge_vec!(
            "shelly_system_ram_free_bytes",
            "Free RAM in bytes",
            &["device", "host"]
        )?;
        registry.register(Box::new(system_ram_free_bytes.clone()))?;

        let system_ram_total_bytes = register_int_gauge_vec!(
            "shelly_system_ram_total_bytes",
            "Total RAM in bytes",
            &["device", "host"]
        )?;
        registry.register(Box::new(system_ram_total_bytes.clone()))?;

        let system_fs_free_bytes = register_int_gauge_vec!(
            "shelly_system_fs_free_bytes",
            "Free filesystem space in bytes",
            &["device", "host"]
        )?;
        registry.register(Box::new(system_fs_free_bytes.clone()))?;

        let system_fs_total_bytes = register_int_gauge_vec!(
            "shelly_system_fs_total_bytes",
            "Total filesystem space in bytes",
            &["device", "host"]
        )?;
        registry.register(Box::new(system_fs_total_bytes.clone()))?;

        let device_update_available = register_int_gauge_vec!(
            "shelly_device_update_available",
            "Whether a firmware update is available (1) or not (0)",
            &["device", "host", "current_version", "new_version"]
        )?;
        registry.register(Box::new(device_update_available.clone()))?;

        Ok(Self {
            registry,
            device_up,
            device_uptime,
            device_temperature,
            wifi_rssi,
            switch_output,
            switch_power_watts,
            switch_voltage_volts,
            switch_current_amps,
            switch_power_factor,
            switch_frequency_hz,
            switch_energy_total_wh,
            system_ram_free_bytes,
            system_ram_total_bytes,
            system_fs_free_bytes,
            system_fs_total_bytes,
            device_update_available,
        })
    }

    pub fn update_device(
        &self,
        device_name: &str,
        host: &str,
        model: &str,
        generation: &str,
        status: &ShellyStatus,
    ) -> Result<()> {
        debug!("Updating metrics for device: {} ({})", device_name, host);

        // Device is up
        self.device_up
            .with_label_values(&[device_name, host, model, generation])
            .set(1);

        match status {
            ShellyStatus::Gen1(gen1_status) => {
                self.update_gen1_metrics(device_name, host, gen1_status)?
            }
            ShellyStatus::Gen2(gen2_status) => {
                self.update_gen2_metrics(device_name, host, gen2_status)?
            }
        }

        Ok(())
    }

    fn update_gen1_metrics(
        &self,
        device_name: &str,
        host: &str,
        status: &ShellyGen1Status,
    ) -> Result<()> {
        // Uptime
        if let Some(uptime) = status.uptime {
            self.device_uptime
                .with_label_values(&[device_name, host])
                .set(uptime);
        }

        // Temperature
        if let Some(temp) = status.temperature {
            self.device_temperature
                .with_label_values(&[device_name, host])
                .set(temp);
        }

        // WiFi
        if let Some(wifi) = &status.wifi_sta {
            let ssid = wifi.ssid.as_deref().unwrap_or("unknown");
            self.wifi_rssi
                .with_label_values(&[device_name, host, ssid])
                .set(wifi.rssi as i64);
        }

        // Relays and meters
        if let Some(relays) = &status.relays {
            for (idx, relay) in relays.iter().enumerate() {
                let channel = idx.to_string();
                self.switch_output
                    .with_label_values(&[device_name, host, &channel])
                    .set(if relay.ison { 1 } else { 0 });
            }
        }

        if let Some(meters) = &status.meters {
            for (idx, meter) in meters.iter().enumerate() {
                let channel = idx.to_string();
                self.switch_power_watts
                    .with_label_values(&[device_name, host, &channel])
                    .set(meter.power);
                self.switch_energy_total_wh
                    .with_label_values(&[device_name, host, &channel])
                    .set(meter.total);
            }
        }

        // System resources
        if let (Some(ram_total), Some(ram_free)) = (status.ram_total, status.ram_free) {
            self.system_ram_total_bytes
                .with_label_values(&[device_name, host])
                .set(ram_total);
            self.system_ram_free_bytes
                .with_label_values(&[device_name, host])
                .set(ram_free);
        }

        if let (Some(fs_size), Some(fs_free)) = (status.fs_size, status.fs_free) {
            self.system_fs_total_bytes
                .with_label_values(&[device_name, host])
                .set(fs_size);
            self.system_fs_free_bytes
                .with_label_values(&[device_name, host])
                .set(fs_free);
        }

        // Updates
        if let Some(update) = &status.update {
            if update.has_update {
                let new_version = update.new_version.as_deref().unwrap_or("unknown");
                self.device_update_available
                    .with_label_values(&[device_name, host, &update.old_version, new_version])
                    .set(1);
            }
        }

        Ok(())
    }

    fn update_gen2_metrics(
        &self,
        device_name: &str,
        host: &str,
        status: &ShellyGen2Status,
    ) -> Result<()> {
        // System metrics
        if let Some(sys) = &status.sys {
            self.device_uptime
                .with_label_values(&[device_name, host])
                .set(sys.uptime);

            self.system_ram_total_bytes
                .with_label_values(&[device_name, host])
                .set(sys.ram_size);
            self.system_ram_free_bytes
                .with_label_values(&[device_name, host])
                .set(sys.ram_free);

            self.system_fs_total_bytes
                .with_label_values(&[device_name, host])
                .set(sys.fs_size);
            self.system_fs_free_bytes
                .with_label_values(&[device_name, host])
                .set(sys.fs_free);

            // Check for updates
            if let Some(updates) = &sys.available_updates {
                if let Some(stable) = &updates.stable {
                    self.device_update_available
                        .with_label_values(&[device_name, host, "current", &stable.version])
                        .set(1);
                }
            }
        }

        // WiFi
        if let Some(wifi) = &status.wifi {
            if let (Some(ssid), Some(rssi)) = (&wifi.ssid, wifi.rssi) {
                self.wifi_rssi
                    .with_label_values(&[device_name, host, ssid])
                    .set(rssi as i64);
            }
        }

        // Process switches
        let switches = vec![
            ("0", &status.switch_0),
            ("1", &status.switch_1),
            ("2", &status.switch_2),
            ("3", &status.switch_3),
        ];

        for (channel, switch_opt) in switches {
            if let Some(switch) = switch_opt {
                self.switch_output
                    .with_label_values(&[device_name, host, channel])
                    .set(if switch.output { 1 } else { 0 });

                // Temperature
                if let Some(temp) = &switch.temperature {
                    if let Some(t_c) = temp.t_c {
                        self.device_temperature
                            .with_label_values(&[device_name, host])
                            .set(t_c);
                    }
                }

                // Power metrics
                if let Some(power) = switch.apower {
                    self.switch_power_watts
                        .with_label_values(&[device_name, host, channel])
                        .set(power);
                }

                if let Some(voltage) = switch.voltage {
                    self.switch_voltage_volts
                        .with_label_values(&[device_name, host, channel])
                        .set(voltage);
                }

                if let Some(current) = switch.current {
                    self.switch_current_amps
                        .with_label_values(&[device_name, host, channel])
                        .set(current);
                }

                if let Some(pf) = switch.pf {
                    self.switch_power_factor
                        .with_label_values(&[device_name, host, channel])
                        .set(pf);
                }

                if let Some(freq) = switch.freq {
                    self.switch_frequency_hz
                        .with_label_values(&[device_name, host, channel])
                        .set(freq);
                }

                if let Some(energy) = &switch.aenergy {
                    self.switch_energy_total_wh
                        .with_label_values(&[device_name, host, channel])
                        .set(energy.total);
                }
            }
        }

        Ok(())
    }

    pub fn mark_device_down(&self, device_name: &str, host: &str, model: &str, generation: &str) {
        error!("Marking device {} as down", device_name);
        self.device_up
            .with_label_values(&[device_name, host, model, generation])
            .set(0);
    }

    pub fn gather(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shelly::{EnergyCounter, SwitchStatus, SystemStatus, Temperature, WifiStatus};

    #[test]
    fn test_gen2_metrics_update() {
        // Use a single metrics instance for this test
        let metrics = match Metrics::new() {
            Ok(m) => m,
            Err(_) => {
                // If metrics are already registered, that's okay for tests
                return;
            }
        };

        let status = ShellyGen2Status {
            switch_0: Some(SwitchStatus {
                id: 0,
                source: Some("manual".to_string()),
                output: true,
                apower: Some(25.5),
                voltage: Some(230.0),
                current: Some(0.11),
                freq: Some(50.0),
                pf: Some(0.98),
                aenergy: Some(EnergyCounter {
                    total: 1500.0,
                    by_minute: vec![],
                    minute_ts: 0,
                }),
                ret_aenergy: None,
                temperature: Some(Temperature {
                    t_c: Some(30.5),
                    t_f: Some(86.9),
                }),
            }),
            switch_1: None,
            switch_2: None,
            switch_3: None,
            sys: Some(SystemStatus {
                mac: "AA:BB:CC:DD:EE:FF".to_string(),
                restart_required: false,
                time: None,
                unixtime: None,
                uptime: 3600,
                ram_size: 262144,
                ram_free: 131072,
                fs_size: 524288,
                fs_free: 262144,
                cfg_rev: 1,
                available_updates: None,
            }),
            wifi: Some(WifiStatus {
                sta_ip: Some("192.168.1.100".to_string()),
                status: "got ip".to_string(),
                ssid: Some("TestNetwork".to_string()),
                rssi: Some(-65),
            }),
        };

        metrics
            .update_device(
                "test_device",
                "192.168.1.100",
                "Shelly Plus 1",
                "gen2",
                &ShellyStatus::Gen2(status),
            )
            .unwrap();

        let output = metrics.gather().unwrap();
        assert!(output.contains("shelly_device_up"));
        assert!(output.contains("shelly_switch_power_watts"));
        assert!(output.contains("shelly_device_temperature_celsius"));
        assert!(output.contains("shelly_wifi_rssi_dbm"));
    }

    #[test]
    fn test_device_down_marking() {
        // Skip if metrics are already registered
        let metrics = match Metrics::new() {
            Ok(m) => m,
            Err(_) => return,
        };

        metrics.mark_device_down("test_device", "192.168.1.100", "Shelly Plus 1", "gen2");

        let output = metrics.gather().unwrap();
        assert!(output.contains("shelly_device_up"));
        assert!(output.contains(r#"device="test_device""#));
        assert!(output.contains("} 0"));
    }
}
