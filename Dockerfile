# Build stage
FROM rust:1.81-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/storychain

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies and CUDA dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    nvidia-cuda-toolkit \
    && rm -rf /var/lib/apt/lists/*

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

# Copy the built executable
COPY --from=builder /usr/src/storychain/target/release/storychain /usr/local/bin/

# Set environment variables
ENV RUST_LOG=info
ENV OLLAMA_HOST=0.0.0.0:11434
ENV OLLAMA_MODEL=deepseek-r1:32b
# Enable GPU support
ENV CUDA_VISIBLE_DEVICES=all

# Create directories for artifacts and output
# These directories are meant to be mounted as volumes:
# - /app/artifacts: Mount your local artifacts directory containing premise files
# - /app/output: Mount your local output directory to persist generated stories
RUN mkdir -p /app/artifacts /app/output

WORKDIR /app

# Create entrypoint script with GPU checks
RUN echo '#!/bin/bash\n\
# Check for NVIDIA GPU\n\
if command -v nvidia-smi &> /dev/null; then\n\
    echo "NVIDIA GPU detected:"\n\
    nvidia-smi\n\
else\n\
    echo "Warning: No NVIDIA GPU detected, falling back to CPU"\n\
fi\n\
\n\
# Start Ollama server in the background\n\
ollama serve &\n\
\n\
# Wait for Ollama server to start\n\
echo "Waiting for Ollama server to start..."\n\
sleep 5\n\
\n\
# Pull the required model\n\
echo "Pulling Deepseek model..."\n\
ollama pull deepseek-r1:32b\n\
\n\
# Execute the command passed to the container\n\
echo "Starting StoryChain application..."\n\
exec "$@"' > /app/entrypoint.sh && chmod +x /app/entrypoint.sh

# Expose Ollama port
EXPOSE 11434

# Set the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]

# Default command (can be overridden)
# Note: Output files will be saved to /app/output/ which should be mounted as a volume
CMD ["storychain", "premise", "--epochs", "5", "--output", "/app/output/story.json"] 