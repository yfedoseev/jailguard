# Multi-stage Dockerfile for JailGuard
# Stage 1: Builder
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build the library and examples in release mode
RUN cargo build --release --features "cpu,semantic-embeddings"

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts from builder
COPY --from=builder /app/target/release/deps /app/deps
COPY --from=builder /app/target/release/examples /app/examples

# Create a non-root user for security
RUN useradd -m -u 1000 jailguard && chown -R jailguard:jailguard /app
USER jailguard

# Health check for the container
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# API server port
EXPOSE 8080

# Metrics port
EXPOSE 9090

# Default command: run the evaluation example
CMD ["./examples/evaluate_detector"]
