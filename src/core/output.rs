use crate::core::story::{Passage, StoryData, StoryFormat};
use indexmap::IndexMap;
use tracing::debug;

// Struct to reduce function parameters
struct StoryInfo<'a> {
    name: &'a str,
    ifid: &'a str,
    format: &'a str,
    format_version: &'a str,
    start_passage: &'a str,
    zoom: f32,
}

pub struct HtmlOutputHandler;

impl HtmlOutputHandler {
    pub async fn generate_html(
        passages: &IndexMap<String, Passage>,
        story_data: &Option<StoryData>,
        context: &crate::cli::BuildContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let data = story_data.as_ref().ok_or("StoryData is required")?;

        let name = data
            .name
            .as_ref()
            .ok_or("Story name is required (missing StoryTitle passage?)")?;

        let ifid = if data.ifid.is_empty() {
            return Err("IFID is required in StoryData".into());
        } else {
            &data.ifid
        };

        let format = &data.format;
        let format_version = &data.format_version;

        let start_passage = data.start.as_deref()
            .or_else(|| {
                if passages.contains_key("Start") {
                    Some("Start")
                } else {
                    passages.keys().next().map(|k| k.as_str())
                }
            })
            .ok_or("No start passage found (either specify 'start' in StoryData or provide at least one passage)")?;

        let zoom = data.zoom.unwrap_or(1.0);

        let story_format = StoryFormat::find_format(format, format_version).await?;

        let story_info = StoryInfo {
            name,
            ifid,
            format,
            format_version,
            start_passage,
            zoom,
        };
        let story_data_xml =
            Self::get_twine2_data_chunk(passages, &story_info, data, context.is_debug)?;

        let html = story_format
            .source
            .replace("{{STORY_NAME}}", name)
            .replace("{{STORY_DATA}}", &story_data_xml);

        Ok(html)
    }

    /// Only update the pages of modified files
    pub async fn update_html(
        passages: &IndexMap<String, Passage>,
        story_data: &Option<StoryData>,
        context: &mut crate::cli::BuildContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(data) = story_data {
            let format_changed =
                context.format_name != data.format || context.format_version != data.format_version;

            if !format_changed && context.story_format.is_some() {
                if let Some(story_format) = &context.story_format {
                    let name = data
                        .name
                        .as_ref()
                        .ok_or("Story name is required (missing StoryTitle passage?)")?;

                    let ifid = if data.ifid.is_empty() {
                        return Err("IFID is required in StoryData".into());
                    } else {
                        &data.ifid
                    };

                    let start_passage = data
                        .start
                        .as_deref()
                        .or_else(|| {
                            if passages.contains_key("Start") {
                                Some("Start")
                            } else {
                                passages.keys().next().map(|k| k.as_str())
                            }
                        })
                        .ok_or("No start passage found")?;

                    let zoom = data.zoom.unwrap_or(1.0);

                    let story_info = StoryInfo {
                        name,
                        ifid,
                        format: &data.format,
                        format_version: &data.format_version,
                        start_passage,
                        zoom,
                    };
                    let story_data_xml =
                        Self::get_twine2_data_chunk(passages, &story_info, data, context.is_debug)?;

                    let html = story_format
                        .source
                        .replace("{{STORY_NAME}}", name)
                        .replace("{{STORY_DATA}}", &story_data_xml);

                    return Ok(html);
                }
            }

            if format_changed {
                context.format_name = data.format.clone();
                context.format_version = data.format_version.clone();

                let story_format =
                    StoryFormat::find_format(&data.format, &data.format_version).await?;
                context.story_format = Some(story_format);

                return Self::generate_html_with_cached_format(passages, data, context);
            }
        }

        Self::generate_html(passages, story_data, context).await
    }

    /// Generate HTML using cached story format (avoid repeated format file lookups)
    fn generate_html_with_cached_format(
        passages: &IndexMap<String, Passage>,
        story_data: &StoryData,
        context: &crate::cli::BuildContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(story_format) = &context.story_format {
            let name = story_data
                .name
                .as_ref()
                .ok_or("Story name is required (missing StoryTitle passage?)")?;

            let ifid = if story_data.ifid.is_empty() {
                return Err("IFID is required in StoryData".into());
            } else {
                &story_data.ifid
            };

            let start_passage = story_data
                .start
                .as_deref()
                .or_else(|| {
                    if passages.contains_key("Start") {
                        Some("Start")
                    } else {
                        passages.keys().next().map(|k| k.as_str())
                    }
                })
                .ok_or("No start passage found")?;

            let zoom = story_data.zoom.unwrap_or(1.0);

            let story_info = StoryInfo {
                name,
                ifid,
                format: &story_data.format,
                format_version: &story_data.format_version,
                start_passage,
                zoom,
            };
            let story_data_xml =
                Self::get_twine2_data_chunk(passages, &story_info, story_data, context.is_debug)?;

            let html = story_format
                .source
                .replace("{{STORY_NAME}}", name)
                .replace("{{STORY_DATA}}", &story_data_xml);

            Ok(html)
        } else {
            Err("No cached story format available".into())
        }
    }

