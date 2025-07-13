use crate::pipeline::{PipeMap, core::PipeNode};
use async_trait::async_trait;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use zip::CompressionMethod;
use zip::write::{FileOptions, ZipWriter};

/// Node for compressing assets using ffmpeg
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Starting asset compression...");

        let assets_dirs = data
            .get::<Vec<PathBuf>>("assets_dirs")
            .ok_or("Missing assets_dirs in pipeline data")?;

        let fast_compression = data.get::<bool>("fast_compression").unwrap_or(&false);

        if assets_dirs.is_empty() {
            debug!("No assets directories specified, skipping compression");
            data.insert("asset_file_map", Vec::<(PathBuf, String)>::new());
            return Ok(data);
        }

        // Check ffmpeg availability
        let ffmpeg_available = match check_ffmpeg_availability() {
            Ok(available) => {
                if !available {
                    warn!("   FFmpeg not found. Media compression will be skipped.");
                    warn!(
                        "   Install FFmpeg from https://ffmpeg.org/ to enable media compression."
                    );
                    warn!("   Files will be included without compression.");
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

            debug!("Processing assets directory: {:?}", assets_dir);

            // Collect all files in the assets directory with relative paths
            let files = collect_asset_files_with_relative_paths(assets_dir)?;

            for (file_path, relative_path) in files {
                debug!(
                    "Processing asset file: {:?} -> {}",
                    file_path.display(),
                    relative_path
                );

                if is_media_file(&file_path) && ffmpeg_available {
                    match compress_media_file(&file_path, &relative_path, *fast_compression).await {
                        Ok((compressed_path, archive_path)) => {
                            asset_file_map.push((compressed_path, archive_path));
                        }
                        Err(e) => {
                            warn!("Failed to compress {}: {}", file_path.display(), e);
                            // Fall back to original file
                            asset_file_map.push((file_path, relative_path));
                        }
                    }
                } else {
                    asset_file_map.push((file_path, relative_path));
                }
            }
        }

        data.insert("asset_file_map", asset_file_map);

        info!("Asset compression completed");
        Ok(data)
    }
}

/// Node for creating archive with all assets
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

    async fn process(
        &self,
        data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Starting archive creation...");

        let output_path = data
            .get::<PathBuf>("pack_output_path")
            .ok_or("Missing pack_output_path in pipeline data")?;

        let html_output_path = data
            .get::<PathBuf>("html_output_path")
            .ok_or("Missing html_output_path in pipeline data")?;

        let default_files = Vec::new();
        let asset_file_map = data
            .get::<Vec<(PathBuf, String)>>("asset_file_map")
            .unwrap_or(&default_files);

        debug!("Creating archive at: {:?}", output_path);

        // Create the archive
        let file = std::fs::File::create(output_path)
            .map_err(|e| format!("Failed to create archive file: {e}"))?;

        let mut zip = ZipWriter::new(file);
        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Add the generated HTML file
        if html_output_path.exists() {
            let html_content = std::fs::read(html_output_path)
                .map_err(|e| format!("Failed to read HTML file: {e}"))?;

            // Try to get story title for HTML filename in archive, fallback to "index.html"
            let html_name = if let Some(context) = data.get::<crate::cli::BuildContext>("context") {
                let (all_passages, _) = context.get_all_cached_data();
                all_passages
                    .get("StoryTitle")
                    .map(|p| format!("{}.html", p.content.trim()))
                    .filter(|name| !name.trim().is_empty() && name != ".html")
                    .unwrap_or_else(|| "index.html".to_string())
            } else {
                "index.html".to_string()
            };

            zip.start_file(&html_name, options)
                .map_err(|e| format!("Failed to start HTML file in archive: {e}"))?;
            zip.write_all(&html_content)
                .map_err(|e| format!("Failed to write HTML content to archive: {e}"))?;

            debug!("Added HTML file to archive: {}", html_name);
        }

        // Add asset files with proper directory structure
        for (file_path, archive_path) in asset_file_map {
            if !file_path.exists() {
                warn!("Asset file does not exist: {:?}", file_path);
                continue;
            }

            let file_content = std::fs::read(file_path)
                .map_err(|e| format!("Failed to read asset file {}: {}", file_path.display(), e))?;

            zip.start_file(archive_path, options)
                .map_err(|e| format!("Failed to start file in archive: {e}"))?;
            zip.write_all(&file_content)
                .map_err(|e| format!("Failed to write file content to archive: {e}"))?;

            debug!("Added asset file to archive: {}", archive_path);
        }

        zip.finish()
            .map_err(|e| format!("Failed to finalize archive: {e}"))?;

        info!("Archive created successfully: {:?}", output_path);
        Ok(data)
    }
}

