use storychain::{StoryChain, StoryChainError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), StoryChainError> {
    // Get the input file from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <story.json>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    
    // Read and parse the JSON file
    let content = std::fs::read_to_string(input_file)?;
    let chain: StoryChain = serde_json::from_str(&content)?;
    
    // Convert to markdown
    let output_file = input_file.replace(".json", ".md");
    chain.export_to_markdown(&output_file)?;
    
    println!("Successfully converted {} to {}", input_file, output_file);
    Ok(())
} 