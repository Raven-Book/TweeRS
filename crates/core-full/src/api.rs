// API module for core-full with I/O operations
// This will be implemented in later steps

use std::path::PathBuf;

/// Input source for core-full - includes files and URLs
#[derive(Clone, Debug)]
pub enum InputSource {
    /// File path on the local filesystem
    FilePath(PathBuf),
    /// Raw text content
    Text { name: String, content: String },
    /// HTTP/HTTPS URL to fetch
    Url(String),
    /// Raw bytes
    Bytes {
        name: String,
        data: Vec<u8>,
        mime_type: Option<String>,
    },
}

/// Build configuration for core-full
#[derive(Clone, Debug)]
pub struct BuildConfig {
    sources: Vec<InputSource>,
    output_path: PathBuf,
    format_name: Option<String>,
    format_version: Option<String>,
    format_source: Option<String>,
    is_debug: bool,
    base64: bool,
    start_passage: Option<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            output_path: PathBuf::from("index.html"),
            format_name: None,
            format_version: None,
            format_source: None,
            is_debug: false,
            base64: false,
            start_passage: None,
        }
    }
}

impl BuildConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_files<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.sources
            .extend(paths.into_iter().map(|p| InputSource::FilePath(p.into())));
        self
    }

    pub fn add_urls<I, S>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sources
            .extend(urls.into_iter().map(|url| InputSource::Url(url.into())));
        self
    }

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

    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = path.into();
        self
    }

    pub fn format(mut self, name: String, version: String) -> Self {
        self.format_name = Some(name);
        self.format_version = Some(version);
        self
    }

    pub fn format_source(mut self, source: String) -> Self {
        self.format_source = Some(source);
        self
    }

    pub fn debug(mut self, is_debug: bool) -> Self {
        self.is_debug = is_debug;
        self
    }

    pub fn base64(mut self, base64: bool) -> Self {
        self.base64 = base64;
        self
    }

    pub fn start_passage(mut self, start_passage: Option<String>) -> Self {
        self.start_passage = start_passage;
        self
    }
}

/// Build output
#[derive(Clone, Debug)]
pub struct BuildOutput {
    pub output_path: PathBuf,
}

/// Build with I/O operations
pub async fn run_build(
    config: BuildConfig,
) -> Result<BuildOutput, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Load or discover story format
    let format_name = config.format_name.clone().ok_or("Format name required")?;
    let format_version = config
        .format_version
        .clone()
        .ok_or("Format version required")?;

    let format_source = if let Some(source) = config.format_source {
        source
    } else {
        crate::format::find_and_load_format(&format_name, &format_version).await?
    };

    // 2. Load all sources (files, URLs, etc.)
    let loaded_sources = crate::io::load_sources(config.sources, config.base64).await?;

    // 3. Convert to core API format
    let format_info = tweers_core::api::StoryFormatInfo {
        name: format_name,
        version: format_version,
        source: format_source,
    };

    let core_config = tweers_core::api::BuildConfig::new(format_info)
        .add_texts(loaded_sources.texts)
        .add_bytes(loaded_sources.bytes)
        .debug(config.is_debug)
        .start_passage(config.start_passage);

    // 4. Call pure core build (synchronous)
    let core_output = tweers_core::api::build(core_config)?;

    // 5. Write output file
    tokio::fs::write(&config.output_path, core_output.html).await?;

    Ok(BuildOutput {
        output_path: config.output_path,
    })
}

/// Pack configuration for core-full
#[derive(Clone, Debug, Default)]
pub struct PackConfig {
    // To be implemented
}

/// Pack output
#[derive(Clone, Debug)]
pub struct PackOutput {
    pub output_path: PathBuf,
}
