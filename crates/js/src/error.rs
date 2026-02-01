use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSError {
    #[error("JavaScript compilation error: {0}")]
    CompilationError(String),
    #[error("JavaScript runtime error: {0}")]
    RuntimeError(String),
    #[error("JavaScript syntax error: {0}")]
    SyntaxError(String),
    #[error("JavaScript module error: {0}")]
    ModuleError(String),
    #[error("JavaScript evaluation error: {0}")]
    EvaluationError(String),
    #[error("V8 engine error: {0}")]
    V8Error(String),
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
}

pub type ScriptError = JSError;

impl JSError {
    pub fn compilation_error(msg: impl Into<String>) -> Self {
        Self::CompilationError(msg.into())
    }

    pub fn runtime_error(msg: impl Into<String>) -> Self {
        Self::RuntimeError(msg.into())
    }

    pub fn syntax_error(msg: impl Into<String>) -> Self {
        Self::SyntaxError(msg.into())
    }

    pub fn module_error(msg: impl Into<String>) -> Self {
        Self::ModuleError(msg.into())
    }

    pub fn evaluation_error(msg: impl Into<String>) -> Self {
        Self::EvaluationError(msg.into())
    }

    pub fn v8_error(msg: impl Into<String>) -> Self {
        Self::V8Error(msg.into())
    }
}

pub type JSResult<T> = Result<T, JSError>;
pub type ScriptResult<T> = JSResult<T>;

// Conversion to TweersError
impl From<JSError> for tweers_core::error::TweersError {
    fn from(err: JSError) -> Self {
        tweers_core::error::TweersError::script(err.to_string())
    }
}
