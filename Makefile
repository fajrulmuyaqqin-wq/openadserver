# OpenAdServer (LiteAds Rust Rewrite) Makefile
# High-performance computational advertising system in Rust

.PHONY: help build check dev test test-fast fmt clean docker-up docker-down docker-logs docker-build db-init db-mock health benchmark

# Default target
help:
	@echo "OpenAdServer (Rust) - Makefile Commands"
	@echo ""
	@echo "Rust Development:"
	@echo "  make check       Verify compilation without code generation"
	@echo "  make build       Compile release binary (target/release/openadserver)"
	@echo "  make dev         Run ad server locally in development mode"
	@echo "  make test        Run all unit & integration tests"
	@echo "  make fmt         Format Rust code with rustfmt"
	@echo "  make clean       Clean cargo target artifacts"
	@echo ""
	@echo "Database & Mocking:"
	@echo "  make db-init     Initialize schema and default tables"
	@echo "  make db-mock     Seed database with realistic mock ad campaigns"
	@echo ""
	@echo "Docker & Compose:"
	@echo "  make docker-up   Start postgres, redis, and ad-server containers"
	@echo "  make docker-down Stop all containers"
	@echo "  make docker-build Build Rust multi-stage Docker container"
	@echo "  make docker-logs Follow container logs"
	@echo ""
	@echo "Observability & Benchmarks:"
	@echo "  make health      Check server /health endpoint"
	@echo "  make metrics     Fetch Prometheus /metrics"
	@echo "  make benchmark   Run high-throughput HTTP benchmark"

# =============================================================================
# Rust Development
# =============================================================================

check:
	cargo check

build:
	cargo build --release

dev:
	RUST_LOG=info cargo run -- serve

test:
	cargo test

test-fast:
	cargo test -- --nocapture

fmt:
	cargo fmt

clean:
	cargo clean

# =============================================================================
# Database & Mock Data
# =============================================================================

db-init:
	cargo run -- init-db

db-mock:
	cargo run -- mock-data

# =============================================================================
# Docker & Deployment
# =============================================================================

docker-up:
	docker compose up -d

docker-up-full:
	docker compose --profile monitoring --profile production up -d

docker-down:
	docker compose down

docker-logs:
	docker compose logs -f ad-server

docker-build:
	docker compose build --no-cache

docker-restart:
	docker compose restart ad-server

# =============================================================================
# Observability & Testing
# =============================================================================

health:
	@curl -s http://localhost:8000/health | jq . || curl -s http://localhost:8000/health

metrics:
	@curl -s http://localhost:8000/metrics

benchmark:
	@echo "Running benchmark against Rust OpenAdServer..."
	@wrk -t8 -c200 -d30s -s scripts/benchmark.lua http://localhost:8000/api/v1/ad/request || echo "wrk not found, install via package manager"
