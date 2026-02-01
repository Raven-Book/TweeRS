/// Format information implementation
use tweers_core::context::FormatInfo;
use tweers_core::core::story::StoryFormat;

#[derive(Clone, Debug)]
pub struct ConcreteFormatInfo {
    pub name: String,
    pub version: String,
    pub story_format: Option<StoryFormat>,
}

impl ConcreteFormatInfo {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            story_format: None,
        }
    }

    pub fn with_format(mut self, story_format: StoryFormat) -> Self {
        self.story_format = Some(story_format);
        self
    }

    pub fn set_format(&mut self, story_format: StoryFormat) {
        self.story_format = Some(story_format);
    }
}

impl FormatInfo for ConcreteFormatInfo {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn story_format(&self) -> Option<&StoryFormat> {
        self.story_format.as_ref()
    }
}
