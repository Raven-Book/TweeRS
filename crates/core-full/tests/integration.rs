// End-to-end integration test for build pipeline
use std::fs;
use std::path::PathBuf;
use tweers_core::pipeline::{BASE64, SOURCES};
use tweers_core_full::commands::BuildContext;
use tweers_core_full::commands::CONTEXT;
use tweers_core_full::pipeline::{nodes::basic::*, PipeMap, Pipeline};

#[tokio::test]
async fn test_file_collection_and_parsing() {
    // Setup test paths
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let story_dir = test_dir.join("test/story");

    // Create a simple pipeline for file collection only
    let mut pipeline = Pipeline::new("test_collection");
    pipeline = pipeline
        .add_node(Box::new(FileCollectorNode))
        .expect("Failed to add FileCollectorNode");

    // Prepare input data
    let mut input = PipeMap::new();
    input.insert_typed(SOURCES, vec![story_dir]);
    input.insert_typed(BASE64, false);

    // Execute pipeline
    let result = pipeline.execute(input).await;

    if let Err(e) = &result {
        eprintln!("Pipeline execution failed: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "Pipeline execution failed: {:?}",
        result.err()
    );

    let output = result.unwrap();

    // Verify output contains collected files
    let files = output.get_typed(tweers_core::pipeline::FILES);
    assert!(files.is_some(), "Files not found in output");
    assert!(!files.unwrap().is_empty(), "No files collected");
}

#[tokio::test]
async fn test_data_aggregator_injects_tweers_paths() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let temp_dir = workspace_dir.join("target/test-tweers-paths");

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("failed to clean test dir");
    }
    fs::create_dir_all(temp_dir.join("story/chapters")).expect("failed to create story dir");
    fs::create_dir_all(temp_dir.join("assets")).expect("failed to create assets dir");

    fs::write(
        temp_dir.join("story/main.twee"),
        r#":: StoryData
{
    "ifid": "12345678-1234-1234-1234-123456789012",
    "format": "SugarCube",
    "format-version": "2.37.3"
}

:: StoryTitle
Test

:: Start
Hello
"#,
    )
    .expect("failed to write main.twee");

    fs::write(
        temp_dir.join("story/chapters/scene.tw"),
        r#":: Scene
World

:: Choice
Next
"#,
    )
    .expect("failed to write scene.tw");

    fs::write(temp_dir.join("assets/logic.js"), "window.answer = 42;")
        .expect("failed to write logic.js");
    fs::write(temp_dir.join("assets/theme.css"), "body { color: red; }")
        .expect("failed to write theme.css");

    let files = vec![
        temp_dir.join("story/main.twee"),
        temp_dir.join("story/chapters/scene.tw"),
        temp_dir.join("assets/logic.js"),
        temp_dir.join("assets/theme.css"),
    ];

    let mut pipeline = Pipeline::new("test_tweers_paths");
    pipeline = pipeline
        .add_node(Box::new(FileParserNode))
        .expect("Failed to add FileParserNode")
        .add_node(Box::new(DataAggregatorNode))
        .expect("Failed to add DataAggregatorNode");

    let mut input = PipeMap::new();
    input.insert_typed(tweers_core::pipeline::MODIFIED_FILES, files.clone());
    input.insert_typed(tweers_core::pipeline::FILES, files.clone());
    input.insert_typed(
        SOURCES,
        vec![temp_dir.join("story"), temp_dir.join("assets")],
    );
    input.insert_typed(CONTEXT, BuildContext::new(false, false, None));

    let output = pipeline.execute(input).await.expect("pipeline failed");
    let passages = output
        .get_typed(tweers_core::pipeline::ALL_PASSAGES)
        .expect("missing all_passages");

    let tweers_paths = passages
        .get("TweersPaths")
        .expect("missing TweersPaths passage");

    let json: serde_json::Value =
        serde_json::from_str(&tweers_paths.content).expect("TweersPaths is not valid json");

    assert_eq!(json["version"], 1);
    let sources = json["sources"]
        .as_array()
        .expect("sources should be an array");
    assert_eq!(sources.len(), 4);

    assert!(sources.iter().any(|item| {
        item["type"] == "twee"
            && item["path"] == "main.twee"
            && item["dir"] == "."
            && item["name"] == "main.twee"
            && item["passages"] == serde_json::json!(["StoryData", "StoryTitle", "Start"])
    }));
    assert!(sources.iter().any(|item| {
        item["type"] == "twee"
            && item["path"] == "chapters/scene.tw"
            && item["dir"] == "chapters"
            && item["name"] == "scene.tw"
            && item["passages"] == serde_json::json!(["Scene", "Choice"])
    }));
    assert!(sources.iter().any(|item| {
        item["type"] == "js"
            && item["path"] == "logic.js"
            && item["dir"] == "."
            && item["name"] == "logic.js"
    }));
    assert!(sources.iter().any(|item| {
        item["type"] == "css"
            && item["path"] == "theme.css"
            && item["dir"] == "."
            && item["name"] == "theme.css"
    }));
}
