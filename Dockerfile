# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/storychain

# Copy source code
COPY . .

# Run tests
RUN cargo test --release

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

# Copy the built executable
COPY --from=builder /usr/src/storychain/target/release/storychain /usr/local/bin/

# Set environment variables
ENV RUST_LOG=info
ENV OLLAMA_HOST=0.0.0.0:11434
ENV OLLAMA_MODEL=deepseek-r1:32b

# Create directory for artifacts
RUN mkdir -p /app/artifacts

WORKDIR /app

# Create and set permissions for the entrypoint script
COPY --from=builder /usr/src/storychain/entrypoint.sh /app/
RUN chmod +x /app/entrypoint.sh

# Expose Ollama port
EXPOSE 11434

# Set the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]

# Default command (can be overridden)
CMD ["storychain", "premise", "--epochs", "5", "--output", "story.json"] 