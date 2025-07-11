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

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Copy source code
COPY . .

# Build the actual app
RUN cargo build --release --locked

####################################################################################################
## Final image
####################################################################################################
# # Start a new stage to create a smaller image without unnecessary build dependencies.
# Use a minimal base image (distroless, alpine, scratch, etc.).
FROM gcr.io/distroless/cc

WORKDIR /app

# Copy the compiled binary from the builder stage.
COPY --from=builder /app/target/release/nexus-network .

ENTRYPOINT ["./nexus-network"]