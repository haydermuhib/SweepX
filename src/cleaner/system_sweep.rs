use crate::cleaner::SafetyValidator;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SweepCategory {
    SnapDisabledRevisions,
    FlatpakUnusedRuntimes,
    NativePackageCache,
    NativeOrphanDependencies,
    BrokenSymlinks,
    SystemUserCache,
}

impl SweepCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SnapDisabledRevisions => "Snap Disabled Revisions",
            Self::FlatpakUnusedRuntimes => "Flatpak Unused Runtimes",
            Self::NativePackageCache => "Native Package Cache",
            Self::NativeOrphanDependencies => "Orphaned Package Dependencies",
            Self::BrokenSymlinks => "Dangling & Broken Symlinks",
            Self::SystemUserCache => "Thumbnail & User Cache",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::SnapDisabledRevisions => "📦",
            Self::FlatpakUnusedRuntimes => "📦",
            Self::NativePackageCache => "💾",
            Self::NativeOrphanDependencies => "🔗",
            Self::BrokenSymlinks => "⛓️",
            Self::SystemUserCache => "🖼️",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemSweepItem {
    pub id: String,
    pub title: String,
    pub category: SweepCategory,
    pub description: String,
    pub reclaimable_bytes: u64,
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SystemSweepReport {
    pub freed_bytes: u64,
    pub items_cleaned: usize,
    pub errors: Vec<String>,
}

/// Parses `snap list --all` output to locate obsolete disabled revisions.
pub fn parse_snap_disabled_revisions(stdout: &str) -> Vec<SystemSweepItem> {
    let mut items = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || idx == 0 {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts[0];
        let version = parts[1];
        let rev = parts[2];
        let notes = parts.get(5).or_else(|| parts.get(4)).copied().unwrap_or("");

        if notes.contains("disabled") {
            let snap_file_name = format!("{}_{}.snap", name, rev);
            let snap_path = PathBuf::from("/var/lib/snapd/snaps").join(&snap_file_name);
            let size = std::fs::metadata(&snap_path).map(|m| m.len()).unwrap_or(0);

            items.push(SystemSweepItem {
                id: format!("snap-rev-{}-{}", name, rev),
                title: format!("Snap: {} (Rev {}, v{})", name, rev, version),
                category: SweepCategory::SnapDisabledRevisions,
                description: format!("Obsolete disabled snap revision retained in /var/lib/snapd/snaps/{}", snap_file_name),
                reclaimable_bytes: size,
                paths: vec![snap_path],
                command: Some(format!("snap remove {} --revision={}", name, rev)),
                selected: true,
            });
        }
    }

    items
}

/// Scans for package download caches across APT, DNF, Pacman.
pub fn scan_native_package_cache() -> Vec<SystemSweepItem> {
    let mut items = Vec::new();

    // 1. APT Cache
    let apt_cache = Path::new("/var/cache/apt/archives");
    if apt_cache.exists() {
        let mut size = 0u64;
        let mut deb_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(apt_cache) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "deb") {
                    if let Ok(meta) = entry.metadata() {
                        size += meta.len();
                        deb_files.push(path);
                    }
                }
            }
        }
        if size > 0 {
            items.push(SystemSweepItem {
                id: "cache-apt".to_string(),
                title: "APT Package Download Cache".to_string(),
                category: SweepCategory::NativePackageCache,
                description: format!("Downloaded .deb archives in /var/cache/apt/archives ({} packages)", deb_files.len()),
                reclaimable_bytes: size,
                paths: deb_files,
                command: Some("apt-get clean".to_string()),
                selected: true,
            });
        }
    }

    // 2. DNF Cache
    let dnf_cache = Path::new("/var/cache/dnf");
    if dnf_cache.exists() {
        let size = crate::scanner::manual::calculate_dir_size(dnf_cache);
        if size > 0 {
            items.push(SystemSweepItem {
                id: "cache-dnf".to_string(),
                title: "DNF Package Metadata & Download Cache".to_string(),
                category: SweepCategory::NativePackageCache,
                description: "Cached repository metadata and RPM packages in /var/cache/dnf".to_string(),
                reclaimable_bytes: size,
                paths: vec![dnf_cache.to_path_buf()],
                command: Some("dnf clean packages".to_string()),
                selected: true,
            });
        }
    }

    // 3. Pacman Cache
    let pacman_cache = Path::new("/var/cache/pacman/pkg");
    if pacman_cache.exists() {
        let size = crate::scanner::manual::calculate_dir_size(pacman_cache);
        if size > 0 {
            items.push(SystemSweepItem {
                id: "cache-pacman".to_string(),
                title: "Pacman Package Cache".to_string(),
                category: SweepCategory::NativePackageCache,
                description: "Cached .pkg.tar.zst packages in /var/cache/pacman/pkg".to_string(),
                reclaimable_bytes: size,
                paths: vec![pacman_cache.to_path_buf()],
                command: Some("pacman -Sc --noconfirm".to_string()),
                selected: true,
            });
        }
    }

    // 4. User Thumbnail Cache
    if let Some(cache_dir) = dirs::cache_dir() {
        let thumb_dir = cache_dir.join("thumbnails");
        if thumb_dir.exists() {
            let size = crate::scanner::manual::calculate_dir_size(&thumb_dir);
            if size > 1024 * 1024 {
                items.push(SystemSweepItem {
                    id: "cache-user-thumbnails".to_string(),
                    title: "User Desktop Thumbnail Cache".to_string(),
                    category: SweepCategory::SystemUserCache,
                    description: "Generated file and image preview thumbnails in ~/.cache/thumbnails/".to_string(),
                    reclaimable_bytes: size,
                    paths: vec![thumb_dir],
                    command: None,
                    selected: true,
                });
            }
        }
    }

    items
}

