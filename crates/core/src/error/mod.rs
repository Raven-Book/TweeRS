/// Centralized error handling for TweeRS
pub mod excel;
pub mod pipeline;
pub mod tweers;

pub use excel::{ExcelParseError, ExcelResult};
pub use pipeline::{PipelineError, PipelineResult, ProcessingError};
pub use tweers::{Result, TweersError};
