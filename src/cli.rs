use crate::core::story::{Passage, StoryData, StoryFormat};
use crate::js::ScriptManager;
use crate::pipeline::nodes::asset::{ArchiveCreatorNode, AssetCompressorNode};
use crate::pipeline::nodes::basic::{
    DataAggregatorNode, FileChangeDetectorNode, FileCollectorNode, FileParserNode, FileWriterNode,
    HtmlGeneratorNode,
};
use crate::pipeline::nodes::script::{DataProcessorNode, HtmlProcessorNode};
use crate::pipeline::{PipeMap, Pipeline};
use crate::util::file::{get_media_passage_type, is_support_file_with_base64};
use clap::{Parser, Subcommand};
use indexmap::IndexMap;
use notify::{EventKind, RecursiveMode};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
        #[clap(short = 'o', long, default_value = "index.html")]
        output_path: PathBuf,
        /// Debug mode
        #[clap(short = 't', long)]
        is_debug: bool,
        /// Convert images to Base64 fragments
        #[clap(short, long)]
        base64: bool,
        /// Start passage name
        #[clap(short = 's', long)]
        start_passage: Option<String>,
    },

    /// Build and pack with compressed assets
    Pack {
        /// Sources
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        /// Assets directories to compress
        #[clap(short = 'a', long = "assets")]
        assets_dirs: Vec<PathBuf>,
        /// Output archive path
        #[clap(short = 'o', long, default_value = "package.zip")]
        output_path: PathBuf,
        /// Enable fast compression (lower quality, faster speed)
        #[clap(short = 'f', long)]
        fast_compression: bool,
        /// Debug mode
        #[clap(short = 't', long)]
        is_debug: bool,
    },

    /// Update TweeRS to the latest release
    Update {
        /// Force update even if already latest version
        #[clap(short = 'f', long)]
        force: bool,
    },
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
            if self.base64
                && let Some(media_type) = get_media_passage_type(path)
            {
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
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting build command");
    debug!("Sources: {:?}", sources);
    debug!("Output: {:?}", dist);
    debug!("Watch mode: {}", watch);
    debug!("Base64 mode: {}", base64);
    debug!("Start passage: {:?}", start_passage);

    let mut context = BuildContext::new(is_debug, base64, start_passage);

    build_once(&sources, &dist, &mut context, false).await?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting pipeline-based build process...");

    let script_manager = ScriptManager::default();

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

    if script_manager.has_data_scripts() {
        pipeline = pipeline.add_node(Box::new(DataProcessorNode::new(script_manager.clone())?))?;
    }

    pipeline = pipeline.add_node(Box::new(HtmlGeneratorNode))?;

    if script_manager.has_html_scripts() {
        pipeline = pipeline.add_node(Box::new(HtmlProcessorNode::new(script_manager.clone())?))?;
    }

    pipeline = pipeline.add_node(Box::new(FileWriterNode))?;

    let mut pipe_data = PipeMap::new();
    pipe_data.insert("sources", sources.to_vec());
    pipe_data.insert("base64", context.base64);
    pipe_data.insert("context", context.clone());
    pipe_data.insert("output_path", dist.to_path_buf());
    pipe_data.insert("is_rebuild", is_rebuild);

    let result = pipeline.execute(pipe_data).await?;

    if let Some(updated_context) = result.get::<BuildContext>("context") {
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
        } else if source.is_file()
            && let Some(parent) = source.parent()
        {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
            debug!("Watching file parent directory: {:?}", parent);
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

                    match build_once(&sources, &dist, &mut context, true).await {
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
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting pack command");
    debug!("Sources: {:?}", sources);
    debug!("Assets directories: {:?}", assets_dirs);
    debug!("Output archive: {:?}", output_path);

    let mut context = BuildContext::with_assets(is_debug, true, assets_dirs.clone());

    let temp_dir = std::env::temp_dir().join(format!("tweers_pack_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    let temp_html = temp_dir.join("temp_index.html");
    build_once(&sources, &temp_html, &mut context, false).await?;

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
        &sources,
        &assets_dirs,
        &temp_html,
        &actual_output_path,
        fast_compression,
        &mut context,
    )
    .await?;

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    Ok(())
}

/// Pack using pipeline system with asset compression
async fn pack_once(
    sources: &[PathBuf],
    assets_dirs: &[PathBuf],
    html_output_path: &Path,
    archive_output_path: &Path,
    fast_compression: bool,
    context: &mut BuildContext,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting pipeline-based pack process...");

    let pipeline = Pipeline::new("TweersPackPipeline")
        .with_external_inputs(vec![
            "sources".to_string(),
            "assets_dirs".to_string(),
            "html_output_path".to_string(),
            "pack_output_path".to_string(),
            "context".to_string(),
        ])
        .add_node(Box::new(AssetCompressorNode))?
        .add_node(Box::new(ArchiveCreatorNode))?;

    let mut pipe_data = PipeMap::new();
    pipe_data.insert("sources", sources.to_vec());
    pipe_data.insert("assets_dirs", assets_dirs.to_vec());
    pipe_data.insert("html_output_path", html_output_path.to_path_buf());
    pipe_data.insert("pack_output_path", archive_output_path.to_path_buf());
    pipe_data.insert("fast_compression", fast_compression);
    pipe_data.insert("context", context.clone());

    let _result = pipeline.execute(pipe_data).await?;

    info!("Pack pipeline completed successfully");
    Ok(())
}

/// GitHub Release API response structure
#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    body: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Update command: automatically download and replace tweers binary from GitHub releases
pub async fn update_command(
    repo_api_url: String,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking for TweeRS updates...");

    let client = reqwest::Client::new();

    let response = client
        .get(&repo_api_url)
        .header("User-Agent", "TweeRS-Updater")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch release info: {}", response.status()).into());
    }

    let release: GithubRelease = response.json().await?;

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = release.tag_name.trim_start_matches('v');

    info!("Current version: {}", current_version);
    info!("Latest version: {}", latest_version);

    if !force && current_version == latest_version {
        info!("Already running the latest version!");
        return Ok(());
    }

    let platform_suffix = if cfg!(target_os = "windows") {
        "windows-x86_64.zip"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-arm64.tar.gz"
        } else {
            "macos-x86_64.tar.gz"
        }
    } else if cfg!(target_os = "linux") {
        "linux-x86_64.tar.gz"
    } else {
        return Err("Unsupported platform for auto-update".into());
    };

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(platform_suffix))
        .ok_or("No suitable release asset found for current platform")?;

    info!("Downloading {} ({} bytes)", asset.name, asset.size);

    let download_response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "TweeRS-Updater")
        .send()
        .await?;

    if !download_response.status().is_success() {
        return Err(format!("Failed to download asset: {}", download_response.status()).into());
    }

    let archive_data = download_response.bytes().await?;
    let current_exe = std::env::current_exe()?;

    let temp_dir = std::env::temp_dir().join(format!("tweers_update_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    if asset.name.ends_with(".zip") {
        extract_zip(&archive_data, &temp_dir)?;
    } else if asset.name.ends_with(".tar.gz") {
        extract_tar_gz(&archive_data, &temp_dir)?;
    } else {
        return Err("Unsupported archive format".into());
    }

    let new_exe_name = if cfg!(target_os = "windows") {
        "tweers.exe"
    } else {
        "tweers"
    };

    let extracted_exe = find_executable(&temp_dir, new_exe_name)?;

    let backup_path = current_exe.with_extension("old");
    if backup_path.exists() {
        std::fs::remove_file(&backup_path)?;
    }
    std::fs::rename(&current_exe, &backup_path)?;

    std::fs::copy(&extracted_exe, &current_exe)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&current_exe, perms)?;
    }

    std::fs::remove_dir_all(&temp_dir)?;

    info!("Successfully updated to version {}", latest_version);
    info!("Old version backed up as: {}", backup_path.display());
    info!("Restart required to use the new version.");

    Ok(())
}

/// Extract ZIP archive to target directory
fn extract_zip(data: &[u8], target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = target_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(outpath)?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Extract TAR.GZ archive to target directory
#[cfg(unix)]
fn extract_tar_gz(data: &[u8], target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let cursor = Cursor::new(data);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    archive.unpack(target_dir)?;

    Ok(())
}

/// Extract TAR.GZ archive to target directory (Windows stub)
#[cfg(windows)]
fn extract_tar_gz(_data: &[u8], _target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Err("TAR.GZ extraction not supported on Windows - this should not be called".into())
}

/// Find executable file in directory recursively
fn find_executable(dir: &Path, exe_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_name().is_some_and(|n| n == exe_name) {
            return Ok(path);
        } else if path.is_dir()
            && let Ok(found) = find_executable(&path, exe_name)
        {
            return Ok(found);
        }
    }

    Err(format!("Executable {exe_name} not found in extracted files").into())
}
