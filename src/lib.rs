//! StoryChain Library - Core functionality for AI-driven narrative generation
//! 
//! This library provides the core components for generating narratives using AI models.
//! It includes structures for managing story nodes, chains of narrative content,
//! and interfaces for AI providers that generate the actual content.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use log::{info, debug, error};
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub mod artifacts;
pub use artifacts::{Artifact, ArtifactManager, ArtifactType};

/// Represents possible errors that can occur during story generation
/// and related operations.
#[derive(Error, Debug)]
pub enum StoryChainError {
    /// Error communicating with the AI server
    #[error("AI server error: {0}")]
    AIServerError(String),
    
    /// Error parsing the AI's response format
    #[error("Invalid reasoning format: {0}")]
    InvalidReasoningFormat(String),
    
    /// File system operation error
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Represents a single node in the story chain, containing the narrative content
/// and metadata about its connections to other nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryNode {
    /// Unique identifier for the node
    pub id: String,
    
    /// The actual narrative content of this node
    pub content: String,
    
    /// The AI's reasoning for generating this content
    pub reasoning: String,
    
    /// ID of the previous node in the chain (if any)
    pub predecessor: Option<String>,
    
    /// ID of the next node in the chain (if any)
    pub successor: Option<String>,
    
    /// Additional metadata associated with this node
    pub metadata: HashMap<String, String>,
}

/// Represents a complete chain of story nodes, forming a narrative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryChain {
    /// Map of node IDs to their corresponding StoryNode instances
    pub nodes: HashMap<String, StoryNode>,
    
    /// ID of the first node in the chain
    pub root_node_id: String,
}

/// Trait defining the interface for AI providers that generate story content.
#[async_trait::async_trait]
pub trait AIProvider {
    /// Generates content based on a given prompt
    /// 
    /// # Arguments
    /// * `prompt` - The prompt to send to the AI model
    /// 
    /// # Returns
    /// A tuple of (reasoning, content) strings or an error
    async fn generate(&self, prompt: &str) -> Result<(String, String), StoryChainError>;
}

/// Implementation of AIProvider using the Deepseek language model
pub struct DeepseekProvider {
    /// The specific Deepseek model to use
    model: String,
    
    /// Path to the file where AI responses will be logged
    log_file: String,
}

impl DeepseekProvider {
    /// Creates a new DeepseekProvider instance
    pub fn new(model: String, log_file: String) -> Self {
        Self { model, log_file }
    }

    /// Logs AI interactions to a file for debugging and analysis
    /// 
    /// # Arguments
    /// * `prompt` - The prompt sent to the AI
    /// * `response` - The AI's response
    fn log_response(&self, prompt: &str, response: &str) -> Result<(), StoryChainError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| StoryChainError::IOError(e))?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        writeln!(file, "=== AI Response at {} ===", timestamp)?;
        writeln!(file, "Prompt: {}", prompt)?;
        writeln!(file, "Response: {}", response)?;
        writeln!(file, "=== End Response ===\n")?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl AIProvider for DeepseekProvider {
    /// Generates story content using the Deepseek model via Ollama
    async fn generate(&self, prompt: &str) -> Result<(String, String), StoryChainError> {
        info!("Sending request to Ollama for model: {}", self.model);
        debug!("Prompt: {}", prompt);

        // Execute Ollama command to generate content
        let output = Command::new("ollama")
            .arg("run")
            .arg(&self.model)
            .arg(prompt)
            .output()
            .map_err(|e| {
                error!("Failed to execute Ollama command: {}", e);
                StoryChainError::AIServerError(format!("Failed to execute Ollama command: {}", e))
            })?;

        // Check for command execution success
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Ollama command failed: {}", stderr);
            return Err(StoryChainError::AIServerError(format!(
                "Ollama command failed: {}",
                stderr
            )));
        }

        // Parse the output into UTF-8 string
        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| {
                error!("Failed to parse Ollama output: {}", e);
                StoryChainError::AIServerError(format!("Failed to parse Ollama output: {}", e))
            })?;

        debug!("Raw AI response: {}", response_text);

        // Log the response for debugging
        self.log_response(prompt, &response_text)?;

        // Parse the response to extract reasoning and content
        let re = regex::Regex::new(r"(?s)<think>(.*?)</think>\s*(.*)").unwrap();
        if let Some(captures) = re.captures(&response_text) {
            let reasoning = captures[1].trim().to_string();
            let content = captures[2].trim().to_string();
            
            // Validate that neither part is empty
            if reasoning.is_empty() || content.is_empty() {
                error!("Empty reasoning or content in response");
                return Err(StoryChainError::InvalidReasoningFormat(
                    "Empty reasoning or content in response".to_string(),
                ));
            }
            
            info!("Successfully parsed reasoning and content from response");
            return Ok((reasoning, content));
        }

        error!("Could not parse AI response: {}", response_text);
        Err(StoryChainError::InvalidReasoningFormat(format!(
            "Could not parse AI response into reasoning and content. Response: {}",
            response_text
        )))
    }
}

