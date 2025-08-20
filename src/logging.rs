//! Logging utilities for TweeRS

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::constants::EXECUTABLE_PATH;

/// Generate unique log file path based on current directory, PID, and timestamp
pub fn get_log_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let folder_name = current_dir
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
        .to_string_lossy();

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let pid = std::process::id();

    let logs_dir = EXECUTABLE_PATH
        .get()
        .ok_or("EXECUTABLE_PATH not initialized")?
        .parent()
        .ok_or("Cannot get executable parent directory")?
        .join("logs");

    std::fs::create_dir_all(&logs_dir)?;

    let log_filename = format!("{folder_name}_{pid}_{timestamp}.log");
    let log_file_path = logs_dir.join(log_filename);

    Ok(log_file_path)
}

/// Create log file with proper options
pub fn create_log_file() -> Result<std::fs::File, Box<dyn std::error::Error>> {
    let log_path = get_log_file_path()?;

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)?;

    println!("Log file created: {}", log_path.display());

    Ok(log_file)
}
