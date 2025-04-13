# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/storychain
COPY . .

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin/storychain

# Install curl for Ollama installation
RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

# Copy the built executable from builder stage
COPY --from=builder /usr/src/storychain/target/release/storychain .
COPY --from=builder /usr/src/storychain/artifacts ./artifacts

# Create directory for AI response logs
RUN mkdir -p /var/log/storychain

# Set environment variables
ENV RUST_LOG=info
ENV OLLAMA_HOST=http://localhost:11434
ENV OLLAMA_MODEL=deepseek-r1:32b

# Expose Ollama port
EXPOSE 11434

# Create entrypoint script
RUN echo '#!/bin/sh\n\
ollama serve &\n\
sleep 5\n\
ollama pull deepseek-r1:32b\n\
exec "$@"' > /entrypoint.sh && \
chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["./storychain", "premise", "--epochs", "5", "--output", "story.json"] 