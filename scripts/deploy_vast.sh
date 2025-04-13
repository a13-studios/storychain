#!/bin/bash

# Default values
MAX_PRICE=1.0
DURATION=24

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --max-price)
            MAX_PRICE="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--max-price <price>] [--duration <hours>]"
            exit 1
            ;;
    esac
done

# Check if vast-cli is installed
if ! command -v vast &> /dev/null; then
    echo "vast-cli not found. Installing..."
    pip install vast-ai-client
fi

# Check if API key is configured
if ! vast show apikey &> /dev/null; then
    echo "Please configure vast-ai with your API key:"
    echo "1. Get your API key from: https://vast.ai/console/account"
    echo "2. Run: vast set apikey YOUR_API_KEY"
    exit 1
fi

# Function to find best instance
find_best_instance() {
    echo "Searching for available RTX 5090 instances..."
    
    # Try RTX 5090 first
    INSTANCES=$(vast search offers 'gpu_name == "RTX 5090"' -o json)
    
    # If no 5090, try 4090 or A6000
    if [ "$(echo "$INSTANCES" | jq length)" -eq 0 ]; then
        echo "No RTX 5090 instances found. Checking for other high-end GPUs..."
        INSTANCES=$(vast search offers 'gpu_name == "RTX 4090" || gpu_name == "A6000"' -o json)
    fi
    
    # Check if any instances found
    if [ "$(echo "$INSTANCES" | jq length)" -eq 0 ]; then
        echo "No suitable GPU instances found."
        exit 1
    fi
    
    # Find best instance based on reliability and performance/cost
    BEST_INSTANCE=$(echo "$INSTANCES" | jq 'sort_by(.reliability * .dlperf_per_dphtotal) | reverse | .[0]')
    
    # Print instance details
    echo -e "\nSelected instance:"
    echo "ID: $(echo "$BEST_INSTANCE" | jq -r '.id')"
    echo "GPU: $(echo "$BEST_INSTANCE" | jq -r '.gpu_name')"
    echo "Cost: $$(echo "$BEST_INSTANCE" | jq -r '.dph_total')/hour"
    echo "Location: $(echo "$BEST_INSTANCE" | jq -r '.location')"
    
    # Return instance ID
    echo "$BEST_INSTANCE" | jq -r '.id'
}

# Create onstart script
create_onstart_script() {
    cat > onstart.sh << 'EOF'
#!/bin/bash

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
fi

# Clone StoryChain repository
git clone https://github.com/YOUR_USERNAME/storychain.git
cd storychain

# Build and run the container
docker build -t storychain .
docker run -d -p 11434:11434 -v $(pwd)/artifacts:/app/artifacts storychain

echo "StoryChain deployment complete!"
EOF

    chmod +x onstart.sh
    echo "$(pwd)/onstart.sh"
}

# Main execution
echo "Deploying StoryChain to vast.ai..."

# Find best instance
INSTANCE_ID=$(find_best_instance)

# Create and prepare onstart script
ONSTART_SCRIPT=$(create_onstart_script)

# Create instance
echo -e "\nCreating instance..."
vast create instance "$INSTANCE_ID" \
    --image pytorch/pytorch \
    --disk 32 \
    --onstart "$ONSTART_SCRIPT" \
    --price "$MAX_PRICE"

# Print helpful information
echo -e "\nInstance created successfully!"
echo "To connect to your instance:"
echo "vast ssh $INSTANCE_ID"
echo -e "\nTo stop the instance when done:"
echo "vast stop instance $INSTANCE_ID"
echo -e "\nTo monitor the deployment:"
echo "vast logs $INSTANCE_ID" 