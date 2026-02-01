/// Unified error type for TweeRS
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TweersError {
    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // Parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Excel parsing error: {0}")]
    Excel(#[from] crate::error::ExcelParseError),

    // Pipeline errors
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] crate::error::ProcessingError),

    // Configuration errors
    #[error("Missing required input: {0}")]
    MissingInput(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    // Format errors
    #[error("Format error: {0}")]
    Format(String),

    // Script errors (for JS integration)
    #[error("Script error: {0}")]
    Script(String),

    // Generic error for compatibility
    #[error("{0}")]
    Other(String),

    // Boxed error for dynamic error types
    #[error("Error: {0}")]
    Boxed(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Result type alias using TweersError
pub type Result<T> = std::result::Result<T, TweersError>;

impl TweersError {
    /// Create a parse error
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a missing input error
    pub fn missing_input(name: impl Into<String>) -> Self {
        Self::MissingInput(name.into())
    }

    /// Create an invalid config error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a format error
    pub fn format(msg: impl Into<String>) -> Self {
        Self::Format(msg.into())
    }

    /// Create a script error
    pub fn script(msg: impl Into<String>) -> Self {
        Self::Script(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

// Conversion from JSError (for tweers-js integration)
// This will be implemented in tweers-js crate to avoid circular dependency

// Conversion from String for convenience
impl From<String> for TweersError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

// Conversion from &str for convenience
impl From<&str> for TweersError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for TweersError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

// Conversion from zip::result::ZipError
