use crate::engine::ScriptEngine;
use crate::error::ScriptError;
use crate::manager::ScriptManager;
use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, error, info, warn};
use tweers_core::error::{Result, TweersError};
use tweers_core::pipeline::{PipeMap, PipeNode};
use tweers_core_full::commands::CONTEXT;

pub struct DataProcessorNode {
    script_manager: ScriptManager,
}

impl DataProcessorNode {
    pub fn new(script_manager: ScriptManager) -> std::result::Result<Self, ScriptError> {
        Ok(Self { script_manager })
    }
}

#[async_trait]
impl PipeNode for DataProcessorNode {
    fn name(&self) -> String {
        "DataProcessor".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec![
            "all_passages".to_string(),
            "story_data".to_string(),
            "context".to_string(),
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
        let passages = data
            .get_typed(tweers_core::pipeline::ALL_PASSAGES)
            .ok_or_else(|| TweersError::missing_input("all_passages"))?;
        let story_data = data
            .get_typed(tweers_core::pipeline::STORY_DATA)
            .ok_or_else(|| TweersError::missing_input("story_data"))?;
        let _context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?;

        if !self.script_manager.has_data_scripts() {
            debug!("No data processing scripts found, skipping");
            return Ok(data);
        }

        let mut current_passages = passages.clone();
        let current_story_data = story_data.clone();

        for script_path in self.script_manager.get_data_scripts() {
            debug!("Executing data script: {:?}", script_path);

            let input_data = json!(current_passages);

            match tokio::fs::read_to_string(script_path).await {
                Ok(script_content) => {
                    let mut engine = ScriptEngine::new()?;
                    let format_info = if let Some(ref sd) = current_story_data {
                        json!({ "name": sd.format, "version": sd.format_version })
                    } else {
                        json!({ "name": "", "version": "" })
                    };

                    match engine.execute_data_processor(
                        &input_data.to_string(),
                        &format_info.to_string(),
                        &script_content,
                    ) {
                        Ok(result) => {
                            if let Ok(new_passages) = serde_json::from_str(&result) {
                                current_passages = new_passages;
                                debug!("Data script executed: {:?}", script_path);
                            } else {
                                warn!("Invalid data structure from: {:?}", script_path);
                            }
                        }
                        Err(e) => warn!("Script failed: {:?} - {}", script_path, e),
                    }
                }
                Err(e) => error!("Failed to read script: {:?} - {}", script_path, e),
            }
        }

        info!(
            "{} data scripts executed",
            self.script_manager.get_data_scripts().len()
        );
        data.insert_typed(tweers_core::pipeline::ALL_PASSAGES, current_passages);
        data.insert_typed(tweers_core::pipeline::STORY_DATA, current_story_data);
        Ok(data)
    }
}

pub struct HtmlProcessorNode {
    script_manager: ScriptManager,
}

impl HtmlProcessorNode {
    pub fn new(script_manager: ScriptManager) -> std::result::Result<Self, ScriptError> {
        Ok(Self { script_manager })
    }
}

#[async_trait]
impl PipeNode for HtmlProcessorNode {
    fn name(&self) -> String {
        "HtmlProcessor".to_string()
    }

    fn input(&self) -> Vec<String> {
        vec![
            "html_content".to_string(),
            "all_passages".to_string(),
            "context".to_string(),
        ]
    }

    fn output(&self) -> Vec<String> {
        vec!["html_content".to_string()]
    }

    async fn process(&self, mut data: PipeMap) -> Result<PipeMap> {
        let html_content = data
            .get_typed(tweers_core::pipeline::HTML_CONTENT)
            .ok_or_else(|| TweersError::missing_input("html_content"))?;
        let passages = data
            .get_typed(tweers_core::pipeline::ALL_PASSAGES)
            .ok_or_else(|| TweersError::missing_input("all_passages"))?;
        let context = data
            .get_typed(CONTEXT)
            .ok_or_else(|| TweersError::missing_input("context"))?;

        if !self.script_manager.has_html_scripts() {
            debug!("No HTML processing scripts found, skipping");
            return Ok(data);
        }

        let mut current_html = html_content.clone();

        for script_path in self.script_manager.get_html_scripts() {
            debug!("Executing HTML script: {:?}", script_path);

            match tokio::fs::read_to_string(script_path).await {
                Ok(script_content) => {
                    let mut engine = ScriptEngine::new()?;
                    let passages_json = serde_json::to_string(&passages)?;
                    let format_info = json!({
                        "name": context.format_name,
                        "version": context.format_version
                    });

                    match engine.execute_html_processor(
                        &current_html,
                        &passages_json,
                        &format_info.to_string(),
                        &script_content,
                    ) {
                        Ok(result) => {
                            current_html = result;
                            debug!("HTML script executed: {:?}", script_path);
                        }
                        Err(e) => warn!("HTML script failed: {:?} - {}", script_path, e),
                    }
                }
                Err(e) => error!("Failed to read HTML script: {:?} - {}", script_path, e),
            }
        }

        info!(
            "{} HTML scripts executed",
            self.script_manager.get_html_scripts().len()
        );
        data.insert_typed(tweers_core::pipeline::HTML_CONTENT, current_html);
        Ok(data)
    }
}
