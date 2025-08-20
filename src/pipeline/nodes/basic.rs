use crate::cli::BuildContext;
use crate::core::output::HtmlOutputHandler;
use crate::core::parser::TweeParser;
use crate::core::story::{Passage, StoryData};
use crate::excel::parser::ExcelParser;
use crate::pipeline::{PipeMap, PipeNode};
use crate::util::file::{collect_files_with_base64, get_media_passage_type, get_mime_type_prefix};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use indexmap::IndexMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let is_rebuild = *data.get::<bool>("is_rebuild").unwrap_or(&false);
        let sources = data
            .get::<Vec<PathBuf>>("sources")
            .ok_or("Missing sources input")?;

        let base64 = data.get::<bool>("base64").unwrap_or(&false);

        debug!(
            "Collecting files from {} sources, base64: {}",
            sources.len(),
            base64
        );

        let files = collect_files_with_base64(sources, *base64, is_rebuild).await?;

        if files.is_empty() {
            return Err("No support files found in the specified sources".into());
        }

        debug!("Found {} support files", files.len());
        data.insert("files", files);
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let files = data
            .get::<Vec<PathBuf>>("files")
            .ok_or("Missing files input")?;

        let context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?;

        let mut modified_files = Vec::new();

        for file_path in files {
            if context.is_file_modified(file_path)? {
                debug!("File modified: {:?}", file_path);
                modified_files.push(file_path.clone());
            }
        }

        debug!("Found {} modified files", modified_files.len());
        data.insert("modified_files", modified_files);
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let is_rebuild = *data.get::<bool>("is_rebuild").unwrap_or(&false);
        let modified_files = data
            .get::<Vec<PathBuf>>("modified_files")
            .ok_or("Missing modified_files input")?;

        let mut context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?
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

        data.insert("parsed_data", parsed_files);
        data.insert("context", context);
        Ok(data)
    }
}

