use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppImageInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_integrated: bool,
    pub desktop_file: Option<PathBuf>,
}

/// Checks if an AppImage already has an associated .desktop entry in ~/.local/share/applications.
pub fn is_appimage_integrated(appimage_path: &Path) -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let apps_dir = home.join(".local/share/applications");
        if apps_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(apps_dir) {
                let path_str = appimage_path.to_string_lossy();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "desktop") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(&*path_str) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extracts the embedded .DirIcon from an AppImage and places it in ~/.local/share/icons.
pub fn extract_appimage_icon(appimage_path: &Path, clean_id: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let icons_dir = home.join(".local/share/icons");
    let _ = std::fs::create_dir_all(&icons_dir);

    // Try extracting .DirIcon into a temporary sandbox
    if let Ok(temp_dir) = tempfile::tempdir() {
        let output = std::process::Command::new(appimage_path)
            .args(["--appimage-extract", ".DirIcon"])
            .current_dir(temp_dir.path())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let extracted_icon = temp_dir.path().join("squashfs-root").join(".DirIcon");
                if extracted_icon.exists() {
                    let dest_icon = icons_dir.join(format!("appimagekit-{}.png", clean_id));
                    if std::fs::copy(&extracted_icon, &dest_icon).is_ok() {
                        return Some(dest_icon.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Integrates an AppImage by creating a desktop launcher entry in ~/.local/share/applications.
pub fn integrate_appimage(appimage_path: &Path, display_name: &str) -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Failed to resolve user home directory".to_string())?;
    let applications_dir = home.join(".local/share/applications");
    std::fs::create_dir_all(&applications_dir)
        .map_err(|e| format!("Failed to create applications directory: {}", e))?;

    let clean_id = display_name
        .to_lowercase()
        .replace(' ', "-")
        .replace(".appimage", "");

    let icon_value = extract_appimage_icon(appimage_path, &clean_id)
        .unwrap_or_else(|| "application-x-executable".to_string());

    let desktop_file_path = applications_dir.join(format!("appimagekit-{}.desktop", clean_id));

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Exec=\"{exec}\" %U\n\
         Icon={icon}\n\
         Comment=AppImage application integrated by SweepX\n\
         Categories=Utility;\n\
         Terminal=false\n\
         X-AppImage-Version=1.0\n",
        name = display_name,
        exec = appimage_path.display(),
        icon = icon_value
    );

    std::fs::write(&desktop_file_path, desktop_content)
        .map_err(|e| format!("Failed to write desktop file: {}", e))?;

    Ok(desktop_file_path)
}
