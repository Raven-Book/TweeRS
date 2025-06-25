use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Missing required input '{required}' for node '{node}'")]
    MissingInput { node: String, required: String },

    #[error("Missing required output '{required}' for node '{node}'")]
    MissingOutput { node: String, required: String },

    #[error("Pipeline configuration error: {message}")]
    ConfigError { message: String },

    #[error("Node processing error: {0}")]
    NodeError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
