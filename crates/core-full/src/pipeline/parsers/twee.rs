use super::r#trait::FileParser;
/// Twee file parser
use async_trait::async_trait;
use indexmap::IndexMap;
use std::path::PathBuf;
use tweers_core::core::parser::TweeParser;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::{Result, TweersError};

pub struct TweeFileParser;

#[async_trait]
impl FileParser for TweeFileParser {
    fn can_parse(&self, extension: &str) -> bool {
        matches!(extension, "twee" | "tw")
    }

    async fn parse(
        &self,
        file_path: &PathBuf,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        let content = tokio::fs::read_to_string(file_path).await?;
        TweeParser::parse(&content).map_err(|e| {
            TweersError::parse(format!("Failed to parse {}: {}", file_path.display(), e))
        })
    }
}
