use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymlinkRecord {
    pub link_path: PathBuf,
    pub target_path: PathBuf,
    pub is_broken: bool,
    pub size_bytes: u64,
}

/// Standard directories where symlinks to binaries and desktop files are located.
pub fn get_symlink_scan_directories() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/usr/local/bin")];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join("bin"));
    }

    dirs
}

/// Scans standard PATH and application launcher directories for symlinks and detects broken links.
pub fn scan_symlink_graph() -> Vec<SymlinkRecord> {
    let scan_dirs = get_symlink_scan_directories();
    let mut records = Vec::new();

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(sym_meta) = std::fs::symlink_metadata(&path) {
                    if sym_meta.file_type().is_symlink() {
                        if let Ok(target) = std::fs::read_link(&path) {
                            let abs_target = if target.is_absolute() {
                                target.clone()
                            } else {
                                path.parent().map(|p| p.join(&target)).unwrap_or(target.clone())
                            };

                            let is_broken = !abs_target.exists();
                            let size = sym_meta.len();

                            records.push(SymlinkRecord {
                                link_path: path,
                                target_path: abs_target,
                                is_broken,
                                size_bytes: size,
                            });
                        }
                    }
                }
            }
        }
    }

    records
}

/// Finds all symlinks across the system pointing into a specific application root directory (e.g. `/opt/myapp`).
pub fn find_symlinks_pointing_to_app(app_root: &Path) -> Vec<PathBuf> {
    let all_symlinks = scan_symlink_graph();
    let normalized_root = crate::cleaner::normalize_path(app_root);

    all_symlinks
        .into_iter()
        .filter(|sym| {
            let norm_target = crate::cleaner::normalize_path(&sym.target_path);
            norm_target.starts_with(&normalized_root) || norm_target == normalized_root
        })
        .map(|sym| sym.link_path)
        .collect()
}

/// Returns all broken symlinks found on the host.
pub fn find_broken_symlinks() -> Vec<SymlinkRecord> {
    scan_symlink_graph()
        .into_iter()
        .filter(|sym| sym.is_broken)
        .collect()
}
