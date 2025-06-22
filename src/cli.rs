use crate::core::output::HtmlOutputHandler;
use crate::core::parser::TweeParser;
use crate::core::story::{Passage, StoryData, StoryFormat};
use crate::util::file::{
    collect_files_with_base64, get_media_passage_type, get_mime_type_prefix,
    is_support_file_with_base64,
};
use base64::{Engine as _, engine::general_purpose};
use clap::{Parser, Subcommand};
use indexmap::IndexMap;
use notify::{EventKind, RecursiveMode};
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{debug, error, info, warn};

#[derive(Subcommand)]
#[command(version, about, long_about = None)]
pub enum Commands {
    /// Convert .twee/.tw files to HTML
    Build {
        /// Sources
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        /// Watch
        #[clap(short, long)]
        watch: bool,
        /// Output path
        #[clap(short, long, default_value = "index.html")]
        dist: PathBuf,
        /// Debug mode
        #[clap(short = 't', long)]
        is_debug: bool,
        /// Convert images to Base64 fragments
        #[clap(short, long)]
        base64: bool,
    },

    Zip {},
}

/// TweeRS Command
#[derive(Parser)]
#[command(about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

/// Cached file info
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub passages: IndexMap<String, Passage>,
    pub story_data: Option<StoryData>,
}

/// Cached build context to avoid reloading story format and re-parsing unchanged files
pub struct BuildContext {
    pub story_format: Option<StoryFormat>,
    pub format_name: String,
    pub format_version: String,
    /// Cache parsed file contents with modification times
    pub file_cache: IndexMap<PathBuf, FileInfo>,
    /// Debug mode flag
    pub is_debug: bool,
    /// Base64 encoding flag for media files
    pub base64: bool,
}

impl Default for BuildContext {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl BuildContext {
    pub fn new(is_debug: bool, base64: bool) -> Self {
        Self {
            story_format: None,
            format_name: String::new(),
            format_version: String::new(),
            file_cache: IndexMap::new(),
            is_debug,
            base64,
        }
    }

    /// Check if file has been modified since last cache
    pub fn is_file_modified(&self, path: &PathBuf) -> Result<bool, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;

        if let Some(cached) = self.file_cache.get(path) {
            // If base64 is enabled and this is a media file, check if it was previously
            // processed as a media file (has media-related tags)
            if self.base64 {
                if let Some(media_type) = get_media_passage_type(path) {
                    // Check if any cached passage for this file has the expected media tag
                    let has_media_passage = cached
                        .passages
                        .values()
                        .any(|p| p.tags.as_ref().is_some_and(|tags| tags == media_type));


                    if !has_media_passage {
                        debug!(
                            "Media file {:?} not previously processed as media, forcing reprocess",
                            path
                        );
                        return Ok(true);
                    }
                }
            }

            Ok(cached.modified != modified)
        } else {
            Ok(true)
        }
    }

    /// Update file cache with new content
    pub fn update_cache(
        &mut self,
        path: PathBuf,
        passages: IndexMap<String, Passage>,
        story_data: Option<StoryData>,
    ) -> Result<(), std::io::Error> {
        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;

        let file_info = FileInfo {
            path: path.clone(),
            modified,
            passages,
            story_data,
        };

        self.file_cache.insert(path, file_info);
        Ok(())
    }

    /// Get cached passages and story data from all files
    pub fn get_all_cached_data(&self) -> (IndexMap<String, Passage>, Option<StoryData>) {
        let mut all_passages = IndexMap::new();
        let mut story_data = None;

        for file_info in self.file_cache.values() {
            for (name, passage) in &file_info.passages {
                all_passages.insert(name.clone(), passage.clone());
            }

            if story_data.is_none() && file_info.story_data.is_some() {
                story_data = file_info.story_data.clone();
            }
        }

        (all_passages, story_data)
    }
}

pub async fn build_command(
    sources: Vec<PathBuf>,
    dist: PathBuf,
    watch: bool,
    is_debug: bool,
    base64: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting build command");
    debug!("Sources: {:?}", sources);
    debug!("Output: {:?}", dist);
    debug!("Watch mode: {}", watch);
    debug!("Base64 mode: {}", base64);

    let mut context = BuildContext::new(is_debug, base64);

    build_once(&sources, &dist, &mut context, false).await?;

    if watch {
        info!("Entering watch mode...");
        watch_and_rebuild(sources, dist, context).await?;
    }

    Ok(())
}