/// Discovers all sweepable items across system, container, and cache tiers.
pub async fn scan_all_sweep_items() -> Vec<SystemSweepItem> {
    let mut all_items = Vec::new();

    // 1. Snap disabled revisions
    if which::which("snap").is_ok() {
        if let Ok(out) = Command::new("snap").arg("list").arg("--all").output().await {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                all_items.extend(parse_snap_disabled_revisions(&stdout));
            }
        }
    }

    // 2. Flatpak unused runtimes check
    if which::which("flatpak").is_ok() {
        if let Ok(out) = Command::new("flatpak")
            .arg("list")
            .arg("--runtime")
            .output()
            .await
        {
            if out.status.success() {
                // If flatpak is present, check for unused runtimes
                // We provide the dedicated flatpak uninstall --unused action
                let stdout = String::from_utf8_lossy(&out.stdout);
                if !stdout.trim().is_empty() {
                    // Item for Flatpak unused runtimes
                    all_items.push(SystemSweepItem {
                        id: "flatpak-unused-runtimes".to_string(),
                        title: "Flatpak Unused Runtimes & Extensions".to_string(),
                        category: SweepCategory::FlatpakUnusedRuntimes,
                        description: "Prune all orphaned Flatpak runtime platforms no longer used by installed applications".to_string(),
                        reclaimable_bytes: 0, // Computed dynamically by flatpak on prune
                        paths: vec![],
                        command: Some("flatpak uninstall --unused -y".to_string()),
                        selected: false,
                    });
                }
            }
        }
    }

    // 3. Package caches and user caches
    all_items.extend(scan_native_package_cache());

    // 4. Broken & Dangling Symlinks in ~/.local/bin, /usr/local/bin, and application launchers
    let broken_symlinks = crate::scanner::symlink_graph::find_broken_symlinks();
    for sym in broken_symlinks {
        let file_name = sym
            .link_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| sym.link_path.display().to_string());

        all_items.push(SystemSweepItem {
            id: format!("broken-symlink-{}", sym.link_path.display()),
            title: format!("Broken Symlink: {}", file_name),
            category: SweepCategory::BrokenSymlinks,
            description: format!(
                "Dangling symlink at {} pointing to missing target: {}",
                sym.link_path.display(),
                sym.target_path.display()
            ),
            reclaimable_bytes: sym.size_bytes,
            paths: vec![sym.link_path],
            command: None,
            selected: true,
        });
    }

    all_items
}

/// Executes optimization for selected sweep items.
pub async fn execute_system_sweep(items: &[SystemSweepItem]) -> SystemSweepReport {
    let mut report = SystemSweepReport::default();

    for item in items.iter().filter(|i| i.selected) {
        // If a dedicated command is configured (e.g. `snap remove ...` or `apt-get clean`)
        if let Some(ref cmd_str) = item.command {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if !parts.is_empty() {
                let program = parts[0];
                let args = &parts[1..];

                let cmd_fut = async {
                    if program == "snap" || program == "apt-get" || program == "dnf" || program == "pacman" {
                        crate::cleaner::run_elevated_command(program, args).await.is_ok()
                    } else {
                        Command::new(program)
                            .args(args)
                            .output()
                            .await
                            .map(|o| o.status.success())
                            .unwrap_or(false)
                    }
                };

                let success = match timeout(Duration::from_secs(60), cmd_fut).await {
                    Ok(res) => res,
                    Err(_) => {
                        report.errors.push(format!("Command timed out after 60s for {}", item.title));
                        false
                    }
                };

                if success {
                    report.freed_bytes += item.reclaimable_bytes;
                    report.items_cleaned += 1;
                } else {
                    report.errors.push(format!("Command failed for {}", item.title));
                }
            }
        } else {
            // Direct file deletion for cache paths with SafetyValidator validation
            let mut file_success = true;
            for path in &item.paths {
                if let Err(violation) = SafetyValidator::is_path_safe_to_delete(path) {
                    file_success = false;
                    report.errors.push(format!(
                        "Safety check blocked cleaning {}: {}",
                        path.display(),
                        violation
                    ));
                    continue;
                }

                if path.is_dir() {
                    if let Err(e) = std::fs::remove_dir_all(path) {
                        file_success = false;
                        report.errors.push(format!("Failed to clean {}: {}", path.display(), e));
                    } else {
                        // Recreate empty cache directory so desktop environment retains the folder
                        let _ = std::fs::create_dir_all(path);
                    }
                } else if path.is_file() {
                    if let Err(e) = std::fs::remove_file(path) {
                        file_success = false;
                        report.errors.push(format!("Failed to delete {}: {}", path.display(), e));
                    }
                }
            }
            if file_success {
                report.freed_bytes += item.reclaimable_bytes;
                report.items_cleaned += 1;
            }
        }
    }

    report
}
