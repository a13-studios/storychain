#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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

    #[tokio::test]
    async fn test_story_chain_creation() {
        let mut chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
            2,
        );

        assert_eq!(chain.nodes.len(), 1);
        assert_eq!(chain.root_node_id, "root");
        assert_eq!(chain.branch_ratio, 2);

        let root_node = chain.nodes.get("root").unwrap();
        assert_eq!(root_node.content, "Initial content");
        assert_eq!(root_node.reasoning, "Initial reasoning");
        assert!(root_node.predecessors.is_empty());
        assert!(root_node.successors.is_empty());
    }

    #[tokio::test]
    async fn test_story_chain_generation() {
        let mut chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
            2,
        );

        let ai_provider = MockAIProvider;
        let new_nodes = chain.generate_next_nodes("root", &ai_provider).await.unwrap();

        assert_eq!(new_nodes.len(), 1);
        assert_eq!(chain.nodes.len(), 2);

        let root_node = chain.nodes.get("root").unwrap();
        assert_eq!(root_node.successors.len(), 1);
        assert_eq!(root_node.successors[0], "root_0");

        let new_node = chain.nodes.get("root_0").unwrap();
        assert_eq!(new_node.predecessors.len(), 1);
        assert_eq!(new_node.predecessors[0], "root");
    }

    #[test]
    fn test_artifact_manager() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ArtifactManager::new(temp_dir.path().to_str().unwrap());

        // Test creating and saving an artifact
        manager
            .create_artifact(
                "test".to_string(),
                "Test content".to_string(),
                ArtifactType::Premise,
            )
            .unwrap();

        // Test loading artifacts
        manager.load_from_dir().unwrap();
        let artifact = manager.get_artifact("test").unwrap();
        assert_eq!(artifact.content, "Test content");
        assert_eq!(artifact.artifact_type, ArtifactType::Premise);

        // Test updating an artifact
        let mut updated_artifact = artifact.clone();
        updated_artifact.content = "Updated content".to_string();
        manager.update_artifact(updated_artifact.clone()).unwrap();

        // Verify the update
        manager.load_from_dir().unwrap();
        let artifact = manager.get_artifact("test").unwrap();
        assert_eq!(artifact.content, "Updated content");

        // Test getting artifacts by type
        let premises = manager.get_artifacts_by_type(&ArtifactType::Premise);
        assert_eq!(premises.len(), 1);
        assert_eq!(premises[0].id, "test");
    }

    #[test]
    fn test_story_chain_serialization() {
        let chain = StoryChain::new(
            "Initial content".to_string(),
            "Initial reasoning".to_string(),
            2,
        );

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: StoryChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.nodes.len(), deserialized.nodes.len());
        assert_eq!(chain.root_node_id, deserialized.root_node_id);
        assert_eq!(chain.branch_ratio, deserialized.branch_ratio);
    }
} 