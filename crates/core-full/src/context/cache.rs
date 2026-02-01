/// File cache implementation
use indexmap::IndexMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::debug;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::Result;
use tweers_core::util::file::get_media_passage_type;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub passages: IndexMap<String, Passage>,
    pub story_data: Option<StoryData>,
}

#[derive(Clone, Debug)]
pub struct FileCache {
    cache: IndexMap<PathBuf, FileInfo>,
    base64: bool,
}

impl FileCache {
    pub fn new(base64: bool) -> Self {
        Self {
            cache: IndexMap::new(),
            base64,
        }
    }

    /// Check if file has been modified since last cache
    pub fn is_modified(&self, path: &PathBuf) -> Result<bool> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;

        if let Some(cached) = self.cache.get(path) {
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
    pub fn update(
        &mut self,
        path: PathBuf,
        passages: IndexMap<String, Passage>,
        story_data: Option<StoryData>,
    ) -> Result<()> {
        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;

        let file_info = FileInfo {
            path: path.clone(),
            modified,
            passages,
            story_data,
        };

        self.cache.insert(path, file_info);
        Ok(())
    }

    /// Get cached passages and story data from all files
    pub fn get_all_data(&self) -> (IndexMap<String, Passage>, Option<StoryData>) {
        let mut all_passages = IndexMap::new();
        let mut story_data = None;

        for file_info in self.cache.values() {
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
