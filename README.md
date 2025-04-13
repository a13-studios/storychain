# StoryChain

StoryChain is a Rust-based narrative generation system that uses AI to create branching stories based on predefined premises. The system uses Ollama with the Deepseek model to generate coherent, flowing narratives.

## Prerequisites

- Rust (latest stable version)
- [Ollama](https://ollama.ai/) installed and running
- The Deepseek model pulled (`ollama pull deepseek-r1:32b`)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd storychain
```

2. Build the project:
```bash
cargo build
```

## Usage

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

Parameters:
- `premise-name`: Name of your premise file (without .yaml extension)
- `--epochs`: Number of story segments to generate (default: 5)
- `--output`: Output JSON file path (default: story.json)

Example:
```bash
cargo run -- premise --epochs 3 --output my_story.json
```

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

## Logging

The system logs AI responses to `ai_responses.log` and general execution information through the standard logging system. Set the `RUST_LOG` environment variable to control log levels:

```bash
RUST_LOG=debug cargo run -- premise
```

## Error Handling

The system handles various error cases:
- AI server errors (Ollama connection issues)
- Invalid response formats
- File I/O errors
- Serialization errors

## License

[Your chosen license] 