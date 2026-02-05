// Stable API facade for external consumers - Pure logic, no I/O

use crate::core::file::{aggregate_sources, parse_bytes_content, parse_text_content};
use crate::core::story::{Passage, StoryData, StoryFormat};
use indexmap::IndexMap;

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

/// Parse output - contains parsed passages and story data
/// Can be directly used with build_from_parsed after setting format_info.source
#[derive(Clone, Debug)]
pub struct ParseOutput {
    pub passages: IndexMap<String, Passage>,
    pub story_data: StoryData,
    /// Format info with empty source - needs to be filled before building
    pub format_info: StoryFormatInfo,
    pub is_debug: bool,
}

/// Build output - returns HTML as string
#[derive(Clone, Debug)]
pub struct BuildOutput {
    pub html: String,
    pub story_data: StoryData,
}

/// Configuration for building from parsed data
#[derive(Clone, Debug)]
pub struct BuildFromParsedConfig {
    pub passages: IndexMap<String, Passage>,
    pub story_data: StoryData,
    pub format_info: StoryFormatInfo,
    pub is_debug: bool,
}

/// Helper function to parse sources into passages and story data
fn parse_sources(
    sources: &[InputSource],
) -> Result<(IndexMap<String, Passage>, StoryData), Box<dyn std::error::Error + Send + Sync>> {
    let mut parsed_sources = Vec::new();

    for source in sources {
        match source {
            InputSource::Text { name, content } => {
                let parsed = parse_text_content(name, content)?;
                parsed_sources.push(parsed);
            }
            InputSource::Bytes {
                name,
                data,
                mime_type: _,
            } => {
                let parsed = parse_bytes_content(name, data)?;
                parsed_sources.push(parsed);
            }
        }
    }

    aggregate_sources(parsed_sources)
}

/// Pure build function - no I/O (synchronous)
pub fn build(config: BuildConfig) -> Result<BuildOutput, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::output::HtmlOutputHandler;
    use crate::util::sort::compare_paths;

    // Parse story format
    let story_format = StoryFormat::parse(&config.format_info.source)?;

    // Parse all sources
    let (passages, mut story_data) = parse_sources(&config.sources)?;

    // Sort passages by source_file using depth-first natural order
    let mut passages_vec: Vec<(String, Passage)> = passages.into_iter().collect();
    passages_vec.sort_by(|a, b| {
        compare_paths(
            a.1.source_file.as_deref().unwrap_or(&a.0),
            b.1.source_file.as_deref().unwrap_or(&b.0),
        )
    });
    let passages: IndexMap<String, Passage> = passages_vec.into_iter().collect();

    // Apply start_passage override if provided
    if config.start_passage.is_some() {
        story_data.start = config.start_passage;
    }

    // Generate HTML
    let html = HtmlOutputHandler::generate_html(
        &passages,
        &Some(story_data.clone()),
        &story_format,
        config.is_debug,
    )?;

    Ok(BuildOutput { html, story_data })
}

/// Parse sources without building HTML
pub fn parse(
    sources: Vec<InputSource>,
) -> Result<ParseOutput, Box<dyn std::error::Error + Send + Sync>> {
    let (passages, story_data) = parse_sources(&sources)?;

    // Extract format info from story_data and create empty source
    let format_info = StoryFormatInfo {
        name: story_data.format.clone(),
        version: story_data.format_version.clone(),
        source: String::new(), // Empty source - to be filled by caller
    };

    Ok(ParseOutput {
        passages,
        story_data,
        format_info,
        is_debug: false, // Default to false, can be set later
    })
}

/// Parse passages only - does not require StoryData
/// Useful for IDE integration where individual files need to be parsed
pub fn passages(
    sources: Vec<InputSource>,
) -> Result<IndexMap<String, Passage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut all_passages = IndexMap::new();

    for source in sources {
        match source {
            InputSource::Text { name, content } => {
                let parsed = parse_text_content(&name, &content)?;
                all_passages.extend(parsed.passages);
            }
            InputSource::Bytes {
                name,
                data,
                mime_type: _,
            } => {
                let parsed = parse_bytes_content(&name, &data)?;
                all_passages.extend(parsed.passages);
            }
        }
    }

    Ok(all_passages)
}

/// Build HTML from already parsed data
pub fn build_from_parsed(
    parsed: ParseOutput,
) -> Result<BuildOutput, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::output::HtmlOutputHandler;
    use crate::core::story::StoryFormat;

    // Parse story format
    let story_format = StoryFormat::parse(&parsed.format_info.source)?;

    // Generate HTML
    let html = HtmlOutputHandler::generate_html(
        &parsed.passages,
        &Some(parsed.story_data.clone()),
        &story_format,
        parsed.is_debug,
    )?;

    Ok(BuildOutput {
        html,
        story_data: parsed.story_data,
    })
}

/// Sort file paths using depth-first natural ordering
///
/// Returns paths sorted with deeper paths first, then natural sort within same depth.
/// This matches the order used for passage processing in build.
pub fn sort_paths(paths: Vec<String>) -> Vec<String> {
    use crate::util::sort::compare_paths;
    let mut sorted = paths;
    sorted.sort_by(|a, b| compare_paths(a, b));
    sorted
}
