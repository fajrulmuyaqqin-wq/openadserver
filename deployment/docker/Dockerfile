# OpenAdServer Rust Dockerfile
# Multi-stage build for ultra-compact, high-performance binary container

# Build stage
FROM rust:1-slim-bookworm AS builder

WORKDIR /usr/src/openadserver

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency specifications for layer caching
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy actual source files and configuration
COPY src ./src
COPY configs ./configs

# Build optimized production release binary
RUN touch src/main.rs src/lib.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash openadserver

# Copy compiled binary and configuration files
COPY --from=builder /usr/src/openadserver/target/release/openadserver /app/openadserver
COPY --from=builder /usr/src/openadserver/configs /app/configs

RUN chown -R openadserver:openadserver /app

USER openadserver

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

ENTRYPOINT ["/app/openadserver"]
CMD ["serve"]
