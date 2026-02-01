use super::filters::FileFilter;
/// Generic file collector with filter support
use std::path::{Path, PathBuf};
use tweers_core::error::Result;

pub struct FileCollector<F: FileFilter> {
    filter: F,
}

impl<F: FileFilter> FileCollector<F> {
    pub fn new(filter: F) -> Self {
        Self { filter }
    }

    /// Async file collection
    pub async fn collect_async(&self, sources: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for source in sources {
            if source.is_file() {
                if self.filter.should_include(source) {
                    files.push(source.clone());
                }
            } else if source.is_dir() {
                self.collect_dir_async(source, &mut files).await?;
            }
        }

        Ok(files)
    }

    fn collect_dir_async<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    self.collect_dir_async(&path, files).await?;
                } else if self.filter.should_include(&path) {
                    files.push(path);
                }
            }

            Ok(())
        })
    }
}
