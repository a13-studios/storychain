#!/bin/bash

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Please install jq first:"
    echo "  Homebrew: brew install jq"
    echo "  Apt: sudo apt-get install jq"
    echo "  Yum: sudo yum install jq"
    exit 1
fi

# Check if input file is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <story.json>"
    exit 1
fi

JSON_FILE="$1"
OUTPUT_FILE="${JSON_FILE%.*}.md"

# Check if input file exists
if [ ! -f "$JSON_FILE" ]; then
    echo "Error: File $JSON_FILE not found"
    exit 1
fi

# Create markdown header
cat > "$OUTPUT_FILE" << EOF
# Generated Story

*Generated on $(date '+%Y-%m-%d %H:%M:%S')*

---

EOF

# Get root node ID
ROOT_ID=$(jq -r '.root_node_id' "$JSON_FILE")
CURRENT_ID="$ROOT_ID"
SCENE_NUM=1

# Function to format a scene
format_scene() {
    local node_id="$1"
    local scene_num="$2"
    
    # Extract node content and reasoning
    local content=$(jq -r ".nodes[\"$node_id\"].content" "$JSON_FILE")
    local reasoning=$(jq -r ".nodes[\"$node_id\"].reasoning" "$JSON_FILE")
    local successor=$(jq -r ".nodes[\"$node_id\"].successor" "$JSON_FILE")
    
    # Write scene to file
    cat >> "$OUTPUT_FILE" << EOF
## Scene $scene_num

$content

<details>
<summary>AI's Reasoning</summary>

$reasoning
</details>

---

EOF
    
    # Return successor ID
    echo "$successor"
}

# Process each node in the chain
VISITED=()
while [ "$CURRENT_ID" != "null" ] && [[ ! " ${VISITED[@]} " =~ " ${CURRENT_ID} " ]]; do
    VISITED+=("$CURRENT_ID")
    CURRENT_ID=$(format_scene "$CURRENT_ID" "$SCENE_NUM")
    ((SCENE_NUM++))
done

echo "Successfully converted story to $OUTPUT_FILE" 