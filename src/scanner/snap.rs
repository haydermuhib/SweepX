use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Identifies if a Snap package is a platform runtime, base system, or shared framework.
pub fn is_snap_runtime_or_dependency(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with("core")
        || n == "bare"
        || n == "snapd"
        || n.starts_with("gnome-")
        || n.starts_with("gtk-")
        || n.starts_with("mesa-")
        || n.starts_with("wine-platform")
        || n.starts_with("kf5-")
        || n.starts_with("kf6-")
        || n.starts_with("kde-frameworks")
        || n.starts_with("qt5-")
        || n.starts_with("qt6-")
        || n.ends_with("-runtime")
        || n.ends_with("-sdk")
}

/// Provides human-readable descriptions for shared Snap runtimes.
pub fn describe_snap_runtime(name: &str) -> Option<String> {
    let n = name.to_lowercase();
    if n.starts_with("wine-platform") {
        Some("Shared Snap Wine compatibility platform (Windows emulation runtime)".to_string())
    } else if n.starts_with("gnome-") || n.starts_with("gtk-") {
        Some("Shared Snap GNOME/GTK desktop application framework".to_string())
    } else if n.starts_with("mesa-") {
        Some("Shared Snap Mesa 3D graphics drivers & runtime".to_string())
    } else if n.starts_with("core") || n == "bare" || n == "snapd" {
        Some("Snap core base system environment".to_string())
    } else if n.starts_with("kf5-")
        || n.starts_with("kf6-")
        || n.starts_with("kde-frameworks")
        || n.starts_with("qt")
    {
        Some("Shared Snap KDE/Qt application framework".to_string())
    } else if n.ends_with("-sdk") {
        Some("Shared Snap development SDK runtime".to_string())
    } else if is_snap_runtime_or_dependency(name) {
        Some("Shared Snap platform runtime dependency".to_string())
    } else {
        None
    }
}

/// Finds all `.snap` squashfs files in `/var/lib/snapd/snaps/` belonging to this snap.
pub fn find_snap_package_files(snap_name: &str) -> Vec<(PathBuf, u64)> {
    let mut files = Vec::new();
    let snaps_dir = Path::new("/var/lib/snapd/snaps");
    if let Ok(entries) = std::fs::read_dir(snaps_dir) {
        let prefix = format!("{}_", snap_name);
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(&prefix) && file_name.ends_with(".snap") {
                    if let Ok(meta) = entry.metadata() {
                        files.push((path, meta.len()));
                    }
                }
            }
        }
    }
    files
}

pub fn parse_snap_list_output(stdout: &str) -> Vec<Application> {
    let mut apps = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || idx == 0 {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0];
        let version = parts.get(1).map(|v| v.to_string());
        let mut app = Application::new(name, name, InstallMethod::Snap);
        app.version = version;

        let is_runtime = is_snap_runtime_or_dependency(name);
        app.is_system = is_runtime;
        if is_runtime {
            app.description = describe_snap_runtime(name);
        }

        // 1. User sandbox & app config directory (~/snap/<name>)
        if let Some(home) = dirs::home_dir() {
            let snap_data_path = home.join("snap").join(name);
            if snap_data_path.exists() {
                let size = crate::scanner::manual::calculate_dir_size(&snap_data_path);
                app.total_size_bytes += size;
                app.artifacts.push(AppArtifact::new(
                    ArtifactKind::SandboxDir,
                    snap_data_path,
                    size,
                    true,
                ));
            }
        }

        // 2. Installed .snap package images (/var/lib/snapd/snaps/<name>_*.snap)
        for (pkg_path, pkg_size) in find_snap_package_files(name) {
            app.total_size_bytes += pkg_size;
            app.artifacts.push(AppArtifact::new(
                ArtifactKind::Binary,
                pkg_path,
                pkg_size,
                false,
            ));
        }

        apps.push(app);
    }

    apps
}

pub async fn scan_snaps() -> Vec<Application> {
    if which::which("snap").is_err() {
        return Vec::new();
    }

    let output = Command::new("snap").arg("list").output().await;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_snap_list_output(&stdout)
        }
        _ => Vec::new(),
    }
}
