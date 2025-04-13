//! Artifact Management System
//! 
//! This module provides functionality for managing various artifacts used in the story
//! generation process, such as premises, character arcs, and world-building details.
//! It handles the persistence and retrieval of these artifacts from the file system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::StoryChainError;

/// Manages the storage and retrieval of story-related artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManager {
    /// Map of artifact IDs to their corresponding Artifact instances
    artifacts: HashMap<String, Artifact>,
    
    /// Directory where artifacts are stored on disk
    artifact_dir: String,
}

impl ArtifactManager {
    /// Creates a new ArtifactManager instance
    /// 
    /// # Arguments
    /// * `artifact_dir` - The directory where artifacts will be stored
    pub fn new(artifact_dir: &str) -> Self {
        Self {
            artifacts: HashMap::new(),
            artifact_dir: artifact_dir.to_string(),
        }
    }

    /// Loads all artifacts from the specified directory
    /// 
    /// Creates the directory if it doesn't exist and loads all JSON files
    /// within it as artifacts.
    pub fn load_from_dir(&mut self) -> Result<(), StoryChainError> {
        let path = Path::new(&self.artifact_dir);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
            return Ok(());
        }

        // Iterate through all files in the directory
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            // Only process JSON files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                let artifact: Artifact = serde_json::from_str(&content)?;
                self.artifacts.insert(artifact.id.clone(), artifact);
            }
        }

        Ok(())
    }

    /// Saves a single artifact to disk
    /// 
    /// # Arguments
    /// * `artifact` - The artifact to save
    pub fn save_artifact(&self, artifact: &Artifact) -> Result<(), StoryChainError> {
        let path = Path::new(&self.artifact_dir)
            .join(format!("{}.json", artifact.id));
        
        let content = serde_json::to_string_pretty(artifact)?;
        std::fs::write(path, content)?;
        
        Ok(())
    }

    /// Retrieves an artifact by its ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the artifact to retrieve
    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    /// Updates an existing artifact or creates a new one
    /// 
    /// # Arguments
    /// * `artifact` - The artifact to update
    pub fn update_artifact(&mut self, artifact: Artifact) -> Result<(), StoryChainError> {
        self.artifacts.insert(artifact.id.clone(), artifact.clone());
        self.save_artifact(&artifact)?;
        Ok(())
    }

    /// Creates a new artifact with the specified parameters
    /// 
    /// # Arguments
    /// * `id` - Unique identifier for the artifact
    /// * `content` - The content of the artifact
    /// * `artifact_type` - The type of artifact being created
    pub fn create_artifact(
        &mut self,
        id: String,
        content: String,
        artifact_type: ArtifactType,
    ) -> Result<(), StoryChainError> {
        let artifact = Artifact {
            id,
            content,
            artifact_type,
            metadata: HashMap::new(),
        };
        
        self.artifacts.insert(artifact.id.clone(), artifact.clone());
        self.save_artifact(&artifact)?;
        
        Ok(())
    }

    /// Retrieves all artifacts of a specific type
    /// 
    /// # Arguments
    /// * `artifact_type` - The type of artifacts to retrieve
    pub fn get_artifacts_by_type(&self, artifact_type: &ArtifactType) -> Vec<&Artifact> {
        self.artifacts
            .values()
            .filter(|a| &a.artifact_type == artifact_type)
            .collect()
    }
}

/// Represents a single story-related artifact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    /// Unique identifier for the artifact
    pub id: String,
    
    /// The actual content of the artifact
    pub content: String,
    
    /// The type of the artifact (e.g., Premise, CharacterArc)
    pub artifact_type: ArtifactType,
    
    /// Additional metadata associated with this artifact
    pub metadata: HashMap<String, String>,
}

/// Enumerates the different types of artifacts that can be managed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactType {
    /// The foundational premise of the story
    Premise,
    
    /// Character development and arc information
    CharacterArc,
    
    /// Overall plot structure and outline
    PlotOutline,
    
    /// World-building details and background
    WorldBuilding,
    
    /// Custom artifact type with specified name
    Custom(String),
} 