// Stable API facade for external consumers - Pure logic, no I/O

use crate::core::story::StoryData;

pub type Error = Box<dyn std::error::Error>;

/// Represents different types of input sources for building (pure logic)
#[derive(Clone, Debug)]
pub enum InputSource {
    /// Raw text content with a name/identifier
    Text { name: String, content: String },
    /// Raw bytes with a name and mime type hint
    Bytes {
        name: String,
        data: Vec<u8>,
        mime_type: Option<String>,
    },
}

/// Story format information (passed in, not discovered)
#[derive(Clone, Debug)]
pub struct StoryFormatInfo {
    pub name: String,
    pub version: String,
    /// The format.js content (already loaded)
    pub source: String,
}

/// Build configuration for pure core (no I/O)
#[derive(Clone, Debug)]
pub struct BuildConfig {
    sources: Vec<InputSource>,
    format_info: StoryFormatInfo,
    is_debug: bool,
    start_passage: Option<String>,
}

impl BuildConfig {
    pub fn new(format_info: StoryFormatInfo) -> Self {
        Self {
            sources: Vec::new(),
            format_info,
            is_debug: false,
            start_passage: None,
        }
    }

    pub fn sources<I>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = InputSource>,
    {
        self.sources = sources.into_iter().collect();
        self
    }

    /// Add text content as sources
    pub fn add_texts<I, S1, S2>(mut self, texts: I) -> Self
    where
        I: IntoIterator<Item = (S1, S2)>,
        S1: Into<String>,
        S2: Into<String>,
    {
        self.sources
            .extend(texts.into_iter().map(|(name, content)| InputSource::Text {
                name: name.into(),
                content: content.into(),
            }));
        self
    }

    /// Add bytes as sources
    pub fn add_bytes<I, S>(mut self, bytes: I) -> Self
    where
        I: IntoIterator<Item = (S, Vec<u8>, Option<String>)>,
        S: Into<String>,
    {
        self.sources.extend(
            bytes
                .into_iter()
                .map(|(name, data, mime_type)| InputSource::Bytes {
                    name: name.into(),
                    data,
                    mime_type,
                }),
        );
        self
    }

    pub fn debug(mut self, is_debug: bool) -> Self {
        self.is_debug = is_debug;
        self
    }

    pub fn start_passage(mut self, start_passage: Option<String>) -> Self {
        self.start_passage = start_passage;
        self
    }
}

/// Build output - returns HTML as string
#[derive(Clone, Debug)]
pub struct BuildOutput {
    pub html: String,
    pub story_data: StoryData,
}

/// Pure build function - no I/O (synchronous)
pub fn build(config: BuildConfig) -> Result<BuildOutput, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::output::HtmlOutputHandler;
    use crate::core::parser::TweeParser;
    use crate::core::story::StoryFormat;
    use indexmap::IndexMap;

    // Parse story format
    let story_format = StoryFormat::parse(&config.format_info.source)?;

    // Parse all sources
    let mut all_passages = IndexMap::new();
    let mut story_data = None;

    for source in &config.sources {
        match source {
            InputSource::Text { name: _, content } => {
                let (passages, data) = TweeParser::parse(content)?;
                for (passage_name, passage) in passages {
                    all_passages.insert(passage_name, passage);
                }
                if story_data.is_none() {
                    story_data = data;
                }
            }
            InputSource::Bytes {
                name,
                data,
                mime_type: _,
            } => {
                // Handle bytes (e.g., Excel files)
                if let Some(ext) = std::path::Path::new(name).extension()
                    && let Some(ext_str) = ext.to_str()
                    && matches!(ext_str, "xlsx" | "xlsm" | "xlsb" | "xls")
                {
                    use crate::excel::parser::ExcelParser;
                    let result = ExcelParser::parse_from_bytes(data.clone())?;

                    if !result.javascript.is_empty() {
                        let passage = crate::core::story::Passage {
                            name: name.clone(),
                            tags: Some("init script".to_string()),
                            position: None,
                            size: None,
                            content: result.javascript,
                        };
                        all_passages.insert(name.clone(), passage);
                    }
                }
            }
        }
    }

    let final_story_data = story_data.ok_or("StoryData is required")?;

    // Generate HTML (synchronous)
    let html = HtmlOutputHandler::generate_html(
        &all_passages,
        &Some(final_story_data.clone()),
        &story_format,
        config.is_debug,
    )?;

    Ok(BuildOutput {
        html,
        story_data: final_story_data,
    })
}
