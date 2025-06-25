use crate::pipeline::{PipeMap, error::PipelineError};
use async_trait::async_trait;
use tracing::debug;

#[async_trait]
pub trait PipeNode: Send + Sync {
    fn name(&self) -> String;

    fn input(&self) -> Vec<String>;

    fn output(&self) -> Vec<String>;

    async fn process(
        &self,
        data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>>;

    fn validate_input(
        &self,
        data: &PipeMap,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for input in self.input() {
            let is_optional = input.ends_with('?');
            let clean_input = if is_optional {
                input.trim_end_matches('?')
            } else {
                input.as_str()
            };

            if !is_optional && !data.contains_key(clean_input) {
                return Err(Box::new(PipelineError::MissingInput {
                    node: self.name(),
                    required: clean_input.to_string(),
                }));
            }
        }
        Ok(())
    }

    fn validate_output(
        &self,
        data: &PipeMap,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for output in self.output() {
            // Output does not support optional syntax, all outputs are required
            if !data.contains_key(&output) {
                return Err(Box::new(PipelineError::MissingOutput {
                    node: self.name(),
                    required: output.to_string(),
                }));
            }
        }
        Ok(())
    }
}

pub struct Pipeline {
    nodes: Vec<Box<dyn PipeNode>>,
    name: String,
    external_inputs: Vec<String>,
}

impl Pipeline {
    pub fn new(name: &str) -> Self {
        Self {
            nodes: Vec::new(),
            name: name.to_string(),
            external_inputs: Vec::new(),
        }
    }

    /// Set external inputs that will be provided via initial PipeMap
    pub fn with_external_inputs(mut self, external_inputs: Vec<String>) -> Self {
        self.external_inputs = external_inputs;
        self
    }

    pub fn add_node(mut self, node: Box<dyn PipeNode>) -> Result<Self, PipelineError> {
        if self.nodes.is_empty() {
            // First node: inputs will be provided externally via initial PipeMap
            debug!(
                "Adding first node '{}' to pipeline '{}'. Inputs: {:?} (will be provided externally)",
                node.name(),
                self.name,
                node.input()
            );
        } else {
            let current_inputs = node.input();
            let mut missing_inputs = Vec::new();
            let mut available_sources = Vec::new();

            // Collect outputs from previous nodes
            for existing_node in &self.nodes {
                available_sources.extend(existing_node.output());
            }

            // Add external inputs as available sources
            available_sources.extend(self.external_inputs.clone());

            for input in &current_inputs {
                let is_optional = input.ends_with('?');
                let clean_input = if is_optional {
                    input.trim_end_matches('?')
                } else {
                    input.as_str()
                };

                let is_available = available_sources.iter().any(|source| source == clean_input);

                if !is_available && !is_optional {
                    missing_inputs.push(input.clone());
                }
            }

            if !missing_inputs.is_empty() {
                return Err(PipelineError::ConfigError {
                    message: format!(
                        "Node '{}' requires inputs {:?} that are not available.\n\
                         Available sources (previous node outputs + external inputs): {:?}\n\
                         Required inputs: {:?}\n\
                         Optional inputs: {:?}\n\
                         Tip: Add '?' suffix to make inputs optional (e.g., 'config?')",
                        node.name(),
                        missing_inputs,
                        available_sources,
                        current_inputs
                            .iter()
                            .filter(|input| !input.ends_with('?'))
                            .collect::<Vec<_>>(),
                        current_inputs
                            .iter()
                            .filter(|input| input.ends_with('?'))
                            .collect::<Vec<_>>()
                    ),
                });
            }

            debug!(
                "Adding node '{}' to pipeline '{}'. Inputs: {:?}, Available sources: {:?}",
                node.name(),
                self.name,
                current_inputs,
                available_sources
            );
        }

        self.nodes.push(node);
        Ok(self)
    }

    pub async fn execute(&self, mut data: PipeMap) -> Result<PipeMap, PipelineError> {
        debug!(
            "Executing pipeline '{}' with {} nodes",
            self.name,
            self.nodes.len()
        );

        for (index, node) in self.nodes.iter().enumerate() {
            debug!("Processing node {}: '{}'", index + 1, node.name());

            node.validate_input(&data)
                .map_err(PipelineError::NodeError)?;

            data = node.process(data).await.map_err(PipelineError::NodeError)?;

            node.validate_output(&data)
                .map_err(PipelineError::NodeError)?;

            debug!("Node '{}' processed successfully", node.name());
        }

        debug!("Pipeline '{}' executed successfully", self.name);
        Ok(data)
    }
}
