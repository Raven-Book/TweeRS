// Integration tests for FileCollector
use std::path::PathBuf;
use tweers_core_full::io::{FileCollector, SupportFileFilter};

#[tokio::test]
async fn test_collect_twee_files() {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test/story");

    let filter = SupportFileFilter::new(false);
    let collector = FileCollector::new(filter);

    let files = collector
        .collect_async(&[test_dir])
        .await
        .expect("Failed to collect files");

    // Should find .twee and .tw files
    assert!(!files.is_empty());
    assert!(files
        .iter()
        .any(|f| f.extension().and_then(|e| e.to_str()) == Some("twee")));
}
