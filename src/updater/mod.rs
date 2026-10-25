use serde::Deserialize;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

const REPO_OWNER: &str = "haydermuhib";
const REPO_NAME: &str = "SweepX";
const GITHUB_API_LATEST: &str = "https://api.github.com/repos/haydermuhib/SweepX/releases/latest";

#[derive(Debug, Clone, Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub size: Option<u64>,
    pub browser_download_url: String,
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
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    pub html_url: String,
    pub has_update: bool,
}

/// Selects the optimal release asset, strongly preferring compressed archives (.tar.gz) over raw binaries.
pub fn select_best_asset<'a>(
    assets: &'a [GithubReleaseAsset],
    arch_target: &str,
) -> Option<&'a GithubReleaseAsset> {
    // Priority 1: Compressed archive for target architecture (.tar.gz / .tgz)
    // Priority 2: Generic linux compressed archive (.tar.gz / .tgz)
    // Priority 3: Architecture-specific uncompressed binary
    // Priority 4: Generic binary named "sweepx"
    assets
        .iter()
        .find(|a| {
            a.name.contains(arch_target)
                && (a.name.ends_with(".tar.gz") || a.name.ends_with(".tgz"))
        })
        .or_else(|| {
            assets.iter().find(|a| {
                a.name.contains("linux")
                    && (a.name.ends_with(".tar.gz") || a.name.ends_with(".tgz"))
            })
        })
        .or_else(|| assets.iter().find(|a| a.name.contains(arch_target)))
        .or_else(|| assets.iter().find(|a| a.name == "sweepx"))
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
            asset_name: None,
            asset_size: None,
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

    let chosen_asset = select_best_asset(&release.assets, arch_target);

    let (download_url, asset_name, asset_size) = match chosen_asset {
        Some(a) => (Some(a.browser_download_url.clone()), Some(a.name.clone()), a.size),
        None => (None, None, None),
    };

    Ok(UpdateInfo {
        current_version,
        latest_version: clean_tag,
        release_notes: release.body.unwrap_or_default(),
        download_url,
        asset_name,
        asset_size,
        html_url: release.html_url,
        has_update,
    })
}

/// Downloads the latest release binary and atomically replaces the running executable.
pub async fn perform_self_update(download_url: &str) -> Result<PathBuf, String> {
    perform_self_update_with_progress(download_url, None, None).await
}

/// Downloads the latest release binary with real-time stream progress and atomically replaces the running executable.
pub async fn perform_self_update_with_progress(
    download_url: &str,
    _total_bytes: Option<u64>,
    progress_bar: Option<&indicatif::ProgressBar>,
) -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;

    let parent_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to find parent directory of executable".to_string())?;

    let temp_download = parent_dir.join(format!(".sweepx-update-{}", std::process::id()));

    // Stream download through curl pipe into temporary file with progress tracking
    let mut child = Command::new("curl")
        .args(["-fsSL", download_url])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn curl download: {}", e))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open curl stdout pipe".to_string())?;

    let mut temp_file = tokio::fs::File::create(&temp_download)
        .await
        .map_err(|e| format!("Failed to create temporary download file: {}", e))?;

    let mut buffer = [0u8; 65536];
    let mut downloaded: u64 = 0;

    loop {
        let bytes_read = stdout
            .read(&mut buffer)
            .await
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp_download);
                format!("Error reading download stream: {}", e)
            })?;

        if bytes_read == 0 {
            break;
        }

        temp_file
            .write_all(&buffer[..bytes_read])
            .await
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp_download);
                format!("Failed to write downloaded data to disk: {}", e)
            })?;

        downloaded += bytes_read as u64;
        if let Some(pb) = progress_bar {
            pb.set_position(downloaded);
        }
    }

    temp_file.flush().await.map_err(|e| {
        let _ = std::fs::remove_file(&temp_download);
        format!("Failed to flush temp file: {}", e)
    })?;
    drop(temp_file);

    let status = child.wait().await.map_err(|e| {
        let _ = std::fs::remove_file(&temp_download);
        format!("Failed waiting for curl download process: {}", e)
    })?;

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

    // Refresh application icon across standard directories, update desktop entry, and invalidate icon caches.
    // Execute the newly installed binary so it extracts and installs ITS newly embedded SVG icons and branding assets.
    let refreshed_via_child = Command::new(&current_exe)
        .arg("install-assets")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !refreshed_via_child {
        let _ = update_installed_assets();
    }

    Ok(current_exe)
}

