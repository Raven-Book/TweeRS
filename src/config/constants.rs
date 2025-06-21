//! Constants for TweeRS

use std::path::PathBuf;
use std::sync::OnceLock;

/// Full path of the executable file
pub static EXECUTABLE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Storage path for StoryFormat
pub const STORY_FORMAT_DIR: &str = "story-format";

/// Storage path for config.toml
pub const CONFIG_FILE: &str = "config.toml";

/// Log file path
pub const LOG_FILE: &str = "tweers.log";

/// Supported Twee extensions
pub const TWEE_EXTENSIONS: &[&str] = &["twee", "tw"];

pub fn init_constants() {
    let exe_path = std::env::current_exe().expect("Failed to get executable path");
    EXECUTABLE_PATH
        .set(exe_path)
        .expect("EXECUTABLE_PATH has already been initialized");
}
