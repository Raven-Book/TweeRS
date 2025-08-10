/// Pipeline processing error types  
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Pipeline configuration error: {0}")]
    ConfigurationError(String),
    #[error("Node processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Dependency error: {0}")]
    DependencyError(String),
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("Missing required input '{required}' for node '{node}'")]
    MissingInput { node: String, required: String },
    #[error("Missing required output '{required}' for node '{node}'")]
    MissingOutput { node: String, required: String },
    #[error("Node processing error: {0}")]
    NodeError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Pipeline configuration error: {message}")]
    ConfigError { message: String },
}

// Legacy alias for backwards compatibility
pub type PipelineError = ProcessingError;

impl ProcessingError {
    /// Create a new ConfigurationError
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        Self::ConfigurationError(msg.into())
    }

    /// Create a new ProcessingFailed error
    pub fn processing_failed(msg: impl Into<String>) -> Self {
        Self::ProcessingFailed(msg.into())
    }

    /// Create a new DependencyError
    pub fn dependency_error(msg: impl Into<String>) -> Self {
        Self::DependencyError(msg.into())
    }

    /// Create a new SerializationError
    pub fn serialization_error(msg: impl Into<String>) -> Self {
        Self::SerializationError(msg.into())
    }

    /// Create a new ContextError
    pub fn context_error(msg: impl Into<String>) -> Self {
        Self::ContextError(msg.into())
    }
}

/// Result type alias for pipeline operations
pub type PipelineResult<T> = Result<T, ProcessingError>;
