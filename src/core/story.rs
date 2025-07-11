use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::format;
use std::path::Path;
use tracing::debug;

use crate::config::constants;

/// StoryData Passage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryData {
    /// Maps to <tw-storydata name>. From StoryTitle Passage
    pub name: Option<String>,
    /// Maps to <tw-storydata ifid>
    pub ifid: String,
    /// Maps to <tw-storydata format>
    pub format: String,
    /// Maps to <tw-storydata format-version>
    #[serde(alias = "format-version")]
    pub format_version: String,
    /// Maps to <tw-passagedata name> of the node whose pid matches <tw-storydata startnode>
    pub start: Option<String>,
    /// Pairs map to <tw-tag> nodes as <tw-tag name>:<tw-tag color>
    #[serde(alias = "tag-colors")]
    pub tag_colors: Option<HashMap<String, String>>,
    /// Maps to <tw-storydata zoom>
    pub zoom: Option<f32>,
}

/// Passage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passage {
    /// The name of the passage
    pub name: String,
    /// Any tags for the passage separated by spaces
    pub tags: Option<String>,
    /// Comma-separated X and Y position of the upper-left of the passage when viewed within the Twine 2 editor
    pub position: Option<String>,
    /// Comma-separated width and height of the passage when viewed within the Twine 2 editor
    pub size: Option<String>,
    /// The content of passage
    pub content: String,
}

/*
/// Color
pub enum Color {
    Gray,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}*/

/// StoryFormat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryFormat {
    /// The name of the story format
    pub name: Option<String>,
    /// The version of story format
    pub version: String,
    ///  True if the story format is a "proofing" format. The distinction is relevant only in the Twine 2 UI
    #[serde(default)]
    pub proofing: bool,
    /// An adequately escaped string containing the full HTML output of the story format, including the two placeholders {{STORY_NAME}} and {{STORY_DATA}}
    pub source: String,

    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

/*

/// Simple version parsing for semantic version comparison
#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    original: String,
}

/// Parse version string into Version struct

impl Version {
    fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = if parts.len() >= 3 {
                parts[2].parse().unwrap_or(0)
            } else {
                0
            };
            Some(Version {
                major,
                minor,
                patch,
                original: version.to_string(),
            })
        } else {
            None
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}
*/

impl StoryData {
    /// Validate required fields
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_none() {
            return Err("Story name is required (missing StoryTitle passage?)".to_string());
        }
        if self.ifid.is_empty() {
            return Err("IFID is required in StoryData".to_string());
        }
        if self.format.is_empty() {
            return Err("Format is required in StoryData".to_string());
        }
        if self.format_version.is_empty() {
            return Err("Format version is required in StoryData".to_string());
        }
        Ok(())
    }
}

impl StoryFormat {
    pub async fn load(path: &Path) -> Result<StoryFormat, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse(&content)
    }

    /// Parse format.js content to extract StoryFormat
    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let re = Regex::new(r"window\.storyFormat\s*\(\s*(\{[\s\S]*}\s*)")?;

        if let Some(caps) = re.captures(content) {
            let json = caps
                .get(1)
                .ok_or("Failed to extract story format json from regex match".to_string())?;
            let json_str = json.as_str();
            let story_format = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to parse story format JSON: {e}"))?;
            Ok(story_format)
        } else {
            Err("Could not find window.storyFormat(...) in format file".into())
        }
    }

    pub async fn find_format(
        story_format: &str,
        version: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
            return Err(std::format!(
                "Story formats directory not found: {}",
                format_dir.display()
            )
            .into());
        }

        // First, try to find by directory name pattern: format-version
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
                return Self::load(&format_file).await;
            } else {
                tracing::warn!(
                    "Directory '{}' exists but missing format.js file",
                    expected_dir_name
                );
            }
        }

        // Fallback: scan all directories and warn about non-standard naming
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

            // Check if directory follows the expected naming pattern
            if !dir_name.contains('-') {
                tracing::warn!(
                    "Story format directory '{}' does not follow the standard naming pattern 'format-version'",
                    dir_name
                );
            }

            let format_file = entry_path.join("format.js");

            if !format_file.exists() {
                tracing::warn!(
                    "Story format directory '{}' is missing format.js file",
                    dir_name
                );
                continue;
            }

            match Self::load(&format_file).await {
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
                        return Ok(story_format_struct);
                    } else {
                        found_formats.push((
                            dir_name.to_string(),
                            story_format_struct.name.clone(),
                            story_format_struct.version.clone(),
                        ));
                    }
                }
                Err(e) => {
                    tracing::warn!(
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
}
