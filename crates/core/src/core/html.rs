use crate::core::story::{Passage, StoryData};
use crate::util::html::HtmlEscape;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashMap;

const STORY_DATA_PASSAGE_NAME: &str = "StoryData";
const STORY_TITLE_PASSAGE_NAME: &str = "StoryTitle";
const USER_STYLESHEET_PASSAGE_NAME: &str = "StoryStylesheet";
const USER_SCRIPT_PASSAGE_NAME: &str = "StoryScript";
pub struct HtmlParseOutput {
    pub passages: IndexMap<String, Passage>,
    pub story_data: StoryData,
    pub is_debug: bool,
}

#[derive(Clone)]
struct CapturedElement {
    attrs: String,
    inner: String,
}

pub struct TwineHtmlParser;

impl TwineHtmlParser {
    pub fn parse(html: &str) -> Result<HtmlParseOutput, Box<dyn std::error::Error + Send + Sync>> {
        let story_element =
            Self::extract_first_element(html, "tw-storydata")?.ok_or_else(|| {
                "Failed to find <tw-storydata> in HTML. Only Twine export HTML is supported."
                    .to_string()
            })?;

        let story_attrs = Self::parse_attributes(&story_element.attrs);

        let story_name = Self::required_attribute(&story_attrs, "name", "tw-storydata")?;
        let ifid = Self::required_attribute(&story_attrs, "ifid", "tw-storydata")?;
        let format = Self::required_attribute(&story_attrs, "format", "tw-storydata")?;
        let format_version =
            Self::required_attribute(&story_attrs, "format-version", "tw-storydata")?;

        let zoom = story_attrs
            .get("zoom")
            .filter(|value| !value.is_empty())
            .map(|value| value.parse::<f32>())
            .transpose()
            .map_err(|e| format!("Failed to parse tw-storydata zoom: {e}"))?;

        let is_debug = story_attrs
            .get("options")
            .map(|value| value.split_whitespace().any(|item| item == "debug"))
            .unwrap_or(false);

        let stylesheet_passage =
            Self::extract_user_asset_passage(&story_element.inner, "style", "stylesheet")?;
        let script_passage =
            Self::extract_user_asset_passage(&story_element.inner, "script", "script")?;

        let tag_color_elements = Self::extract_all_elements(&story_element.inner, "tw-tag")?;
        let mut tag_colors = HashMap::new();
        for tag in tag_color_elements {
            let attrs = Self::parse_attributes(&tag.attrs);
            let Some(name) = attrs.get("name").filter(|value| !value.is_empty()).cloned() else {
                continue;
            };
            let Some(color) = attrs
                .get("color")
                .filter(|value| !value.is_empty())
                .cloned()
            else {
                continue;
            };
            tag_colors.insert(name, color);
        }

        let passage_elements = Self::extract_all_elements(&story_element.inner, "tw-passagedata")?;
        let mut pid_to_name = HashMap::new();
        let mut normal_passages = Vec::new();

        for element in passage_elements {
            let attrs = Self::parse_attributes(&element.attrs);

            let pid = Self::required_attribute(&attrs, "pid", "tw-passagedata")?;
            let name = Self::required_attribute(&attrs, "name", "tw-passagedata")?;
            let tags = attrs.get("tags").cloned().filter(|value| !value.is_empty());
            let position = attrs
                .get("position")
                .cloned()
                .filter(|value| !value.is_empty());
            let size = attrs.get("size").cloned().filter(|value| !value.is_empty());
            let content = HtmlEscape::unescape(&element.inner);

            pid_to_name.insert(pid, name.clone());
            normal_passages.push(Passage {
                name,
                tags,
                position,
                size,
                content,
                source_file: None,
                source_line: None,
            });
        }

        let start = match story_attrs
            .get("startnode")
            .filter(|value| !value.is_empty())
        {
            Some(startnode) => Some(
                pid_to_name
                    .get(startnode)
                    .cloned()
                    .ok_or_else(|| format!("Failed to resolve startnode '{startnode}'"))?,
            ),
            None => None,
        };

        let story_data = StoryData {
            name: Some(story_name.clone()),
            ifid,
            format: format.clone(),
            format_version: format_version.clone(),
            start,
            tag_colors: (!tag_colors.is_empty()).then_some(tag_colors),
            zoom,
        };

        let mut passages = IndexMap::new();
        passages.insert(
            STORY_DATA_PASSAGE_NAME.to_string(),
            Passage {
                name: STORY_DATA_PASSAGE_NAME.to_string(),
                tags: None,
                position: None,
                size: None,
                content: Self::story_data_to_twee_json(&story_data),
                source_file: None,
                source_line: None,
            },
        );
        passages.insert(
            STORY_TITLE_PASSAGE_NAME.to_string(),
            Passage {
                name: STORY_TITLE_PASSAGE_NAME.to_string(),
                tags: None,
                position: None,
                size: None,
                content: story_name,
                source_file: None,
                source_line: None,
            },
        );

        if let Some(passage) = stylesheet_passage {
            passages.insert(passage.name.clone(), passage);
        }
        if let Some(passage) = script_passage {
            passages.insert(passage.name.clone(), passage);
        }
        for passage in normal_passages {
            passages.insert(passage.name.clone(), passage);
        }

        Ok(HtmlParseOutput {
            passages,
            story_data,
            is_debug,
        })
    }

