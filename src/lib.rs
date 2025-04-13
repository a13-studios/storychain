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

#[derive(Error, Debug)]
pub enum StoryChainError {
    #[error("AI server error: {0}")]
    AIServerError(String),
    #[error("Invalid reasoning format: {0}")]
    InvalidReasoningFormat(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryNode {
    pub id: String,
    pub content: String,
    pub reasoning: String,
    pub predecessor: Option<String>,
    pub successor: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryChain {
    pub nodes: HashMap<String, StoryNode>,
    pub root_node_id: String,
}

#[async_trait::async_trait]
pub trait AIProvider {
    async fn generate(&self, prompt: &str) -> Result<(String, String), StoryChainError>;
}

pub struct DeepseekProvider {
    model: String,
    log_file: String,
}

impl DeepseekProvider {
    pub fn new(model: String, log_file: String) -> Self {
        Self { model, log_file }
    }

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
    async fn generate(&self, prompt: &str) -> Result<(String, String), StoryChainError> {
        info!("Sending request to Ollama for model: {}", self.model);
        debug!("Prompt: {}", prompt);

        let output = Command::new("ollama")
            .arg("run")
            .arg(&self.model)
            .arg(prompt)
            .output()
            .map_err(|e| {
                error!("Failed to execute Ollama command: {}", e);
                StoryChainError::AIServerError(format!("Failed to execute Ollama command: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Ollama command failed: {}", stderr);
            return Err(StoryChainError::AIServerError(format!(
                "Ollama command failed: {}",
                stderr
            )));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| {
                error!("Failed to parse Ollama output: {}", e);
                StoryChainError::AIServerError(format!("Failed to parse Ollama output: {}", e))
            })?;

        debug!("Raw AI response: {}", response_text);

        // Log the response for debugging and persistence
        self.log_response(prompt, &response_text)?;

        // Try to parse the response for think block using a more robust regex
        let re = regex::Regex::new(r"(?s)<think>(.*?)</think>\s*(.*)").unwrap();
        if let Some(captures) = re.captures(&response_text) {
            let reasoning = captures[1].trim().to_string();
            let content = captures[2].trim().to_string();
            
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

    pub async fn generate_next_nodes(
        &mut self,
        current_node_id: &str,
        ai_provider: &dyn AIProvider,
        premise: Option<&str>,
    ) -> Result<Vec<String>, StoryChainError> {
        let start_time = std::time::Instant::now();
        debug!("Generating next node for: {}", current_node_id);
        let current_node = self.nodes.get(current_node_id)
            .ok_or_else(|| StoryChainError::AIServerError("Node not found".to_string()))?;

        let mut prompt = String::new();
        
        if let Some(premise) = premise {
            debug!("Including premise in prompt");
            prompt.push_str(&format!("Story Premise:\n{}\n\n", premise));
        }
        
        let prompt_time = start_time.elapsed();
        debug!("Prompt preparation took: {:?}", prompt_time);
        
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
        
        // Create new node with a unique ID based on the number of existing nodes
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
        
        // Update current node's successor
        if let Some(node) = self.nodes.get_mut(current_node_id) {
            node.successor = Some(new_id.clone());
            debug!("Updated successor for node: {}", current_node_id);
        }

        self.nodes.insert(new_id.clone(), new_node);
        let total_time = start_time.elapsed();
        info!("Total node generation took: {:?}", total_time);
        Ok(vec![new_id])
    }

    pub fn export_to_file(&self, path: &str) -> Result<(), StoryChainError> {
        info!("Exporting story chain to file: {}", path);
        let serialized = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, serialized)?;
        info!("Successfully exported story chain");
        Ok(())
    }
} 