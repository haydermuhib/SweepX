use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use tokio::process::Command;

pub fn parse_snap_list_output(stdout: &str) -> Vec<Application> {
    let mut apps = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || idx == 0 {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0];
        if name.starts_with("core") || name == "bare" || name == "snapd" {
            continue;
        }

        let version = parts.get(1).map(|v| v.to_string());
        let mut app = Application::new(name, name, InstallMethod::Snap);
        app.version = version;

        if let Some(home) = dirs::home_dir() {
            let snap_data_path = home.join("snap").join(name);
            if snap_data_path.exists() {
                let size = crate::scanner::manual::calculate_dir_size(&snap_data_path);
                app.total_size_bytes += size;
                app.artifacts.push(AppArtifact::new(
                    ArtifactKind::SandboxDir,
                    snap_data_path,
                    size,
                    true,
                ));
            }
        }

        apps.push(app);
    }

    apps
}

pub async fn scan_snaps() -> Vec<Application> {
    if which::which("snap").is_err() {
        return Vec::new();
    }

    let output = Command::new("snap").arg("list").output().await;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_snap_list_output(&stdout)
        }
        _ => Vec::new(),
    }
}
