// Pipeline node registry for dynamic node management
use super::core::PipeNode;
use std::collections::HashMap;

/// Factory function type for creating pipeline nodes
pub type NodeFactory = Box<dyn Fn() -> Box<dyn PipeNode> + Send + Sync>;

/// Registry for pipeline nodes
pub struct NodeRegistry {
    nodes: HashMap<String, NodeFactory>,
}

impl NodeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Register a node factory with a name
    pub fn register<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: Fn() -> Box<dyn PipeNode> + Send + Sync + 'static,
    {
        self.nodes.insert(name.into(), Box::new(factory));
    }

    /// Create a node instance by name
    pub fn create(&self, name: &str) -> Option<Box<dyn PipeNode>> {
        self.nodes.get(name).map(|factory| factory())
    }

    /// Check if a node is registered
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    /// List all registered node names
    pub fn list_nodes(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// Get the number of registered nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
