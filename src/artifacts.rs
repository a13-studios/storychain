use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::StoryChainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManager {
    artifacts: HashMap<String, Artifact>,
    artifact_dir: String,
}

impl ArtifactManager {
    pub fn new(artifact_dir: &str) -> Self {
        Self {
            artifacts: HashMap::new(),
            artifact_dir: artifact_dir.to_string(),
        }
    }

    pub fn load_from_dir(&mut self) -> Result<(), StoryChainError> {
        let path = Path::new(&self.artifact_dir);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                let artifact: Artifact = serde_json::from_str(&content)?;
                self.artifacts.insert(artifact.id.clone(), artifact);
            }
        }

        Ok(())
    }

    pub fn save_artifact(&self, artifact: &Artifact) -> Result<(), StoryChainError> {
        let path = Path::new(&self.artifact_dir)
            .join(format!("{}.json", artifact.id));
        
        let content = serde_json::to_string_pretty(artifact)?;
        std::fs::write(path, content)?;
        
        Ok(())
    }

    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    pub fn update_artifact(&mut self, artifact: Artifact) -> Result<(), StoryChainError> {
        self.artifacts.insert(artifact.id.clone(), artifact.clone());
        self.save_artifact(&artifact)?;
        Ok(())
    }

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

    pub fn get_artifacts_by_type(&self, artifact_type: &ArtifactType) -> Vec<&Artifact> {
        self.artifacts
            .values()
            .filter(|a| &a.artifact_type == artifact_type)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    pub id: String,
    pub content: String,
    pub artifact_type: ArtifactType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactType {
    Premise,
    CharacterArc,
    PlotOutline,
    WorldBuilding,
    Custom(String),
} 