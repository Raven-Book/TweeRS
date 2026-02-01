use crate::core::story::StoryFormat;
/// Context traits for build configuration
use std::path::PathBuf;

/// Build configuration interface
pub trait BuildConfig: Send + Sync {
    fn is_debug(&self) -> bool;
    fn base64(&self) -> bool;
    fn start_passage(&self) -> Option<&str>;
    fn assets_dirs(&self) -> &[PathBuf];
}

/// Format information interface
pub trait FormatInfo: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn story_format(&self) -> Option<&StoryFormat>;
}

/// Build context interface (combines config and format)
pub trait BuildContext: Send + Sync {
    fn config(&self) -> &dyn BuildConfig;
    fn format(&self) -> &dyn FormatInfo;
}
