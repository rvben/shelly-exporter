# Shelly Prometheus Exporter

A Prometheus exporter for Shelly smart home devices, supporting both Gen1 and Gen2 devices.

## Features

- **Multi-generation support**: Works with both Shelly Gen1 and Gen2 devices
- **Auto-detection**: Automatically detects device generation
- **Multiple devices**: Monitor multiple Shelly devices simultaneously
- **Comprehensive metrics**: Power consumption, energy usage, temperature, WiFi signal, and more
- **mDNS discovery**: Optional automatic device discovery (coming soon)
- **Low resource usage**: Efficient Rust implementation

## Supported Devices

This exporter supports all Shelly devices with HTTP API access, including:
- Shelly 1/1PM
- Shelly 2.5
- Shelly Plus series (1, 1PM, 2PM, etc.)
- Shelly Pro series
- And more...

## Metrics

The exporter provides the following metrics:

| Metric | Description | Labels |
|--------|-------------|--------|
| `shelly_device_up` | Device availability (1=up, 0=down) | device, host, model, generation |
| `shelly_device_uptime_seconds` | Device uptime in seconds | device, host |
| `shelly_device_temperature_celsius` | Device temperature | device, host |
| `shelly_wifi_rssi_dbm` | WiFi signal strength | device, host, ssid |
| `shelly_switch_output` | Switch state (1=on, 0=off) | device, host, channel |
| `shelly_switch_power_watts` | Instantaneous power consumption | device, host, channel |
| `shelly_switch_voltage_volts` | Voltage measurement | device, host, channel |
| `shelly_switch_current_amps` | Current measurement | device, host, channel |
| `shelly_switch_power_factor` | Power factor | device, host, channel |
| `shelly_switch_frequency_hz` | AC frequency | device, host, channel |
| `shelly_switch_energy_total_wh` | Total energy consumed | device, host, channel |
| `shelly_system_ram_free_bytes` | Free RAM | device, host |
| `shelly_system_ram_total_bytes` | Total RAM | device, host |
| `shelly_system_fs_free_bytes` | Free filesystem space | device, host |
| `shelly_system_fs_total_bytes` | Total filesystem space | device, host |
| `shelly_device_update_available` | Firmware update availability | device, host, current_version, new_version |

## Installation

### Using Docker

```bash
docker run -d \
  --name shelly-exporter \
  -p 9925:9925 \
  -e SHELLY_HOSTS="http://192.168.1.100,http://192.168.1.101" \
  -e SHELLY_NAMES="Living Room,Kitchen" \
  ghcr.io/rvben/shelly-exporter:latest
```

### Using Docker Compose

```yaml
version: '3.8'

services:
  shelly-exporter:
    image: ghcr.io/rvben/shelly-exporter:latest
    container_name: shelly-exporter
    restart: unless-stopped
    ports:
      - "9925:9925"
    environment:
      SHELLY_HOSTS: "http://192.168.1.100,http://192.168.1.101"
      SHELLY_NAMES: "Living Room,Kitchen"
      SHELLY_LOG_LEVEL: info
      SHELLY_POLL_INTERVAL: 30
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/rvben/shelly-exporter
cd shelly-exporter

# Build with Cargo
cargo build --release

# Run the exporter
SHELLY_HOSTS="http://192.168.1.100" ./target/release/shelly-exporter
```

## Configuration

The exporter can be configured using command-line arguments or environment variables:

| CLI Argument | Environment Variable | Description | Default |
|--------------|---------------------|-------------|---------|
| `--hosts` | `SHELLY_HOSTS` | Comma-separated list of device URLs (required) | - |
| `--names` | `SHELLY_NAMES` | Comma-separated list of device names | IP addresses |
| `--username` | `SHELLY_USERNAME` | Authentication username | admin |
| `--password` | `SHELLY_PASSWORD` | Authentication password | - |
| `--port` | `SHELLY_EXPORTER_PORT` | Metrics server port | 9925 |
| `--bind` | `SHELLY_EXPORTER_BIND` | Metrics server bind address | 0.0.0.0 |
| `--poll-interval` | `SHELLY_POLL_INTERVAL` | Poll interval in seconds | 30 |
| `--http-timeout` | `SHELLY_HTTP_TIMEOUT` | HTTP timeout in seconds | 10 |
| `--log-level` | `SHELLY_LOG_LEVEL` | Log level (trace/debug/info/warn/error) | info |
| `--enable-discovery` | `SHELLY_DISCOVERY` | Enable mDNS discovery | false |
| `--discovery-interval` | `SHELLY_DISCOVERY_INTERVAL` | Discovery interval in seconds | 300 |

### Examples

Monitor multiple devices with custom names:
```bash
SHELLY_HOSTS="http://192.168.1.100,http://192.168.1.101,http://192.168.1.102" \
SHELLY_NAMES="Living Room,Kitchen,Bedroom" \
shelly-exporter
```

With authentication:
```bash
SHELLY_HOSTS="http://192.168.1.100" \
SHELLY_USERNAME="admin" \
SHELLY_PASSWORD="secret" \
shelly-exporter
```

## Prometheus Configuration

Add the following to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'shelly'
    static_configs:
      - targets: ['localhost:9925']
    scrape_interval: 30s
```

## Grafana Dashboard

A sample Grafana dashboard is available in `grafana-dashboard.json`. Import it into your Grafana instance to visualize:
- Device status and uptime
- Power consumption trends
- Energy usage over time
- Temperature monitoring
- WiFi signal strength
- System resource usage

## Development

### Running Tests

```bash
cargo test
```

### Building Docker Image

```bash
docker build -t shelly-exporter .
```

## Troubleshooting

### Device Not Detected

1. Ensure the device URL is correct and accessible
2. Check if authentication is required
3. Verify the device is on the same network
4. Try accessing the device URL in a browser

### Wrong Generation Detected

The exporter automatically detects device generation by trying Gen2 endpoints first, then Gen1. If detection fails:
1. Check device firmware is up to date
2. Ensure the device supports HTTP API
3. Check authentication settings

### High Memory Usage

Adjust the poll interval to reduce frequency of metric updates:
```bash
SHELLY_POLL_INTERVAL=60 shelly-exporter
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.