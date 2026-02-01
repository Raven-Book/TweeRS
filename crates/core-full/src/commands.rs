use crate::io::is_support_file_with_base64;
use crate::pipeline::nodes::basic::{
    DataAggregatorNode, FileChangeDetectorNode, FileCollectorNode, FileParserNode, FileWriterNode,
    HtmlGeneratorNode,
};
use crate::pipeline::{PipeMap, PipeNode, Pipeline};
use indexmap::IndexMap;
use notify::{EventKind, RecursiveMode};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, error, info, warn};
use tweers_core::core::story::{Passage, StoryData, StoryFormat};
use tweers_core::pipeline::TypedKey;
use tweers_core::util::file::get_media_passage_type;

/// Cached file info
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub passages: IndexMap<String, Passage>,
    pub story_data: Option<StoryData>,
}

/// Cached build context to avoid reloading story format and re-parsing unchanged files
#[derive(Clone)]
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
    /// Assets directories for pack command
    pub assets_dirs: Vec<PathBuf>,
    /// Start passage name
    pub start_passage: Option<String>,
}

/// Type-safe key for BuildContext in pipeline (re-exported for asset/js crates)
pub const CONTEXT: TypedKey<BuildContext> = TypedKey::new("context");

impl Default for BuildContext {
    fn default() -> Self {
        Self::new(false, false, None)
    }
}

impl BuildContext {
    pub fn new(is_debug: bool, base64: bool, start_passage: Option<String>) -> Self {
        Self {
            story_format: None,
            format_name: String::new(),
            format_version: String::new(),
            file_cache: IndexMap::new(),
            is_debug,
            base64,
            assets_dirs: Vec::new(),
            start_passage,
        }
    }

    pub fn with_assets(is_debug: bool, base64: bool, assets_dirs: Vec<PathBuf>) -> Self {
        Self {
            story_format: None,
            format_name: String::new(),
            format_version: String::new(),
            file_cache: IndexMap::new(),
            is_debug,
            base64,
            assets_dirs,
            start_passage: None,
        }
    }

    /// Check if file has been modified since last cache
    pub fn is_file_modified(&self, path: &PathBuf) -> Result<bool, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;

