/// File parser trait for strategy pattern
use async_trait::async_trait;
use indexmap::IndexMap;
use std::path::Path;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::Result;

#[async_trait]
pub trait FileParser: Send + Sync {
    /// Check if this parser can handle the given file extension
    fn can_parse(&self, extension: &str) -> bool;

    /// Parse the file and return passages and optional story data
    async fn parse(
        &self,
        file_path: &Path,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)>;
}
