# Multi-stage build for optimized Rust application
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for better layer caching
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN mkdir -p ./data
ENV DATABASE_URL="sqlite:./data/trading_data.db"
RUN touch src/main.rs && cargo build --release --no-default-features

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create necessary directories
RUN mkdir -p /app/data /app/logs && chown -R appuser:appuser /app

# Copy the binary
COPY --from=builder /app/target/release/binance-trading-bot /app/

# Copy configuration template
COPY config.example.toml /app/config.toml

# Set ownership and permissions
RUN chown -R appuser:appuser /app
RUN chmod +x /app/binance-trading-bot

# Switch to app user
USER appuser

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./binance-trading-bot", "--config", "config.toml"] 