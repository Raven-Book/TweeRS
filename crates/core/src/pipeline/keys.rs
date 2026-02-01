use crate::core::story::{Passage, StoryData};
use indexmap::IndexMap;
/// Type-safe keys for PipeMap
use std::marker::PhantomData;
use std::path::PathBuf;

/// A type-safe key for PipeMap that enforces compile-time type checking
pub struct TypedKey<T> {
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> TypedKey<T> {
    /// Create a new typed key with a static name
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }

    /// Get the key name
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Clone for TypedKey<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for TypedKey<T> {}

// Common key definitions
// These will be expanded as we discover more keys in the codebase

/// Source file paths
pub const SOURCES: TypedKey<Vec<PathBuf>> = TypedKey::new("sources");

/// Base64 encoding flag
pub const BASE64: TypedKey<bool> = TypedKey::new("base64");

/// Output file path
pub const OUTPUT_PATH: TypedKey<PathBuf> = TypedKey::new("output_path");

/// Collected files
pub const FILES: TypedKey<Vec<PathBuf>> = TypedKey::new("files");

/// Modified files
pub const MODIFIED_FILES: TypedKey<Vec<PathBuf>> = TypedKey::new("modified_files");

/// HTML content
pub const HTML_CONTENT: TypedKey<String> = TypedKey::new("html_content");

/// Is rebuild flag
pub const IS_REBUILD: TypedKey<bool> = TypedKey::new("is_rebuild");

/// Assets directories
pub const ASSETS_DIRS: TypedKey<Vec<PathBuf>> = TypedKey::new("assets_dirs");

/// HTML output path
pub const HTML_OUTPUT_PATH: TypedKey<PathBuf> = TypedKey::new("html_output_path");

/// Pack output path
pub const PACK_OUTPUT_PATH: TypedKey<PathBuf> = TypedKey::new("pack_output_path");

/// Fast compression flag
pub const FAST_COMPRESSION: TypedKey<bool> = TypedKey::new("fast_compression");

/// Build context (using Any type since BuildContext is in core-full)
pub const CONTEXT: TypedKey<crate::commands::BuildContext> = TypedKey::new("context");

/// All passages map
pub const ALL_PASSAGES: TypedKey<IndexMap<String, Passage>> = TypedKey::new("all_passages");

/// Story data
pub const STORY_DATA: TypedKey<Option<StoryData>> = TypedKey::new("story_data");

/// Parsed file data: (path, passages, story_data) per file
pub type ParsedDataEntry = (PathBuf, IndexMap<String, Passage>, Option<StoryData>);
/// Parsed data from file parser node
pub const PARSED_DATA: TypedKey<Vec<ParsedDataEntry>> = TypedKey::new("parsed_data");

/// Asset file map: (local path, archive path) for pack
pub const ASSET_FILE_MAP: TypedKey<Vec<(PathBuf, String)>> = TypedKey::new("asset_file_map");
