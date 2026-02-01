// Pipeline builder for fluent pipeline construction
use super::core::Pipeline;
use super::registry::NodeRegistry;
use crate::error::{Result, TweersError};
use std::sync::Arc;

/// Builder for constructing pipelines with registered nodes
pub struct PipelineBuilder {
    name: String,
    registry: Arc<NodeRegistry>,
    node_names: Vec<String>,
    external_inputs: Vec<String>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(name: impl Into<String>, registry: Arc<NodeRegistry>) -> Self {
        Self {
            name: name.into(),
            registry,
            node_names: Vec::new(),
            external_inputs: Vec::new(),
        }
    }

    /// Set external inputs for the pipeline
    pub fn with_external_inputs(mut self, inputs: Vec<String>) -> Self {
        self.external_inputs = inputs;
        self
    }

    /// Add a node by name
    pub fn add_node(mut self, name: impl Into<String>) -> Result<Self> {
        let node_name = name.into();
        if !self.registry.contains(&node_name) {
            return Err(TweersError::invalid_config(format!(
                "Node '{}' not found in registry",
                node_name
            )));
        }
        self.node_names.push(node_name);
        Ok(self)
    }

    /// Add multiple nodes by names
    pub fn add_nodes<I, S>(mut self, names: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for name in names {
            let node_name = name.into();
            if !self.registry.contains(&node_name) {
                return Err(TweersError::invalid_config(format!(
                    "Node '{}' not found in registry",
                    node_name
                )));
            }
            self.node_names.push(node_name);
        }
        Ok(self)
    }

    /// Build the pipeline
    pub fn build(self) -> Result<Pipeline> {
        let mut nodes = Vec::new();
        for name in &self.node_names {
            let node = self.registry.create(name).ok_or_else(|| {
                TweersError::invalid_config(format!("Node '{}' not found", name))
            })?;
            nodes.push(node);
        }

        let mut pipeline = Pipeline::new(self.name);
        pipeline = pipeline.with_external_inputs(self.external_inputs);

        for node in nodes {
            pipeline = pipeline.add_node(node)?;
        }

        Ok(pipeline)
    }
}
