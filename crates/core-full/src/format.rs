// Story format discovery and loading module

use std::path::Path;
use tracing::{debug, warn};
use tweers_core::config::constants;
use tweers_core::core::story::StoryFormat;

/// Load format.js content from a specific path
pub async fn load_format_from_path(
    path: &Path,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

/// Find and load story format from file system
/// Returns the format.js content as a String
pub async fn find_and_load_format(
    story_format: &str,
    version: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let exe_path = constants::EXECUTABLE_PATH
        .get()
        .ok_or("Executable path not initialized")?;

    let parent_dir = exe_path
        .parent()
        .ok_or("Failed to get parent directory".to_string())?;
    let format_dir = parent_dir.join(constants::STORY_FORMAT_DIR);

    debug!(
        "Searching for story format '{}' version '{}' in directory: {}",
        story_format,
        version,
        format_dir.display()
    );

    if !format_dir.exists() {
        return Err(format!(
            "Story formats directory not found: {}",
            format_dir.display()
        )
        .into());
    }

    // Try direct directory name match first
    let expected_dir_name = format!("{}-{}", story_format.to_lowercase(), version);
    let target_dir = format_dir.join(&expected_dir_name);

    debug!("Looking for directory: {}", target_dir.display());

    if target_dir.exists() && target_dir.is_dir() {
        let format_file = target_dir.join("format.js");
        if format_file.exists() {
            debug!(
                "Found format directory by name pattern: {}",
                expected_dir_name
            );
            return load_format_from_path(&format_file).await;
        } else {
            warn!(
                "Directory '{}' exists but missing format.js file",
                expected_dir_name
            );
        }
    }

    // Scan all directories to find matching format
    let mut entries = tokio::fs::read_dir(&format_dir)
        .await
        .map_err(|_| format!("Failed to read directory: {}", format_dir.display()))?;

    let mut found_formats = Vec::new();
    let mut entry_count = 0;

    while let Some(entry) = entries.next_entry().await? {
        entry_count += 1;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        let dir_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if !dir_name.contains('-') {
            warn!(
                "Story format directory '{}' does not follow the standard naming pattern 'format-version'",
                dir_name
            );
        }

        let format_file = entry_path.join("format.js");

        if !format_file.exists() {
            warn!(
                "Story format directory '{}' is missing format.js file",
                dir_name
            );
            continue;
        }

        // Load and parse the format to check name and version
        match load_format_from_path(&format_file).await {
            Ok(content) => match StoryFormat::parse(&content) {
                Ok(story_format_struct) => {
                    debug!(
                        "Successfully loaded story format from: {}",
                        format_file.display()
                    );
                    debug!(
                        "Format name: {:?}, version: {}",
                        story_format_struct.name, story_format_struct.version
                    );

                    let name_match = match story_format_struct.name {
                        Some(ref name) => {
                            let matches = name.eq_ignore_ascii_case(story_format);
                            debug!(
                                "Name comparison: '{}' vs '{}' = {}",
                                name, story_format, matches
                            );
                            matches
                        }
                        None => {
                            debug!("Story format has no name field");
                            continue;
                        }
                    };

                    if name_match && story_format_struct.version == version {
                        debug!(
                            "Found exact match: name='{}', version='{}'",
                            story_format, version
                        );
                        return Ok(content);
                    } else {
                        found_formats.push((
                            dir_name.to_string(),
                            story_format_struct.name.clone(),
                            story_format_struct.version.clone(),
                        ));
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to parse story format from {}: {}",
                        format_file.display(),
                        e
                    );
                    continue;
                }
            },
            Err(e) => {
                warn!(
                    "Failed to load story format from {}: {}",
                    format_file.display(),
                    e
                );
                continue;
            }
        }
    }

    debug!(
        "Found {} entries in format directory: {}",
        entry_count,
        format_dir.display()
    );

    if !found_formats.is_empty() {
        debug!("Available formats found:");
        for (dir, name, ver) in &found_formats {
            debug!(
                "  Directory: '{}' -> Name: {:?}, Version: '{}'",
                dir, name, ver
            );
        }
    }

    Err(format!(
        "Story format '{story_format}' version '{version}' not found. Available formats: {found_formats:?}"
    )
    .into())
}
