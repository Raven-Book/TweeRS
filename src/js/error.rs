use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Script execution failed: {0}")]
    ExecutionError(String),

    #[error("Script file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid script output: {0}")]
    InvalidOutput(String),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("V8 initialization error: {0}")]
    V8InitError(String),

    #[error("Script compilation error: {0}")]
    CompilationError(String),
}

pub type ScriptResult<T> = Result<T, ScriptError>;
