use crate::models::{Application, InstallMethod};
use std::path::PathBuf;
use tokio::process::Command;

pub struct DnfManager;

impl DnfManager {
    pub fn name(&self) -> &'static str {
        "DNF / RPM"
    }

    pub fn is_available(&self) -> bool {
        which::which("rpm").is_ok() || which::which("dnf").is_ok()
    }

    /// Parses output from `rpm -qa --queryformat '%{NAME}\t%{VERSION}-%{RELEASE}\t%{SIZE}\t%{GROUP}\t%{SUMMARY}\n'`
    pub fn parse_rpm_query_output(stdout: &str) -> Vec<Application> {
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
            let size_bytes: u64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
            let group = parts.get(3).map(|s| s.trim().to_string());
            let summary = parts.get(4).map(|s| s.trim().to_string());

            let is_system = group
                .as_deref()
                .map(|g| g.contains("System") || g.contains("Kernel") || g.contains("Libraries"))
                .unwrap_or(false);

            let mut app = Application::new(pkg_name, pkg_name, InstallMethod::NativeDnf);
            app.version = version;
            app.total_size_bytes = size_bytes;
            app.description = summary;
            app.is_system = is_system;

            apps.push(app);
        }

        apps
    }

    pub async fn list_installed(&self) -> Vec<Application> {
        if !self.is_available() {
            return Vec::new();
        }

        let output = Command::new("rpm")
            .args([
                "-qa",
                "--queryformat",
                "%{NAME}\t%{VERSION}-%{RELEASE}\t%{SIZE}\t%{GROUP}\t%{SUMMARY}\n",
            ])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_rpm_query_output(&stdout)
            }
            _ => Vec::new(),
        }
    }

    pub async fn get_package_files(&self, pkg: &str) -> Vec<PathBuf> {
        let output = Command::new("rpm").args(["-ql", pkg]).output().await;

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
        let output = Command::new("rpm").args(["-q", "--requires", pkg]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
