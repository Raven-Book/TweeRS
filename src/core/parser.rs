use crate::core::story::{Passage, StoryData};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
struct PassageMetadata {
    position: String,
    size: String,
}

// Type aliases to reduce complexity
type PassageHeader = (String, Option<String>, Option<String>, Option<String>);
type ParseResult = Result<(IndexMap<String, Passage>, Option<StoryData>), String>;

// Struct to reduce function parameters
struct PassageContext<'a> {
    story_title: &'a mut Option<String>,
    passages: &'a mut IndexMap<String, Passage>,
    story_data: &'a mut Option<StoryData>,
}

pub struct TweeParser;

impl TweeParser {
    /// Parse twee3 file content
    pub fn parse(content: &str) -> ParseResult {
        debug!(
            "Starting to parse content with {} lines",
            content.lines().count()
        );
        let mut passages = IndexMap::new();
        let mut story_data = None;
        let mut story_title: Option<String> = None;

        let mut current_passage: Option<PassageHeader> = None;

        let mut current_content: Vec<&str> = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            debug!("Processing line {}: {:?}", line_num + 1, line);
            if line.starts_with("::") {
                if let Some((name, tags, position, size)) = current_passage.take() {
                    let mut context = PassageContext {
                        story_title: &mut story_title,
                        passages: &mut passages,
                        story_data: &mut story_data,
                    };
                    Self::save_passage(name, tags, position, size, &current_content, &mut context)?;
                    current_content.clear();
                }
                let header = line.trim_start_matches("::").trim();
                debug!("Parsing header: {:?}", header);
                let metadata = Self::parse_header(header)?;
                debug!("Parsed metadata: {:?}", metadata);
                current_passage = Some(metadata);
            } else if line.starts_with("\\::") {
                current_content.push(line.trim_start_matches("\\").trim());
            } else {
                current_content.push(line);
            }
        }

        if let Some((name, tags, position, size)) = current_passage {
            let mut context = PassageContext {
                story_title: &mut story_title,
                passages: &mut passages,
                story_data: &mut story_data,
            };
            Self::save_passage(name, tags, position, size, &current_content, &mut context)?;
        }

        Ok((passages, story_data))
    }

    fn save_passage(
        name: String,
        tags: Option<String>,
        position: Option<String>,
        size: Option<String>,
        content_lines: &Vec<&str>,
        context: &mut PassageContext,
    ) -> Result<(), String> {
        use tracing::debug;

        let content = content_lines.join("\n").trim().to_string();
        debug!(
            "Saving passage '{}' with content length: {}",
            name,
            content.len()
        );

        if name == "StoryData" {
            debug!("Processing StoryData with content: {:?}", content);
            match serde_json::from_str::<StoryData>(&content) {
                Ok(data) => {
                    *context.story_data = Some(data);
                }
                Err(e) => {
                    return Err(std::format!(
                        "Failed to parse StoryData JSON: {}. \nContent: '{}'",
                        e,
                        content
                    ));
                }
            }
        } else if name == "StoryTitle" {
            *context.story_title = Some(content.clone());
            let passage = Passage {
                name: name.clone(),
                tags,
                position,
                size,
                content,
            };
            context.passages.insert(name, passage);
        } else {
            let passage = Passage {
                name: name.clone(),
                tags,
                position,
                size,
                content,
            };
            context.passages.insert(name, passage);
        }

        Ok(())
    }

    fn parse_header(header: &str) -> Result<PassageHeader, String> {
        let mut chars = header.chars().peekable();

        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        let mut name = String::new();
        let mut escaped = false;

        while let Some(&ch) = chars.peek() {
            if escaped {
                chars.next();
                name.push(ch);
                escaped = false;
            } else if ch == '\\' {
                chars.next();
                escaped = true;
            } else if ch == '[' || ch == '{' || ch == ' ' {
                break;
            } else {
                chars.next();
                name.push(ch);
            }
        }

        if name.is_empty() {
            return Err("Empty passage name".to_string());
        }

        let remainder: String = chars.collect();
        let remainder = remainder.trim();

        let mut tags = None;
        let mut position = None;
        let mut size = None;

        let mut i = 0;
        let remainder_chars: Vec<char> = remainder.chars().collect();

        while i < remainder.len() {
            match remainder_chars[i] {
                '[' => {
                    let (tag_content, end_pos) = Self::parse_bracket_block(&remainder_chars, i)?;
                    if !tag_content.trim().is_empty() {
                        tags = Some(tag_content.trim().to_string());
                    }
                    i = end_pos + 1;
                }
                '{' => {
                    let (json_content, end_pos) = Self::parse_brace_block(&remainder_chars, i)?;
                    if let Ok(metadata) = serde_json::from_str::<PassageMetadata>(&json_content) {
                        position = Some(metadata.position);
                        size = Some(metadata.size);
                    }
                    i = end_pos + 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok((name, tags, position, size))
    }

    fn parse_bracket_block(chars: &[char], start: usize) -> Result<(String, usize), String> {
        if start >= chars.len() || chars[start] != '[' {
            return Err("Expected '[' at start position".to_string());
        }

        let mut content = String::new();
        let mut i = start + 1;
        let mut escaped = false;

        while i < chars.len() {
            let ch = chars[i];

            if escaped {
                content.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == ']' {
                return Ok((content, i));
            } else {
                content.push(ch);
            }

            i += 1;
        }

        Err("Unclosed '[' bracket".to_string())
    }
    fn parse_brace_block(chars: &[char], start: usize) -> Result<(String, usize), String> {
        if start >= chars.len() || chars[start] != '{' {
            return Err("Expected '{' at start position".to_string());
        }

        let mut content = String::new();
        let mut i = start;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut string_delimiter = None;
        let mut escaped = false;

        while i < chars.len() {
            let ch = chars[i];
            content.push(ch);

            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' || ch == '\'' {
                if in_string {
                    if Some(ch) == string_delimiter {
                        in_string = false;
                        string_delimiter = None;
                    }
                } else {
                    in_string = true;
                    string_delimiter = Some(ch);
                }
            } else if !in_string {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            return Ok((content, i));
                        }
                    }
                    _ => {}
                }
            }

            i += 1;
        }

        Err("Unclosed '{' brace".to_string())
    }
}
