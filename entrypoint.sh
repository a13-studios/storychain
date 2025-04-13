#!/bin/sh

# Start Ollama server in the background
ollama serve &

# Wait for Ollama server to start
echo "Waiting for Ollama server to start..."
sleep 5

# Pull the required model
echo "Pulling Deepseek model..."
ollama pull deepseek-r1:32b

# Execute the command passed to the container
echo "Starting StoryChain application..."
exec "$@" 