// JavaScript-friendly types for WASM bindings

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// JavaScript-friendly input source
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum JsInputSource {
    #[serde(rename = "text")]
    Text { name: String, content: String },
    #[serde(rename = "bytes")]
    Bytes {
        name: String,
        data: Vec<u8>,
        mime_type: Option<String>,
    },
}

/// JavaScript-friendly story format info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsStoryFormatInfo {
    pub name: String,
    pub version: String,
    pub source: String,
}

/// JavaScript-friendly build configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsBuildConfig {
    pub sources: Vec<JsInputSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_info: Option<JsStoryFormatInfo>,
    #[serde(default)]
    pub is_debug: bool,
    pub start_passage: Option<String>,
}

/// JavaScript-friendly Passage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsPassage {
    pub name: String,
    pub tags: Option<String>,
    pub position: Option<String>,
    pub size: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_line: Option<u32>,
}

/// JavaScript-friendly StoryData
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsStoryData {
    pub name: Option<String>,
    pub ifid: String,
    pub format: String,
    #[serde(rename = "format-version")]
    pub format_version: String,
    pub start: Option<String>,
    #[serde(rename = "tag-colors")]
    pub tag_colors: Option<std::collections::HashMap<String, String>>,
    pub zoom: Option<f32>,
}

/// JavaScript-friendly parse output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsParseOutput {
    pub passages: std::collections::HashMap<String, JsPassage>,
    pub story_data: JsStoryData,
    pub format_info: JsStoryFormatInfo,
    pub is_debug: bool,
}

/// JavaScript-friendly build output
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct JsBuildOutput {
    html: String,
}

#[wasm_bindgen]
impl JsBuildOutput {
    #[wasm_bindgen(getter)]
    pub fn html(&self) -> String {
        self.html.clone()
    }

    #[wasm_bindgen(constructor)]
    pub fn new(html: String) -> Self {
        Self { html }
    }
}

// ============================================================================
// Type conversions between JS types and Rust API types
// ============================================================================

impl From<JsInputSource> for crate::api::InputSource {
    fn from(js_source: JsInputSource) -> Self {
        match js_source {
            JsInputSource::Text { name, content } => {
                crate::api::InputSource::Text { name, content }
            }
            JsInputSource::Bytes {
                name,
                data,
                mime_type,
            } => crate::api::InputSource::Bytes {
                name,
                data,
                mime_type,
            },
        }
    }
}

impl From<JsStoryFormatInfo> for crate::api::StoryFormatInfo {
    fn from(js_info: JsStoryFormatInfo) -> Self {
        crate::api::StoryFormatInfo {
            name: js_info.name,
            version: js_info.version,
            source: js_info.source,
        }
    }
}

impl From<JsBuildConfig> for crate::api::BuildConfig {
    fn from(js_config: JsBuildConfig) -> Self {
        // Use a dummy format_info if not provided (for parse-only operations)
        let format_info = js_config
            .format_info
            .map(|info| info.into())
            .unwrap_or_else(|| crate::api::StoryFormatInfo {
                name: String::new(),
                version: String::new(),
                source: String::new(),
            });

        let sources: Vec<crate::api::InputSource> =
            js_config.sources.into_iter().map(|s| s.into()).collect();

        crate::api::BuildConfig::new(format_info)
            .sources(sources)
            .debug(js_config.is_debug)
            .start_passage(js_config.start_passage)
    }
}

impl From<crate::core::story::Passage> for JsPassage {
    fn from(passage: crate::core::story::Passage) -> Self {
        JsPassage {
            name: passage.name,
            tags: passage.tags,
            position: passage.position,
            size: passage.size,
            content: passage.content,
            source_file: passage.source_file,
            source_line: passage.source_line,
        }
    }
}

impl From<JsPassage> for crate::core::story::Passage {
    fn from(js_passage: JsPassage) -> Self {
        crate::core::story::Passage {
            name: js_passage.name,
            tags: js_passage.tags,
            position: js_passage.position,
            size: js_passage.size,
            content: js_passage.content,
            source_file: js_passage.source_file,
            source_line: js_passage.source_line,
        }
    }
}

impl From<crate::core::story::StoryData> for JsStoryData {
    fn from(story_data: crate::core::story::StoryData) -> Self {
        JsStoryData {
            name: story_data.name,
            ifid: story_data.ifid,
            format: story_data.format,
            format_version: story_data.format_version,
            start: story_data.start,
            tag_colors: story_data.tag_colors,
            zoom: story_data.zoom,
        }
    }
}

impl From<JsStoryData> for crate::core::story::StoryData {
    fn from(js_story_data: JsStoryData) -> Self {
        crate::core::story::StoryData {
            name: js_story_data.name,
            ifid: js_story_data.ifid,
            format: js_story_data.format,
            format_version: js_story_data.format_version,
            start: js_story_data.start,
            tag_colors: js_story_data.tag_colors,
            zoom: js_story_data.zoom,
        }
    }
}

impl From<crate::api::ParseOutput> for JsParseOutput {
    fn from(parse_output: crate::api::ParseOutput) -> Self {
        let passages: std::collections::HashMap<String, JsPassage> = parse_output
            .passages
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        JsParseOutput {
            passages,
            story_data: parse_output.story_data.into(),
            format_info: JsStoryFormatInfo {
                name: parse_output.format_info.name,
                version: parse_output.format_info.version,
                source: parse_output.format_info.source,
            },
            is_debug: parse_output.is_debug,
        }
    }
}