/// Perform a single build operation
async fn build_once(
    sources: &[PathBuf],
    dist: &PathBuf,
    context: &mut BuildContext,
    is_rebuild: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting build process...");
    debug!("Base64 mode enabled: {}", context.base64);

    let files = collect_files_with_base64(sources, context.base64).await?;
    debug!("Found {} support files", files.len());

    // Add debug info about found files
    if context.base64 {
        for file in &files {
            if let Some(media_type) = get_media_passage_type(file) {
                debug!("Found media file: {:?} (type: {})", file, media_type);
            }
        }
    }

    if files.is_empty() {
        return Err("No support files found in the specified sources".into());
    }

    let mut all_passages = IndexMap::new();
    let mut story_data = None;
    let mut modified_files = Vec::new();

    // First pass: process modified files and update cache
    for file_path in &files {
        debug!("Checking if file is modified: {:?}", file_path);
        if context.is_file_modified(file_path)? {
            if is_rebuild {
                debug!("Parsing modified file: {:?}", file_path);
            } else {
                info!("Parsing file: {:?}", file_path);
            }
            modified_files.push(file_path.clone());

            let (passages, file_story_data) = if let Some(extension) = file_path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                debug!("File extension: {}", ext_str);
                match ext_str.as_str() {
                    "js" => {
                        let content = tokio::fs::read_to_string(file_path).await?;
                        let passage_name = file_path.to_string_lossy().to_string();

                        debug!("Creating JS passage with name: {}", passage_name);
                        let mut passages = IndexMap::new();
                        let passage = Passage {
                            name: passage_name.clone(),
                            tags: Some("script".to_string()),
                            position: None,
                            size: None,
                            content: content.clone(),
                        };
                        passages.insert(passage_name.clone(), passage);
                        debug!(
                            "Created JS passage: {} with {} characters",
                            passage_name,
                            content.len()
                        );
                        (passages, None)
                    }
                    "css" => {
                        let content = tokio::fs::read_to_string(file_path).await?;
                        let passage_name = file_path.to_string_lossy().to_string();

                        debug!("Creating CSS passage with name: {}", passage_name);
                        let mut passages = IndexMap::new();
                        let passage = Passage {
                            name: passage_name.clone(),
                            tags: Some("stylesheet".to_string()),
                            position: None,
                            size: None,
                            content: content.clone(),
                        };
                        passages.insert(passage_name.clone(), passage);
                        debug!(
                            "Created CSS passage: {} with {} characters",
                            passage_name,
                            content.len()
                        );
                        (passages, None)
                    }
                    _ => {
                        // Check if it's a media file and base64 is enabled
                        debug!(
                            "Checking if file is media file, base64 enabled: {}",
                            context.base64
                        );
                        if context.base64 {
                            if let Some(media_type) = get_media_passage_type(file_path) {
                                debug!("Processing media file: {:?} as {}", file_path, media_type);

                                let binary_content = tokio::fs::read(file_path).await?;
                                let base64_content =
                                    general_purpose::STANDARD.encode(&binary_content);
                                let mime_prefix = get_mime_type_prefix(file_path);
                                let full_content = format!("{}{}", mime_prefix, base64_content);
                                let passage_name = file_path.to_string_lossy().to_string();

                                debug!(
                                    "Creating media passage with name: {} (type: {})",
                                    passage_name, media_type
                                );
                                debug!("MIME prefix: {}", mime_prefix);
                                let mut passages = IndexMap::new();
                                debug!(
                                    "Created media passage: {} with {} characters of data URL content",
                                    passage_name,
                                    full_content.len()
                                );
                                let passage = Passage {
                                    name: passage_name.clone(),
                                    tags: Some(media_type.to_string()),
                                    position: None,
                                    size: None,
                                    content: full_content,
                                };
                                passages.insert(passage_name.clone(), passage);
                                (passages, None)
                            } else {
                                debug!("File is not a media file, parsing as Twee file");
                                let content = tokio::fs::read_to_string(file_path).await?;
                                debug!("Parsing as Twee file");
                                TweeParser::parse(&content).map_err(|e| {
                                    format!("Failed to parse {}: {}", file_path.display(), e)
                                })?
                            }
                        } else {
                            debug!("Base64 not enabled, parsing as Twee file");
                            let content = tokio::fs::read_to_string(file_path).await?;
                            debug!("Parsing as Twee file");
                            TweeParser::parse(&content).map_err(|e| {
                                format!("Failed to parse {}: {}", file_path.display(), e)
                            })?
                        }
                    }
                }
            } else {
                let content = tokio::fs::read_to_string(file_path).await?;
                debug!("No extension, parsing as Twee file");
                TweeParser::parse(&content)
                    .map_err(|e| format!("Failed to parse {}: {}", file_path.display(), e))?
            };

            context.update_cache(file_path.clone(), passages.clone(), file_story_data.clone())?;

            if story_data.is_none() && file_story_data.is_some() {
                story_data = file_story_data;
            }
        } else {
            debug!("File not modified, skipping: {:?}", file_path);
        }
    }

    // Second pass: build all_passages in correct file order
    for file_path in &files {
        if let Some(file_info) = context.file_cache.get(file_path) {
            for (name, passage) in &file_info.passages {
                debug!(
                    "Adding passage to all_passages in order: {} from file {:?} with tags: {:?}",
                    name, file_path, passage.tags
                );
                if all_passages.contains_key(name) {
                    warn!(
                        "Duplicate passage name '{}' found in file {:?}. Overwriting existing passage.",
                        name, file_path
                    );
                }
                all_passages.insert(name.clone(), passage.clone());
            }

            if story_data.is_none() && file_info.story_data.is_some() {
                story_data = file_info.story_data.clone();
            }
        }
    }

    // Extract StoryTitle from all passages and set it on StoryData
    if let Some(ref mut data) = story_data {
        if let Some(title_passage) = all_passages.get("StoryTitle") {
            data.name = Some(title_passage.content.clone());
            debug!("Set story name from StoryTitle passage: {:?}", data.name);
        }

        // Validate StoryData after all files have been processed
        data.validate()
            .map_err(|e| std::format!("StoryData validation failed: {}", e))?;
    }

    if all_passages.is_empty() {
        return Err("No passages found in any files".into());
    }

    debug!(
        "Total passages before HTML generation: {}",
        all_passages.len()
    );
    let script_count = all_passages
        .values()
        .filter(|p| p.tags.as_ref().is_some_and(|tags| tags.contains("script")))
        .count();
    let stylesheet_count = all_passages
        .values()
        .filter(|p| {
            p.tags
                .as_ref()
                .is_some_and(|tags| tags.contains("stylesheet"))
        })
        .count();
    let media_count = all_passages
        .values()
        .filter(|p| {
            p.tags.as_ref().is_some_and(|tags| {
                tags.starts_with("Twine.image")
                    || tags.starts_with("Twine.audio")
                    || tags.starts_with("Twine.video")
                    || tags.starts_with("Twine.vtt")
            })
        })
        .count();
    debug!(
        "Script passages: {}, Stylesheet passages: {}, Media passages: {}",
        script_count, stylesheet_count, media_count
    );

    if context.base64 && media_count > 0 {
        info!(
            "Successfully processed {} media files with base64 encoding",
            media_count
        );
        for passage in all_passages.values() {
            if let Some(ref tags) = passage.tags {
                if tags.starts_with("Twine.") {
                    info!("Media passage: {} ({})", passage.name, tags);
                }
            }
        }
    }

    let html = if modified_files.is_empty() {
        debug!("Using incremental update (no files modified)");
        HtmlOutputHandler::update_html(&all_passages, &story_data, context).await?
    } else {
        debug!("Files modified, generating HTML with potential format caching");

        if let Some(ref data) = story_data {
            let format_changed =
                context.format_name != data.format || context.format_version != data.format_version;

            if context.story_format.is_none() || format_changed {
                debug!(
                    "Loading and caching story format: {} v{}",
                    data.format, data.format_version
                );
                context.format_name = data.format.clone();
                context.format_version = data.format_version.clone();
                let story_format =
                    StoryFormat::find_format(&data.format, &data.format_version).await?;
                context.story_format = Some(story_format);
            } else {
                debug!(
                    "Using cached story format: {} v{}",
                    context.format_name, context.format_version
                );
            }
        }

        HtmlOutputHandler::update_html(&all_passages, &story_data, context).await?
    };

    if let Some(parent) = dist.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(dist, html).await?;
    if is_rebuild {
        debug!(
            "Rebuild completed successfully. Output written to: {:?}",
            dist
        );
    } else {
        info!(
            "Build completed successfully. Output written to: {:?}",
            dist
        );
    }

    Ok(())
}

