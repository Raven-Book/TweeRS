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

/// Supported image extensions for base64 encoding
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp"];

/// Supported audio extensions for base64 encoding
pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg", "aac", "m4a", "flac"];

/// Supported video extensions for base64 encoding
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "ogv", "avi", "mov", "wmv"];

/// Supported VTT extensions for base64 encoding
pub const VTT_EXTENSIONS: &[&str] = &["vtt"];

pub fn init_constants() {
    let exe_path = std::env::current_exe().expect("Failed to get executable path");
    EXECUTABLE_PATH
        .set(exe_path)
        .expect("EXECUTABLE_PATH has already been initialized");
}
