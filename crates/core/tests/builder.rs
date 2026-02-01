// Unit tests for PipelineBuilder
use tweers_core::pipeline::{NodeRegistry, PipelineBuilder, PipeMap, PipeNode};
use tweers_core::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

struct TestNode {
    name: String,
}

#[async_trait]
impl PipeNode for TestNode {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn input(&self) -> Vec<String> {
        vec![]
    }

    fn output(&self) -> Vec<String> {
        vec![]
    }

    async fn process(&self, data: PipeMap) -> Result<PipeMap> {
        Ok(data)
    }
}

#[test]
fn test_builder_basic() {
    let mut registry = NodeRegistry::new();
    registry.register("test_node", || {
        Box::new(TestNode {
            name: "test".to_string(),
        })
    });

    let registry = Arc::new(registry);
    let _builder = PipelineBuilder::new("test_pipeline", registry);

    // Builder should be created successfully
    assert!(true);
}

#[test]
fn test_builder_add_node_success() {
    let mut registry = NodeRegistry::new();
    registry.register("node1", || {
        Box::new(TestNode {
            name: "node1".to_string(),
        })
    });

    let registry = Arc::new(registry);
    let result = PipelineBuilder::new("test", registry)
        .add_node("node1");

    assert!(result.is_ok());
}

#[test]
fn test_builder_add_nonexistent_node() {
    let registry = NodeRegistry::new();
    let registry = Arc::new(registry);

    let result = PipelineBuilder::new("test", registry)
        .add_node("nonexistent");

    assert!(result.is_err());
}