    pub fn to_twee(passages: &IndexMap<String, Passage>, story_data: &StoryData) -> String {
        let mut stylesheet_passages = Vec::new();
        let mut script_passages = Vec::new();
        let mut normal_passages = Vec::new();

        for passage in passages.values() {
            if passage.name == STORY_DATA_PASSAGE_NAME || passage.name == STORY_TITLE_PASSAGE_NAME {
                continue;
            }

            if Self::passage_has_tag(passage, "stylesheet") {
                stylesheet_passages.push(passage);
            } else if Self::passage_has_tag(passage, "script") {
                script_passages.push(passage);
            } else {
                normal_passages.push(passage);
            }
        }

        let mut sections = Vec::new();
        sections.push(Self::render_passage(
            STORY_DATA_PASSAGE_NAME,
            None,
            None,
            None,
            &Self::story_data_to_twee_json(story_data),
        ));

        if let Some(title) = story_data.name.as_ref() {
            sections.push(Self::render_passage(
                STORY_TITLE_PASSAGE_NAME,
                None,
                None,
                None,
                title,
            ));
        }

        for passage in stylesheet_passages {
            sections.push(Self::render_existing_passage(passage));
        }
        for passage in script_passages {
            sections.push(Self::render_existing_passage(passage));
        }
        for passage in normal_passages {
            sections.push(Self::render_existing_passage(passage));
        }

        sections.join("\n\n")
    }

    fn extract_user_asset_passage(
        story_inner: &str,
        tag_name: &str,
        role: &str,
    ) -> Result<Option<Passage>, Box<dyn std::error::Error + Send + Sync>> {
        let elements = Self::extract_all_elements(story_inner, tag_name)?;
        for element in elements {
            let attrs = Self::parse_attributes(&element.attrs);
            let matches_role = attrs
                .get("role")
                .map(|value| value == role)
                .unwrap_or(false);
            let matches_id = match tag_name {
                "style" => attrs
                    .get("id")
                    .map(|value| value == "twine-user-stylesheet")
                    .unwrap_or(false),
                "script" => attrs
                    .get("id")
                    .map(|value| value == "twine-user-script")
                    .unwrap_or(false),
                _ => false,
            };

            if !matches_role || !matches_id {
                continue;
            }

            let (name, tags) = match tag_name {
                "style" => (USER_STYLESHEET_PASSAGE_NAME, "stylesheet"),
                "script" => (USER_SCRIPT_PASSAGE_NAME, "script"),
                _ => continue,
            };

            return Ok(Some(Passage {
                name: name.to_string(),
                tags: Some(tags.to_string()),
                position: None,
                size: None,
                content: element.inner.trim().to_string(),
                source_file: None,
                source_line: None,
            }));
        }

        Ok(None)
    }

