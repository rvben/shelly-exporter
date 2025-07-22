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

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 shelly && \
    adduser -D -u 1000 -G shelly shelly

# Copy the binary from builder
COPY --from=builder /app/shelly-exporter /usr/local/bin/shelly-exporter

# Change ownership
RUN chown shelly:shelly /usr/local/bin/shelly-exporter

# Switch to non-root user
USER shelly

# Expose metrics port
EXPOSE 9925

# Set default environment variables
ENV SHELLY_LOG_LEVEL=info
ENV SHELLY_EXPORTER_BIND=0.0.0.0
ENV SHELLY_EXPORTER_PORT=9925

ENTRYPOINT ["/usr/local/bin/shelly-exporter"]