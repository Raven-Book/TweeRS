use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::skip::skip;

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
    /// Parse format.js content to extract StoryFormat
    /// Handles non-standard JSON that may contain JavaScript functions
    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        const KNOWN_FIELDS: &[&str] = &[
            "name",
            "version",
            "proofing",
            "source",
            "author",
            "description",
            "image",
            "url",
            "license",
        ];

        let json_obj = skip(content, KNOWN_FIELDS)?;

        serde_json::from_str(&json_obj)
            .map_err(|e| format!("Failed to parse story format JSON: {e}").into())
    }
}
