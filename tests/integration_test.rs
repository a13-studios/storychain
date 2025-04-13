use storychain::{StoryChain, DeepseekProvider, AIProvider, StoryChainError};
use std::path::Path;

/// A mock AI provider for testing that returns predefined responses
struct MockAIProvider;

#[async_trait::async_trait]
impl AIProvider for MockAIProvider {
    async fn generate(&self, _prompt: &str) -> Result<(String, String), StoryChainError> {
        Ok((
            "Test scene reasoning: establishing the setting".to_string(),
            "The sun cast long shadows across the quiet street.".to_string(),
        ))
    }
}

#[tokio::test]
async fn test_basic_story_generation() -> Result<(), StoryChainError> {
    // Create a temporary test premise
    let test_dir = "test_artifacts";
    std::fs::create_dir_all(test_dir)?;
    let premise_path = Path::new(test_dir).join("test_premise.yaml");
    std::fs::write(&premise_path, "A story about a quiet neighborhood.")?;

    // Initialize story chain
    let mut chain = StoryChain::new(
        "Initial scene content".to_string(),
        "Initial scene reasoning".to_string(),
    );

    // Generate a few scenes
    let ai_provider = MockAIProvider;
    let mut current_node = "root".to_string();
    
    for _ in 0..2 {
        let next_nodes = chain.generate_next_nodes(
            &current_node,
            &ai_provider,
            Some("A story about a quiet neighborhood."),
        ).await?;
        
        if next_nodes.is_empty() {
            break;
        }
        current_node = next_nodes[0].clone();
    }

    // Verify chain structure
    assert!(chain.nodes.len() > 1, "Story chain should have multiple nodes");
    
    // Clean up test artifacts
    std::fs::remove_dir_all(test_dir)?;
    
    Ok(())
}

#[test]
fn test_story_export() -> Result<(), StoryChainError> {
    let chain = StoryChain::new(
        "Test content".to_string(),
        "Test reasoning".to_string(),
    );

    let test_output = "test_story.json";
    chain.export_to_file(test_output)?;

    // Verify file was created and contains valid JSON
    let content = std::fs::read_to_string(test_output)?;
    let _: StoryChain = serde_json::from_str(&content)?;

    // Clean up
    std::fs::remove_file(test_output)?;

    Ok(())
} 