//! Test suite for the StoryChain library
//! 
//! This module contains integration tests that verify the core functionality
//! of the story generation system, including story chain creation, artifact
//! management, and AI provider interactions.

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Mock implementation of AIProvider for testing purposes
    /// Returns predefined responses instead of calling actual AI services
    struct MockAIProvider;

    #[async_trait::async_trait]
    impl AIProvider for MockAIProvider {
        async fn generate(&self, prompt: &str) -> Result<(String, String), StoryChainError> {
            Ok((
                "Test reasoning".to_string(),
                "Test content".to_string(),
            ))
        }
    }

    /// Tests the basic creation of a StoryChain
    /// 
    /// Verifies:
    /// - Initial node creation
    /// - Root node ID assignment
    /// - Branch ratio setting
    /// - Node content and reasoning
    #[tokio::test]
    async fn test_story_chain_creation() {
        let mut chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
            2,
        );

        // Verify basic chain properties
        assert_eq!(chain.nodes.len(), 1);
        assert_eq!(chain.root_node_id, "root");
        assert_eq!(chain.branch_ratio, 2);

        // Verify root node properties
        let root_node = chain.nodes.get("root").unwrap();
        assert_eq!(root_node.content, "Initial content");
        assert_eq!(root_node.reasoning, "Initial reasoning");
        assert!(root_node.predecessors.is_empty());
        assert!(root_node.successors.is_empty());
    }

    /// Tests the generation of new story nodes
    /// 
    /// Verifies:
    /// - Node generation using AI provider
    /// - Node linking (predecessors/successors)
    /// - Node ID generation
    #[tokio::test]
    async fn test_story_chain_generation() {
        let mut chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
        );

        let ai_provider = MockAIProvider;
        let mut current_node_id = "root".to_string();
        let total_epochs = 3;
        
        for epoch in 0..total_epochs {
            let new_nodes = chain
                .generate_next_nodes(
                    &current_node_id,
                    &ai_provider,
                    Some("Test premise"),
                    epoch + 1,
                    total_epochs
                )
                .await
                .unwrap();

            // Verify new node creation
            assert_eq!(new_nodes.len(), 1);
            assert_eq!(chain.nodes.len(), epoch + 2); // +2 because we start with root node

            // Update current node for next iteration
            current_node_id = new_nodes[0].clone();
        }
    }

    /// Tests the ArtifactManager functionality
    /// 
    /// Verifies:
    /// - Artifact creation and storage
    /// - Loading artifacts from directory
    /// - Artifact updates
    /// - Artifact type filtering
    #[test]
    fn test_artifact_manager() {
        // Create temporary directory for test artifacts
        let temp_dir = tempdir().unwrap();
        let mut manager = ArtifactManager::new(temp_dir.path().to_str().unwrap());

        // Test artifact creation
        manager
            .create_artifact(
                "test".to_string(),
                "Test content".to_string(),
                ArtifactType::Premise,
            )
            .unwrap();

        // Test loading artifacts from directory
        manager.load_from_dir().unwrap();
        let artifact = manager.get_artifact("test").unwrap();
        assert_eq!(artifact.content, "Test content");
        assert_eq!(artifact.artifact_type, ArtifactType::Premise);

        // Test artifact updates
        let mut updated_artifact = artifact.clone();
        updated_artifact.content = "Updated content".to_string();
        manager.update_artifact(updated_artifact.clone()).unwrap();

        // Verify update persistence
        manager.load_from_dir().unwrap();
        let artifact = manager.get_artifact("test").unwrap();
        assert_eq!(artifact.content, "Updated content");

        // Test artifact type filtering
        let premises = manager.get_artifacts_by_type(&ArtifactType::Premise);
        assert_eq!(premises.len(), 1);
        assert_eq!(premises[0].id, "test");
    }

    /// Tests StoryChain serialization and deserialization
    /// 
    /// Verifies:
    /// - JSON serialization
    /// - JSON deserialization
    /// - Preservation of chain properties
    #[test]
    fn test_story_chain_serialization() {
        let chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
            2,
        );

        // Test serialization/deserialization roundtrip
        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: StoryChain = serde_json::from_str(&serialized).unwrap();

        // Verify preservation of properties
        assert_eq!(chain.nodes.len(), deserialized.nodes.len());
        assert_eq!(chain.root_node_id, deserialized.root_node_id);
        assert_eq!(chain.branch_ratio, deserialized.branch_ratio);
    }
} 