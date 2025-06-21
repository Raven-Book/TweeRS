use crate::config::constants;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs as async_fs;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{debug, info};

/// Check if the file is a support file
pub fn is_support_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        let ext_str = ext_str.as_str();
        return constants::TWEE_EXTENSIONS.contains(&ext_str)
            || ext_str == "js"
            || ext_str == "css";
    }
    false
}

/// Recursively collect all files from given paths
pub async fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, std::io::Error> {
    info!("Starting file collection, path count: {}", paths.len());
    for path in paths {
        debug!("Processing path: {:?}", path);
    }

    let mut set = JoinSet::new();
    let files = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    for path in paths {
        let path = path.clone();
        let files_clone = Arc::clone(&files);
        set.spawn(async move {
            let metadata = async_fs::metadata(&path).await?;
            if metadata.is_dir() {
                debug!("Processing directory: {:?}", path);
                process_path(path, files_clone).await?
            } else if is_support_file(&path) {
                debug!("Found support file: {:?}", path);
                files_clone.lock().await.push(path);
            }
            Ok::<_, std::io::Error>(())
        });
    }

    set.join_all().await;

    files.lock().await.sort();
    let result = Arc::try_unwrap(files).unwrap().into_inner();
    tracing::info!(
        "File collection completed, found {} support files",
        result.len()
    );
    Ok(result)
}

fn process_path(
    path: PathBuf,
    files: Arc<Mutex<Vec<PathBuf>>>,
) -> std::pin::Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> {
    Box::pin(async move {
        let metadata = tokio::fs::metadata(&path).await?;

        if metadata.is_file() {
            if is_support_file(&path) {
                debug!("Adding support file: {:?}", path);
                files.lock().await.push(path);
            }
        } else if metadata.is_dir() {
            debug!("Recursively processing directory: {:?}", path);
            process_directory(path, files).await?;
        }

        Ok(())
    })
}

fn process_directory(
    dir: PathBuf,
    files: Arc<Mutex<Vec<PathBuf>>>,
) -> std::pin::Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>> {
    Box::pin(async move {
        if let Some(dir_name) = dir.file_name() {
            if dir_name.to_string_lossy().starts_with('.') {
                debug!("Skipping hidden directory: {:?}", dir);
                return Ok(());
            }
        }

        debug!("Starting to process directory: {:?}", dir);
        let mut read_dir = tokio::fs::read_dir(&dir).await?;
        let mut child_tasks = JoinSet::new();
        let mut entry_count = 0;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            entry_count += 1;
            let files_clone = Arc::clone(&files);
            child_tasks.spawn(async move { process_path(path, files_clone).await });
        }

        debug!("Found {} entries in directory {:?}", entry_count, dir);

        while let Some(result) = child_tasks.join_next().await {
            result??;
        }

        debug!("Finished processing directory: {:?}", dir);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::debug;

    #[tokio::test]
    async fn test_collect_files() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = vec![
            project_root.join("test/story/Part 1"),
            project_root.join("test/story/Part 2"),
            project_root.join("test/story/A.twee"),
        ];

        debug!("Project root directory: {:?}", project_root);
        debug!("Story directory path: {:?}", paths);

        match collect_files(&paths).await {
            Ok(paths) => {
                debug!("Count of found files: {:?}", paths.len());
                debug!("Found files: {:?}", paths);
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
