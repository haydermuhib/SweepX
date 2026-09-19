use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub app_id: String,
    pub app_name: String,
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub created_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FilesystemSnapshot {
    pub file_entries: HashMap<PathBuf, (u64, u64)>, // Path -> (size_bytes, mtime_secs)
}

/// Standard paths monitored during an installation command.
pub fn get_monitored_install_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/opt"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/local/share"),
    ];

    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".local/bin"));
        roots.push(home.join(".local/share"));
        roots.push(home.join(".local/opt"));
        roots.push(home.join(".config"));
        roots.push(home.join("Applications"));
    }

    roots
}

impl FilesystemSnapshot {
    /// Captures a point-in-time filesystem state across all monitored roots.
    pub fn capture() -> Self {
        let roots = get_monitored_install_roots();
        let mut file_entries = HashMap::new();

        for root in roots {
            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(&root)
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path().to_path_buf();
                if let Ok(meta) = entry.metadata() {
                    let size = meta.len();
                    let mtime = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    file_entries.insert(path, (size, mtime));
                }
            }
        }

        Self { file_entries }
    }

    /// Computes the diff between this pre-install snapshot and a post-install snapshot.
    pub fn diff(
        &self,
        post: &FilesystemSnapshot,
        app_id: &str,
        app_name: &str,
        command: &str,
    ) -> InstallManifest {
        let mut created_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut total_size_bytes = 0u64;

        for (path, &(size, mtime)) in &post.file_entries {
            match self.file_entries.get(path) {
                None => {
                    // Newly created file
                    created_files.push(path.clone());
                    total_size_bytes += size;
                }
                Some(&(_old_size, old_mtime)) if mtime > old_mtime => {
                    // Modified existing file
                    modified_files.push(path.clone());
                }
                _ => {}
            }
        }

        created_files.sort();
        modified_files.sort();

        InstallManifest {
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            command: command.to_string(),
            timestamp: Utc::now(),
            created_files,
            modified_files,
            total_size_bytes,
        }
    }
}
