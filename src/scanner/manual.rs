use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File extensions that must NEVER be registered as executable applications.
const FORBIDDEN_EXTENSIONS: &[&str] = &[
    "tar.gz", "tgz", "tar.xz", "txz", "tar.bz2", "tbz2", "tar.zst", "tar", "zip",
    "7z", "rar", "gz", "xz", "bz2", "zst", "iso", "deb", "rpm", "apk", "flatpakref",
    "sh", "bash", "zsh", "py", "pyc", "pl", "rb", "js", "ts", "json", "yaml", "yml",
    "xml", "html", "htm", "txt", "md", "log", "bak", "old", "orig", "png", "jpg",
    "jpeg", "svg", "ico", "lock", "db", "sqlite", "sqlite3"
];

/// Scans `/opt`, user binaries in `~/.local/bin`, `~/Applications`, `~/bin`, `/usr/local/bin`, and detects AppImages and standalone binaries.
pub async fn scan_manual_installations() -> Vec<Application> {
    tokio::task::spawn_blocking(|| {
        let mut apps = Vec::new();
        let mut seen_paths = HashSet::new();

        // 1. Scan /opt directory for unmanaged software suites
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

                        // Check if this /opt directory or its children are owned by RPM, DPKG, or Pacman
                        if is_path_owned_by_package_manager(&path) {
                            continue;
                        }

                        let size = calculate_dir_size(&path);
                        if size == 0 {
                            continue;
                        }

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

                    // Check extension blacklist
                    if is_forbidden_extension(&file_name) {
                        continue;
                    }

                    let is_appimage = file_name.to_lowercase().ends_with(".appimage") || is_appimage_binary(path);
                    let is_elf = is_elf_binary(path);

                    if (is_appimage || is_elf) && is_file_executable(path) {
                        // Skip if owned by package manager
                        if is_path_owned_by_package_manager(path) {
                            continue;
                        }

                        if seen_paths.insert(path.to_path_buf()) {
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
        }

        apps
    })
    .await
    .unwrap_or_default()
}

/// Checks if a file or directory is owned by RPM, DPKG, or Pacman.
pub fn is_path_owned_by_package_manager(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // 1. Check RPM (Fedora, RHEL, CentOS, openSUSE)
    if which::which("rpm").is_ok() {
        if let Ok(output) = std::process::Command::new("rpm")
            .args(["-qf", &path_str])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("is not owned by any package") && !stdout.trim().is_empty() {
                    return true;
                }
            }
        }

        // If directory, check immediate children
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten().take(5) {
                    let child_str = entry.path().to_string_lossy().to_string();
                    if let Ok(output) = std::process::Command::new("rpm")
                        .args(["-qf", &child_str])
                        .output()
                    {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if !stdout.contains("is not owned by any package") && !stdout.trim().is_empty() {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check DPKG (Ubuntu, Debian, Mint)
    if which::which("dpkg").is_ok() {
        if let Ok(output) = std::process::Command::new("dpkg")
            .args(["-S", &path_str])
            .output()
        {
            if output.status.success() {
                return true;
            }
        }

        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten().take(5) {
                    let child_str = entry.path().to_string_lossy().to_string();
                    if let Ok(output) = std::process::Command::new("dpkg")
                        .args(["-S", &child_str])
                        .output()
                    {
                        if output.status.success() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 3. Check Pacman (Arch Linux, Manjaro)
    if which::which("pacman").is_ok() {
        if let Ok(output) = std::process::Command::new("pacman")
            .args(["-Qo", &path_str])
            .output()
        {
            if output.status.success() {
                return true;
            }
        }
    }

    false
}

/// Checks if a file name has an extension on the forbidden blacklist.
fn is_forbidden_extension(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    for ext in FORBIDDEN_EXTENSIONS {
        if lower.ends_with(&format!(".{}", ext)) {
            return true;
        }
    }
    false
}

/// Checks ELF binary magic bytes (0x7F 'E' 'L' 'F').
fn is_elf_binary(path: &Path) -> bool {
    use std::io::Read;
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut magic = [0u8; 4];
        if file.read_exact(&mut magic).is_ok() {
            return magic == [0x7f, b'E', b'L', b'F'];
        }
    }
    false
}

/// Checks AppImage type 1 or type 2 magic bytes.
fn is_appimage_binary(path: &Path) -> bool {
    use std::io::Read;
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut header = [0u8; 12];
        if file.read_exact(&mut header).is_ok() {
            // AppImage Type 2: 0x7F 'E' 'L' 'F' ... 'A' 'I' 0x02
            if header[0..4] == [0x7f, b'E', b'L', b'F'] && header[8..11] == [b'A', b'I', 0x02] {
                return true;
            }
        }
    }
    false
}

/// Recursively calculates the byte size of a directory with depth limit.
pub fn calculate_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    WalkDir::new(path)
        .max_depth(5)
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
            if path.is_file() && is_file_executable(&path) && is_elf_binary(&path) {
                return Some(path);
            }
        }
    }
    None
}
