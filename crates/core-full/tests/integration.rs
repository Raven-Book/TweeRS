// End-to-end integration test for build pipeline
use tweers_core_full::pipeline::{
    PipeMap, Pipeline,
    nodes::basic::*,
};
use tweers_core::pipeline::{SOURCES, BASE64};
use std::path::PathBuf;

#[tokio::test]
async fn test_file_collection_and_parsing() {
    // Setup test paths
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap();

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

    assert!(result.is_ok(), "Pipeline execution failed: {:?}", result.err());

    let output = result.unwrap();

    // Verify output contains collected files
    let files = output.get_typed(tweers_core::pipeline::FILES);
    assert!(files.is_some(), "Files not found in output");
    assert!(!files.unwrap().is_empty(), "No files collected");
}
