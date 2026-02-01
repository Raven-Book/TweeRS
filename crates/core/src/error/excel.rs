use calamine::XlsxError;
/// Excel parsing error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExcelParseError {
    #[error("Failed to open Excel file: {0}")]
    FileError(#[from] XlsxError),
    #[error("Worksheet '{0}' not found")]
    WorksheetNotFound(String),
    #[error("Invalid table format: {0}")]
    InvalidFormat(String),
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    #[error("Array index validation failed: {0}")]
    ArrayIndexError(String),
    #[error("Type parsing error: {0}")]
    TypeParseError(String),
    #[error("Data validation error: {0}")]
    DataValidationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl ExcelParseError {
    /// Create a new InvalidFormat error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a new MissingHeader error
    pub fn missing_header(header: impl Into<String>) -> Self {
        Self::MissingHeader(header.into())
    }

    /// Create a new ArrayIndexError
    pub fn array_index_error(msg: impl Into<String>) -> Self {
        Self::ArrayIndexError(msg.into())
    }

    /// Create a new TypeParseError
    pub fn type_parse_error(msg: impl Into<String>) -> Self {
        Self::TypeParseError(msg.into())
    }

    /// Create a new DataValidationError
    pub fn data_validation_error(msg: impl Into<String>) -> Self {
        Self::DataValidationError(msg.into())
    }

    /// Create a new ConfigError
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }
}

/// Result type alias for Excel parsing operations
pub type ExcelResult<T> = Result<T, ExcelParseError>;
