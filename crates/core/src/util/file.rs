use crate::config::constants;
use std::path::Path;

/// Check if the file is a support file
pub fn is_support_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        let ext_str = ext_str.as_str();
        return constants::TWEE_EXTENSIONS.contains(&ext_str)
            || ext_str == "js"
            || ext_str == "css"
            || ext_str == "xlsx"
            || ext_str == "xlsm"
            || ext_str == "xlsb"
            || ext_str == "xls";
    }
    false
}

/// Check if extension is a media file
pub fn is_media_extension(ext: &str) -> bool {
    constants::IMAGE_EXTENSIONS.contains(&ext)
        || constants::AUDIO_EXTENSIONS.contains(&ext)
        || constants::VIDEO_EXTENSIONS.contains(&ext)
        || constants::VTT_EXTENSIONS.contains(&ext)
}

/// Get MIME type prefix for extension
pub fn get_mime_type_prefix(ext: &str) -> Option<&'static str> {
    if constants::IMAGE_EXTENSIONS.contains(&ext) {
        Some("image")
    } else if constants::AUDIO_EXTENSIONS.contains(&ext) {
        Some("audio")
    } else if constants::VIDEO_EXTENSIONS.contains(&ext) {
        Some("video")
    } else if constants::VTT_EXTENSIONS.contains(&ext) {
        Some("text")
    } else {
        None
    }
}

/// Get media passage type from extension
pub fn get_media_passage_type(ext: &str) -> Option<&'static str> {
    if constants::IMAGE_EXTENSIONS.contains(&ext) {
        Some("image")
    } else if constants::AUDIO_EXTENSIONS.contains(&ext) {
        Some("audio")
    } else if constants::VIDEO_EXTENSIONS.contains(&ext) {
        Some("video")
    } else if constants::VTT_EXTENSIONS.contains(&ext) {
        Some("vtt")
    } else {
        None
    }
}
