//! Shared file handling logic for CLI and API
//!
//! This module provides unified file type detection and passage creation
//! to ensure consistent behavior between CLI pipeline and API.

use crate::core::story::{Passage, StoryData};
use crate::excel::parser::ExcelParser;
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::warn;

pub const TWEERS_PATHS_PASSAGE: &str = "TweersPaths";

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct TweersPathSource {
    #[serde(rename = "type")]
    source_type: String,
    path: String,
    dir: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    passages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
struct TweersPathsPayload {
    version: u32,
    sources: Vec<TweersPathSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TweersPathKey {
    source_type: String,
    path: String,
    dir: String,
    name: String,
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
            let mut passage = create_script_passage(name.to_string(), content.to_string());
            passage.source_file = Some(name.to_string());
            Ok(ParsedSource::new().with_passage(name.to_string(), passage))
        }
        FileType::Css => {
            let mut passage = create_stylesheet_passage(name.to_string(), content.to_string());
            passage.source_file = Some(name.to_string());
            Ok(ParsedSource::new().with_passage(name.to_string(), passage))
        }
        FileType::Twee | FileType::Unknown => {
            // Parse as Twee file
            let (mut passages, story_data) = TweeParser::parse(content)?;
            set_passages_source_file(&mut passages, name);
            Ok(ParsedSource {
                passages,
                story_data,
            })
        }
        FileType::Excel | FileType::Media => {
            // Text content shouldn't be Excel or Media - treat as Twee
            let (mut passages, story_data) = TweeParser::parse(content)?;
            set_passages_source_file(&mut passages, name);
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
                let mut passage = create_init_script_passage(name.to_string(), result.javascript);
                passage.source_file = Some(name.to_string());
                parsed.passages.insert(name.to_string(), passage);
            }

            // Create HTML passage if there's HTML code
            if !result.html.is_empty() {
                let html_name = format!("{}_html", name);
                let mut passage = create_html_passage(html_name.clone(), result.html);
                passage.source_file = Some(name.to_string());
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

pub fn build_tweers_paths_content(
    source_roots: &[PathBuf],
    source_paths: &[PathBuf],
    passages: &IndexMap<String, Passage>,
) -> String {
    let mut collected: BTreeMap<TweersPathKey, Vec<(u32, usize, String)>> = BTreeMap::new();

    for source_path in source_paths {
        if let Some(source_key) = tweers_path_key_from_path(source_path.as_path(), source_roots) {
            collected.entry(source_key).or_default();
        }
    }

    for (index, passage) in passages.values().enumerate() {
        let Some(source_file) = passage.source_file.as_deref() else {
            continue;
        };

        let Some(source_key) = tweers_path_key_from_path(Path::new(source_file), source_roots)
        else {
            continue;
        };

        let source_type = source_key.source_type.clone();
        let entry = collected.entry(source_key).or_default();
        if source_type == "twee" {
            entry.push((
                passage.source_line.unwrap_or(u32::MAX),
                index,
                passage.name.clone(),
            ));
        }
    }

    let sources = collected
        .into_iter()
        .map(|(source_key, mut passage_entries)| {
            let passages = if source_key.source_type == "twee" {
                passage_entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

                let mut names = Vec::new();
                for (_, _, name) in passage_entries {
                    if !names.iter().any(|existing| existing == &name) {
                        names.push(name);
                    }
                }
                Some(names)
            } else {
                None
            };

            TweersPathSource {
                source_type: source_key.source_type,
                path: source_key.path,
                dir: source_key.dir,
                name: source_key.name,
                passages,
            }
        })
        .collect();

    let payload = TweersPathsPayload {
        version: 1,
        sources,
    };

    serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|_| "{\"version\":1,\"sources\":[]}".to_string())
}

pub fn inject_tweers_paths(
    passages: &mut IndexMap<String, Passage>,
    source_roots: &[PathBuf],
    source_paths: &[PathBuf],
) {
    if passages.contains_key(TWEERS_PATHS_PASSAGE) {
        warn!(
            "Passage name '{}' is reserved and will be overwritten",
            TWEERS_PATHS_PASSAGE
        );
    }

    let content = build_tweers_paths_content(source_roots, source_paths, passages);
    passages.insert(
        TWEERS_PATHS_PASSAGE.to_string(),
        Passage {
            name: TWEERS_PATHS_PASSAGE.to_string(),
            tags: None,
            position: None,
            size: None,
            content,
            source_file: None,
            source_line: None,
        },
    );
}

fn set_passages_source_file(passages: &mut IndexMap<String, Passage>, source_file: &str) {
    let source_file = source_file.to_string();
    for passage in passages.values_mut() {
        passage.source_file = Some(source_file.clone());
    }
}

fn tweers_path_key_from_path(file_path: &Path, roots: &[PathBuf]) -> Option<TweersPathKey> {
    let ext = file_path.extension()?.to_str()?.to_lowercase();
    let source_type = match ext.as_str() {
        "twee" | "tw" => "twee",
        "js" => "js",
        "css" => "css",
        _ => return None,
    };

    let normalized_original = normalize_path_string(&file_path.to_string_lossy());
    let relative_path = roots
        .iter()
        .find_map(|root| file_path.strip_prefix(root).ok())
        .map(|relative| normalize_path_string(&relative.to_string_lossy()))
        .unwrap_or(normalized_original);

    let normalized_relative = relative_path.trim_start_matches("./").to_string();
    let normalized_relative = if normalized_relative.is_empty() {
        file_path.file_name()?.to_string_lossy().to_string()
    } else {
        normalized_relative
    };

    let dir = Path::new(&normalized_relative)
        .parent()
        .and_then(|parent| parent.to_str())
        .filter(|parent| !parent.is_empty())
        .map(normalize_path_string)
        .unwrap_or_else(|| ".".to_string());

    Some(TweersPathKey {
        source_type: source_type.to_string(),
        path: normalized_relative,
        dir,
        name: file_path.file_name()?.to_string_lossy().to_string(),
    })
}

fn normalize_path_string(path: &str) -> String {
    path.replace('\\', "/")
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

    #[test]
    fn test_parse_text_content_sets_source_file_for_twee() {
        let parsed = parse_text_content("story/main.twee", ":: Start\nHello\n\n:: Second\nWorld\n")
            .expect("parse should succeed");

        assert_eq!(
            parsed.passages["Start"].source_file.as_deref(),
            Some("story/main.twee")
        );
        assert_eq!(
            parsed.passages["Second"].source_file.as_deref(),
            Some("story/main.twee")
        );
    }

    #[test]
    fn test_build_tweers_paths_content_includes_twee_passages() {
        let mut passages = IndexMap::new();
        passages.insert(
            "Start".to_string(),
            Passage {
                name: "Start".to_string(),
                tags: None,
                position: None,
                size: None,
                content: "Hello".to_string(),
                source_file: Some("story/main.twee".to_string()),
                source_line: Some(10),
            },
        );
        passages.insert(
            "StoryTitle".to_string(),
            Passage {
                name: "StoryTitle".to_string(),
                tags: None,
                position: None,
                size: None,
                content: "Test".to_string(),
                source_file: Some("story/main.twee".to_string()),
                source_line: Some(1),
            },
        );
        passages.insert(
            "assets/theme.css".to_string(),
            Passage {
                name: "assets/theme.css".to_string(),
                tags: Some("stylesheet".to_string()),
                position: None,
                size: None,
                content: "body {}".to_string(),
                source_file: Some("assets/theme.css".to_string()),
                source_line: None,
            },
        );

        let content = build_tweers_paths_content(
            &[],
            &[
                PathBuf::from("story/main.twee"),
                PathBuf::from("assets/theme.css"),
            ],
            &passages,
        );

        let json: serde_json::Value =
            serde_json::from_str(&content).expect("TweersPaths should be valid JSON");
        let sources = json["sources"]
            .as_array()
            .expect("sources should be an array");

        assert!(sources.iter().any(|item| {
            item["type"] == "twee"
                && item["path"] == "story/main.twee"
                && item["dir"] == "story"
                && item["name"] == "main.twee"
                && item["passages"] == serde_json::json!(["StoryTitle", "Start"])
        }));
        assert!(sources.iter().any(|item| {
            item["type"] == "css"
                && item["path"] == "assets/theme.css"
                && item["dir"] == "assets"
                && item["name"] == "theme.css"
                && item.get("passages").is_none()
        }));
    }
}
