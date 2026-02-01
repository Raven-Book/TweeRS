/// Build configuration implementation
use std::path::PathBuf;
use tweers_core::context::BuildConfig;

#[derive(Clone, Debug)]
pub struct ConcreteBuildConfig {
    pub is_debug: bool,
    pub base64: bool,
    pub start_passage: Option<String>,
    pub assets_dirs: Vec<PathBuf>,
}

impl ConcreteBuildConfig {
    pub fn new(is_debug: bool, base64: bool, start_passage: Option<String>) -> Self {
        Self {
            is_debug,
            base64,
            start_passage,
            assets_dirs: Vec::new(),
        }
    }

    pub fn with_assets(mut self, assets_dirs: Vec<PathBuf>) -> Self {
        self.assets_dirs = assets_dirs;
        self
    }
}

impl BuildConfig for ConcreteBuildConfig {
    fn is_debug(&self) -> bool {
        self.is_debug
    }

    fn base64(&self) -> bool {
        self.base64
    }

    fn start_passage(&self) -> Option<&str> {
        self.start_passage.as_deref()
    }

    fn assets_dirs(&self) -> &[PathBuf] {
        &self.assets_dirs
    }
}