/// Collect all files in an assets directory recursively with relative paths
fn collect_asset_files_with_relative_paths(
    base_dir: &Path,
) -> Result<Vec<(PathBuf, String)>, String> {
    let mut files = Vec::new();

    fn visit_dir(
        dir: &Path,
        base_dir: &Path,
        files: &mut Vec<(PathBuf, String)>,
    ) -> Result<(), String> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, base_dir, files)?;
            } else if path.is_file() {
                // Calculate relative path from base_dir
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", path.display()))?;

                // Convert to forward slashes for zip archives and include base dir name
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

/// Check if a file is a media file that could benefit from compression
fn is_media_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
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

/// Compress a media file using ffmpeg
async fn compress_media_file(
    file_path: &Path,
    archive_path: &str,
    fast_compression: bool,
) -> Result<(PathBuf, String), String> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Create temporary directory for compressed file
    let temp_dir = std::env::temp_dir().join(format!("tweers_compress_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {e}"))?;

    // Compress videos
    if matches!(
        extension.as_str(),
        "mp4" | "avi" | "mov" | "mkv" | "webm" | "ogv"
    ) {
        let compressed_name = format!(
            "compressed_{}.mp4",
            file_path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("video")
        );
        let compressed_path = temp_dir.join(&compressed_name);

        // Configure compression settings based on fast_compression flag
        let (crf, preset, audio_bitrate) = if fast_compression {
            ("30", "ultrafast", "96k") // Lower quality, much faster
        } else {
            ("23", "medium", "128k") // Better quality, good balance
        };

        info!(
            "Compressing video: {} ({})",
            file_path.display(),
            if fast_compression {
                "fast mode"
            } else {
                "quality mode"
            }
        );

        // Run ffmpeg with progress output
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
                audio_bitrate,
                "-movflags",
                "+faststart",
                "-progress",
                "pipe:1",
                "-y",
                &compressed_path.to_string_lossy(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ffmpeg: {e}"))?;

        // Get video duration first
        let duration = get_video_duration(file_path)?;

        // Read progress from stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(time_ms) = time_str.parse::<u64>() {
                        let current_sec = time_ms / 1000000;
                        let progress = if duration > 0 {
                            (current_sec as f64 / duration as f64 * 100.0).min(100.0)
                        } else {
                            0.0
                        };

                        // Create progress bar
                        let bar_width = 30;
                        let filled = (progress / 100.0 * bar_width as f64) as usize;
                        let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);

                        print!(
                            "\r[{}] {:.1}% ({}:{:02}:{:02})",
                            bar,
                            progress,
                            current_sec / 3600,
                            (current_sec % 3600) / 60,
                            current_sec % 60
                        );
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                }
            }
        }

        let result = child
            .wait()
            .map_err(|e| format!("Failed to wait for ffmpeg: {e}"))?;

        println!(); // New line after progress

        if result.success() {
            return check_and_return_compressed(file_path, &compressed_path, archive_path);
        } else {
            warn!("Video compression failed for {}", file_path.display());
        }
    }
    // Compress images
    else if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tiff") {
        let compressed_name = if extension == "png" {
            format!(
                "compressed_{}.png",
                file_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("image")
            )
        } else {
            format!(
                "compressed_{}.jpg",
                file_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("image")
            )
        };
        let compressed_path = temp_dir.join(&compressed_name);

        info!("Compressing image: {}", file_path.display());

        let result = if extension == "png" {
            let compression_level = if fast_compression { "6" } else { "9" };
            // Use ffmpeg for PNG compression
            Command::new("ffmpeg")
                .args([
                    "-i",
                    &file_path.to_string_lossy(),
                    "-compression_level",
                    compression_level,
                    "-y",
                    &compressed_path.to_string_lossy(),
                ])
                .output()
        } else {
            let quality = if fast_compression { "65" } else { "80" }; // Better quality for images
            // Use ffmpeg for JPEG compression
            Command::new("ffmpeg")
                .args([
                    "-i",
                    &file_path.to_string_lossy(),
                    "-q:v",
                    quality,
                    "-y",
                    &compressed_path.to_string_lossy(),
                ])
                .output()
        };

        match result {
            Ok(output) if output.status.success() => {
                return check_and_return_compressed(file_path, &compressed_path, archive_path);
            }
            Ok(_) => {
                warn!("Image compression failed for {}", file_path.display());
            }
            Err(e) => {
                warn!("Failed to run ffmpeg for image compression: {}", e);
            }
        }
    }

    // For other files or failed compression, return original
    Ok((file_path.to_path_buf(), archive_path.to_string()))
}

/// Check compressed file size and return appropriate result
fn check_and_return_compressed(
    original_path: &Path,
    compressed_path: &Path,
    archive_path: &str,
) -> Result<(PathBuf, String), String> {
    let original_size = std::fs::metadata(original_path)
        .map_err(|e| format!("Failed to get original file size: {e}"))?
        .len();
    let compressed_size = std::fs::metadata(compressed_path)
        .map_err(|e| format!("Failed to get compressed file size: {e}"))?
        .len();

    if compressed_size < original_size {
        let reduction = (original_size - compressed_size) as f64 / original_size as f64 * 100.0;
        info!(
            "✓ Compressed {} ({} -> {} bytes, {:.1}% smaller)",
            original_path.file_name().unwrap().to_string_lossy(),
            format_bytes(original_size),
            format_bytes(compressed_size),
            reduction
        );
        Ok((compressed_path.to_path_buf(), archive_path.to_string()))
    } else {
        debug!("Compressed file is not smaller, using original");
        std::fs::remove_file(compressed_path).ok();
        Ok((original_path.to_path_buf(), archive_path.to_string()))
    }
}

/// Get video duration in seconds using ffprobe
fn get_video_duration(file_path: &Path) -> Result<u64, String> {
    use std::process::Command;

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            &file_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {e}"))?;

    if output.status.success() {
        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration_f64: f64 = duration_str
            .trim()
            .parse()
            .map_err(|_| "Failed to parse duration".to_string())?;
        Ok(duration_f64 as u64)
    } else {
        // Fallback: return 0 to disable progress percentage
        Ok(0)
    }
}

/// Check if ffmpeg and ffprobe are available
fn check_ffmpeg_availability() -> Result<bool, String> {
    use std::process::Command;

    // Check ffmpeg
    let ffmpeg_check = Command::new("ffmpeg").args(["-version"]).output();

    if ffmpeg_check.is_err() {
        return Ok(false);
    }

    // Check ffprobe
    let ffprobe_check = Command::new("ffprobe").args(["-version"]).output();

    if ffprobe_check.is_err() {
        return Ok(false);
    }

    Ok(true)
}

/// Format bytes in human readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
