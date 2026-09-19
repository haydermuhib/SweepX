use crate::models::{Application, InstallMethod};
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

    /// Parses output from `dpkg-query -W -f='${Package}\t${Version}\t${Installed-Size}\t${Section}\t${Description}\n'`
    pub fn parse_dpkg_query_output(stdout: &str) -> Vec<Application> {
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

            let is_system = section
                .as_deref()
                .map(|s| {
                    s.contains("libs")
                        || s.contains("kernel")
                        || s.contains("oldlibs")
                        || s.contains("libdevel")
                })
                .unwrap_or(false);

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

        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Package}\t${Version}\t${Installed-Size}\t${Section}\t${Description}\n"])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_dpkg_query_output(&stdout)
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
