use serde::{Deserialize, Serialize};
use crate::core::story::{Passage, StoryData};

#[derive(Debug, Serialize, Deserialize)]
struct PassageMetadata {
    position: String,
    size: String,
}

pub struct TweeParser;

impl TweeParser {

    /// Parse twee3 file content
    pub fn parse(content: &str) -> Result<(Vec<Passage>, Option<StoryData>), String> {
        let mut passages = Vec::new();
        let mut story_data = None;
        let mut story_title: Option<String> = None;


        // (name, tag, position, size)
        let mut current_passage: Option<(String, Option<String>, Option<String>, Option<String>)> = None;

        let mut current_content: Vec<&str> = Vec::new();

        for line in content.lines() {
            if line.starts_with("::") {
                if let Some((name, tags, position, size)) = current_passage.take() {
                    Self::save_passage(
                        name, tags, position, size,
                        &current_content,
                        &mut story_title,
                        &mut passages,
                        &mut story_data
                    )?;
                    current_content.clear();
                }
                let header = line.trim_start_matches("::").trim();
                let metadata = Self::parse_header(header)?;
                current_passage = Some(metadata);
            } else if line.starts_with("\\::") {
                current_content.push(line.trim_start_matches("\\").trim());
            } else {
                current_content.push(line);
            }
        }

        if let Some((name, tags, position, size)) = current_passage {
            Self::save_passage(
                name, tags, position, size,
                &current_content,
                &mut story_title,
                &mut passages,
                &mut story_data
            )?;
        }

        if let Some(ref mut data) = story_data {
            data.validate().map_err(|e| std::format!("StoryData validation failed: {}", e))?;
        }

        Ok((passages, story_data))
    }

    fn save_passage(
        name: String,
        tags: Option<String>,
        position: Option<String>,
        size: Option<String>,
        content_lines: &Vec<&str>,
        story_title: &mut Option<String>,
        passages: &mut Vec<Passage>,
        story_data: &mut Option<StoryData>,
    ) -> Result<(), String> {
        let content = content_lines.join("\n").trim().to_string();

        if name == "StoryData" {
            match serde_json::from_str::<StoryData>(&content) {
                Ok(mut data) => {
                    if let Some(title) = story_title {
                        data.name = Some(title.clone());
                    }
                    *story_data = Some(data);
                },
                Err(e) => return Err(std::format!("Failed to parse StoryData JSON: {}. \nContent: '{}'", e, content)),
            }
        } else if name == "StoryTitle" {
            *story_title = Some(content);
        } else {
            passages.push(Passage {
                name,
                tags,
                position,
                size,
                content,
            });
        }

        Ok(())
    }

    fn parse_header(header: &str) ->  Result<(String, Option<String>, Option<String>, Option<String>), String> {

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
            } else if ch == '\\' {
                chars.next();
                escaped = true;
            } else if ch == '[' || ch == '{' || ch == ' ' {
                break;
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
                },
                '{' => {
                    let (json_content, end_pos) = Self::parse_brace_block(&remainder_chars, i)?;
                    if let Ok(metadata) = serde_json::from_str::<PassageMetadata>(&json_content) {
                        position = Some(metadata.position);
                        size = Some(metadata.size);
                    }
                    i = end_pos + 1;
                },
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
                    },
                    _ => {}
                }
            }

            i += 1;
        }

        Err("Unclosed '{' brace".to_string())
    }


}