/// Refreshes the application SVG icons and desktop entry in user directories.
pub fn update_installed_assets() -> Result<(), String> {
    if let Some(home) = dirs::home_dir() {
        install_assets_to(&home)
    } else {
        Err("Unable to locate user home directory".to_string())
    }
}

/// Installs latest embedded assets, creates index.theme if needed, clears stale raster caches, and writes desktop entry.
pub fn install_assets_to(home: &std::path::Path) -> Result<(), String> {
    const SWEEPX_ICON_SVG: &str = include_str!("../../assets/icons/sweepx.svg");

    let hicolor_dir = home.join(".local/share/icons/hicolor");
    let scalable_dir = hicolor_dir.join("scalable/apps");
    let _ = std::fs::create_dir_all(&scalable_dir);

    // Repair: Clean up any truncated index.theme or broken icon-theme.cache created by previous SweepX versions.
    // Truncated themes (listing only scalable/apps) override system hicolor themes and hide all raster PNG icons.
    let index_theme = hicolor_dir.join("index.theme");
    if index_theme.exists() {
        if let Ok(content) = std::fs::read_to_string(&index_theme) {
            if content.contains("Directories=scalable/apps") && content.lines().count() < 25 {
                let _ = std::fs::remove_file(&index_theme);
                let _ = std::fs::remove_file(hicolor_dir.join("icon-theme.cache"));
            }
        }
    }

    let icon_targets = [
        scalable_dir.join("sweepx.svg"),
        home.join(".local/share/icons/sweepx.svg"),
        home.join(".local/share/pixmaps/sweepx.svg"),
    ];

    for path in &icon_targets {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, SWEEPX_ICON_SVG);
    }

    // Clean up any stale legacy raster PNG icons that take precedence over scalable SVG in DE caches
    let legacy_pngs = [
        home.join(".local/share/icons/sweepx.png"),
        home.join(".local/share/pixmaps/sweepx.png"),
        hicolor_dir.join("48x48/apps/sweepx.png"),
        hicolor_dir.join("128x128/apps/sweepx.png"),
        hicolor_dir.join("256x256/apps/sweepx.png"),
    ];
    for stale in &legacy_pngs {
        if stale.exists() {
            let _ = std::fs::remove_file(stale);
        }
    }

    let apps_dir = home.join(".local/share/applications");
    let _ = std::fs::create_dir_all(&apps_dir);
    let desktop_path = apps_dir.join("sweepx.desktop");
    let icon_path = scalable_dir.join("sweepx.svg");

    let bin_path = if let Ok(current) = std::env::current_exe() {
        let s = current.to_string_lossy();
        if !s.contains("/target/") && !s.contains("/tmp/") {
            current
        } else {
            home.join(".local/bin/sweepx")
        }
    } else {
        home.join(".local/bin/sweepx")
    };

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=SweepX\n\
         Comment=Unified Linux Application Tracker & Deep Uninstaller\n\
         Exec={} gui\n\
         Icon={}\n\
         Categories=System;Utility;Settings;\n\
         Terminal=false\n\
         StartupNotify=true\n",
        bin_path.display(),
        icon_path.display()
    );
    let _ = std::fs::write(&desktop_path, desktop_content);

    // Trigger cache updates if utilities are present and index.theme exists
    if hicolor_dir.join("index.theme").exists() {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .args(["-f", "-t", hicolor_dir.to_str().unwrap_or("")])
            .output();
    }

    let _ = std::process::Command::new("update-desktop-database")
        .arg(apps_dir.to_str().unwrap_or(""))
        .output();

    let _ = std::process::Command::new("touch")
        .args([hicolor_dir.as_os_str(), apps_dir.as_os_str()])
        .output();

    Ok(())
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
