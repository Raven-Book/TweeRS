use super::r#trait::FileParser;
/// Excel file parser
use async_trait::async_trait;
use indexmap::IndexMap;
use std::path::Path;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::Result;
use tweers_core::excel::parser::ExcelParser;

pub struct ExcelFileParser;

#[async_trait]
impl FileParser for ExcelFileParser {
    fn can_parse(&self, extension: &str) -> bool {
        matches!(extension, "xlsx" | "xls")
    }

    async fn parse(
        &self,
        file_path: &Path,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        let bytes = tokio::fs::read(file_path).await?;
        let result = ExcelParser::parse_from_bytes(bytes)?;

        let passage_name = file_path.to_string_lossy().to_string();
        let mut passages = IndexMap::new();

        // Create JS passage
        if !result.javascript.is_empty() {
            let js_passage = Passage {
                name: format!("{}_script", passage_name),
                tags: Some("script".to_string()),
                position: None,
                size: None,
                content: result.javascript,
            };
            passages.insert(js_passage.name.clone(), js_passage);
        }

        // Create HTML passage
        if !result.html.is_empty() {
            let html_passage = Passage {
                name: passage_name.clone(),
                tags: None,
                position: None,
                size: None,
                content: result.html,
            };
            passages.insert(html_passage.name.clone(), html_passage);
        }

        Ok((passages, None))
    }
}
