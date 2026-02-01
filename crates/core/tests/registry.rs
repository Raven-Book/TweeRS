// Unit tests for NodeRegistry
use tweers_core::pipeline::{NodeRegistry, PipeMap, PipeNode};
use tweers_core::error::Result;
use async_trait::async_trait;

// Test node implementation
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
fn test_registry_new() {
    let registry = NodeRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_register_and_create() {
    let mut registry = NodeRegistry::new();

    registry.register("test_node", || {
        Box::new(TestNode {
            name: "test".to_string(),
        })
    });

    assert!(registry.contains("test_node"));
    assert_eq!(registry.len(), 1);

    let node = registry.create("test_node");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name(), "test");
}

#[test]
fn test_registry_create_nonexistent() {
    let registry = NodeRegistry::new();
    let node = registry.create("nonexistent");
    assert!(node.is_none());
}

#[test]
fn test_registry_multiple_nodes() {
    let mut registry = NodeRegistry::new();

    registry.register("node1", || {
        Box::new(TestNode {
            name: "node1".to_string(),
        })
    });

    registry.register("node2", || {
        Box::new(TestNode {
            name: "node2".to_string(),
        })
    });

    assert_eq!(registry.len(), 2);
    let names = registry.list_nodes();
    assert!(names.contains(&"node1".to_string()));
    assert!(names.contains(&"node2".to_string()));
}
