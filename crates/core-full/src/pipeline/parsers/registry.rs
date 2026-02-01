use super::excel::ExcelFileParser;
use super::media::MediaFileParser;
/// Parser registry for managing file parsers
use super::r#trait::FileParser;
use super::text::TextFileParser;
use super::twee::TweeFileParser;
use indexmap::IndexMap;
use std::path::Path;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::{Result, TweersError};

pub struct FileParserRegistry {
    parsers: Vec<Box<dyn FileParser>>,
}

impl FileParserRegistry {
    pub fn new(base64: bool) -> Self {
        let parsers: Vec<Box<dyn FileParser>> = vec![
            Box::new(TextFileParser::new("js", "script")),
            Box::new(TextFileParser::new("css", "stylesheet")),
            Box::new(TweeFileParser),
            Box::new(ExcelFileParser),
            Box::new(MediaFileParser::new(base64)),
        ];

        Self { parsers }
    }

    pub fn add_parser(&mut self, parser: Box<dyn FileParser>) {
        self.parsers.push(parser);
    }

    pub async fn parse(
        &self,
        file_path: &Path,
    ) -> Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| TweersError::parse("File has no extension"))?
            .to_lowercase();

        for parser in &self.parsers {
            if parser.can_parse(&extension) {
                return parser.parse(file_path).await;
            }
        }

        Err(TweersError::parse(format!(
            "No parser found for extension: {}",
            extension
        )))
    }
}
