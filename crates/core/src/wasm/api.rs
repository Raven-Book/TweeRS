// WASM API bindings - JavaScript-callable functions

use super::types::{JsBuildConfig, JsBuildOutput, JsParseOutput, JsStoryFormatInfo};
use wasm_bindgen::prelude::*;

/// Build a Twee story from the given configuration
///
/// # Arguments
/// * `config_js` - JavaScript object containing build configuration
///
/// # Returns
/// * `JsBuildOutput` - Contains the generated HTML
///
/// # Errors
/// Returns a JsValue error if the build fails
#[wasm_bindgen]
pub fn build(config_js: JsValue) -> Result<JsBuildOutput, JsValue> {
    // Convert JS value to Rust type
    let js_config: JsBuildConfig = serde_wasm_bindgen::from_value(config_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

    // Convert to internal API type
    let config: crate::api::BuildConfig = js_config.into();

    // Call the core build function
    let output = crate::api::build(config)
        .map_err(|e| JsValue::from_str(&format!("Build failed: {}", e)))?;

    // Convert output to JS-friendly type
    Ok(JsBuildOutput::new(output.html))
}

/// Parse Twee sources without building HTML
///
/// # Arguments
/// * `config_js` - JavaScript object containing build configuration
///
/// # Returns
/// * `JsValue` - Contains passages and story data as JSON
///
/// # Errors
/// Returns a JsValue error if parsing fails
#[wasm_bindgen]
pub fn parse(config_js: JsValue) -> Result<JsValue, JsValue> {
    // Convert JS value to Rust type
    let js_config: JsBuildConfig = serde_wasm_bindgen::from_value(config_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

    // Convert to internal API type
    let config: crate::api::BuildConfig = js_config.into();

    // Call the core parse function
    let output = crate::api::parse(config)
        .map_err(|e| JsValue::from_str(&format!("Parse failed: {}", e)))?;

    // Convert output to JS-friendly type
    let js_output: JsParseOutput = output.into();

    // Serialize to JsValue
    serde_wasm_bindgen::to_value(&js_output)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize output: {}", e)))
}

/// Build HTML from already parsed data
///
/// # Arguments
/// * `parsed_js` - JavaScript object containing parsed passages and story data
/// * `format_info_js` - JavaScript object containing story format info
/// * `is_debug` - Whether to build in debug mode
///
/// # Returns
/// * `JsBuildOutput` - Contains the generated HTML
///
/// # Errors
/// Returns a JsValue error if the build fails
#[wasm_bindgen]
pub fn build_from_parsed(
    parsed_js: JsValue,
    format_info_js: JsValue,
    is_debug: bool,
) -> Result<JsBuildOutput, JsValue> {
    use indexmap::IndexMap;

    // Parse JS values
    let js_parsed: JsParseOutput = serde_wasm_bindgen::from_value(parsed_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse parsed data: {}", e)))?;

    let js_format_info: JsStoryFormatInfo = serde_wasm_bindgen::from_value(format_info_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse format info: {}", e)))?;

    // Convert passages from HashMap to IndexMap
    let passages: IndexMap<String, crate::core::story::Passage> = js_parsed
        .passages
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect();

    // Convert story data
    let story_data: crate::core::story::StoryData = js_parsed.story_data.into();

    // Convert format info
    let format_info: crate::api::StoryFormatInfo = js_format_info.into();

    // Create build config
    let config = crate::api::BuildFromParsedConfig {
        passages,
        story_data,
        format_info,
        is_debug,
    };

    // Call the core build function
    let output = crate::api::build_from_parsed(config)
        .map_err(|e| JsValue::from_str(&format!("Build failed: {}", e)))?;

    // Convert output to JS-friendly type
    Ok(JsBuildOutput::new(output.html))
}
