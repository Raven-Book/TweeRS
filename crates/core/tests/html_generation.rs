// End-to-end HTML generation test
use std::fs;
use std::path::PathBuf;
use tweers_core::api::{BuildConfig, InputSource, StoryFormatInfo};

fn collect_story_files(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_story_files(&path, files);
            } else if let Some(ext) = path.extension() {
                if ext == "twee" || ext == "tw" {
                    files.push(path);
                }
            }
        }
    }
}

#[test]
fn test_html_generation_from_test_story() {
    // Setup test paths
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let story_dir = test_dir.join("test/story");
    let format_file = test_dir.join("test/story-format/sugarcube-2.37.3/format.js");

    // Load format source
    let format_source = std::fs::read_to_string(&format_file).expect("Failed to read format file");

    // Collect all .twee and .tw files
    let mut story_files = Vec::new();
    collect_story_files(&story_dir, &mut story_files);
    assert!(!story_files.is_empty(), "No story files found");

    // Sort files to ensure deterministic ordering across different filesystems
    story_files.sort();

    // Read all story files
    let mut sources = Vec::new();
    for file in story_files {
        let content = std::fs::read_to_string(&file).expect(&format!("Failed to read {:?}", file));
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        sources.push(InputSource::Text { name, content });
    }

    // Create format info
    let format_info = StoryFormatInfo {
        name: "SugarCube".to_string(),
        version: "2.37.3".to_string(),
        source: format_source,
    };

    // Create build config
    let config = BuildConfig::new(format_info).sources(sources).debug(false);

    // Build HTML (synchronous)
    let result = tweers_core::api::build(config);
    assert!(result.is_ok(), "Build failed: {:?}", result.err());

    let output = result.unwrap();

    // Verify HTML output
    assert!(!output.html.is_empty(), "HTML output is empty");
    assert!(
        output.html.contains("Test Story"),
        "HTML should contain story title"
    );
    assert!(
        output.html.contains("Welcome to the test story"),
        "HTML should contain passage content"
    );
}

#[test]
fn test_passages_api() {
    use tweers_core::api::{InputSource, passages};

    let content = ":: Start\nHello world\n\n:: Second [tag1 tag2]\nAnother passage";
    let sources = vec![InputSource::Text {
        name: "test.twee".to_string(),
        content: content.to_string(),
    }];

    let result = passages(sources);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "passages failed: {:?}", result.err());

    let passages = result.unwrap();
    println!("Passages count: {}", passages.len());
    for (name, p) in &passages {
        println!("- {}: line {:?}", name, p.source_line);
    }
    assert_eq!(passages.len(), 2, "Expected 2 passages");
}
