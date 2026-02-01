use super::r#trait::FileParser;
/// Media file parser (images, audio, video)
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use indexmap::IndexMap;
use std::path::Path;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::Result;
use tweers_core::util::file::{get_media_passage_type, get_mime_type_prefix};

pub struct MediaFileParser {
    base64: bool,
}

impl MediaFileParser {
    pub fn new(base64: bool) -> Self {
        Self { base64 }
    }
}

fn normalize_media_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[async_trait]
impl FileParser for MediaFileParser {
    fn can_parse(&self, extension: &str) -> bool {
        get_media_passage_type(extension).is_some()
    }

    async fn parse(
        &self,
        file_path: &Path,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        if !self.base64 {
            return Ok((IndexMap::new(), None));
        }

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let media_type = get_media_passage_type(ext).unwrap();
        let binary_content = tokio::fs::read(file_path).await?;
        let base64_content = general_purpose::STANDARD.encode(&binary_content);
        let mime_prefix = get_mime_type_prefix(ext).unwrap_or("");
        let full_content = format!("{mime_prefix}{base64_content}");
        let passage_name = normalize_media_path(&file_path.to_string_lossy());

        let mut passages = IndexMap::new();
        let passage = Passage {
            name: passage_name.clone(),
            tags: Some(media_type.to_string()),
            position: None,
            size: None,
            content: full_content,
        };
        passages.insert(passage_name, passage);
        Ok((passages, None))
    }
}
