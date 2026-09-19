use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Locations where desktop entries are typically found according to XDG spec.
pub fn get_desktop_entry_directories() -> Vec<PathBuf> {
    let mut dirs_list = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs_list.push(home.join(".local/share/applications"));
        // Flatpak desktop exports
        dirs_list.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    // Global flatpak & snap exports
    dirs_list.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs_list.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    dirs_list
}

/// Parses an INI-style `.desktop` file content.
pub fn parse_desktop_entry_content(content: &str, file_path: &Path) -> Option<Application> {
    let mut in_desktop_entry_section = false;
    let mut name = None;
    let mut display_name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut no_display = false;
    let mut entry_type = None;

    let mut snap_instance_name = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = &trimmed[1..trimmed.len() - 1];
            in_desktop_entry_section = section == "Desktop Entry";
            continue;
        }

        if !in_desktop_entry_section {
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim();

            match key {
                "Name" if name.is_none() => {
                    name = Some(val.to_string());
                    display_name = Some(val.to_string());
                }
                "GenericName" if display_name.as_deref() == name.as_deref() => {
                    if let Some(ref n) = name {
                        display_name = Some(format!("{} ({})", n, val));
                    }
                }
                "Exec" if exec.is_none() => {
                    exec = Some(val.to_string());
                }
                "Icon" if icon.is_none() => {
                    icon = Some(val.to_string());
                }
                "Comment" if comment.is_none() => {
                    comment = Some(val.to_string());
                }
                "X-SnapInstanceName" => {
                    snap_instance_name = Some(val.to_string());
                }
                "NoDisplay" => {
                    no_display = val.eq_ignore_ascii_case("true");
                }
                "Type" => {
                    entry_type = Some(val.to_string());
                }
                _ => {}
            }
        }
    }

    if no_display || entry_type.as_deref() == Some("Directory") {
        return None;
    }

    let app_name = name?;
    let file_stem = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| app_name.to_lowercase());

    // Determine install method based on path and exec
    let path_str = file_path.to_string_lossy();
    let install_method = if path_str.contains("flatpak") {
        InstallMethod::Flatpak
    } else if path_str.contains("snap") || snap_instance_name.is_some() {
        InstallMethod::Snap
    } else if let Some(ref e) = exec {
        if e.contains("/opt/") {
            InstallMethod::ManualOpt
        } else if e.to_lowercase().contains(".appimage") {
            InstallMethod::AppImage
        } else {
            InstallMethod::CustomDesktop
        }
    } else {
        InstallMethod::CustomDesktop
    };

    // Normalize app ID for snaps to match snap name (e.g. notepad-plus-plus)
    let app_id = if let Some(snap_name) = snap_instance_name {
        snap_name
    } else if install_method == InstallMethod::Snap {
        if let Some((first, _)) = file_stem.split_once('_') {
            first.to_string()
        } else {
            file_stem
        }
    } else {
        file_stem
    };

    let mut app = Application::new(app_id, app_name, install_method);
    app.display_name = display_name.unwrap_or_else(|| app.name.clone());
    app.description = comment;
    app.icon = icon;
    app.desktop_file = Some(file_path.to_path_buf());

    // Resolve executable binary
    if let Some(raw_exec) = exec {
        let clean_binary = extract_binary_from_exec(&raw_exec);
        let resolved_path = resolve_binary_path(&clean_binary);
        app.exec_path = resolved_path.clone();

        if let Some(bin_path) = resolved_path {
            if let Ok(meta) = std::fs::metadata(&bin_path) {
                app.total_size_bytes += meta.len();
                app.artifacts.push(AppArtifact::new(
                    ArtifactKind::Binary,
                    bin_path,
                    meta.len(),
                    true,
                ));
            }
        }
    }

    // Add desktop file artifact
    if let Ok(meta) = std::fs::metadata(file_path) {
        app.total_size_bytes += meta.len();
        app.artifacts.push(AppArtifact::new(
            ArtifactKind::DesktopEntry,
            file_path.to_path_buf(),
            meta.len(),
            true,
        ));
    }

    Some(app)
}

/// Cleans Exec field by removing XDG parameter codes (%f, %u, %F, %U, etc.) and quotes.
pub fn extract_binary_from_exec(exec: &str) -> String {
    let mut parts = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for ch in exec.chars() {
        match ch {
            '"' | '\'' => {
                in_quote = !in_quote;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    let first = parts.into_iter().next().unwrap_or_default();
    first
}

/// Resolves a binary command to its absolute PathBuf if it exists on disk or in PATH.
pub fn resolve_binary_path(binary: &str) -> Option<PathBuf> {
    if binary.is_empty() {
        return None;
    }

    let path = PathBuf::from(binary);
    if path.is_absolute() && path.exists() {
        return Some(path);
    }

    which::which(binary).ok()
}

/// Asynchronously scans all standard desktop entry directories on the host.
pub async fn scan_desktop_entries() -> Vec<Application> {
    let dirs = get_desktop_entry_directories();
    tokio::task::spawn_blocking(move || {
        let mut apps = Vec::new();
        let mut seen_ids = HashSet::new();

        for dir in dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "desktop") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Some(app) = parse_desktop_entry_content(&content, path) {
                            if seen_ids.insert(app.id.clone()) {
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }

        apps
    })
    .await
    .unwrap_or_default()
}
