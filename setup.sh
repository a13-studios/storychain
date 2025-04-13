#!/bin/bash
# Exit immediately if a command exits with a non-zero status.
set -e

echo "----------------------------------------------"
echo "Step 1: Updating system packages..."
apt update && apt upgrade -y

echo "----------------------------------------------"
echo "Step 2: Installing required utilities (pciutils, lshw, wget, tar, curl, build-essential)..."
apt install -y pciutils lshw wget tar curl build-essential

echo "----------------------------------------------"
echo "Step 3: Checking available hardware..."

echo ">> NVIDIA devices via lspci:"
lspci | grep -i nvidia || echo "No NVIDIA hardware found."

echo ">> NVIDIA details via lshw:"
lshw -short | grep -i nvidia || echo "No NVIDIA hardware details found."

# Check if the system is macOS (Darwin) to verify Apple Silicon / MPS availability.
if [[ "$(uname)" == "Darwin" ]]; then
    echo ">> Apple MPS/Apple Silicon GPU details (via system_profiler):"
    system_profiler SPDisplaysDataType || echo "No Apple GPU details found."
else
    echo ">> Not a Darwin-based system; skipping Apple Silicon/MPS check."
fi

echo "----------------------------------------------"
echo "Step 4: Installing Rust via rustup..."
# Non-interactive Rust installation with default options
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source Rust environment (adjust HOME if running as non-root)
source $HOME/.cargo/env

echo "Verifying Rust installation:"
rustc --version
cargo --version

echo "----------------------------------------------"
echo "Step 5: Installing Ollama..."
curl -fsSL https://ollama.com/install.sh | sh

echo "----------------------------------------------"
echo "Step 6: Starting the Ollama API server in the background..."
# Start the Ollama service (using the 'serve' subcommand) in the background
nohup ollama serve > ollama.log 2>&1 &
# Allow a few seconds for the server to initialize
sleep 10

echo "Verifying that Ollama is running (listing running models):"
ollama ps || echo "Ollama service may not be fully initialized yet."

echo "----------------------------------------------"
echo "Step 7: Pulling the deepseek-r1:32b model..."
ollama pull deepseek-r1:32b

echo "----------------------------------------------"
echo "Step 8: Verifying installed models..."
ollama list

echo "----------------------------------------------"
echo "Setup complete."

# download story md locally
# scp -P 30845 root@58.224.7.136:/workspace/storychain/story.md ~/Downloads/