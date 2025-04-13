//! StoryChain - A narrative generation system using AI
//! 
//! This is the main entry point for the StoryChain application, which generates
//! linear narratives using AI models. The application takes a premise file as input
//! and generates a sequence of connected scenes that form a coherent story.

use storychain::{StoryChain, DeepseekProvider, AIProvider, StoryChainError};
use log::info;
use clap::{Command, Arg};

/// The main entry point for the StoryChain application.
/// 
/// # Error
/// Returns a `StoryChainError` if any operation fails during story generation
/// or file operations.
#[tokio::main]
async fn main() -> Result<(), StoryChainError> {
    // Initialize logging system for application-wide logging
    env_logger::init();
    info!("Starting StoryChain application");

    // Set up command-line argument parsing using clap
    let matches = Command::new("storychain")
        .version("0.1.0")
        .about("Generates a linear narrative using AI")
        .arg(
            // Required premise file argument that specifies the story's foundation
            Arg::new("premise")
                .help("The premise file to use")
                .required(true)
                .index(1),
        )
        .arg(
            // Optional number of epochs (story generation iterations)
            Arg::new("epochs")
                .long("epochs")
                .help("Number of epochs to generate")
                .default_value("5")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            // Optional output file path for the generated story
            Arg::new("output")
                .long("output")
                .help("Output file path")
                .default_value("story.json"),
        )
        .get_matches();

    // Extract command line arguments
    let premise_file = matches.get_one::<String>("premise").unwrap();
    let epochs = *matches.get_one::<usize>("epochs").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();

    info!("Starting story generation with {} epochs", epochs);

    // Load the premise from the specified YAML file in the artifacts directory
    let start_time = std::time::Instant::now();
    let premise = std::fs::read_to_string(format!("artifacts/{}.yaml", premise_file))
        .map_err(|e| StoryChainError::IOError(e))?;
    info!("Loaded premise from artifacts/{}.yaml", premise_file);

    // Initialize the AI provider with the Deepseek model for story generation
    let provider = DeepseekProvider::new(
        "deepseek-r1:32b".to_string(),  // Using the 32B parameter Deepseek model
        "ai_responses.log".to_string(),  // Log file for AI responses
    );

    // Generate the initial scene based on the premise
    info!("Generating initial scene");
    let initial_start = std::time::Instant::now();
    let (reasoning, content) = provider.generate(&format!(
        // Construct the prompt for the initial scene generation
        "You are tasked with writing a scene in the style specified by the premise.\n\n\
        IMPORTANT: Format your response EXACTLY as follows:\n\
        <think>\n\
        Write your reasoning here in a single paragraph, explaining your narrative choices and how they connect to the premise.\n\
        </think>\n\
        Write your scene content here, using proper paragraphs and formatting.\n\n\
        Story Premise:\n{}\n\n\
        Remember: \n\
        - Put your reasoning in a SINGLE paragraph inside <think> tags\n\
        - Write your scene content immediately after the </think> tag\n\
        - Use proper paragraphs in your scene content\n\
        - Do NOT add any extra formatting or tags",
        premise
    )).await?;
    let initial_time = initial_start.elapsed();
    info!("Initial scene generation took: {:?}", initial_time);

    // Initialize the story chain with the generated content and reasoning
    let mut chain = StoryChain::new(content, reasoning);

    // Generate subsequent scenes for the specified number of epochs
    let mut current_node_id = "root".to_string();
    for epoch in 0..epochs {
        let epoch_start = std::time::Instant::now();
        info!("Starting epoch {} of {}", epoch + 1, epochs);
        
        // Generate the next scene based on the current one
        let next_node_ids = chain
            .generate_next_nodes(&current_node_id, &provider, Some(&premise))
            .await?;
            
        // Break if no more nodes can be generated
        if next_node_ids.is_empty() {
            break;
        }
        
        // Update the current node to the first generated successor
        current_node_id = next_node_ids[0].clone();
        let epoch_time = epoch_start.elapsed();
        info!("Epoch {} took: {:?}", epoch + 1, epoch_time);
    }

    // Export the complete story chain to the specified output file
    chain.export_to_file(output_file)?;
    let total_time = start_time.elapsed();
    info!("Story chain exported to {}", output_file);
    info!("Total story generation took: {:?}", total_time);

    Ok(())
}
