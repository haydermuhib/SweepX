use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scans `/opt`, user binaries in `~/.local/bin`, `~/Applications`, `~/bin`, `/usr/local/bin`, and detects AppImages and standalone binaries.
pub async fn scan_manual_installations() -> Vec<Application> {
    tokio::task::spawn_blocking(|| {
        let mut apps = Vec::new();
        let mut seen_paths = HashSet::new();

        // 1. Scan /opt directory for installed software suites
        let opt_dir = Path::new("/opt");
        if opt_dir.exists() && opt_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(opt_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }

                        let size = calculate_dir_size(&path);
                        let id = format!("opt-{}", name.to_lowercase());

                        if seen_paths.insert(path.clone()) {
                            let mut app = Application::new(&id, &name, InstallMethod::ManualOpt);
                            app.total_size_bytes = size;
                            app.artifacts.push(AppArtifact::new(
                                ArtifactKind::AppDir,
                                path.clone(),
                                size,
                                true,
                            ));

                            if let Some(exec) = find_candidate_binary_in_dir(&path) {
                                app.exec_path = Some(exec);
                            }

                            apps.push(app);
                        }
                    }
                }
            }
        }

        // 2. Scan standard standalone binaries, user CLI tools, and AppImages
        let mut custom_scan_dirs = vec![PathBuf::from("/usr/local/bin")];
        if let Some(home) = dirs::home_dir() {
            custom_scan_dirs.push(home.join(".local/bin"));
            custom_scan_dirs.push(home.join("Applications"));
            custom_scan_dirs.push(home.join("bin"));
        }

        for dir in custom_scan_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir).max_depth(2).into_iter().flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if file_name.starts_with('.') {
                        continue;
                    }

                    let is_appimage = file_name.to_lowercase().ends_with(".appimage");
                    let is_executable = is_file_executable(path);

                    if (is_appimage || is_executable) && seen_paths.insert(path.to_path_buf()) {
                        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                        let clean_name = if is_appimage {
                            file_name
                                .trim_end_matches(".AppImage")
                                .trim_end_matches(".appimage")
                                .to_string()
                        } else {
                            file_name.clone()
                        };

                        let method = if is_appimage {
                            InstallMethod::AppImage
                        } else {
                            InstallMethod::ManualOpt
                        };

                        let mut app = Application::new(
                            clean_name.to_lowercase(),
                            &clean_name,
                            method,
                        );
                        app.total_size_bytes = size;
                        app.exec_path = Some(path.to_path_buf());
                        app.artifacts.push(AppArtifact::new(
                            ArtifactKind::Binary,
                            path.to_path_buf(),
                            size,
                            true,
                        ));

                        // Attach associated user config, cache, and data directories if they exist
                        if let Some(home) = dirs::home_dir() {
                            let config_path = home.join(".config").join(&clean_name);
                            if config_path.exists() {
                                let c_size = calculate_dir_size(&config_path);
                                app.total_size_bytes += c_size;
                                app.artifacts.push(AppArtifact::new(
                                    ArtifactKind::ConfigDir,
                                    config_path,
                                    c_size,
                                    true,
                                ));
                            }

                            let data_path = home.join(".local/share").join(&clean_name);
                            if data_path.exists() {
                                let d_size = calculate_dir_size(&data_path);
                                app.total_size_bytes += d_size;
                                app.artifacts.push(AppArtifact::new(
                                    ArtifactKind::DataDir,
                                    data_path,
                                    d_size,
                                    true,
                                ));
                            }

                            let cache_path = home.join(".cache").join(&clean_name);
                            if cache_path.exists() {
                                let ca_size = calculate_dir_size(&cache_path);
                                app.total_size_bytes += ca_size;
                                app.artifacts.push(AppArtifact::new(
                                    ArtifactKind::CacheDir,
                                    cache_path,
                                    ca_size,
                                    true,
                                ));
                            }
                        }

                        apps.push(app);
                    }
                }
            }
        }

        apps
    })
    .await
    .unwrap_or_default()
}

/// Recursively calculates the byte size of a directory.
pub fn calculate_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Checks if a file has execute permissions.
fn is_file_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            return meta.permissions().mode() & 0o111 != 0;
        }
    }
    false
}

/// Searches top level of an /opt directory for an executable binary.
fn find_candidate_binary_in_dir(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_file_executable(&path) {
                return Some(path);
            }
        }
    }
    None
}
