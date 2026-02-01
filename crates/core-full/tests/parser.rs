// Integration tests for FileParser
use tweers_core_full::pipeline::parsers::twee::TweeFileParser;
use tweers_core_full::pipeline::parsers::FileParser;
use tweers_core::error::Result;
use std::path::PathBuf;

#[tokio::test]
async fn test_twee_parser() {
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test/story/A.twee");

    let parser = TweeFileParser;
    let result: Result<_> = parser.parse(&test_file).await;

    assert!(result.is_ok());
    let (passages, story_data) = result.unwrap();

    // Should have Start and Next passages
    assert!(passages.contains_key("Start"));
    assert!(passages.contains_key("Next"));
    assert!(story_data.is_some());
}
