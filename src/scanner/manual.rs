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

        // 2. Scan standard user binary locations and dynamic user-space $PATH directories
        let custom_scan_dirs = get_user_binary_scan_dirs();

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
                    let is_prog = is_executable_program(path);

                    if (is_appimage || is_prog) && is_file_executable(path) {
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
    // System package managers never own files inside user home directories
    if let Some(home) = dirs::home_dir() {
        if path.starts_with(&home) {
            return false;
        }
    }

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

/// Checks if a file starts with a script shebang (#!).
fn has_shebang(path: &Path) -> bool {
    use std::io::Read;
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut magic = [0u8; 2];
        if file.read_exact(&mut magic).is_ok() {
            return magic == [b'#', b'!'];
        }
    }
    false
}

/// Checks if a file is an executable program (ELF, AppImage, or shebang script).
fn is_executable_program(path: &Path) -> bool {
    if !is_file_executable(path) {
        return false;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if is_forbidden_extension(file_name) {
        return false;
    }
    is_appimage_binary(path) || is_elf_binary(path) || has_shebang(path)
}

/// Recursively calculates the complete byte size of a directory.
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

/// Searches top level of an /opt directory or user suite for an executable binary.
fn find_candidate_binary_in_dir(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_executable_program(&path) {
                return Some(path);
            }
        }
    }
    None
}

/// Dynamically extracts user-space scan directories from standard locations and $PATH.
pub fn get_user_binary_scan_dirs() -> Vec<PathBuf> {
    extract_user_binary_scan_dirs(std::env::var_os("PATH"))
}

/// Helper function to parse user-space binary directories from an optional PATH OsString without hardcoding.
pub fn extract_user_binary_scan_dirs(path_var: Option<std::ffi::OsString>) -> Vec<PathBuf> {
    let mut scan_dirs = Vec::new();
    let mut seen = HashSet::new();

    let home = dirs::home_dir();

    // 1. Standard user binary locations
    if let Some(ref h) = home {
        for standard in &[h.join(".local/bin"), h.join("Applications"), h.join("bin")] {
            if standard.exists() && seen.insert(standard.clone()) {
                scan_dirs.push(standard.clone());
            }
        }
    }
    let usr_local = PathBuf::from("/usr/local/bin");
    if usr_local.exists() && seen.insert(usr_local.clone()) {
        scan_dirs.push(usr_local);
    }

    // 2. Dynamic ingestion of user-space directories from $PATH
    if let Some(path_str) = path_var {
        for p in std::env::split_paths(&path_str) {
            if !p.exists() || !p.is_dir() {
                continue;
            }
            let s = p.to_string_lossy();

            // Exclude system binaries, container wrapper paths, and editor/agent internals
            if s.starts_with("/bin")
                || s == "/usr/bin"
                || s.starts_with("/sbin")
                || s.starts_with("/usr/sbin")
                || s.contains("/snap/")
                || s.contains("/flatpak/")
                || s.contains(".gemini")
                || s.contains(".vscode")
                || s.contains(".vscode-server")
                || s.contains(".antigravity")
            {
                continue;
            }

            // Only include user-owned or local directories ($HOME or /usr/local)
            let is_user_path = home.as_ref().map(|h| p.starts_with(h)).unwrap_or(false)
                || s.starts_with("/usr/local");

            if is_user_path && seen.insert(p.clone()) {
                scan_dirs.push(p);
            }
        }
    }

    scan_dirs
}

