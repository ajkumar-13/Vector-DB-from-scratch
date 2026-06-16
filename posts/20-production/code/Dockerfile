# Multi-stage Docker build for VectorDB
# Optimized for minimal image size and fast builds with dependency caching

# Stage 1: Planner (for dependency caching with cargo-chef)
FROM lukemathwalker/cargo-chef:latest-rust-1.75 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder (compile dependencies + code)
FROM chef AS builder

# Copy dependency recipe
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (this layer is cached unless dependencies change)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin vectordb

# Strip debug symbols to reduce binary size
RUN strip target/release/vectordb

# Stage 3: Runtime (minimal image with only the binary)
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
# - ca-certificates: For HTTPS requests
# - libssl3: OpenSSL runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/vectordb /usr/local/bin/vectordb

# Create data directory
RUN mkdir -p /data && chmod 755 /data

# Create non-root user for security
RUN useradd -m -u 1000 vectordb && chown -R vectordb:vectordb /data
USER vectordb

# Volume for persistent data
VOLUME /data

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["vectordb", "--data-dir", "/data", "--host", "0.0.0.0", "--port", "8080"]
