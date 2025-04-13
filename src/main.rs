use storychain::{StoryChain, DeepseekProvider, AIProvider, StoryChainError};
use log::info;
use clap::{Command, Arg};

#[tokio::main]
async fn main() -> Result<(), StoryChainError> {
    env_logger::init();
    info!("Starting StoryChain application");

    let matches = Command::new("storychain")
        .version("0.1.0")
        .about("Generates a linear narrative using AI")
        .arg(
            Arg::new("premise")
                .help("The premise file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("epochs")
                .long("epochs")
                .help("Number of epochs to generate")
                .default_value("5")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .help("Output file path")
                .default_value("story.json"),
        )
        .get_matches();

    let premise_file = matches.get_one::<String>("premise").unwrap();
    let epochs = *matches.get_one::<usize>("epochs").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();

    info!("Starting story generation with {} epochs", epochs);

    // Load premise from file
    let start_time = std::time::Instant::now();
    let premise = std::fs::read_to_string(format!("artifacts/{}.yaml", premise_file))
        .map_err(|e| StoryChainError::IOError(e))?;
    info!("Loaded premise from artifacts/{}.yaml", premise_file);

    // Initialize AI provider
    let provider = DeepseekProvider::new(
        "deepseek-r1:32b".to_string(),
        "ai_responses.log".to_string(),
    );

    // Generate initial scene
    info!("Generating initial scene");
    let initial_start = std::time::Instant::now();
    let (reasoning, content) = provider.generate(&format!(
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

    // Create story chain with content and reasoning in the correct order
    let mut chain = StoryChain::new(content, reasoning);

    // Generate next nodes
    let mut current_node_id = "root".to_string();
    for epoch in 0..epochs {
        let epoch_start = std::time::Instant::now();
        info!("Starting epoch {} of {}", epoch + 1, epochs);
        let next_node_ids = chain
            .generate_next_nodes(&current_node_id, &provider, Some(&premise))
            .await?;
        if next_node_ids.is_empty() {
            break;
        }
        current_node_id = next_node_ids[0].clone();
        let epoch_time = epoch_start.elapsed();
        info!("Epoch {} took: {:?}", epoch + 1, epoch_time);
    }

    // Export story chain
    chain.export_to_file(output_file)?;
    let total_time = start_time.elapsed();
    info!("Story chain exported to {}", output_file);
    info!("Total story generation took: {:?}", total_time);

    Ok(())
}
