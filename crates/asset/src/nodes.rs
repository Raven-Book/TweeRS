use async_trait::async_trait;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use tweers_core::error::{Result, TweersError};
use tweers_core::pipeline::{PipeMap, PipeNode};
use tweers_core_full::commands::CONTEXT;
use zip::CompressionMethod;
use zip::write::{FileOptions, ZipWriter};

pub struct AssetCompressorNode;

#[async_trait]
impl PipeNode for AssetCompressorNode {
    fn name(&self) -> String {
        "AssetCompressorNode".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec!["assets_dirs".to_string(), "fast_compression?".to_string()]
    }

    fn output(&self) -> Vec<String> {
        vec!["asset_file_map".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        debug!("Starting asset compression...");

        let assets_dirs = data
            .get_typed(tweers_core::pipeline::ASSETS_DIRS)
            .ok_or("Missing assets_dirs in pipeline data")?;

        let fast_compression = data
            .get_typed(tweers_core::pipeline::FAST_COMPRESSION)
            .unwrap_or(&false);

        if assets_dirs.is_empty() {
            debug!("No assets directories specified, skipping compression");
            data.insert_typed(
                tweers_core::pipeline::ASSET_FILE_MAP,
                Vec::<(PathBuf, String)>::new(),
            );
            return Ok(data);
        }

        let ffmpeg_available = match check_ffmpeg_availability() {
            Ok(available) => {
                if !available {
                    warn!("FFmpeg not found. Media compression will be skipped.");
                }
                available
            }
            Err(e) => {
                warn!("Failed to check FFmpeg availability: {}", e);
                false
            }
        };

        let mut asset_file_map = Vec::new();

        for assets_dir in assets_dirs {
            if !assets_dir.exists() {
                warn!("Assets directory does not exist: {:?}", assets_dir);
                continue;
            }

            let files = collect_asset_files_with_relative_paths(assets_dir)?;

            for (file_path, relative_path) in files {
                if is_media_file(&file_path) && ffmpeg_available {
                    match compress_media_file(&file_path, &relative_path, *fast_compression).await {
                        Ok((compressed_path, archive_path)) => {
                            asset_file_map.push((compressed_path, archive_path));
                        }
                        Err(e) => {
                            warn!("Failed to compress {}: {}", file_path.display(), e);
                            asset_file_map.push((file_path, relative_path));
                        }
                    }
                } else {
                    asset_file_map.push((file_path, relative_path));
                }
            }
        }

        data.insert_typed(tweers_core::pipeline::ASSET_FILE_MAP, asset_file_map);
        info!("Asset compression completed");
        Ok(data)
    }
}

pub struct ArchiveCreatorNode;

#[async_trait]
impl PipeNode for ArchiveCreatorNode {
    fn name(&self) -> String {
        "ArchiveCreatorNode".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec![
            "pack_output_path".to_string(),
            "html_output_path".to_string(),
            "asset_file_map?".to_string(),
        ]
    }

    fn output(&self) -> Vec<String> {
        vec![]
    }

    async fn process(&self, data: PipeMap) -> Result<PipeMap> {
        debug!("Starting archive creation...");

        let output_path = data
            .get_typed(tweers_core::pipeline::PACK_OUTPUT_PATH)
            .ok_or("Missing pack_output_path")?;

        let html_output_path = data
            .get_typed(tweers_core::pipeline::HTML_OUTPUT_PATH)
            .ok_or("Missing html_output_path")?;

        let default_files = Vec::new();
        let asset_file_map = data
            .get_typed(tweers_core::pipeline::ASSET_FILE_MAP)
            .unwrap_or(&default_files);

        let file = std::fs::File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        if html_output_path.exists() {
            let html_content = std::fs::read(html_output_path)?;
            let html_name = get_html_name(&data);
            zip.start_file(&html_name, options)
                .map_err(|e| TweersError::other(format!("Zip error: {}", e)))?;
            zip.write_all(&html_content)?;
            debug!("Added HTML: {}", html_name);
        }

        for (file_path, archive_path) in asset_file_map {
            if !file_path.exists() {
                warn!("Asset not found: {:?}", file_path);
                continue;
            }
            let content = std::fs::read(file_path)?;
            zip.start_file(archive_path, options)
                .map_err(|e| TweersError::other(format!("Zip error: {}", e)))?;
            zip.write_all(&content)?;
            debug!("Added asset: {}", archive_path);
        }

        zip.finish()
            .map_err(|e| TweersError::other(format!("Zip error: {}", e)))?;
        info!("Archive created: {:?}", output_path);
        Ok(data)
    }
}

fn get_html_name(data: &PipeMap) -> String {
    if let Some(context) = data.get_typed(CONTEXT) {
        let (all_passages, _) = context.get_all_cached_data();
        all_passages
            .get("StoryTitle")
            .map(|p| format!("{}.html", p.content.trim()))
            .filter(|name| !name.trim().is_empty() && name != ".html")
            .unwrap_or_else(|| "index.html".to_string())
    } else {
        "index.html".to_string()
    }
}

fn collect_asset_files_with_relative_paths(
    base_dir: &Path,
) -> std::result::Result<Vec<(PathBuf, String)>, String> {
    let mut files = Vec::new();

    fn visit_dir(
        dir: &Path,
        base_dir: &Path,
        files: &mut Vec<(PathBuf, String)>,
    ) -> std::result::Result<(), String> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, base_dir, files)?;
            } else if path.is_file() {
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", path.display()))?;

                let base_name = base_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("assets");

                let relative_str = relative_path
                    .to_str()
                    .ok_or_else(|| format!("Invalid UTF-8 in path: {}", relative_path.display()))?
                    .replace('\\', "/");

                let archive_path = format!("{base_name}/{relative_str}");
                files.push((path, archive_path));
            }
        }
        Ok(())
    }

    visit_dir(base_dir, base_dir, &mut files)?;
    Ok(files)
}

