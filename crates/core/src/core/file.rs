//! Shared file handling logic for CLI and API
//!
//! This module provides unified file type detection and passage creation
//! to ensure consistent behavior between CLI pipeline and API.

use crate::core::story::{Passage, StoryData};
use crate::excel::parser::ExcelParser;
use indexmap::IndexMap;

/// Detected file type based on extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    JavaScript,
    Css,
    Excel,
    Twee,
    Media,
    Unknown,
}

/// Result of parsing a single source
#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub passages: IndexMap<String, Passage>,
    pub story_data: Option<StoryData>,
}

impl ParsedSource {
    pub fn new() -> Self {
        Self {
            passages: IndexMap::new(),
            story_data: None,
        }
    }

    pub fn with_passage(mut self, name: String, passage: Passage) -> Self {
        self.passages.insert(name, passage);
        self
    }
}

impl Default for ParsedSource {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect file type from filename/path
pub fn detect_file_type(filename: &str) -> FileType {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("js") => FileType::JavaScript,
        Some("css") => FileType::Css,
        Some("xlsx" | "xlsm" | "xlsb" | "xls") => FileType::Excel,
        Some("twee" | "tw") => FileType::Twee,
        Some(ext) if is_media_extension(ext) => FileType::Media,
        _ => FileType::Unknown,
    }
}

/// Check if extension is a media file
fn is_media_extension(ext: &str) -> bool {
    use crate::config::constants;
    constants::IMAGE_EXTENSIONS.contains(&ext)
        || constants::AUDIO_EXTENSIONS.contains(&ext)
        || constants::VIDEO_EXTENSIONS.contains(&ext)
        || constants::VTT_EXTENSIONS.contains(&ext)
}

/// Create a script passage from JavaScript content
pub fn create_script_passage(name: String, content: String) -> Passage {
    Passage {
        name,
        tags: Some("script".to_string()),
        position: None,
        size: None,
        content,
        source_file: None,
        source_line: None,
    }
}

/// Create a stylesheet passage from CSS content
pub fn create_stylesheet_passage(name: String, content: String) -> Passage {
    Passage {
        name,
        tags: Some("stylesheet".to_string()),
        position: None,
        size: None,
        content,
        source_file: None,
        source_line: None,
    }
}

/// Create an init script passage (used for Excel-generated JavaScript)
pub fn create_init_script_passage(name: String, content: String) -> Passage {
    Passage {
        name,
        tags: Some("init script".to_string()),
        position: None,
        size: None,
        content,
        source_file: None,
        source_line: None,
    }
}

/// Create an HTML passage (used for Excel-generated HTML)
pub fn create_html_passage(name: String, content: String) -> Passage {
    Passage {
        name,
        tags: Some("html".to_string()),
        position: None,
        size: None,
        content,
        source_file: None,
        source_line: None,
    }
}

/// Parse text content based on detected file type
///
/// This is the core parsing function that handles different file types:
/// - `.js` files become script passages
/// - `.css` files become stylesheet passages
/// - `.twee`/`.tw` files are parsed as Twee format
/// - Unknown extensions are treated as Twee files
pub fn parse_text_content(
    name: &str,
    content: &str,
) -> Result<ParsedSource, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::parser::TweeParser;

    let file_type = detect_file_type(name);

    match file_type {
        FileType::JavaScript => {
            let passage = create_script_passage(name.to_string(), content.to_string());
            Ok(ParsedSource::new().with_passage(name.to_string(), passage))
        }
        FileType::Css => {
            let passage = create_stylesheet_passage(name.to_string(), content.to_string());
            Ok(ParsedSource::new().with_passage(name.to_string(), passage))
        }
        FileType::Twee | FileType::Unknown => {
            // Parse as Twee file
            let (passages, story_data) = TweeParser::parse(content)?;
            Ok(ParsedSource {
                passages,
                story_data,
            })
        }
        FileType::Excel | FileType::Media => {
            // Text content shouldn't be Excel or Media - treat as Twee
            let (passages, story_data) = TweeParser::parse(content)?;
            Ok(ParsedSource {
                passages,
                story_data,
            })
        }
    }
}

/// Parse binary content (Excel files)
pub fn parse_bytes_content(
    name: &str,
    data: &[u8],
) -> Result<ParsedSource, Box<dyn std::error::Error + Send + Sync>> {
    let file_type = detect_file_type(name);

    match file_type {
        FileType::Excel => {
            let result = ExcelParser::parse_from_bytes(data.to_vec())?;
            let mut parsed = ParsedSource::new();

            // Create JavaScript passage if there's JavaScript code
            if !result.javascript.is_empty() {
                let passage = create_init_script_passage(name.to_string(), result.javascript);
                parsed.passages.insert(name.to_string(), passage);
            }

            // Create HTML passage if there's HTML code
            if !result.html.is_empty() {
                let html_name = format!("{}_html", name);
                let passage = create_html_passage(html_name.clone(), result.html);
                parsed.passages.insert(html_name, passage);
            }

            Ok(parsed)
        }
        _ => {
            // Non-Excel binary files - return empty result
            Ok(ParsedSource::new())
        }
    }
}

/// Aggregate multiple parsed sources into final passages and story data
///
/// This handles:
/// - Merging all passages from multiple sources
/// - Taking the first StoryData found
/// - Cross-file StoryTitle merge (if StoryData.name is None, get from StoryTitle passage)
pub fn aggregate_sources(
    sources: Vec<ParsedSource>,
) -> Result<(IndexMap<String, Passage>, StoryData), Box<dyn std::error::Error + Send + Sync>> {
    let mut all_passages = IndexMap::new();
    let mut story_data: Option<StoryData> = None;

    for source in sources {
        for (name, passage) in source.passages {
            all_passages.insert(name, passage);
        }
        if story_data.is_none() && source.story_data.is_some() {
            story_data = source.story_data;
        }
    }

    let mut final_story_data = story_data.ok_or("StoryData is required")?;

    // Cross-file StoryTitle merge
    if final_story_data.name.is_none()
        && let Some(title_passage) = all_passages.get("StoryTitle")
    {
        final_story_data.name = Some(title_passage.content.trim().to_string());
    }

    Ok((all_passages, final_story_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type("script.js"), FileType::JavaScript);
        assert_eq!(detect_file_type("style.css"), FileType::Css);
        assert_eq!(detect_file_type("story.twee"), FileType::Twee);
        assert_eq!(detect_file_type("story.tw"), FileType::Twee);
        assert_eq!(detect_file_type("data.xlsx"), FileType::Excel);
        assert_eq!(detect_file_type("image.png"), FileType::Media);
        assert_eq!(detect_file_type("unknown.txt"), FileType::Unknown);
    }

    #[test]
    fn test_create_script_passage() {
        let passage =
            create_script_passage("test.js".to_string(), "console.log('hi');".to_string());
        assert_eq!(passage.name, "test.js");
        assert_eq!(passage.tags, Some("script".to_string()));
        assert_eq!(passage.content, "console.log('hi');");
    }

    #[test]
    fn test_create_stylesheet_passage() {
        let passage = create_stylesheet_passage("test.css".to_string(), "body { }".to_string());
        assert_eq!(passage.name, "test.css");
        assert_eq!(passage.tags, Some("stylesheet".to_string()));
        assert_eq!(passage.content, "body { }");
    }
}
