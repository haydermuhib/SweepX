use std::path::{Path, PathBuf};

/// Detailed classification of why a path cannot be safely deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyViolation {
    EmptyPath,
    RelativePath,
    SystemRoot,
    UserHomeRoot,
    SystemDirectory(String),
    ProtectedXdgBase(String),
    InsufficientPathDepth,
    SymlinkEscape,
    DoesNotExist,
}

impl std::fmt::Display for SafetyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyViolation::EmptyPath => write!(f, "Target path is empty"),
            SafetyViolation::RelativePath => write!(f, "Target path must be an absolute path"),
            SafetyViolation::SystemRoot => write!(f, "Cannot delete root directory '/'"),
            SafetyViolation::UserHomeRoot => write!(f, "Cannot delete user home directory"),
            SafetyViolation::SystemDirectory(dir) => {
                write!(f, "Target path is a protected system directory: {}", dir)
            }
            SafetyViolation::ProtectedXdgBase(dir) => {
                write!(f, "Target path is a protected base directory: {}", dir)
            }
            SafetyViolation::InsufficientPathDepth => {
                write!(f, "Target path has insufficient depth to delete safely")
            }
            SafetyViolation::SymlinkEscape => {
                write!(f, "Target symlink escapes safe containment bounds")
            }
            SafetyViolation::DoesNotExist => write!(f, "Target path does not exist"),
        }
    }
}

/// Critical system directories that must NEVER be deleted directly as a whole.
const PROTECTED_SYSTEM_DIRS: &[&str] = &[
    "/",
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/home",
    "/lib",
    "/lib32",
    "/lib64",
    "/lost+found",
    "/media",
    "/mnt",
    "/opt",
    "/proc",
    "/root",
    "/run",
    "/sbin",
    "/srv",
    "/sys",
    "/tmp",
    "/usr",
    "/usr/bin",
    "/usr/include",
    "/usr/lib",
    "/usr/lib64",
    "/usr/local",
    "/usr/local/bin",
    "/usr/local/lib",
    "/usr/local/share",
    "/usr/sbin",
    "/usr/share",
    "/usr/share/applications",
    "/usr/share/icons",
    "/usr/src",
    "/var",
    "/var/cache",
    "/var/lib",
    "/var/local",
    "/var/lock",
    "/var/log",
    "/var/mail",
    "/var/opt",
    "/var/run",
    "/var/spool",
    "/var/tmp",
    "/var/lib/flatpak",
    "/var/lib/snapd",
    "/snap",
    "/snap/bin",
    "/usr/bin/flatpak",
    "/usr/bin/snap",
];

pub struct SafetyValidator;

impl SafetyValidator {
    /// Validates if a path is safe to delete or move to trash.
    pub fn is_path_safe_to_delete(path: &Path) -> Result<(), SafetyViolation> {
        let path_str = path.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err(SafetyViolation::EmptyPath);
        }

        if !path.is_absolute() {
            return Err(SafetyViolation::RelativePath);
        }

        // Canonicalize or normalize path representation
        let normalized = normalize_path(path);
        let normalized_str = normalized.to_string_lossy();

        // 1. Root check
        if normalized_str == "/" {
            return Err(SafetyViolation::SystemRoot);
        }

        // 2. Protected system directories check
        for &protected in PROTECTED_SYSTEM_DIRS {
            if normalized_str == protected || normalized_str == format!("{}/", protected) {
                return Err(SafetyViolation::SystemDirectory(protected.to_string()));
            }
        }

        // 3. User home directory checks
        if let Some(home_dir) = dirs::home_dir() {
            let norm_home = normalize_path(&home_dir);
            if normalized == norm_home {
                return Err(SafetyViolation::UserHomeRoot);
            }

            // Protected base XDG directories check (e.g. ~/.config itself, ~/.cache itself)
            let protected_user_bases = [
                norm_home.join(".config"),
                norm_home.join(".cache"),
                norm_home.join(".cache/flatpak"),
                norm_home.join(".local"),
                norm_home.join(".local/share"),
                norm_home.join(".local/share/flatpak"),
                norm_home.join(".local/share/flatpak/repo"),
                norm_home.join(".local/state"),
                norm_home.join(".local/bin"),
                norm_home.join(".local/share/applications"),
                norm_home.join(".local/share/icons"),
                norm_home.join(".var"),
                norm_home.join(".var/app"),
                norm_home.join("snap"),
                norm_home.join("Desktop"),
                norm_home.join("Documents"),
                norm_home.join("Downloads"),
                norm_home.join("Music"),
                norm_home.join("Pictures"),
                norm_home.join("Videos"),
            ];

            for base in &protected_user_bases {
                if normalized == *base {
                    return Err(SafetyViolation::ProtectedXdgBase(
                        base.to_string_lossy().to_string(),
                    ));
                }
            }
        }

        // 4. Component depth check: Must have at least 2 components from root (e.g. /opt/app or /home/user/.config/app)
        let components: Vec<_> = normalized.components().collect();
        if components.len() <= 2 {
            return Err(SafetyViolation::InsufficientPathDepth);
        }

        Ok(())
    }

    /// Sanitizes a list of candidate paths, partitioning into safe deletion targets and rejected violations.
    pub fn sanitize_deletion_targets(
        paths: &[PathBuf],
    ) -> (Vec<PathBuf>, Vec<(PathBuf, SafetyViolation)>) {
        let mut safe_paths = Vec::new();
        let mut rejected = Vec::new();

        for path in paths {
            match Self::is_path_safe_to_delete(path) {
                Ok(()) => safe_paths.push(path.clone()),
                Err(violation) => rejected.push((path.clone(), violation)),
            }
        }

        (safe_paths, rejected)
    }
}

/// Helper to normalize path components (resolves `.` and `..` without requiring the path to exist on disk).
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}
