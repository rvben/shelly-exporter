# Build stage - use the target platform's rust image
FROM rust:1.88-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application for the native platform
RUN cargo build --release --target $(rustc -vV | sed -n 's/host: //p') && \
    cp target/$(rustc -vV | sed -n 's/host: //p')/release/shelly-exporter /app/shelly-exporter

# Runtime stage
FROM alpine:3.21

# OCI labels for GitHub Container Registry
LABEL org.opencontainers.image.source=https://github.com/rvben/shelly-exporter
LABEL org.opencontainers.image.description="Prometheus exporter for Shelly smart home devices"
LABEL org.opencontainers.image.licenses=MIT

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 exporter && \
    adduser -D -u 1000 -G exporter exporter

# Copy the binary from builder
COPY --from=builder /app/shelly-exporter /usr/local/bin/shelly-exporter

# Change ownership
RUN chown exporter:exporter /usr/local/bin/shelly-exporter

# Switch to non-root user
USER exporter

# Expose metrics port
EXPOSE 9925

# Set default environment variables
ENV LOG_LEVEL=info
ENV SHELLY_EXPORTER_BIND=0.0.0.0
ENV METRICS_PORT=9925

ENTRYPOINT ["/usr/local/bin/shelly-exporter"]