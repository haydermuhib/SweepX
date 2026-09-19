use crate::models::{Application, InstallMethod};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::process::Command;

pub struct AptManager;

impl AptManager {
    pub fn name(&self) -> &'static str {
        "APT / dpkg"
    }

    pub fn is_available(&self) -> bool {
        which::which("dpkg-query").is_ok() || which::which("apt").is_ok()
    }

    /// Fetches the set of manually installed packages by the user.
    pub async fn get_manual_installed_set() -> HashSet<String> {
        let output = Command::new("apt-mark").arg("showmanual").output().await;

        let mut manual_installed = HashSet::new();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let name = line.trim();
                    if !name.is_empty() {
                        manual_installed.insert(name.to_lowercase());
                    }
                }
            }
        }
        manual_installed
    }

    /// Parses output from `dpkg-query -W -f='${Package}\t${Version}\t${Installed-Size}\t${Section}\t${Description}\n'`
    pub fn parse_dpkg_query_output(
        stdout: &str,
        manual_installed: &HashSet<String>,
    ) -> Vec<Application> {
        let mut apps = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.is_empty() {
                continue;
            }

            let pkg_name = parts[0].trim();
            if pkg_name.is_empty() {
                continue;
            }

            let version = parts.get(1).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
            let size_kb: u64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
            let section = parts.get(3).map(|s| s.trim().to_string());
            let description = parts.get(4).map(|d| d.trim().lines().next().unwrap_or("").to_string());

            let name_lower = pkg_name.to_lowercase();
            let is_explicit = manual_installed.contains(&name_lower);

            let is_lib = name_lower.starts_with("lib")
                || name_lower.ends_with("-dev")
                || name_lower.ends_with("-data")
                || section
                    .as_deref()
                    .map(|s| {
                        s.contains("libs")
                            || s.contains("kernel")
                            || s.contains("oldlibs")
                            || s.contains("libdevel")
                    })
                    .unwrap_or(false);

            let is_system = !is_explicit || is_lib;

            let mut app = Application::new(pkg_name, pkg_name, InstallMethod::NativeApt);
            app.version = version;
            app.total_size_bytes = size_kb * 1024;
            app.description = description;
            app.is_system = is_system;

            apps.push(app);
        }

        apps
    }

    pub async fn list_installed(&self) -> Vec<Application> {
        if !self.is_available() {
            return Vec::new();
        }

        let (manual_installed, dpkg_output) = tokio::join!(
            Self::get_manual_installed_set(),
            Command::new("dpkg-query")
                .args(["-W", "-f=${Package}\t${Version}\t${Installed-Size}\t${Section}\t${Description}\n"])
                .output()
        );

        match dpkg_output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_dpkg_query_output(&stdout, &manual_installed)
            }
            _ => Vec::new(),
        }
    }

    pub async fn get_package_files(&self, pkg: &str) -> Vec<PathBuf> {
        let output = Command::new("dpkg").args(["-L", pkg]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .map(|l| PathBuf::from(l.trim()))
                    .filter(|p| p.is_file())
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub async fn get_dependencies(&self, pkg: &str) -> Vec<String> {
        let output = Command::new("apt-cache").args(["depends", pkg]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.starts_with("Depends:") {
                            Some(trimmed.trim_start_matches("Depends:").trim().to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
