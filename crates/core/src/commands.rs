// Commands module - moved to core-full
// This is a placeholder for core compilation

use crate::core::story::StoryFormat;

pub struct BuildContext {
    pub is_debug: bool,
    pub format_name: String,
    pub format_version: String,
    pub story_format: Option<StoryFormat>,
}

impl BuildContext {
    pub fn new(is_debug: bool) -> Self {
        Self {
            is_debug,
            format_name: String::new(),
            format_version: String::new(),
            story_format: None,
        }
    }
}
