use super::r#trait::FileParser;
/// Text file parser (JS/CSS)
use async_trait::async_trait;
use indexmap::IndexMap;
use std::path::Path;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::Result;

pub struct TextFileParser {
    extension: String,
    tag: String,
}

impl TextFileParser {
    pub fn new(extension: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            extension: extension.into(),
            tag: tag.into(),
        }
    }
}

#[async_trait]
impl FileParser for TextFileParser {
    fn can_parse(&self, extension: &str) -> bool {
        extension == self.extension
    }

    async fn parse(
        &self,
        file_path: &Path,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let passage_name = file_path.to_string_lossy().to_string();

        let mut passages = IndexMap::new();
        let passage = Passage {
            name: passage_name.clone(),
            tags: Some(self.tag.clone()),
            position: None,
            size: None,
            content,
            source_file: Some(file_path.to_string_lossy().to_string()),
            source_line: Some(1),
        };
        passages.insert(passage_name, passage);
        Ok((passages, None))
    }
}