        if let Some(cached) = self.file_cache.get(path) {
            if self.base64 {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(media_type) = get_media_passage_type(ext) {
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
    start_passage: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    build_command_with_nodes(
        sources,
        dist,
        watch,
        is_debug,
        base64,
        start_passage,
        vec![],
        vec![],
    )
    .await
}

/// Build command with external node injection support
pub async fn build_command_with_nodes(
    sources: Vec<PathBuf>,
    dist: PathBuf,
    watch: bool,
    is_debug: bool,
    base64: bool,
    start_passage: Option<String>,
    data_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
    html_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting build command");
    debug!("Sources: {:?}", sources);
    debug!("Output: {:?}", dist);

    let mut context = BuildContext::new(is_debug, base64, start_passage);

    build_once(&sources, &dist, &mut context, false, data_nodes, html_nodes).await?;

    if watch {
        info!("Entering watch mode...");
        watch_and_rebuild(sources, dist, context).await?;
    }

    Ok(())
}

/// Build using pipeline system
async fn build_once(
    sources: &[PathBuf],
    dist: &Path,
    context: &mut BuildContext,
    is_rebuild: bool,
    data_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
    html_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting pipeline-based build process...");

    // Load story format if not already loaded
    if context.story_format.is_none() {
        // First, we need to parse files to get StoryData
        let files =
            crate::io::collect_files_with_base64(sources, context.base64, is_rebuild).await?;

        for file_path in &files {
            if context.is_file_modified(file_path)? {
                // Check if it's a text file (twee/tw)
                if let Some(ext) = file_path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if matches!(ext_str, "twee" | "tw") {
                            let content = tokio::fs::read_to_string(file_path).await?;
                            if let Ok((passages, story_data)) =
                                tweers_core::core::parser::TweeParser::parse(&content)
                            {
                                context.update_cache(
                                    file_path.clone(),
                                    passages,
                                    story_data.clone(),
                                )?;

                                if let Some(data) = story_data {
                                    // Load story format based on StoryData
                                    let format_source = crate::format::find_and_load_format(
                                        &data.format,
                                        &data.format_version,
                                    )
                                    .await?;

                                    let story_format =
                                        tweers_core::core::story::StoryFormat::parse(
                                            &format_source,
                                        )?;
                                    context.story_format = Some(story_format);
                                    context.format_name = data.format.clone();
                                    context.format_version = data.format_version.clone();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if context.story_format.is_none() {
            return Err("Failed to load story format: no StoryData found in source files".into());
        }
    }

    let mut pipeline = Pipeline::new("TweersBuildPipeline")
        .with_external_inputs(vec![
            "sources".to_string(),
            "base64".to_string(),
            "context".to_string(),
            "output_path".to_string(),
            "is_rebuild".to_string(),
        ])
        .add_node(Box::new(FileCollectorNode))?
        .add_node(Box::new(FileChangeDetectorNode))?
        .add_node(Box::new(FileParserNode))?
        .add_node(Box::new(DataAggregatorNode))?;

    // Add external data processing nodes
    for node in data_nodes {
        pipeline = pipeline.add_node(node)?;
    }

    pipeline = pipeline.add_node(Box::new(HtmlGeneratorNode))?;

    // Add external HTML processing nodes
    for node in html_nodes {
        pipeline = pipeline.add_node(node)?;
    }

    pipeline = pipeline.add_node(Box::new(FileWriterNode))?;

    let mut pipe_data = PipeMap::new();
    pipe_data.insert_typed(tweers_core::pipeline::SOURCES, sources.to_vec());
    pipe_data.insert_typed(tweers_core::pipeline::BASE64, context.base64);
    pipe_data.insert_typed(CONTEXT, context.clone());
    pipe_data.insert_typed(tweers_core::pipeline::OUTPUT_PATH, dist.to_path_buf());
    pipe_data.insert_typed(tweers_core::pipeline::IS_REBUILD, is_rebuild);

    let result = pipeline.execute(pipe_data).await?;

    if let Some(updated_context) = result.get_typed(CONTEXT) {
        *context = updated_context.clone();
    }

    if is_rebuild {
        debug!("Pipeline rebuild completed successfully");
    } else {
        info!("Pipeline build completed successfully");
    }

    Ok(())
}

/// Watch for file changes and rebuild
async fn watch_and_rebuild(
    sources: Vec<PathBuf>,
    dist: PathBuf,
    mut context: BuildContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let mut pending_changes: HashSet<PathBuf> = HashSet::new();
    let mut last_event_time = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    let relevant_paths: Vec<_> = event
                        .paths
                        .iter()
                        .filter(|path| is_support_file_with_base64(path, context.base64))
                        .cloned()
                        .collect();

                    if !relevant_paths.is_empty() {
                        for path in relevant_paths {
                            pending_changes.insert(path);
                        }
                        last_event_time = std::time::Instant::now();
                    }
                }
                _ => {}
            },
            Ok(Err(e)) => {
                warn!("Watch error: {}", e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !pending_changes.is_empty()
                    && last_event_time.elapsed() >= Duration::from_millis(200)
                {
                    let changed_files: Vec<_> = pending_changes.iter().cloned().collect();
                    pending_changes.clear();

                    info!("Detected changes in source files: {:?}", changed_files);

                    match build_once(&sources, &dist, &mut context, true, vec![], vec![]).await {
                        Ok(()) => debug!("Rebuild completed successfully"),
                        Err(e) => error!("Rebuild failed: {}", e),
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                error!("Watch channel disconnected");
                break;
            }
        }
    }

    Ok(())
}

pub async fn pack_command(
    sources: Vec<PathBuf>,
    assets_dirs: Vec<PathBuf>,
    output_path: PathBuf,
    fast_compression: bool,
    is_debug: bool,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    pack_command_with_nodes(
        sources,
        assets_dirs,
        output_path,
        fast_compression,
        is_debug,
        vec![],
    )
    .await
}

/// Pack command with external node injection support
pub async fn pack_command_with_nodes(
    sources: Vec<PathBuf>,
    assets_dirs: Vec<PathBuf>,
    output_path: PathBuf,
    fast_compression: bool,
    is_debug: bool,
    pack_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting pack command");

    let mut context = BuildContext::with_assets(is_debug, true, assets_dirs.clone());

    let temp_dir = std::env::temp_dir().join(format!("tweers_pack_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    let temp_html = temp_dir.join("temp_index.html");
    build_once(&sources, &temp_html, &mut context, false, vec![], vec![]).await?;

    let (all_passages, _) = context.get_all_cached_data();
    let story_title = all_passages
        .get("StoryTitle")
        .map(|p| p.content.trim().to_string())
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "story".to_string());

    let actual_output_path = if output_path == Path::new("package.zip") {
        PathBuf::from(format!("{story_title}.zip"))
    } else {
        output_path
    };

    pack_once(
        &assets_dirs,
        &temp_html,
        &actual_output_path,
        fast_compression,
        &context,
        pack_nodes,
    )
    .await?;

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    Ok(actual_output_path)
}

/// Pack using pipeline system with external nodes
async fn pack_once(
    assets_dirs: &[PathBuf],
    html_output_path: &Path,
    archive_output_path: &Path,
    fast_compression: bool,
    context: &BuildContext,
    pack_nodes: Vec<Box<dyn PipeNode + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting pipeline-based pack process...");

    let mut pipeline = Pipeline::new("TweersPackPipeline").with_external_inputs(vec![
        "assets_dirs".to_string(),
        "html_output_path".to_string(),
        "pack_output_path".to_string(),
        "fast_compression".to_string(),
        "context".to_string(),
    ]);

    for node in pack_nodes {
        pipeline = pipeline.add_node(node)?;
    }

    let mut pipe_data = PipeMap::new();
    pipe_data.insert_typed(tweers_core::pipeline::ASSETS_DIRS, assets_dirs.to_vec());
    pipe_data.insert_typed(
        tweers_core::pipeline::HTML_OUTPUT_PATH,
        html_output_path.to_path_buf(),
    );
    pipe_data.insert_typed(
        tweers_core::pipeline::PACK_OUTPUT_PATH,
        archive_output_path.to_path_buf(),
    );
    pipe_data.insert_typed(tweers_core::pipeline::FAST_COMPRESSION, fast_compression);
    pipe_data.insert_typed(CONTEXT, context.clone());

    let _result = pipeline.execute(pipe_data).await?;

    info!("Pack pipeline completed successfully");
    Ok(())
}
