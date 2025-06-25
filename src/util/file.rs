use crate::config::constants;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs as async_fs;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{debug, info};

/// Check if the file is a support file
pub fn is_support_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        let ext_str = ext_str.as_str();
        return constants::TWEE_EXTENSIONS.contains(&ext_str)
            || ext_str == "js"
            || ext_str == "css";
    }
    false
}

/// Check if the file is a support file (including media files when base64 is enabled)
pub fn is_support_file_with_base64(path: &Path, base64_enabled: bool) -> bool {
    debug!(
        "Checking file: {:?}, base64_enabled: {}",
        path, base64_enabled
    );

    if is_support_file(path) {
        debug!("File is a regular support file: {:?}", path);
        return true;
    }

    if base64_enabled {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            let ext_str = ext_str.as_str();
            debug!("Checking extension: {} for file: {:?}", ext_str, path);

            let is_image = constants::IMAGE_EXTENSIONS.contains(&ext_str);
            let is_audio = constants::AUDIO_EXTENSIONS.contains(&ext_str);
            let is_video = constants::VIDEO_EXTENSIONS.contains(&ext_str);
            let is_vtt = constants::VTT_EXTENSIONS.contains(&ext_str);

            debug!(
                "Extension check results - image: {}, audio: {}, video: {}, vtt: {}",
                is_image, is_audio, is_video, is_vtt
            );

            if is_image || is_audio || is_video || is_vtt {
                debug!("File is a media file: {:?}", path);
                return true;
            }
        } else {
            debug!("File has no extension: {:?}", path);
        }
    }

    debug!("File is not a support file: {:?}", path);
    false
}

/// Determine the Twine passage type for a media file
pub fn get_media_passage_type(path: &Path) -> Option<&'static str> {
    debug!("Getting media passage type for: {:?}", path);

    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        let ext_str = ext_str.as_str();

        debug!("Extension: {}", ext_str);

        if constants::IMAGE_EXTENSIONS.contains(&ext_str) {
            debug!("Returning Twine.image for: {:?}", path);
            return Some("Twine.image");
        } else if constants::AUDIO_EXTENSIONS.contains(&ext_str) {
            debug!("Returning Twine.audio for: {:?}", path);
            return Some("Twine.audio");
        } else if constants::VIDEO_EXTENSIONS.contains(&ext_str) {
            debug!("Returning Twine.video for: {:?}", path);
            return Some("Twine.video");
        } else if constants::VTT_EXTENSIONS.contains(&ext_str) {
            debug!("Returning Twine.vtt for: {:?}", path);
            return Some("Twine.vtt");
        }
    }

    debug!("No media type found for: {:?}", path);
    None
}

/// Get MIME type prefix for base64 data URL
pub fn get_mime_type_prefix(path: &Path) -> String {
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        let ext_str = ext_str.as_str();

        match ext_str {
            // Image types
            "png" => "data:image/png;base64,",
            "jpg" | "jpeg" => "data:image/jpeg;base64,",
            "gif" => "data:image/gif;base64,",
            "bmp" => "data:image/bmp;base64,",
            "svg" => "data:image/svg+xml;base64,",
            "webp" => "data:image/webp;base64,",

            // Audio types
            "mp3" => "data:audio/mpeg;base64,",
            "wav" => "data:audio/wav;base64,",
            "ogg" => "data:audio/ogg;base64,",
            "aac" => "data:audio/aac;base64,",
            "m4a" => "data:audio/mp4;base64,",
            "flac" => "data:audio/flac;base64,",

            // Video types
            "mp4" => "data:video/mp4;base64,",
            "webm" => "data:video/webm;base64,",
            "ogv" => "data:video/ogg;base64,",
            "avi" => "data:video/x-msvideo;base64,",
            "mov" => "data:video/quicktime;base64,",
            "wmv" => "data:video/x-ms-wmv;base64,",

            // VTT types
            "vtt" => "data:text/vtt;base64,",

            // Default fallback
            _ => "data:application/octet-stream;base64,",
        }
        .to_string()
    } else {
        "data:application/octet-stream;base64,".to_string()
    }
}

