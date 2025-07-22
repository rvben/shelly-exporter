.PHONY: help build test run clean docker-build docker-buildx docker-push docker-push-ghcr docker-run lint format check release all-checks dev-run gh-secrets coverage

# Default target
help:
	@echo "Available targets:"
	@echo "  make build        - Build the binary in debug mode"
	@echo "  make release      - Build the binary in release mode"
	@echo "  make run          - Run the exporter"
	@echo "  make test         - Run tests"
	@echo "  make coverage     - Generate code coverage report"
	@echo "  make lint         - Run clippy linter"
	@echo "  make format       - Format code"
	@echo "  make check        - Run format check and linter"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make docker-build - Build Docker image (local)"
	@echo "  make docker-buildx - Build multi-arch Docker image (local)"
	@echo "  make docker-push  - Build and push multi-arch to Docker Hub"
	@echo "  make docker-push-ghcr - Build and push multi-arch to GitHub Container Registry"
	@echo "  make docker-run   - Run Docker container"
	@echo "  make dev-run      - Run with example hosts for development"
	@echo "  make gh-secrets   - Set GitHub Actions secrets from .env file"

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run tests
test:
	cargo test --verbose

# Generate code coverage report
coverage:
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html

# Run the exporter locally
run:
	RUST_LOG=info cargo run

# Clean build artifacts
clean:
	cargo clean

# Build Docker image
docker-build:
	docker build -t shelly-exporter:latest .

# Build multi-arch Docker image (local)
docker-buildx:
	docker buildx build --platform linux/amd64,linux/arm64 -t shelly-exporter .

# Build and push multi-arch Docker image to Docker Hub
docker-push:
	@if [ -z "$$DOCKER_USERNAME" ]; then \
		echo "Error: DOCKER_USERNAME environment variable is required"; \
		echo "Usage: DOCKER_USERNAME=youruser DOCKER_PASSWORD=yourpass make docker-push"; \
		exit 1; \
	fi
	@if [ -z "$$DOCKER_PASSWORD" ]; then \
		echo "Error: DOCKER_PASSWORD environment variable is required"; \
		echo "Usage: DOCKER_USERNAME=youruser DOCKER_PASSWORD=yourpass make docker-push"; \
		exit 1; \
	fi
	@echo "Logging in to Docker Hub..."
	@echo "$$DOCKER_PASSWORD" | docker login -u "$$DOCKER_USERNAME" --password-stdin
	@echo "Building and pushing multi-arch images..."
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t $$DOCKER_USERNAME/shelly-exporter:latest \
		-t $$DOCKER_USERNAME/shelly-exporter:$$(git describe --tags --always) \
		--push .
	@echo "Successfully pushed to Docker Hub!"

# Build and push to GitHub Container Registry
docker-push-ghcr:
	@if [ -z "$$GITHUB_TOKEN" ]; then \
		echo "Error: GITHUB_TOKEN environment variable is required"; \
		exit 1; \
	fi
	@echo "Logging in to GitHub Container Registry..."
	@echo "$$GITHUB_TOKEN" | docker login ghcr.io -u $$GITHUB_ACTOR --password-stdin
	@echo "Building and pushing multi-arch images to GHCR..."
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t ghcr.io/$$GITHUB_REPOSITORY_OWNER/shelly-exporter:latest \
		-t ghcr.io/$$GITHUB_REPOSITORY_OWNER/shelly-exporter:$$(git describe --tags --always) \
		--push .
	@echo "Successfully pushed to GitHub Container Registry!"

# Run Docker container
docker-run:
	docker run --rm -p 9925:9925 \
		-e SHELLY_HOSTS="$${SHELLY_HOSTS}" \
		-e SHELLY_LOG_LEVEL=info \
		shelly-exporter:latest

# Run linter
lint:
	cargo clippy -- -D warnings

# Format code
format:
	cargo fmt

# Check formatting
check:
	cargo fmt -- --check
	cargo clippy -- -D warnings

# Run all checks (format, lint, test)
all-checks: check test

# Development run with example hosts
dev-run:
	SHELLY_HOSTS="http://192.168.1.100,http://192.168.1.101" \
	SHELLY_NAMES="Test Device 1,Test Device 2" \
	RUST_LOG=debug \
	cargo run

# Set GitHub Actions secrets from .env file
gh-secrets:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found"; \
		echo "Copy .env.example to .env and fill in your values"; \
		exit 1; \
	fi
	@echo "Setting GitHub Actions secrets from .env file..."
	@export $$(cat .env | grep -v '^#' | xargs) && \
		gh secret set DOCKER_USERNAME --body "$$DOCKER_USERNAME" && \
		gh secret set DOCKER_PASSWORD --body "$$DOCKER_PASSWORD" && \
		gh secret set CRATES_IO_TOKEN --body "$$CRATES_IO_TOKEN"
	@echo "GitHub Actions secrets have been set successfully!"