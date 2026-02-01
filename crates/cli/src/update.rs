use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::info;

/// GitHub Release API response structure
#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Update command: automatically download and replace tweers binary from GitHub releases
pub async fn update_command(
    repo_api_url: String,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Checking for TweeRS updates...");

    let client = reqwest::Client::new();

    let response = client
        .get(&repo_api_url)
        .header("User-Agent", "TweeRS-Updater")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to fetch release info: {} - {}", status, error_body).into());
    }

    let response_text = response.text().await?;
    let release: GithubRelease = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse release info: {}. Response body: {}",
            e,
            if response_text.len() > 500 {
                format!("{}...", &response_text[..500])
            } else {
                response_text.clone()
            }
        )
    })?;

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = release
        .tag_name
        .trim_start_matches("tweers-cli-v")
        .trim_start_matches('v');

    info!("Current version: {}", current_version);
    info!("Latest version: {}", latest_version);

    if !force && current_version == latest_version {
        info!("Already running the latest version!");
        return Ok(());
    }

    let platform_suffix = if cfg!(target_os = "windows") {
        "windows-x86_64.zip"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-arm64.tar.gz"
        } else {
            "macos-x86_64.tar.gz"
        }
    } else if cfg!(target_os = "linux") {
        "linux-x86_64.tar.gz"
    } else {
        return Err("Unsupported platform for auto-update".into());
    };

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(platform_suffix))
        .ok_or("No suitable release asset found for current platform")?;

    info!("Downloading {} ({} bytes)", asset.name, asset.size);

    let download_response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "TweeRS-Updater")
        .send()
        .await?;

    if !download_response.status().is_success() {
        return Err(format!("Failed to download asset: {}", download_response.status()).into());
    }

    let archive_data = download_response.bytes().await?;
    let current_exe = std::env::current_exe()?;

    let temp_dir = std::env::temp_dir().join(format!("tweers_update_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    if asset.name.ends_with(".zip") {
        extract_zip(&archive_data, &temp_dir)?;
    } else if asset.name.ends_with(".tar.gz") {
        extract_tar_gz(&archive_data, &temp_dir)?;
    } else {
        return Err("Unsupported archive format".into());
    }

    let new_exe_name = if cfg!(target_os = "windows") {
        "tweers.exe"
    } else {
        "tweers"
    };

    let extracted_exe = find_executable(&temp_dir, new_exe_name)?;

    let backup_path = current_exe.with_extension("old");
    if backup_path.exists() {
        std::fs::remove_file(&backup_path)?;
    }
    std::fs::rename(&current_exe, &backup_path)?;

    std::fs::copy(&extracted_exe, &current_exe)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&current_exe, perms)?;
    }

    std::fs::remove_dir_all(&temp_dir)?;

    info!("Successfully updated to version {}", latest_version);
    info!("Old version backed up as: {}", backup_path.display());
    info!("Restart required to use the new version.");

    Ok(())
}

/// Extract ZIP archive to target directory
fn extract_zip(
    data: &[u8],
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = target_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(outpath)?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Extract TAR.GZ archive to target directory
#[cfg(unix)]
fn extract_tar_gz(
    data: &[u8],
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let cursor = Cursor::new(data);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    archive.unpack(target_dir)?;

    Ok(())
}

/// Extract TAR.GZ archive to target directory (Windows stub)
#[cfg(windows)]
fn extract_tar_gz(
    _data: &[u8],
    _target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("TAR.GZ extraction not supported on Windows - this should not be called".into())
}

/// Find executable file in directory recursively
fn find_executable(
    dir: &Path,
    exe_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_name().is_some_and(|n| n == exe_name) {
            return Ok(path);
        } else if path.is_dir()
            && let Ok(found) = find_executable(&path, exe_name)
        {
            return Ok(found);
        }
    }

    Err(format!("Executable {exe_name} not found in extracted files").into())
}
