// Basic pipeline nodes for I/O operations

use crate::commands::BuildContext;
use crate::commands::CONTEXT;
use crate::io::collect_files_with_base64;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use indexmap::IndexMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};
use tweers_core::core::file::{
    detect_file_type, parse_bytes_content, parse_text_content, FileType,
};
use tweers_core::core::output::HtmlOutputHandler;
use tweers_core::core::story::{Passage, StoryData};
use tweers_core::error::{Result, TweersError};
use tweers_core::pipeline::{PipeMap, PipeNode};
use tweers_core::util::file::{get_media_passage_type, get_mime_type_prefix};

/// File collector node - collect source files to be processed
pub struct FileCollectorNode;

#[async_trait]
impl PipeNode for FileCollectorNode {
    fn name(&self) -> String {
        "FileCollector".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec!["sources".to_string(), "base64?".to_string()]
    }

    fn output(&self) -> Vec<String> {
        vec!["files".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let is_rebuild = *data
            .get_typed(tweers_core::pipeline::IS_REBUILD)
            .unwrap_or(&false);
        let sources = data
            .get_typed(tweers_core::pipeline::SOURCES)
            .ok_or_else(|| TweersError::missing_input("sources"))?;

        let base64 = data
            .get_typed(tweers_core::pipeline::BASE64)
            .unwrap_or(&false);

        debug!(
            "Collecting files from {} sources, base64: {}",
            sources.len(),
            base64
        );

        let files = collect_files_with_base64(sources, *base64, is_rebuild).await?;

        if files.is_empty() {
            return Err(TweersError::other(
                "No support files found in the specified sources",
            ));
        }

        debug!("Found {} support files", files.len());
        data.insert_typed(tweers_core::pipeline::FILES, files);
        Ok(data)
    }
}

/// File change detector node - check which files need to be reprocessed
pub struct FileChangeDetectorNode;

#[async_trait]
impl PipeNode for FileChangeDetectorNode {
    fn name(&self) -> String {
        "FileChangeDetector".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec!["files".to_string(), "context".to_string()]
    }

    fn output(&self) -> Vec<String> {
        vec!["modified_files".to_string(), "context".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let files = data
            .get_typed(tweers_core::pipeline::FILES)
            .ok_or_else(|| TweersError::missing_input("files"))?;

        let context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?;

        let mut modified_files = Vec::new();

        for file_path in files {
            if context.is_file_modified(file_path)? {
                debug!("File modified: {:?}", file_path);
                modified_files.push(file_path.clone());
            }
        }

        debug!("Found {} modified files", modified_files.len());
        data.insert_typed(tweers_core::pipeline::MODIFIED_FILES, modified_files);
        Ok(data)
    }
}

/// File parser node - parse various types of files
pub struct FileParserNode;

#[async_trait]
impl PipeNode for FileParserNode {
    fn name(&self) -> String {
        "FileParser".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec!["modified_files".to_string(), "context".to_string()]
    }

    fn output(&self) -> Vec<String> {
        vec!["parsed_data".to_string(), "context".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let is_rebuild = *data
            .get_typed(tweers_core::pipeline::IS_REBUILD)
            .unwrap_or(&false);
        let modified_files = data
            .get_typed(tweers_core::pipeline::MODIFIED_FILES)
            .ok_or_else(|| TweersError::missing_input("modified_files"))?;

        let mut context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?
            .clone();

        let mut parsed_files = Vec::new();

        for file_path in modified_files {
            if !is_rebuild {
                info!("Parsing file: {:?}", file_path);
            } else {
                debug!("Parsing file: {:?}", file_path);
            }

            let (passages, story_data) = self.parse_file(file_path, &context).await?;

            context.update_cache(file_path.clone(), passages.clone(), story_data.clone())?;

            parsed_files.push((file_path.clone(), passages, story_data));
        }

        data.insert_typed(tweers_core::pipeline::PARSED_DATA, parsed_files);
        data.insert_typed(CONTEXT, context);
        Ok(data)
    }
}

impl FileParserNode {
    async fn parse_file(
        &self,
        file_path: &PathBuf,
        context: &BuildContext,
    ) -> tweers_core::error::Result<(IndexMap<String, Passage>, Option<StoryData>)> {
        let file_type = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| detect_file_type(&format!("file.{}", ext)))
            .unwrap_or(FileType::Unknown);

        match file_type {
            FileType::JavaScript | FileType::Css | FileType::Twee | FileType::Unknown => {
                // Handle media files specially when base64 is enabled
                if context.base64 {
                    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                        if let Some(media_type) = get_media_passage_type(ext) {
                            return self.parse_media_file(file_path, ext, media_type).await;
                        }
                    }
                }

                // Read as text and use shared logic
                let content = tokio::fs::read_to_string(file_path).await?;
                let filename = file_path.to_string_lossy();
                let parsed = parse_text_content(&filename, &content).map_err(|e| {
                    TweersError::parse(format!("Failed to parse {}: {}", file_path.display(), e))
                })?;
                Ok((parsed.passages, parsed.story_data))
            }
            FileType::Excel => {
                // Read as bytes and use shared logic
                let bytes = tokio::fs::read(file_path).await?;
                let filename = file_path.to_string_lossy();
                let parsed = parse_bytes_content(&filename, &bytes).map_err(|e| {
                    TweersError::parse(format!("Failed to parse {}: {}", file_path.display(), e))
                })?;
                Ok((parsed.passages, parsed.story_data))
            }
            FileType::Media => {
                if context.base64 {
                    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                        if let Some(media_type) = get_media_passage_type(ext) {
                            return self.parse_media_file(file_path, ext, media_type).await;
                        }
                    }
                }
                // Non-base64 media files - return empty
                Ok((IndexMap::new(), None))
            }
        }
    }