/// Recursively collect all files from given paths
pub async fn collect_files(
    paths: &[PathBuf],
    is_rebuild: bool,
) -> Result<Vec<PathBuf>, std::io::Error> {
    collect_files_with_base64(paths, false, is_rebuild).await
}

/// Recursively collect all files from given paths with base64 support
pub async fn collect_files_with_base64(
    paths: &[PathBuf],
    base64_enabled: bool,
    is_rebuild: bool,
) -> Result<Vec<PathBuf>, std::io::Error> {
    if !is_rebuild {
        info!("Starting file collection, path count: {}", paths.len());
    } else {
        debug!("Starting file collection, path count: {}", paths.len());
    }
    for path in paths {
        debug!("Processing path: {:?}", path);
    }

    let mut set = JoinSet::new();
    let files = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    for path in paths {
        let path = path.clone();
        let files_clone = Arc::clone(&files);
        set.spawn(async move {
            let metadata = async_fs::metadata(&path).await?;
            if metadata.is_dir() {
                debug!("Processing directory: {:?}", path);
                process_path_with_base64(path, files_clone, base64_enabled).await?
            } else if is_support_file_with_base64(&path, base64_enabled) {
                debug!("Found support file: {:?}", path);
                files_clone.lock().await.push(path);
            }
            Ok::<_, std::io::Error>(())
        });
    }

    set.join_all().await;

    files.lock().await.sort();
    let result = Arc::try_unwrap(files).unwrap().into_inner();
    if !is_rebuild {
        tracing::info!(
            "File collection completed, found {} support files",
            result.len()
        );
    }
    Ok(result)
}

fn process_path_with_base64(
    path: PathBuf,
    files: Arc<Mutex<Vec<PathBuf>>>,
    base64_enabled: bool,
) -> std::pin::Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> {
    Box::pin(async move {
        let metadata = tokio::fs::metadata(&path).await?;

        if metadata.is_file() {
            if is_support_file_with_base64(&path, base64_enabled) {
                debug!("Adding support file: {:?}", path);
                files.lock().await.push(path);
            }
        } else if metadata.is_dir() {
            debug!("Recursively processing directory: {:?}", path);
            process_directory_with_base64(path, files, base64_enabled).await?;
        }

        Ok(())
    })
}

fn process_directory_with_base64(
    dir: PathBuf,
    files: Arc<Mutex<Vec<PathBuf>>>,
    base64_enabled: bool,
) -> std::pin::Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> {
    Box::pin(async move {
        if let Some(dir_name) = dir.file_name() {
            if dir_name.to_string_lossy().starts_with('.') {
                debug!("Skipping hidden directory: {:?}", dir);
                return Ok(());
            }
        }

        debug!("Starting to process directory: {:?}", dir);
        let mut read_dir = tokio::fs::read_dir(&dir).await?;
        let mut child_tasks = JoinSet::new();
        let mut entry_count = 0;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            entry_count += 1;
            let files_clone = Arc::clone(&files);
            child_tasks.spawn(async move {
                process_path_with_base64(path, files_clone, base64_enabled).await
            });
        }

        debug!("Found {} entries in directory {:?}", entry_count, dir);

        while let Some(result) = child_tasks.join_next().await {
            result??;
        }

        debug!("Finished processing directory: {:?}", dir);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::debug;

    #[tokio::test]
    async fn test_collect_files() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = vec![
            project_root.join("test/story/Part 1"),
            project_root.join("test/story/Part 2"),
            project_root.join("test/story/A.twee"),
        ];

        debug!("Project root directory: {:?}", project_root);
        debug!("Story directory path: {:?}", paths);

        match collect_files(&paths, false).await {
            Ok(paths) => {
                debug!("Count of found files: {:?}", paths.len());
                debug!("Found files: {:?}", paths);
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
