# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-01-23

### Added
- Dependabot configuration for automated dependency updates
- Enhanced Cargo.toml metadata for better crates.io discoverability

### Fixed
- Musl toolchain installation in release workflow for binary builds
- Test failures with enum boxing
- GitHub release creation with proper binary artifacts

## [0.1.0] - 2025-01-22

### Added
- Initial Prometheus exporter for Shelly smart home devices
- Support for both Gen1 and Gen2 Shelly devices
- Automatic generation detection
- Multi-device monitoring support
- mDNS device discovery (future enhancement)
- Health check endpoint
- Docker support with multi-stage builds
- GitHub Actions CI/CD pipeline
- OCI labels for GitHub Container Registry integration
- Make release target for automated release process
- Multi-platform Docker builds (linux/amd64, linux/arm64, linux/arm/v7)

### Features
- Gen1 device support (relays, meters, temperature, WiFi status)
- Gen2 device support (switches, system info, WiFi status)
- Real-time power consumption monitoring
- Device status and connectivity tracking
- Configurable polling intervals
- TLS-enabled HTTP client
- Authentication support for protected devices

### Technical
- Async/await architecture with Tokio
- Axum web framework for metrics endpoint
- Structured logging with tracing
- Error handling with anyhow/thiserror
- JSON API integration with reqwest