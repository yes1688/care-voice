# Synchronous backend without any async dependencies
FROM docker.io/rust:1.85-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml ./

# Create a dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() { println!(\"dummy\"); }" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Copy actual source code (will overwrite dummy)
COPY src/ ./src/

# Clean to force rebuild with new source
RUN cargo clean

# Build the actual application
RUN cargo build --release

# Verify the binary was built
RUN ls -la target/release/ && \
    echo "Built binary info:" && \
    file target/release/care-voice-sync 2>/dev/null || echo "file command not available"

# Runtime stage
FROM docker.io/debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/care-voice-sync ./care-voice

# Verify binary exists and is executable
RUN ls -la ./care-voice && chmod +x ./care-voice

# Expose port
EXPOSE 8000

# Set environment for better debugging
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

# Use exec form to ensure proper signal handling
CMD ["./care-voice"]