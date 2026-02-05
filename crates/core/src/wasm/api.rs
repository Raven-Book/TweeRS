// WASM API bindings - JavaScript-callable functions

use super::types::{JsBuildConfig, JsBuildOutput, JsParseOutput};
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
/// * `sources_js` - JavaScript array of input sources
///
/// # Returns
/// * `JsValue` - Contains passages, story data, and format info (with empty source)
///
/// # Errors
/// Returns a JsValue error if parsing fails
#[wasm_bindgen]
pub fn parse(sources_js: JsValue) -> Result<JsValue, JsValue> {
    use super::types::JsInputSource;

    // Convert JS value to sources array
    let js_sources: Vec<JsInputSource> = serde_wasm_bindgen::from_value(sources_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse sources: {}", e)))?;

    // Convert to internal API types
    let sources: Vec<crate::api::InputSource> = js_sources.into_iter().map(|s| s.into()).collect();

    // Call the core parse function
    let output = crate::api::parse(sources)
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
/// * `parsed_js` - JavaScript object containing parsed passages, story data, and format info
///
/// # Returns
/// * `JsBuildOutput` - Contains the generated HTML
///
/// # Errors
/// Returns a JsValue error if the build fails
#[wasm_bindgen]
pub fn build_from_parsed(parsed_js: JsValue) -> Result<JsBuildOutput, JsValue> {
    use indexmap::IndexMap;

    // Parse JS value
    let js_parsed: JsParseOutput = serde_wasm_bindgen::from_value(parsed_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse parsed data: {}", e)))?;

    // Convert passages from HashMap to IndexMap with depth-first natural sorting by source_file
    let mut passages_vec: Vec<(String, crate::core::story::Passage)> = js_parsed
        .passages
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect();

    // Sort by source_file using depth-first natural order
    passages_vec.sort_by(|a, b| {
        crate::util::sort::compare_paths(
            a.1.source_file.as_deref().unwrap_or(&a.0),
            b.1.source_file.as_deref().unwrap_or(&b.0),
        )
    });

    let passages: IndexMap<String, crate::core::story::Passage> =
        passages_vec.into_iter().collect();

    // Convert to ParseOutput
    let parse_output = crate::api::ParseOutput {
        passages,
        story_data: js_parsed.story_data.into(),
        format_info: js_parsed.format_info.into(),
        is_debug: js_parsed.is_debug,
    };

    // Call the core build function
    let output = crate::api::build_from_parsed(parse_output)
        .map_err(|e| JsValue::from_str(&format!("Build failed: {}", e)))?;

    // Convert output to JS-friendly type
    Ok(JsBuildOutput::new(output.html))
}

/// Parse passages only - does not require StoryData
#[wasm_bindgen]
pub fn passages(sources_js: JsValue) -> Result<JsValue, JsValue> {
    use super::types::{JsInputSource, JsPassage};

    let js_sources: Vec<JsInputSource> = serde_wasm_bindgen::from_value(sources_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse sources: {}", e)))?;

    let sources: Vec<crate::api::InputSource> = js_sources.into_iter().map(|s| s.into()).collect();

    let output = crate::api::passages(sources)
        .map_err(|e| JsValue::from_str(&format!("Parse failed: {}", e)))?;

    let js_passages: std::collections::HashMap<String, JsPassage> =
        output.into_iter().map(|(k, v)| (k, v.into())).collect();

    serde_wasm_bindgen::to_value(&js_passages)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize output: {}", e)))
}

/// Sort file paths using depth-first natural ordering
///
/// Returns paths sorted with deeper paths first, then natural sort within same depth.
/// This matches the order used for passage processing in build.
#[wasm_bindgen]
pub fn sort_paths(paths_js: JsValue) -> Result<JsValue, JsValue> {
    let paths: Vec<String> = serde_wasm_bindgen::from_value(paths_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse paths: {}", e)))?;

    let sorted = crate::api::sort_paths(paths);

    serde_wasm_bindgen::to_value(&sorted)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize output: {}", e)))
}
