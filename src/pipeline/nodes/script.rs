use crate::cli::BuildContext;
use crate::core::story::{Passage, StoryData};
use crate::js::{ScriptEngine, ScriptError, ScriptManager};
use crate::pipeline::{PipeMap, PipeNode};
use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::json;
use tracing::{debug, error, info, warn};

pub struct DataProcessorNode {
    script_manager: ScriptManager,
}

impl DataProcessorNode {
    pub fn new(script_manager: ScriptManager) -> Result<Self, ScriptError> {
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let passages = data
            .get::<IndexMap<String, Passage>>("all_passages")
            .ok_or("Missing all_passages input")?;
        let story_data = data
            .get::<Option<StoryData>>("story_data")
            .ok_or("Missing story_data input")?;
        let _context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?;

        // If there are no data processing scripts, return original data directly
        if !self.script_manager.has_data_scripts() {
            debug!("No data processing scripts found, skipping data processing");
            return Ok(data);
        }

        let mut current_passages = passages.clone();
        let current_story_data = story_data.clone();

        // Execute all data processing scripts in sequence
        for script_path in self.script_manager.get_data_scripts() {
            debug!("Executing data script: {:?}", script_path);

            // Debug: Check if story data has name field
            if let Some(ref story_data) = current_story_data {
                debug!("Story data name before script: {:?}", story_data.name);
            }

            // Prepare script input data (just passages)
            let input_data = json!(current_passages);

            // Read and execute script
            match tokio::fs::read_to_string(script_path).await {
                Ok(script_content) => {
                    // Since V8 is not Send, we need to execute on the current thread
                    let mut engine = ScriptEngine::new()?;
                    let format_info = if let Some(ref story_data) = current_story_data {
                        json!({
                            "name": story_data.format,
                            "version": story_data.format_version
                        })
                    } else {
                        json!({
                            "name": "",
                            "version": ""
                        })
                    };
                    match engine.execute_data_processor(
                        &input_data.to_string(),
                        &format_info.to_string(),
                        &script_content,
                    ) {
                        Ok(result) => {
                            // debug!("Script result: {}", result);
                            // Deserialize modified data
                            match serde_json::from_str::<serde_json::Value>(&result) {
                                Ok(processed_data) => {
                                    // The result should be the new passages data
                                    if let Ok(new_passages) = serde_json::from_value(processed_data)
                                    {
                                        current_passages = new_passages;

                                        // Debug: Check if story data has name field after script
                                        if let Some(ref story_data) = current_story_data {
                                            debug!(
                                                "Story data name after script: {:?}",
                                                story_data.name
                                            );
                                        }

                                        debug!(
                                            "Data script executed successfully: {:?}",
                                            script_path
                                        );
                                    } else {
                                        warn!(
                                            "Data script returned invalid data structure, skipping: {:?}",
                                            script_path
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to parse script result, skipping: {:?} - {}",
                                        script_path, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Data script execution failed, skipping: {:?} - {}",
                                script_path, e
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read data script: {:?} - {}", script_path, e);
                }
            }
        }

        info!(
            "Data processing completed, {} scripts executed",
            self.script_manager.get_data_scripts().len()
        );

        data.insert("all_passages", current_passages);
        data.insert("story_data", current_story_data);
        Ok(data)
    }
}

/// HTML processing injection node
pub struct HtmlProcessorNode {
    script_manager: ScriptManager,
}

impl HtmlProcessorNode {
    pub fn new(script_manager: ScriptManager) -> Result<Self, ScriptError> {
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

    async fn process(
        &self,
        mut data: PipeMap,
    ) -> Result<PipeMap, Box<dyn std::error::Error + Send + Sync>> {
        let html_content = data
            .get::<String>("html_content")
            .ok_or("Missing html_content input")?;
        let passages = data
            .get::<IndexMap<String, Passage>>("all_passages")
            .ok_or("Missing all_passages input")?;
        let context = data
            .get::<BuildContext>("context")
            .ok_or("Missing context input")?;

        // If there are no HTML processing scripts, return original HTML directly
        if !self.script_manager.has_html_scripts() {
            debug!("No HTML processing scripts found, skipping HTML processing");
            return Ok(data);
        }

        let mut current_html = html_content.clone();

        // Execute all HTML processing scripts in sequence
        for script_path in self.script_manager.get_html_scripts() {
            debug!("Executing HTML script: {:?}", script_path);

            // No need for script input preparation, variables are set directly in engine

            // Read and execute script
            match tokio::fs::read_to_string(script_path).await {
                Ok(script_content) => {
                    // Since V8 is not Send, we need to execute on the current thread
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
                            debug!("HTML script executed successfully: {:?}", script_path);
                        }
                        Err(e) => {
                            warn!(
                                "HTML script execution failed, skipping: {:?} - {}",
                                script_path, e
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read HTML script: {:?} - {}", script_path, e);
                }
            }
        }

        info!(
            "HTML processing completed, {} scripts executed",
            self.script_manager.get_html_scripts().len()
        );
        data.insert("html_content", current_html);

        Ok(data)
    }
}
