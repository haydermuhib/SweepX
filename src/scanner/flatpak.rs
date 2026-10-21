use crate::models::{AppArtifact, Application, ArtifactKind, InstallMethod};
use tokio::process::Command;

/// Parses stdout from `flatpak list --app --columns=name,application,version,size,installation,options`
pub fn parse_flatpak_list_output(stdout: &str) -> Vec<Application> {
    let mut apps = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let name = parts[0].trim();
        let app_id = parts[1].trim();
        if app_id.is_empty() {
            continue;
        }

        let version = parts.get(2).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let size_str = parts.get(3).map(|s| s.trim()).unwrap_or("");
        let parsed_size = parse_flatpak_size_string(size_str);

        let mut app = Application::new(app_id, name, InstallMethod::Flatpak);
        app.version = version;
        app.total_size_bytes = parsed_size;

        if let Some(home) = dirs::home_dir() {
            let sandbox_path = home.join(".var/app").join(app_id);
            if sandbox_path.exists() {
                let sandbox_size = crate::scanner::manual::calculate_dir_size(&sandbox_path);
                app.artifacts.push(AppArtifact::new(
                    ArtifactKind::SandboxDir,
                    sandbox_path,
                    sandbox_size,
                    true,
                ));
            }
        }

        apps.push(app);
    }

    apps
}

pub fn parse_flatpak_size_string(s: &str) -> u64 {
    let clean = s.replace('\u{00a0}', " ").trim().to_lowercase();
    if clean.is_empty() {
        return 0;
    }

    // Split numeric prefix and unit suffix
    let mut num_str = String::new();
    let mut unit_str = String::new();
    let mut found_unit = false;

    for ch in clean.chars() {
        if !found_unit && (ch.is_ascii_digit() || ch == '.' || ch == ',') {
            if ch == ',' {
                num_str.push('.');
            } else {
                num_str.push(ch);
            }
        } else if ch.is_alphabetic() {
            found_unit = true;
            unit_str.push(ch);
        }
    }

    let val: f64 = num_str.parse().unwrap_or(0.0);
    let unit = unit_str.trim();

    if unit.starts_with('t') {
        (val * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64
    } else if unit.starts_with('g') {
        (val * 1024.0 * 1024.0 * 1024.0) as u64
    } else if unit.starts_with('m') {
        (val * 1024.0 * 1024.0) as u64
    } else if unit.starts_with('k') {
        (val * 1024.0) as u64
    } else {
        val as u64
    }
}

pub async fn scan_flatpaks() -> Vec<Application> {
    if which::which("flatpak").is_err() {
        return Vec::new();
    }

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(4),
        Command::new("flatpak")
            .args([
                "list",
                "--app",
                "--columns=name,application,version,size,installation,options",
            ])
            .output(),
    )
    .await;

    match output {
        Ok(Ok(out)) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_flatpak_list_output(&stdout)
        }
        _ => Vec::new(),
    }
}