fn is_media_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "mp4"
                | "avi"
                | "mov"
                | "mkv"
                | "webm"
                | "ogv"
                | "mp3"
                | "wav"
                | "ogg"
                | "aac"
                | "m4a"
                | "flac"
                | "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "bmp"
                | "tiff"
        )
    } else {
        false
    }
}

fn check_ffmpeg_availability() -> std::result::Result<bool, String> {
    use std::process::Command;
    let ffmpeg = Command::new("ffmpeg").args(["-version"]).output();
    if ffmpeg.is_err() {
        return Ok(false);
    }
    let ffprobe = Command::new("ffprobe").args(["-version"]).output();
    Ok(ffprobe.is_ok())
}

async fn compress_media_file(
    file_path: &Path,
    archive_path: &str,
    fast_compression: bool,
) -> std::result::Result<(PathBuf, String), String> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let temp_dir = std::env::temp_dir().join(format!("tweers_compress_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {e}"))?;

    if matches!(ext.as_str(), "mp4" | "avi" | "mov" | "mkv" | "webm" | "ogv") {
        return compress_video(file_path, archive_path, fast_compression, &temp_dir).await;
    } else if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tiff") {
        return compress_image(file_path, archive_path, fast_compression, &temp_dir, &ext);
    }

    Ok((file_path.to_path_buf(), archive_path.to_string()))
}

async fn compress_video(
    file_path: &Path,
    archive_path: &str,
    fast: bool,
    temp_dir: &Path,
) -> std::result::Result<(PathBuf, String), String> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let name = format!(
        "compressed_{}.mp4",
        file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("video")
    );
    let out_path = temp_dir.join(&name);

    let (crf, preset, audio) = if fast {
        ("30", "ultrafast", "96k")
    } else {
        ("23", "medium", "128k")
    };

    info!("Compressing video: {}", file_path.display());

    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
            &file_path.to_string_lossy(),
            "-c:v",
            "libx264",
            "-preset",
            preset,
            "-crf",
            crf,
            "-c:a",
            "aac",
            "-b:a",
            audio,
            "-movflags",
            "+faststart",
            "-progress",
            "pipe:1",
            "-y",
            &out_path.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {e}"))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(std::result::Result::ok) {
            if line.starts_with("out_time_ms=") {
                // Progress tracking
            }
        }
    }

    let result = child
        .wait()
        .map_err(|e| format!("ffmpeg wait failed: {e}"))?;
    println!();

    if result.success() {
        check_and_return_compressed(file_path, &out_path, archive_path)
    } else {
        Ok((file_path.to_path_buf(), archive_path.to_string()))
    }
}

fn compress_image(
    file_path: &Path,
    archive_path: &str,
    fast: bool,
    temp_dir: &Path,
    ext: &str,
) -> std::result::Result<(PathBuf, String), String> {
    use std::process::Command;

    let stem = file_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    let (out_name, args) = if ext == "png" {
        let level = if fast { "6" } else { "9" };
        (
            format!("compressed_{stem}.png"),
            vec!["-compression_level", level],
        )
    } else {
        let q = if fast { "65" } else { "80" };
        (format!("compressed_{stem}.jpg"), vec!["-q:v", q])
    };

    let out_path = temp_dir.join(&out_name);
    info!("Compressing image: {}", file_path.display());

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-i", &file_path.to_string_lossy()]);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.args(["-y", &out_path.to_string_lossy()]);

    match cmd.output() {
        Ok(output) if output.status.success() => {
            check_and_return_compressed(file_path, &out_path, archive_path)
        }
        _ => Ok((file_path.to_path_buf(), archive_path.to_string())),
    }
}

fn check_and_return_compressed(
    original: &Path,
    compressed: &Path,
    archive_path: &str,
) -> std::result::Result<(PathBuf, String), String> {
    let orig_size = std::fs::metadata(original)
        .map_err(|e| format!("Failed to get original size: {e}"))?
        .len();
    let comp_size = std::fs::metadata(compressed)
        .map_err(|e| format!("Failed to get compressed size: {e}"))?
        .len();

    if comp_size < orig_size {
        let reduction = (orig_size - comp_size) as f64 / orig_size as f64 * 100.0;
        info!(
            "Compressed {} ({} -> {} bytes, {:.1}% smaller)",
            original.file_name().unwrap().to_string_lossy(),
            format_bytes(orig_size),
            format_bytes(comp_size),
            reduction
        );
        Ok((compressed.to_path_buf(), archive_path.to_string()))
    } else {
        debug!("Compressed not smaller, using original");
        std::fs::remove_file(compressed).ok();
        Ok((original.to_path_buf(), archive_path.to_string()))
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", bytes, UNITS[idx])
    } else {
        format!("{:.1} {}", size, UNITS[idx])
    }
}
