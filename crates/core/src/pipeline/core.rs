// Pipeline core - pure logic framework

use super::keys::TypedKey;
use crate::error::Result;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Pipeline data map for passing data between nodes
#[derive(Clone)]
pub struct PipeMap {
    data: HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl PipeMap {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a value with a typed key (compile-time type checking)
    pub fn insert_typed<T: Any + Send + Sync>(&mut self, key: TypedKey<T>, value: T) {
        self.data.insert(key.name().to_string(), Arc::new(value));
    }

    /// Get a value with a typed key (compile-time type checking)
    pub fn get_typed<T: Any + Send + Sync>(&self, key: TypedKey<T>) -> Option<&T> {
        self.data
            .get(key.name())
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Legacy string-based insert (deprecated, use insert_typed)
    #[deprecated(note = "Use insert_typed for type safety")]
    pub fn insert<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.data.insert(key.into(), Arc::new(value));
    }

    /// Legacy string-based get (deprecated, use get_typed)
    #[deprecated(note = "Use get_typed for type safety")]
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>())
    }
}

impl Default for PipeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline node trait
#[async_trait]
pub trait PipeNode: Send + Sync {
    fn name(&self) -> String;
    fn input(&self) -> Vec<String>;
    fn output(&self) -> Vec<String>;

    async fn process(&self, data: PipeMap) -> Result<PipeMap>;
}

/// Pipeline - orchestrates execution of nodes
pub struct Pipeline {
    name: String,
    nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
    external_inputs: Vec<String>,
}

impl Pipeline {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            external_inputs: Vec::new(),
        }
    }

    pub fn with_external_inputs(mut self, inputs: Vec<String>) -> Self {
        self.external_inputs = inputs;
        self
    }

    pub fn add_node(mut self, node: Box<dyn PipeNode + Send + Sync>) -> Result<Self> {
        self.nodes.push(node);
        Ok(self)
    }

    pub async fn execute(&self, mut data: PipeMap) -> Result<PipeMap> {
        for node in &self.nodes {
            data = node.process(data).await?;
        }
        Ok(data)
    }
}
