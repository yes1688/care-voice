# Multi-stage build for Rust backend with whisper-rs
FROM docker.io/rust:1.85-slim AS builder

# Install build dependencies for whisper-rs
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml ./

# Create a dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm src/main.rs target/release/deps/care_voice*

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM docker.io/debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/care-voice ./

# Create models directory and copy model file
RUN mkdir -p models
COPY models/ggml-base.bin ./models/

# Expose port
EXPOSE 8000

# Set environment
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# Run the application
CMD ["./care-voice"]