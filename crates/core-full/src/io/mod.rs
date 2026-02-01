// I/O operations module
// Handles file reading, URL downloading, and file collection

pub mod collector;
pub mod filters;

pub use collector::FileCollector;
pub use filters::{FileFilter, SupportFileFilter};

use crate::api::InputSource;
use std::path::{Path, PathBuf};
use tweers_core::util::file::{get_media_passage_type, is_support_file};

/// Loaded sources ready for core
pub struct LoadedSources {
    pub texts: Vec<(String, String)>,
    pub bytes: Vec<(String, Vec<u8>, Option<String>)>,
}

/// Load all sources and convert to core format
pub async fn load_sources(
    sources: Vec<InputSource>,
    base64: bool,
) -> Result<LoadedSources, Box<dyn std::error::Error + Send + Sync>> {
    let mut texts = Vec::new();
    let mut bytes = Vec::new();

    // Process each source
    for source in sources {
        match source {
            InputSource::FilePath(path) => {
                load_file(&path, base64, &mut texts, &mut bytes).await?;
            }
            InputSource::Url(url) => {
                load_url(&url, &mut texts).await?;
            }
            InputSource::Text { name, content } => {
                texts.push((name, content));
            }
            InputSource::Bytes {
                name,
                data,
                mime_type,
            } => {
                bytes.push((name, data, mime_type));
            }
        }
    }

    Ok(LoadedSources { texts, bytes })
}

/// Load a file from disk
async fn load_file(
    path: &Path,
    _base64: bool,
    texts: &mut Vec<(String, String)>,
    bytes: &mut Vec<(String, Vec<u8>, Option<String>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Check file extension
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Read file content
    let content = tokio::fs::read(path).await?;

    // Determine if it's a text file or binary
    if ext == "twee" || ext == "tw" || ext == "js" || ext == "css" {
        let text = String::from_utf8(content)?;
        texts.push((name, text));
    } else {
        // Binary file (images, etc.)
        let mime_type = get_mime_type(ext);
        bytes.push((name, content, mime_type));
    }

    Ok(())
}

/// Download from URL
async fn load_url(
    url: &str,
    texts: &mut Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    let name = url.split('/').last().unwrap_or("downloaded").to_string();
    texts.push((name, content));
    Ok(())
}

/// Get MIME type from file extension
fn get_mime_type(ext: &str) -> Option<String> {
    let mime = match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => return None,
    };
    Some(mime.to_string())
}

/// Collect files from sources with base64 support
/// Refactored to use the new FileCollector with filter pattern
pub async fn collect_files_with_base64(
    sources: &[PathBuf],
    base64: bool,
    _is_rebuild: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let filter = SupportFileFilter::new(base64);
    let collector = FileCollector::new(filter);
    Ok(collector.collect_async(sources).await?)
}

/// Check if file is supported with base64 consideration
pub fn is_support_file_with_base64(path: &Path, base64: bool) -> bool {
    if is_support_file(path) {
        return true;
    }

    if base64 {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if get_media_passage_type(ext).is_some() {
                return true;
            }
        }
    }

    false
}
