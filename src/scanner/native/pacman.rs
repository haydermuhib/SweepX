use crate::models::{Application, InstallMethod};
use std::path::PathBuf;
use tokio::process::Command;

pub struct PacmanManager;

impl PacmanManager {
    pub fn name(&self) -> &'static str {
        "Pacman / ALPM"
    }

    pub fn is_available(&self) -> bool {
        which::which("pacman").is_ok()
    }

    /// Parses `pacman -Qi` output into Application structs
    pub fn parse_pacman_qi_output(stdout: &str) -> Vec<Application> {
        let mut apps = Vec::new();
        let mut current_name = None;
        let mut current_version = None;
        let mut current_desc = None;
        let mut current_size = 0;
        let mut current_is_explicit = false;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(name) = current_name.take() {
                    let mut app = Application::new(&name, &name, InstallMethod::NativePacman);
                    app.version = current_version.take();
                    app.description = current_desc.take();
                    app.total_size_bytes = current_size;
                    app.is_system = !current_is_explicit;
                    apps.push(app);
                }
                current_size = 0;
                current_is_explicit = false;
                continue;
            }

            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim();
                let val = val.trim();

                match key {
                    "Name" => current_name = Some(val.to_string()),
                    "Version" => current_version = Some(val.to_string()),
                    "Description" => current_desc = Some(val.to_string()),
                    "Installed Size" => {
                        current_size = crate::scanner::flatpak::parse_flatpak_size_string(val);
                    }
                    "Install Reason" => {
                        current_is_explicit = val.contains("Explicitly installed");
                    }
                    _ => {}
                }
            }
        }

        if let Some(name) = current_name {
            let mut app = Application::new(&name, &name, InstallMethod::NativePacman);
            app.version = current_version;
            app.description = current_desc;
            app.total_size_bytes = current_size;
            app.is_system = !current_is_explicit;
            apps.push(app);
        }

        apps
    }

    pub async fn list_installed(&self) -> Vec<Application> {
        if !self.is_available() {
            return Vec::new();
        }

        let output = Command::new("pacman").args(["-Qi"]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_pacman_qi_output(&stdout)
            }
            _ => Vec::new(),
        }
    }

    pub async fn get_package_files(&self, pkg: &str) -> Vec<PathBuf> {
        let output = Command::new("pacman").args(["-Ql", pkg]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|l| {
                        let parts: Vec<&str> = l.split_whitespace().collect();
                        parts.get(1).map(|p| PathBuf::from(p))
                    })
                    .filter(|p| p.is_file())
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub async fn get_dependencies(&self, pkg: &str) -> Vec<String> {
        let output = Command::new("pacman").args(["-Si", pkg]).output().await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if line.starts_with("Depends On") {
                        if let Some((_, deps)) = line.split_once(':') {
                            return deps
                                .split_whitespace()
                                .map(|s| s.to_string())
                                .filter(|s| s != "None")
                                .collect();
                        }
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
}