/// Watch for file changes and rebuild
async fn watch_and_rebuild(
    sources: Vec<PathBuf>,
    dist: PathBuf,
    mut context: BuildContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use notify::{Config, RecommendedWatcher, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                error!("Failed to send watch event: {}", e);
            }
        },
        Config::default(),
    )?;

    for source in &sources {
        if source.is_dir() {
            watcher.watch(source, RecursiveMode::Recursive)?;
            debug!("Watching directory: {:?}", source);
        } else if source.is_file() {
            if let Some(parent) = source.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
                debug!("Watching file parent directory: {:?}", parent);
            }
        }
    }

    debug!("File watcher initialized. Waiting for changes...");

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    let has_relevant_changes = event
                        .paths
                        .iter()
                        .any(|path| is_support_file_with_base64(path, context.base64));

                    if has_relevant_changes {
                        info!("Detected changes in source files: {:?}", event.paths);

                        tokio::time::sleep(Duration::from_millis(100)).await;

                        match build_once(&sources, &dist, &mut context, true).await {
                            Ok(()) => debug!("Rebuild completed successfully"),
                            Err(e) => error!("Rebuild failed: {}", e),
                        }
                    }
                }
                _ => {}
            },
            Ok(Err(e)) => {
                warn!("Watch error: {}", e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                error!("Watch channel disconnected");
                break;
            }
        }
    }

    Ok(())
}
