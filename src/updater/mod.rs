use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;

const REPO_OWNER: &str = "haydermuhib";
const REPO_NAME: &str = "SweepX";
const GITHUB_API_LATEST: &str = "https://api.github.com/repos/haydermuhib/SweepX/releases/latest";

#[derive(Debug, Clone, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: Option<String>,
    pub html_url: String,
    pub has_update: bool,
}

/// Queries GitHub Releases API to check if a newer version of SweepX is available.
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            "User-Agent: SweepX-App",
            "-H",
            "Accept: application/vnd.github.v3+json",
            GITHUB_API_LATEST,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute curl to check updates: {}", e))?;

    if !output.status.success() {
        return Err("Failed to fetch release information from GitHub".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() || stdout.contains("\"message\": \"Not Found\"") {
        return Ok(UpdateInfo {
            current_version: current_version.clone(),
            latest_version: current_version,
            release_notes: String::new(),
            download_url: None,
            html_url: format!("https://github.com/{}/{}", REPO_OWNER, REPO_NAME),
            has_update: false,
        });
    }

    let release: GithubReleaseResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse GitHub release JSON: {}", e))?;

    let clean_tag = release.tag_name.trim_start_matches('v').to_string();
    let has_update = is_newer_version(&clean_tag, &current_version);

    // Look for matching architecture asset
    let arch_target = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.contains(arch_target) || a.name.contains("linux") || a.name == "sweepx")
        .map(|a| a.browser_download_url.clone());

    Ok(UpdateInfo {
        current_version,
        latest_version: clean_tag,
        release_notes: release.body.unwrap_or_default(),
        download_url,
        html_url: release.html_url,
        has_update,
    })
}

/// Downloads the latest release binary and atomically replaces the running executable.
pub async fn perform_self_update(download_url: &str) -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;

    let parent_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to find parent directory of executable".to_string())?;

    let temp_download = parent_dir.join(format!(".sweepx-update-{}", std::process::id()));

    // Download with curl
    let status = Command::new("curl")
        .args(["-fsSL", "-o", temp_download.to_str().unwrap(), download_url])
        .status()
        .await
        .map_err(|e| format!("Failed to download update binary: {}", e))?;

    if !status.success() {
        let _ = std::fs::remove_file(&temp_download);
        return Err("Download failed with non-zero exit code".to_string());
    }

    // If the download is a tar.gz archive, extract the sweepx binary
    if download_url.ends_with(".tar.gz") || download_url.ends_with(".tgz") {
        let extract_dir = parent_dir.join(format!(".sweepx-extract-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&extract_dir);
        let tar_status = Command::new("tar")
            .args(["-xzf", temp_download.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
            .status()
            .await;
        
        let _ = std::fs::remove_file(&temp_download);

        if let Ok(status) = tar_status {
            if status.success() {
                let extracted_binary = extract_dir.join("sweepx");
                if extracted_binary.exists() {
                    let _ = std::fs::rename(&extracted_binary, &temp_download);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&extract_dir);

        if !temp_download.exists() {
            return Err("Failed to extract binary from downloaded archive".to_string());
        }
    }

    // Set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&temp_download) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&temp_download, perms);
        }
    }

    // Replace current executable
    let backup_path = parent_dir.join(format!(".sweepx-old-{}", std::process::id()));
    if std::fs::rename(&current_exe, &backup_path).is_ok() {
        if let Err(e) = std::fs::rename(&temp_download, &current_exe) {
            // Rollback on failure
            let _ = std::fs::rename(&backup_path, &current_exe);
            let _ = std::fs::remove_file(&temp_download);
            return Err(format!("Failed to install update binary: {}", e));
        }
        let _ = std::fs::remove_file(&backup_path);
    } else {
        // Direct write / replace fallback
        std::fs::copy(&temp_download, &current_exe)
            .map_err(|e| format!("Failed to replace executable: {}", e))?;
        let _ = std::fs::remove_file(&temp_download);
    }

    Ok(current_exe)
}

/// Simple semver comparison helper (e.g. "0.2.0" > "0.1.0").
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_parts = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok())
            .collect()
    };

    let latest_parts = parse_parts(latest);
    let current_parts = parse_parts(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}
