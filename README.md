# StoryChain

StoryChain is a Rust-based narrative generation system that uses AI to create branching stories based on predefined premises. The system uses Ollama with the Deepseek model to generate coherent, flowing narratives.

## Prerequisites

- Rust (latest stable version)
- [Ollama](https://ollama.ai/) installed and running
- The Deepseek model pulled (`ollama pull deepseek-r1:32b`)

Or alternatively:
- Docker (for containerized usage)

## Installation

### Local Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd storychain
```

2. Build the project:
```bash
cargo build
```

### Docker Installation

Build the Docker image:
```bash
docker build -t storychain .
```

This will:
1. Run all tests
2. Build the application
3. Create a container with all dependencies

## Usage

### Local Usage

1. First, ensure Ollama is running:
```bash
brew services start ollama  # on macOS
```

2. Create a premise file in the `artifacts` directory with a `.yaml` extension. The premise should include:
   - Title and genre
   - Setting and time period
   - Main premise
   - Characters and their arcs
   - Themes
   - Plot elements

Example premise structure:
```yaml
title: Your Story Title
genre: Genre
setting: Setting Description
time_period: Time Period

premise: |
  Your main premise here...

characters:
  - name: Character Name
    description: Character Description
    arc: Character Arc

themes:
  - Theme 1
  - Theme 2

plot_elements:
  - Plot Element 1
  - Plot Element 2
```

3. Run the story generation:
```bash
cargo run -- <premise-name> --epochs <number> --output <output-file>
```

### Docker Usage

1. Create your premise file as described above in a local `artifacts` directory

2. Create a local directory for output files:
```bash
mkdir -p output
```

3. Run the container with mounted volumes:
```bash
docker run \
  -v $(pwd)/artifacts:/app/artifacts \
  -v $(pwd)/output:/app/output \
  -p 11434:11434 \
  storychain
```

This will:
- Mount your local `artifacts` directory to access premise files
- Mount your local `output` directory to save generated stories
- Expose port 11434 for Ollama

The generated files will be available in your local `output` directory:
- `story.json`: The raw story data
- `story.md`: A readable markdown version

Parameters for Docker usage:
- `premise-name`: Name of your premise file (without .yaml extension)
- `--epochs`: Number of story segments to generate (default: 5)
- `--output`: Output JSON file path (default: /app/output/story.json)

Example with custom parameters:
```bash
docker run \
  -v $(pwd)/artifacts:/app/artifacts \
  -v $(pwd)/output:/app/output \
  -p 11434:11434 \
  storychain storychain my_premise --epochs 3 --output /app/output/my_story.json
```

Note: Always use `/app/output/` as the base path for output files when running in Docker to ensure they are saved to your mounted volume.

## Output

The generated story is saved in JSON format with the following structure:
```json
{
  "nodes": {
    "root": {
      "id": "root",
      "content": "Story content...",
      "reasoning": "Generation reasoning...",
      "predecessor": null,
      "successor": "node_1"
    },
    "node_1": {
      "id": "node_1",
      "content": "Next scene content...",
      "reasoning": "Generation reasoning...",
      "predecessor": "root",
      "successor": "node_2"
    }
    // ... more nodes
  },
  "root_node_id": "root"
}
```

### Converting to Readable Format

The story output can be converted to a readable markdown format using the provided Python script:

```bash
python3 scripts/story_to_markdown.py story.json
```

This will create `story.md` with:
- Numbered scenes
- Story content in a readable format
- AI's reasoning for each scene in collapsible sections
- Clear scene separators

The markdown file can be viewed in any markdown reader or GitHub for a pleasant reading experience.

## Logging

The system logs AI responses to `ai_responses.log` and general execution information through the standard logging system. Set the `RUST_LOG` environment variable to control log levels:

```bash
# Local
RUST_LOG=debug cargo run -- premise

# Docker
docker run -e RUST_LOG=debug -v $(pwd)/artifacts:/app/artifacts -p 11434:11434 storychain
```

## Error Handling

The system handles various error cases:
- AI server errors (Ollama connection issues)
- Invalid response formats
- File I/O errors
- Serialization errors

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

## License

[Your chosen license] 