/// Centralized error handling for TweeRS
pub mod excel;
pub mod js;
pub mod pipeline;

pub use excel::{ExcelParseError, ExcelResult};
pub use js::{JSError, JSResult, ScriptError, ScriptResult};
pub use pipeline::{PipelineError, PipelineResult, ProcessingError};