impl StoryChain {
    /// Creates a new StoryChain with an initial root node
    pub fn new(root_content: String, root_reasoning: String) -> Self {
        info!("Creating new story chain");
        let root_node = StoryNode {
            id: "root".to_string(),
            content: root_content,
            reasoning: root_reasoning,
            predecessor: None,
            successor: None,
            metadata: HashMap::new(),
        };

        let mut nodes = HashMap::new();
        nodes.insert("root".to_string(), root_node);

        Self {
            nodes,
            root_node_id: "root".to_string(),
        }
    }

    /// Generates the next node(s) in the story chain
    /// 
    /// # Arguments
    /// * `current_node_id` - ID of the node to generate from
    /// * `ai_provider` - The AI provider to use for generation
    /// * `premise` - Optional premise to include in generation
    pub async fn generate_next_nodes(
        &mut self,
        current_node_id: &str,
        ai_provider: &dyn AIProvider,
        premise: Option<&str>,
    ) -> Result<Vec<String>, StoryChainError> {
        let start_time = std::time::Instant::now();
        debug!("Generating next node for: {}", current_node_id);
        
        // Get the current node or return error if not found
        let current_node = self.nodes.get(current_node_id)
            .ok_or_else(|| StoryChainError::AIServerError("Node not found".to_string()))?;

        let mut prompt = String::new();
        
        // Include premise in prompt if provided
        if let Some(premise) = premise {
            debug!("Including premise in prompt");
            prompt.push_str(&format!("Story Premise:\n{}\n\n", premise));
        }
        
        let prompt_time = start_time.elapsed();
        debug!("Prompt preparation took: {:?}", prompt_time);
        
        // Construct the prompt for the next scene
        prompt.push_str(&format!(
            "You are continuing a story. Here is the previous scene and its reasoning:\n\n\
            Previous Scene Reasoning:\n{}\n\n\
            Previous Scene Content:\n{}\n\n\
            Now continue the story, maintaining consistency with the previous scene and the overall premise.\n\n\
            IMPORTANT: Format your response EXACTLY as follows:\n\
            <think>\n\
            Your reasoning about how this scene continues the story and develops the narrative.\n\
            </think>\n\
            Write your scene content here, making sure it flows naturally from the previous scene...",
            current_node.reasoning,
            current_node.content
        ));

        debug!("Sending prompt to AI provider");
        let generation_start = std::time::Instant::now();
        let (reasoning, content) = ai_provider.generate(&prompt).await?;
        let generation_time = generation_start.elapsed();
        info!("AI generation took: {:?}", generation_time);
        
        // Create new node with unique ID
        let new_id = format!("node_{}", self.nodes.len());
        debug!("Creating new node: {}", new_id);
        
        let new_node = StoryNode {
            id: new_id.clone(),
            content,
            reasoning,
            predecessor: Some(current_node_id.to_string()),
            successor: None,
            metadata: HashMap::new(),
        };
        
        // Update the current node's successor reference
        if let Some(node) = self.nodes.get_mut(current_node_id) {
            node.successor = Some(new_id.clone());
            debug!("Updated successor for node: {}", current_node_id);
        }

        self.nodes.insert(new_id.clone(), new_node);
        let total_time = start_time.elapsed();
        info!("Total node generation took: {:?}", total_time);
        Ok(vec![new_id])
    }

    /// Exports the story chain to a JSON file
    pub fn export_to_file(&self, path: &str) -> Result<(), StoryChainError> {
        info!("Exporting story chain to file: {}", path);
        let serialized = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, serialized)?;
        info!("Successfully exported story chain");
        Ok(())
    }

    /// Exports the story chain to a markdown file
    /// 
    /// # Arguments
    /// * `path` - The path where the markdown file should be saved
    pub fn export_to_markdown(&self, path: &str) -> Result<(), StoryChainError> {
        let mut content = String::new();
        
        // Add header
        content.push_str("# Generated Story\n\n");
        content.push_str(&format!("*Generated on {}*\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        content.push_str("---\n\n");

        // Start with root node
        let mut current_id = &self.root_node_id;
        let mut scene_num = 1;

        // Process each node in sequence
        while let Some(node) = self.nodes.get(current_id) {
            // Add scene header
            content.push_str(&format!("## Scene {}\n\n", scene_num));
            
            // Add scene content
            content.push_str(&node.content);
            content.push_str("\n\n");
            
            // Add AI's reasoning in a collapsible section
            content.push_str("<details>\n<summary>AI's Reasoning</summary>\n\n");
            content.push_str(&node.reasoning);
            content.push_str("\n</details>\n\n---\n\n");
            
            // Move to next node if it exists
            if let Some(next_id) = &node.successor {
                current_id = next_id;
                scene_num += 1;
            } else {
                break;
            }
        }

        // Write to file
        std::fs::write(path, content)?;
        Ok(())
    }
} 