    /// Generate Twine 2 data chunk following tweego format exactly
    fn get_twine2_data_chunk(
        passages: &IndexMap<String, Passage>,
        story_info: &StoryInfo,
        story_data: &StoryData,
        is_debug: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        let mut start_id = String::new();

        let mut scripts = Vec::new();
        let mut stylesheets = Vec::new();

        for passage in passages.values() {
            if let Some(ref tags) = passage.tags {
                let tags_list: Vec<&str> = tags.split_whitespace().collect();
                if tags_list.contains(&"script") {
                    scripts.push(passage);
                } else if tags_list.contains(&"stylesheet") {
                    stylesheets.push(passage);
                }
            }
        }

        debug!(
            "Found {} script passages and {} stylesheet passages",
            scripts.len(),
            stylesheets.len()
        );

        if !stylesheets.is_empty() {
            data.extend_from_slice(
                b"<style role=\"stylesheet\" id=\"twine-user-stylesheet\" type=\"text/twine-css\">",
            );
            if stylesheets.len() == 1 {
                data.extend_from_slice(stylesheets[0].content.as_bytes());
            } else {
                let mut pid = 1;
                for passage in &stylesheets {
                    if pid > 1 && !data.is_empty() && data[data.len() - 1] != b'\n' {
                        data.push(b'\n');
                    }
                    data.extend_from_slice(
                        format!("/* twine-user-stylesheet #{}: {:?} */\n", pid, passage.name)
                            .as_bytes(),
                    );
                    data.extend_from_slice(passage.content.as_bytes());
                    pid += 1;
                }
            }
            data.extend_from_slice(b"</style>");
        }

        if !scripts.is_empty() {
            data.extend_from_slice(
                b"<script role=\"script\" id=\"twine-user-script\" type=\"text/twine-javascript\">",
            );
            if scripts.len() == 1 {
                data.extend_from_slice(scripts[0].content.as_bytes());
            } else {
                let mut pid = 1;
                for passage in &scripts {
                    if pid > 1 && !data.is_empty() && data[data.len() - 1] != b'\n' {
                        data.push(b'\n');
                    }
                    data.extend_from_slice(
                        format!("/* twine-user-script #{}: {:?} */\n", pid, passage.name)
                            .as_bytes(),
                    );
                    data.extend_from_slice(passage.content.as_bytes());
                    pid += 1;
                }
            }
            data.extend_from_slice(b"</script>");
        }

        if let Some(tag_colors) = &story_data.tag_colors {
            for (tag, color) in tag_colors {
                data.extend_from_slice(
                    format!("<tw-tag name={:?} color={:?}></tw-tag>", tag, color).as_bytes(),
                );
            }
        }

        let mut pid = 1u32;
        let mut passage_start_id: Option<String> = None;

        for passage in passages.values() {
            if passage.name == "Start" {
                passage_start_id = Some(passage.name.clone());
            }

            if passage.name == "StoryTitle" || passage.name == "StoryData" {
                continue;
            }

            if let Some(ref tags) = passage.tags {
                let tags_list: Vec<&str> = tags.split_whitespace().collect();
                if tags_list.contains(&"script") || tags_list.contains(&"stylesheet") {
                    continue;
                }
            }

            let escaped_content = passage
                .content
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("\"", "&quot;")
                .replace("'", "&#39;");

            let tags = passage.tags.as_deref().unwrap_or("");
            let position = passage.position.as_deref().unwrap_or("");
            let size = passage.size.as_deref().unwrap_or("");

            data.extend_from_slice(
                format!(
                    "<tw-passagedata pid=\"{}\" name={:?} tags={:?} position={:?} size={:?}>{}</tw-passagedata>",
                    pid, passage.name, tags, position, size, escaped_content
                ).as_bytes()
            );

            if story_info.start_passage == passage.name {
                start_id = pid.to_string();
            }
            pid += 1;
        }

        if start_id.is_empty() {
            if let Some(passage_start_id) = passage_start_id {
                start_id = passage_start_id;
            } else {
                return Err("Start is required in StoryData".into());
            }
        }

        let options = if is_debug { "debug" } else { "" };

        let story_data_xml = std::format!(
            "<tw-storydata name={:?} startnode={:?} creator={:?} creator-version={:?} ifid={:?} zoom={:?} format={:?} format-version={:?} options={:?} hidden>{}</tw-storydata>",
            story_info.name,
            start_id,
            "TweeRS",
            "0.1.0",
            story_info.ifid,
            story_info.zoom.to_string(),
            story_info.format,
            story_info.format_version,
            options,
            String::from_utf8(data).map_err(|e| std::format!("UTF-8 conversion error: {}", e))?
        );

        Ok(story_data_xml)
    }
}