    fn extract_first_element(
        html: &str,
        tag_name: &str,
    ) -> Result<Option<CapturedElement>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self::extract_all_elements(html, tag_name)?
            .into_iter()
            .next())
    }

    fn extract_all_elements(
        html: &str,
        tag_name: &str,
    ) -> Result<Vec<CapturedElement>, Box<dyn std::error::Error + Send + Sync>> {
        let pattern = format!(r"(?is)<{tag_name}\b(?P<attrs>[^>]*)>(?P<inner>.*?)</{tag_name}>");
        let regex = Regex::new(&pattern)
            .map_err(|e| format!("Failed to compile HTML extraction regex for {tag_name}: {e}"))?;

        Ok(regex
            .captures_iter(html)
            .filter_map(|captures| {
                captures.get(0)?;
                let attrs = captures.name("attrs")?;
                let inner = captures.name("inner")?;

                Some(CapturedElement {
                    attrs: attrs.as_str().to_string(),
                    inner: inner.as_str().to_string(),
                })
            })
            .collect())
    }

    fn parse_attributes(attrs: &str) -> HashMap<String, String> {
        let mut parsed = HashMap::new();
        let regex = Regex::new(
            r#"(?is)([A-Za-z_:][-A-Za-z0-9_:.]*)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?"#,
        )
        .expect("attribute regex should be valid");

        for captures in regex.captures_iter(attrs) {
            let Some(name_match) = captures.get(1) else {
                continue;
            };

            let name = name_match.as_str().to_lowercase();
            let raw_value = captures
                .get(2)
                .or_else(|| captures.get(3))
                .or_else(|| captures.get(4))
                .map(|value| value.as_str())
                .unwrap_or("");

            parsed.insert(name, Self::decode_attribute_value(raw_value));
        }

        parsed
    }

    fn required_attribute(
        attrs: &HashMap<String, String>,
        name: &str,
        tag_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        attrs
            .get(name)
            .filter(|value| !value.is_empty())
            .cloned()
            .ok_or_else(|| format!("Missing required attribute '{name}' on <{tag_name}>").into())
    }

    fn decode_attribute_value(value: &str) -> String {
        let value = HtmlEscape::unescape(value);
        let mut decoded = String::with_capacity(value.len());
        let mut chars = value.chars();

        while let Some(ch) = chars.next() {
            if ch != '\\' {
                decoded.push(ch);
                continue;
            }

            match chars.next() {
                Some('\\') => decoded.push('\\'),
                Some('"') => decoded.push('"'),
                Some('\'') => decoded.push('\''),
                Some('n') => decoded.push('\n'),
                Some('r') => decoded.push('\r'),
                Some('t') => decoded.push('\t'),
                Some(other) => {
                    decoded.push('\\');
                    decoded.push(other);
                }
                None => decoded.push('\\'),
            }
        }

        decoded
    }

    fn passage_has_tag(passage: &Passage, target_tag: &str) -> bool {
        passage
            .tags
            .as_deref()
            .map(|tags| tags.split_whitespace().any(|tag| tag == target_tag))
            .unwrap_or(false)
    }

    fn render_existing_passage(passage: &Passage) -> String {
        Self::render_passage(
            &passage.name,
            passage.tags.as_deref(),
            passage.position.as_deref(),
            passage.size.as_deref(),
            &passage.content,
        )
    }

    fn render_passage(
        name: &str,
        tags: Option<&str>,
        position: Option<&str>,
        size: Option<&str>,
        content: &str,
    ) -> String {
        let mut header = format!(":: {}", Self::escape_header_name(name));

        if let Some(tags) = tags.filter(|value| !value.is_empty()) {
            header.push(' ');
            header.push('[');
            header.push_str(&Self::escape_tag_block(tags));
            header.push(']');
        }

        if position.is_some() || size.is_some() {
            let metadata = format!(
                "{{\"position\":{},\"size\":{}}}",
                serde_json::to_string(position.unwrap_or("")).unwrap_or_else(|_| "\"\"".into()),
                serde_json::to_string(size.unwrap_or("")).unwrap_or_else(|_| "\"\"".into())
            );
            header.push(' ');
            header.push_str(&metadata);
        }

        let content = Self::escape_content_lines(content);
        if content.is_empty() {
            header
        } else {
            format!("{header}\n{content}")
        }
    }

    fn escape_header_name(name: &str) -> String {
        name.replace('\\', "\\\\")
            .replace('[', "\\[")
            .replace('{', "\\{")
    }

    fn escape_tag_block(tags: &str) -> String {
        tags.replace('\\', "\\\\").replace(']', "\\]")
    }

    fn escape_content_lines(content: &str) -> String {
        content
            .lines()
            .map(|line| {
                if line.starts_with("::") {
                    format!("\\{line}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn story_data_to_twee_json(story_data: &StoryData) -> String {
        let mut fields = Vec::new();
        fields.push(("ifid", serde_json::to_string(&story_data.ifid).unwrap()));
        fields.push(("format", serde_json::to_string(&story_data.format).unwrap()));
        fields.push((
            "format-version",
            serde_json::to_string(&story_data.format_version).unwrap(),
        ));

        if let Some(start) = story_data.start.as_ref() {
            fields.push(("start", serde_json::to_string(start).unwrap()));
        }
        if let Some(tag_colors) = story_data.tag_colors.as_ref() {
            fields.push(("tag-colors", serde_json::to_string(tag_colors).unwrap()));
        }
        if let Some(zoom) = story_data.zoom {
            fields.push(("zoom", serde_json::to_string(&zoom).unwrap()));
        }

        let mut lines = Vec::with_capacity(fields.len() + 2);
        lines.push("{".to_string());
        for (index, (key, value)) in fields.iter().enumerate() {
            let suffix = if index + 1 == fields.len() { "" } else { "," };
            lines.push(format!("    \"{key}\": {value}{suffix}"));
        }
        lines.push("}".to_string());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::TwineHtmlParser;

    #[test]
    fn test_parse_missing_storydata() {
        let result = TwineHtmlParser::parse("<html><body><p>No story</p></body></html>");
        assert!(result.is_err());
    }

    #[test]
    fn test_render_content_lines_escape_twee_headers() {
        let rendered = TwineHtmlParser::to_twee(
            &indexmap::IndexMap::from([(
                "Start".to_string(),
                crate::core::story::Passage {
                    name: "Start".to_string(),
                    tags: None,
                    position: None,
                    size: None,
                    content: ":: not a header".to_string(),
                    source_file: None,
                    source_line: None,
                },
            )]),
            &crate::core::story::StoryData {
                name: Some("Story".to_string()),
                ifid: "ifid".to_string(),
                format: "SugarCube".to_string(),
                format_version: "2.37.3".to_string(),
                start: Some("Start".to_string()),
                tag_colors: None,
                zoom: None,
            },
        );

        assert!(rendered.contains("\\:: not a header"));
    }
}