impl FileParserNode {
    async fn parse_file(
        &self,
        file_path: &PathBuf,
        context: &BuildContext,
    ) -> Result<
        (IndexMap<String, Passage>, Option<StoryData>),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if let Some(extension) = file_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "js" => {
                    let content = tokio::fs::read_to_string(file_path).await?;
                    let passage_name = file_path.to_string_lossy().to_string();

                    let mut passages = IndexMap::new();
                    let passage = Passage {
                        name: passage_name.clone(),
                        tags: Some("script".to_string()),
                        position: None,
                        size: None,
                        content,
                    };
                    passages.insert(passage_name, passage);
                    Ok((passages, None))
                }
                "css" => {
                    let content = tokio::fs::read_to_string(file_path).await?;
                    let passage_name = file_path.to_string_lossy().to_string();

                    let mut passages = IndexMap::new();
                    let passage = Passage {
                        name: passage_name.clone(),
                        tags: Some("stylesheet".to_string()),
                        position: None,
                        size: None,
                        content,
                    };
                    passages.insert(passage_name, passage);
                    Ok((passages, None))
                }
                "xlsx" | "xlsm" | "xlsb" | "xls" => {
                    let mut passages = IndexMap::new();

                    let passage_name = file_path.to_string_lossy().to_string();

                    match ExcelParser::parse_file(file_path).await {
                        Ok(js_code) => {
                            if !js_code.is_empty() {
                                let passage = Passage {
                                    name: passage_name.clone(),
                                    tags: Some("init script".to_string()),
                                    position: None,
                                    size: None,
                                    content: js_code,
                                };

                                passages.insert(passage_name, passage);
                                debug!(
                                    "Created JavaScript passage from Excel file: {:?}",
                                    file_path
                                );
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse Excel file {:?}: {}", file_path, e);
                        }
                    }

                    Ok((passages, None))
                }
                _ => {
                    if context.base64
                        && let Some(media_type) = get_media_passage_type(file_path)
                    {
                        let binary_content = tokio::fs::read(file_path).await?;
                        let base64_content = general_purpose::STANDARD.encode(&binary_content);
                        let mime_prefix = get_mime_type_prefix(file_path);
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
                        return Ok((passages, None));
                    }
                    let content = tokio::fs::read_to_string(file_path).await?;
                    TweeParser::parse(&content).map_err(|e| {
                        format!("Failed to parse {}: {}", file_path.display(), e).into()
                    })
                }
            }
        } else {
            let content = tokio::fs::read_to_string(file_path).await?;
            TweeParser::parse(&content)
                .map_err(|e| format!("Failed to parse {}: {}", file_path.display(), e).into())
        }
    }
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let files = data
            .get::<Vec<PathBuf>>("files")
            .ok_or("Missing files input")?;

        let context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?;

        let mut all_passages = IndexMap::new();
        let mut story_data = None;

        for file_path in files {
            if let Some(file_info) = context.file_cache.get(file_path) {
                for (name, passage) in &file_info.passages {
                    if all_passages.contains_key(name) {
                        warn!(
                            "Duplicate passage name '{}' found in file {:?}. Overwriting existing passage.",
                            name, file_path
                        );
                    }
                    all_passages.insert(name.clone(), passage.clone());
                }

                if story_data.is_none() && file_info.story_data.is_some() {
                    story_data = file_info.story_data.clone();
                }
            }
        }

        if let Some(ref mut data) = story_data {
            if let Some(title_passage) = all_passages.get("StoryTitle") {
                data.name = Some(title_passage.content.clone());
                debug!("Set story name from StoryTitle passage: {:?}", data.name);
            }

            if let Some(ref start_passage) = context.start_passage {
                if all_passages.get(start_passage).is_none() {
                    return Err(format!(
                        "Start passage '{start_passage}' does not exist in the loaded passages"
                    )
                    .into());
                }

                data.start = Some(start_passage.clone());
            }

            data.validate()
                .map_err(|e| format!("StoryData validation failed: {e}"))?;
        }

        if all_passages.is_empty() {
            return Err("No passages found in any files".into());
        }

        debug!("Total passages aggregated: {}", all_passages.len());

        data.insert("all_passages", all_passages);
        data.insert("story_data", story_data);
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let is_rebuild = data.get::<bool>("is_rebuild").unwrap_or(&false);

        let all_passages = data
            .get::<IndexMap<String, Passage>>("all_passages")
            .ok_or("Missing all_passages input")?;

        let story_data = data
            .get::<Option<StoryData>>("story_data")
            .ok_or("Missing story_data input")?;

        debug!("HtmlGenerator received {} passages", all_passages.len());
        if all_passages.contains_key("StoryTitle") {
            debug!("StoryTitle passage exists in HtmlGenerator input");
        } else {
            debug!("StoryTitle passage is MISSING in HtmlGenerator input!");
        }

        let mut context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?
            .clone();

        debug!("Generating HTML for {} passages", all_passages.len());

        let html = HtmlOutputHandler::update_html(all_passages, story_data, &mut context)
            .await
            .map_err(|e| format!("HTML generation failed: {e}"))?;

        if !is_rebuild {
            info!("HTML generation completed successfully");
        }

        data.insert("html_content", html);
        data.insert("context", context);
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
        vec!["success".to_string()]
    }

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let is_rebuild = *data.get::<bool>("is_rebuild").unwrap_or(&false);
        let html_content = data
            .get::<String>("html_content")
            .ok_or("Missing html_content input")?;

        let output_path = data
            .get::<PathBuf>("output_path")
            .ok_or("Missing output_path input")?;

        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(output_path, html_content).await?;

        if !is_rebuild {
            info!(
                "Build completed successfully. Output written to: {:?}",
                output_path
            );
        }

        data.insert("success", true);
        Ok(data)
    }
}

/// Normalize media file path for cross-platform consistency
fn normalize_media_path(path_str: &str) -> String {
    let mut normalized = path_str.replace('\\', "/");

    if normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    normalized
}
