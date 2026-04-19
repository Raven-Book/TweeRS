use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tweers"))
}

fn sample_html() -> &'static str {
    r#"<!doctype html>
<html>
<body>
<tw-storydata name="CLI Story" startnode="1" creator="Twine" creator-version="2.9.0" ifid="12345678-1234-1234-1234-123456789012" zoom="1" format="SugarCube" format-version="2.37.3" options="" hidden>
<style role="stylesheet" id="twine-user-stylesheet" type="text/twine-css">/* twine-user-stylesheet #1: "StoryStylesheet" */
body { color: red; }</style>
<script role="script" id="twine-user-script" type="text/twine-javascript">/* twine-user-script #1: "StoryScript" */
window.answer = 42;</script>
<tw-passagedata pid="1" name="Start" tags="" position="" size="">Hello</tw-passagedata>
</tw-storydata>
</body>
</html>"#
}

fn reset_test_dir(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path).expect("failed to remove existing test dir");
    }
    fs::create_dir_all(path).expect("failed to create test dir");
}

#[test]
fn test_html2twee_command_default_output_name() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let temp_dir = workspace_dir.join("target/test-html2twee-default");
    reset_test_dir(&temp_dir);

    let input_path = temp_dir.join("story.html");
    let output_path = temp_dir.join("story.twee");

    fs::write(&input_path, sample_html()).expect("failed to write input html");

    let output = Command::new(cli_bin())
        .arg("html2twee")
        .arg(&input_path)
        .current_dir(&workspace_dir)
        .output()
        .expect("failed to run tweers-cli");

    assert!(output.status.success(), "html2twee command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("Output written to: {}", output_path.display())));

    let twee = fs::read_to_string(output_path).expect("failed to read output twee");
    assert!(twee.contains(":: StoryData"));
    assert!(twee.contains(":: StoryTitle"));
    assert!(twee.contains(":: StoryStylesheet [stylesheet]"));
    assert!(twee.contains(":: StoryScript [script]"));
    assert!(twee.contains("/* twine-user-stylesheet #1: \"StoryStylesheet\" */"));
    assert!(twee.contains("/* twine-user-script #1: \"StoryScript\" */"));
    assert!(twee.contains(":: Start"));
}

#[test]
fn test_html2twee_command_output_directory() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let temp_dir = workspace_dir.join("target/test-html2twee-dir");
    reset_test_dir(&temp_dir);
    let output_dir = temp_dir.join("out");
    fs::create_dir_all(&output_dir).expect("failed to create output dir");

    let input_path = temp_dir.join("chapter.html");
    let output_path = output_dir.join("chapter.twee");
    fs::write(&input_path, sample_html()).expect("failed to write input html");

    let output = Command::new(cli_bin())
        .arg("html2twee")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_dir)
        .current_dir(&workspace_dir)
        .output()
        .expect("failed to run tweers-cli");

    assert!(output.status.success(), "html2twee command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("Output written to: {}", output_path.display())));
    assert!(output_path.exists(), "expected output file to be created in directory");
}

#[test]
fn test_html2twee_command_overwrite_prompt_accepts_yes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let temp_dir = workspace_dir.join("target/test-html2twee-overwrite");
    reset_test_dir(&temp_dir);

    let input_path = temp_dir.join("story.html");
    let output_path = temp_dir.join("story.twee");
    fs::write(&input_path, sample_html()).expect("failed to write input html");
    fs::write(&output_path, "old content").expect("failed to seed output file");

    let mut child = Command::new(cli_bin())
        .arg("html2twee")
        .arg(&input_path)
        .current_dir(&workspace_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn tweers-cli");

    child
        .stdin
        .as_mut()
        .expect("missing stdin")
        .write_all(b"y\n")
        .expect("failed to write confirmation");

    let output = child.wait_with_output().expect("failed to wait for cli");
    assert!(output.status.success(), "html2twee command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Overwrite? [Y/N]:"));
    assert!(stdout.contains(&format!("Output written to: {}", output_path.display())));

    let twee = fs::read_to_string(output_path).expect("failed to read overwritten output");
    assert!(twee.contains(":: StoryData"));
    assert!(twee.contains("/* twine-user-script #1: \"StoryScript\" */"));
}
