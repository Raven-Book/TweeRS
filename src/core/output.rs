use crate::core::story::{Passage, StoryData, StoryFormat};
pub struct HtmlOutputHandler;

impl HtmlOutputHandler {
    pub fn generate_html(passages: &[Passage], story_data: &Option<StoryData>) -> Result<String, Box<dyn std::error::Error>> {
        let data = story_data.as_ref()
            .ok_or("StoryData passage is required")?;

        let name = if let Some (title) = &data.name {
            title.clone()
        } else {
            return Err("Story name is required (missing StoryTitle passage?)".into());

        };

        let ifid = if data.ifid.is_empty() {
            return Err("IFID is required in StoryData".into());
        } else {
            data.ifid.clone()
        };

        let format = if data.format.is_empty() {
            return Err("Format is required in StoryData".into());
        } else {
            data.format.clone()
        };

        let format_version = if data.format_version.is_empty() {
            return Err("Format version is required in StoryData".into());
        } else {
            data.format_version.clone()
        };

        let start_passage = data.start.as_ref()
            .map(|s| s.clone())
            .or_else(|| passages.first().map(|p| p.name.clone()))
            .ok_or("No start passage found (either specify 'start' in StoryData or provide at least one passage)")?;

        let zoom = data.zoom.unwrap_or(1.0);

        let story_format = StoryFormat::find_format(&format, &format_version)?;

        let story_data_xml = Self::get_twine2_data_chunk(
            passages,
            &name,
            &ifid,
            &format,
            &format_version,
            &start_passage,
            zoom,
            data
        )?;

        let html = story_format.source
            .replace("{{STORY_NAME}}", &name)
            .replace("{{STORY_DATA}}", &story_data_xml);

        Ok(html)
    }

    /// Generate Twine 2 data chunk following tweego format exactly
    fn get_twine2_data_chunk(
        passages: &[Passage],
        story_name: &str,
        ifid: &str,
        format: &str,
        format_version: &str,
        start_passage: &str,
        zoom: f32,
        story_data: &StoryData
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        let mut start_id = String::new();

        let mut scripts = Vec::new();
        let mut stylesheets = Vec::new();

        for passage in passages {
            if let Some(ref tags) = passage.tags {
                let tags_list: Vec<&str> = tags.split_whitespace().collect();
                if tags_list.contains(&"script") {
                    scripts.push(passage);
                } else if tags_list.contains(&"stylesheet") {
                    stylesheets.push(passage);
                }
            }
        }

        if !stylesheets.is_empty() {
            data.extend_from_slice(b"<style role=\"stylesheet\" id=\"twine-user-stylesheet\" type=\"text/twine-css\">");
            if stylesheets.len() == 1 {
                data.extend_from_slice(stylesheets[0].content.as_bytes());
            } else {
                let mut pid = 1;
                for passage in &stylesheets {
                    if pid > 1 && !data.is_empty() && data[data.len()-1] != b'\n' {
                        data.push(b'\n');
                    }
                    data.extend_from_slice(format!("/* twine-user-stylesheet #{}: {:?} */\n", pid, passage.name).as_bytes());
                    data.extend_from_slice(passage.content.as_bytes());
                    pid += 1;
                }
            }
            data.extend_from_slice(b"</style>");
        }

        if !scripts.is_empty() {
            data.extend_from_slice(b"<script role=\"script\" id=\"twine-user-script\" type=\"text/twine-javascript\">");
            if scripts.len() == 1 {
                data.extend_from_slice(scripts[0].content.as_bytes());
            } else {
                let mut pid = 1;
                for passage in &scripts {
                    if pid > 1 && !data.is_empty() && data[data.len()-1] != b'\n' {
                        data.push(b'\n');
                    }
                    data.extend_from_slice(format!("/* twine-user-script #{}: {:?} */\n", pid, passage.name).as_bytes());
                    data.extend_from_slice(passage.content.as_bytes());
                    pid += 1;
                }
            }
            data.extend_from_slice(b"</script>");
        }

        if let Some(tag_colors) = &story_data.tag_colors {
            for (tag, color) in tag_colors {
                data.extend_from_slice(format!("<tw-tag name={:?} color={:?}></tw-tag>", tag, color).as_bytes());
            }
        }

        let mut pid = 1u32;

        let mut passage_start_id: Option<String> = None;

        for passage in passages {
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

            // Escape passage content
            let escaped_content = passage.content
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("\"", "&quot;")
                .replace("'", "&#39;");

            let tags = passage.tags.as_ref().map(|s| s.as_str()).unwrap_or("");
            let position = passage.position.as_ref().map(|s| s.as_str()).unwrap_or("");
            let size = passage.size.as_ref().map(|s| s.as_str()).unwrap_or("");

            data.extend_from_slice(
                format!(
                    "<tw-passagedata pid=\"{}\" name={:?} tags={:?} position={:?} size={:?}>{}</tw-passagedata>",
                    pid, passage.name, tags, position, size, escaped_content
                ).as_bytes()
            );

            if start_passage == passage.name {
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

        let options = "";

        let story_data_xml = format!(
            "<tw-storydata name={:?} startnode={:?} creator={:?} creator-version={:?} ifid={:?} zoom={:?} format={:?} format-version={:?} options={:?} hidden>{}</tw-storydata>",
            story_name,
            start_id,
            "TweeRS",
            "0.1.0",
            ifid,
            zoom.to_string(),
            format,
            format_version,
            options,
            String::from_utf8(data).map_err(|e| format!("UTF-8 conversion error: {}", e))?
        );

        Ok(story_data_xml)
    }
}
