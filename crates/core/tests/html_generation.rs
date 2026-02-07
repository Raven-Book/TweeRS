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

#[test]
fn test_non_ascii_tags() {
    use tweers_core::api::{InputSource, passages};

    let content = ":: Start [你好 世界]\nHello\n\n:: Second [café naïve]\nWorld";
    let sources = vec![InputSource::Text {
        name: "test.twee".to_string(),
        content: content.to_string(),
    }];

    let result = passages(sources);
    assert!(result.is_ok(), "non-ASCII tags failed: {:?}", result.err());

    let passages = result.unwrap();
    assert_eq!(passages.len(), 2);
    assert_eq!(passages["Start"].tags.as_deref(), Some("你好 世界"));
    assert_eq!(passages["Second"].tags.as_deref(), Some("café naïve"));
}

#[test]
fn test_start_passage_override() {
    use tweers_core::api::{BuildConfig, InputSource, StoryFormatInfo, build};

    // Setup test paths
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let format_file = test_dir.join("test/story-format/sugarcube-2.37.3/format.js");
    let format_source = fs::read_to_string(&format_file).expect("Failed to read format file");

    let content = r#":: StoryData
{
    "ifid": "12345678-1234-1234-1234-123456789012",
    "format": "SugarCube",
    "format-version": "2.37.3"
}

:: StoryTitle
Test

:: Start
Default start

:: CustomStart
Custom start passage
"#;

    let sources = vec![InputSource::Text {
        name: "test.twee".to_string(),
        content: content.to_string(),
    }];

    let format_info = StoryFormatInfo {
        name: "SugarCube".to_string(),
        version: "2.37.3".to_string(),
        source: format_source,
    };

    let config = BuildConfig::new(format_info)
        .sources(sources)
        .start_passage(Some("CustomStart".to_string()));

    let result = build(config);
    assert!(result.is_ok(), "Build failed: {:?}", result.err());

    let output = result.unwrap();

    // Print startnode for debugging
    if let Some(cap) = regex::Regex::new(r#"startnode="(\d+)""#)
        .unwrap()
        .captures(&output.html)
    {
        println!("startnode = {}", &cap[1]);
    }

    // Check that CustomStart is set as startnode
    assert!(
        output.html.contains(r#"startnode="2""#) || output.html.contains("CustomStart"),
        "HTML should reference CustomStart as start passage"
    );
    println!("story_data.start = {:?}", output.story_data.start);
    assert_eq!(output.story_data.start, Some("CustomStart".to_string()));
}

#[test]
fn test_parse_api() {
    use tweers_core::api::{InputSource, parse};

    let content = r#":: StoryData
{
    "ifid": "12345678-1234-1234-1234-123456789012",
    "format": "SugarCube",
    "format-version": "2.37.3"
}

:: StoryTitle
Parse Test

:: Start
Hello world

:: Second [tag1 tag2]
Another passage
"#;

    let sources = vec![InputSource::Text {
        name: "test.twee".to_string(),
        content: content.to_string(),
    }];

    let result = parse(sources);
    assert!(result.is_ok(), "parse failed: {:?}", result.err());

    let output = result.unwrap();

    // Verify passages
    assert_eq!(output.passages.len(), 4);
    assert!(output.passages.contains_key("Start"));
    assert!(output.passages.contains_key("Second"));

    // Verify story data
    assert_eq!(output.story_data.name, Some("Parse Test".to_string()));
    assert_eq!(
        output.story_data.ifid,
        "12345678-1234-1234-1234-123456789012"
    );

    // Verify format info
    assert_eq!(output.format_info.name, "SugarCube");
    assert_eq!(output.format_info.version, "2.37.3");
    assert!(output.format_info.source.is_empty());
}

#[test]
fn test_build_from_parsed_api() {
    use tweers_core::api::{InputSource, StoryFormatInfo, build_from_parsed, parse};

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let format_file = test_dir.join("test/story-format/sugarcube-2.37.3/format.js");
    let format_source = fs::read_to_string(&format_file).expect("Failed to read format file");

    let content = r#":: StoryData
{
    "ifid": "12345678-1234-1234-1234-123456789012",
    "format": "SugarCube",
    "format-version": "2.37.3"
}

:: StoryTitle
Build From Parsed Test

:: Start
Hello from parsed build
"#;

    let sources = vec![InputSource::Text {
        name: "test.twee".to_string(),
        content: content.to_string(),
    }];

    // First parse
    let mut parsed = parse(sources).expect("parse failed");

    // Fill in format source
    parsed.format_info = StoryFormatInfo {
        name: parsed.format_info.name,
        version: parsed.format_info.version,
        source: format_source,
    };

    // Then build from parsed
    let result = build_from_parsed(parsed);
    assert!(
        result.is_ok(),
        "build_from_parsed failed: {:?}",
        result.err()
    );

    let output = result.unwrap();

    assert!(!output.html.is_empty());
    assert!(output.html.contains("Build From Parsed Test"));
    assert!(output.html.contains("Hello from parsed build"));
}
