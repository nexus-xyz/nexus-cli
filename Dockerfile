# --- Build Stage ---
FROM rust:1.85-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the entire project
COPY . .

# Build the CLI binary
WORKDIR /app/clients/cli
RUN cargo build --release --locked

####################################################################################################
## Final image
####################################################################################################
# Use a minimal base image with glibc support
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/clients/cli/target/release/nexus-network /app/nexus-cli

ENTRYPOINT ["/app/nexus-cli"]
