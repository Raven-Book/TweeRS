use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::skip::parse_js_object;

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
    /// Source file path (for IDE integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// Line number in source file (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line: Option<u32>,
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
        let start = content
            .find("window.storyFormat")
            .ok_or("Could not find window.storyFormat in format file")?;
        let obj_start = content[start..]
            .find('{')
            .map(|i| start + i)
            .ok_or("Could not find opening brace")?;

        let fields = parse_js_object(&content[obj_start..])?;

        let parse_str = |key: &str| -> Option<String> {
            fields.get(key).and_then(|s| serde_json::from_str(s).ok())
        };

        Ok(StoryFormat {
            name: parse_str("name"),
            version: parse_str("version").ok_or("version is required")?,
            proofing: fields.get("proofing").map(|s| s == "true").unwrap_or(false),
            source: parse_str("source").ok_or("source is required")?,
            author: parse_str("author"),
            description: parse_str("description"),
            image: parse_str("image"),
            url: parse_str("url"),
            license: parse_str("license"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::StoryFormat;

    #[test]
    fn parses_story_format_with_unknown_function_before_source() {
        let format = r#"window.storyFormat({
            name: "TestFormat",
            version: "1.0.0",
            setup: function () {
                const pattern = /[,{}]/g;
                const message = `value: ${{ nested: { ok: true } }.nested.ok}`;
                return { pattern, message };
            },
            proofing: false,
            source: "<html>{{STORY_NAME}}{{STORY_DATA}}</html>",
            hydrate: function () {
                return { ignored: true };
            }
        });"#;

        let parsed = StoryFormat::parse(format).expect("story format should parse");

        assert_eq!(parsed.name.as_deref(), Some("TestFormat"));
        assert_eq!(parsed.version, "1.0.0");
        assert!(!parsed.proofing);
        assert_eq!(parsed.source, "<html>{{STORY_NAME}}{{STORY_DATA}}</html>");
    }

    #[test]
    fn parses_story_format_with_js_string_escapes() {
        let format = r#"window.storyFormat({
            "name": "Escaped",
            "version": "1.0.0",
            "proofing": false,
            "source": "line\nbullet:\u2022 quote:\""
        });"#;

        let parsed = StoryFormat::parse(format).expect("story format should parse");

        assert_eq!(parsed.source, "line\nbullet:• quote:\"");
    }
}