    async fn parse_media_file(
        &self,
        file_path: &PathBuf,
        ext: &str,
        media_type: &str,
    ) -> tweers_core::error::Result<(IndexMap<String, Passage>, Option<StoryData>)> {
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
            source_file: Some(file_path.to_string_lossy().to_string()),
            source_line: Some(1),
        };
        passages.insert(passage_name, passage);
        Ok((passages, None))
    }
}

fn normalize_media_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Data aggregator node - aggregate all parsed data
pub struct DataAggregatorNode;

#[async_trait]
impl PipeNode for DataAggregatorNode {
    fn name(&self) -> String {
        "DataAggregator".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec![
            "files".to_string(),
            "context".to_string(),
            "parsed_data?".to_string(),
        ]
    }

    fn output(&self) -> Vec<String> {
        vec![
            "all_passages".to_string(),
            "story_data".to_string(),
            "context".to_string(),
        ]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let files = data
            .get_typed(tweers_core::pipeline::FILES)
            .ok_or_else(|| TweersError::missing_input("files"))?;

        let context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?;

        let mut all_passages = IndexMap::new();
        let mut story_data = None;

        for file_path in files {
            if let Some(file_info) = context.file_cache.get(file_path) {
                for (name, passage) in &file_info.passages {
                    if all_passages.contains_key(name) {
                        warn!("Duplicate passage name: {}", name);
                    }
                    all_passages.insert(name.clone(), passage.clone());
                }

                if story_data.is_none() && file_info.story_data.is_some() {
                    story_data = file_info.story_data.clone();
                }
            }
        }

        if let Some(mut data_obj) = story_data.clone() {
            if let Some(title_passage) = all_passages.get("StoryTitle") {
                data_obj.name = Some(title_passage.content.clone());
                debug!(
                    "Set story name from StoryTitle passage: {:?}",
                    data_obj.name
                );
            }

            if let Some(ref start_passage) = context.start_passage {
                if all_passages.get(start_passage).is_none() {
                    return Err(format!(
                        "Start passage '{start_passage}' does not exist in the loaded passages"
                    )
                    .into());
                }

                data_obj.start = Some(start_passage.clone());
            }

            data_obj
                .validate()
                .map_err(|e| format!("StoryData validation failed: {e}"))?;
            story_data = Some(data_obj);
        }

        if all_passages.is_empty() {
            return Err(TweersError::other("No passages found in any files"));
        }

        debug!("Total passages aggregated: {}", all_passages.len());

        data.insert_typed(tweers_core::pipeline::ALL_PASSAGES, all_passages);
        data.insert_typed(tweers_core::pipeline::STORY_DATA, story_data);
        Ok(data)
    }
}

/// HTML generator node - generate final HTML output
pub struct HtmlGeneratorNode;

#[async_trait]
impl PipeNode for HtmlGeneratorNode {
    fn name(&self) -> String {
        "HtmlGenerator".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec![
            "all_passages".to_string(),
            "story_data".to_string(),
            "context".to_string(),
        ]
    }

    fn output(&self) -> Vec<String> {
        vec!["html_content".to_string(), "context".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let is_rebuild = data
            .get_typed(tweers_core::pipeline::IS_REBUILD)
            .unwrap_or(&false);

        let all_passages = data
            .get_typed(tweers_core::pipeline::ALL_PASSAGES)
            .ok_or_else(|| TweersError::missing_input("all_passages"))?;

        let story_data = data
            .get_typed(tweers_core::pipeline::STORY_DATA)
            .ok_or_else(|| TweersError::missing_input("story_data"))?;

        debug!("HtmlGenerator received {} passages", all_passages.len());

        let context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?
            .clone();

        debug!("Generating HTML for {} passages", all_passages.len());

        let html_content = HtmlOutputHandler::generate_html(
            all_passages,
            story_data,
            context
                .story_format
                .as_ref()
                .ok_or_else(|| TweersError::invalid_config("Story format not loaded"))?,
            context.is_debug,
        )?;

        if !is_rebuild {
            info!("HTML generation completed");
        }

        data.insert_typed(tweers_core::pipeline::HTML_CONTENT, html_content);
        data.insert_typed(CONTEXT, context);
        Ok(data)
    }
}

/// File writer node - write HTML content to file
pub struct FileWriterNode;

#[async_trait]
impl PipeNode for FileWriterNode {
    fn name(&self) -> String {
        "FileWriter".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec!["html_content".to_string(), "output_path".to_string()]
    }

    fn output(&self) -> Vec<String> {
        vec![]
    }

    async fn process(&self, data: PipeMap) -> Result<PipeMap> {
        let is_rebuild = data
            .get_typed(tweers_core::pipeline::IS_REBUILD)
            .unwrap_or(&false);

        let html_content = data
            .get_typed(tweers_core::pipeline::HTML_CONTENT)
            .ok_or_else(|| TweersError::missing_input("html_content"))?;

        let output_path = data
            .get_typed(tweers_core::pipeline::OUTPUT_PATH)
            .ok_or_else(|| TweersError::missing_input("output_path"))?;

        tokio::fs::write(output_path, html_content).await?;

        if !is_rebuild {
            info!("Output written to: {:?}", output_path);
        } else {
            debug!("Output written to: {:?}", output_path);
        }

        Ok(data)
    }
}
