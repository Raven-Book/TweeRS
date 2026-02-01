/// Context module for core-full
pub mod cache;
pub mod config;
pub mod format;

pub use cache::{FileCache, FileInfo};
pub use config::ConcreteBuildConfig;
pub use format::ConcreteFormatInfo;

use tweers_core::context::{BuildConfig, BuildContext as BuildContextTrait, FormatInfo};

/// Concrete build context implementation
#[derive(Clone)]
pub struct ConcreteBuildContext {
    config: ConcreteBuildConfig,
    format: ConcreteFormatInfo,
    cache: FileCache,
}

impl ConcreteBuildContext {
    pub fn new(is_debug: bool, base64: bool, start_passage: Option<String>) -> Self {
        Self {
            config: ConcreteBuildConfig::new(is_debug, base64, start_passage),
            format: ConcreteFormatInfo::new(String::new(), String::new()),
            cache: FileCache::new(base64),
        }
    }

    pub fn with_assets(is_debug: bool, base64: bool, assets_dirs: Vec<std::path::PathBuf>) -> Self {
        Self {
            config: ConcreteBuildConfig::new(is_debug, base64, None).with_assets(assets_dirs),
            format: ConcreteFormatInfo::new(String::new(), String::new()),
            cache: FileCache::new(base64),
        }
    }

    pub fn cache(&self) -> &FileCache {
        &self.cache
    }

    pub fn cache_mut(&mut self) -> &mut FileCache {
        &mut self.cache
    }

    pub fn format_mut(&mut self) -> &mut ConcreteFormatInfo {
        &mut self.format
    }
}

impl BuildContextTrait for ConcreteBuildContext {
    fn config(&self) -> &dyn BuildConfig {
        &self.config
    }

    fn format(&self) -> &dyn FormatInfo {
        &self.format
    }
}
