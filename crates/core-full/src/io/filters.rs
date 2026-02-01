use crate::io::is_support_file_with_base64;
/// File filter implementations
use std::path::Path;

/// File filter trait
pub trait FileFilter: Send + Sync {
    fn should_include(&self, path: &Path) -> bool;
}

/// Support file filter (for build operations)
pub struct SupportFileFilter {
    base64: bool,
}

impl SupportFileFilter {
    pub fn new(base64: bool) -> Self {
        Self { base64 }
    }
}

impl FileFilter for SupportFileFilter {
    fn should_include(&self, path: &Path) -> bool {
        is_support_file_with_base64(path, self.base64)
    }
}
