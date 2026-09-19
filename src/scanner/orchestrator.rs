use crate::models::{Application, InstallMethod};
use crate::scanner::desktop_entry::scan_desktop_entries;
use crate::scanner::flatpak::scan_flatpaks;
use crate::scanner::manual::scan_manual_installations;
use crate::scanner::native::detect_available_package_managers;
use crate::scanner::snap::scan_snaps;
use std::collections::HashMap;

/// Concurrently scans and aggregates applications across all system packaging tiers.
pub async fn scan_all_applications() -> Vec<Application> {
    let (desktop_res, manual_res, flatpak_res, snap_res) = tokio::join!(
        scan_desktop_entries(),
        scan_manual_installations(),
        scan_flatpaks(),
        scan_snaps()
    );

    // Concurrently query active native package managers
    let native_managers = detect_available_package_managers();
    let mut native_apps = Vec::new();
    for pm in native_managers {
        let mut apps = pm.list_installed().await;
        native_apps.append(&mut apps);
    }

    // Merge and deduplicate by App ID and executable path
    let mut app_map: HashMap<String, Application> = HashMap::new();

    // 1. Add Desktop Entries first (provides display names, icons, descriptions)
    for app in desktop_res {
        app_map.insert(app.id.clone(), app);
    }

    // 2. Merge Flatpaks (enrich or insert)
    for app in flatpak_res {
        if let Some(existing) = app_map.get_mut(&app.id) {
            existing.install_method = InstallMethod::Flatpak;
            existing.version = app.version.or(existing.version.clone());
            existing.total_size_bytes = existing.total_size_bytes.max(app.total_size_bytes);
            existing.artifacts.extend(app.artifacts);
        } else {
            app_map.insert(app.id.clone(), app);
        }
    }

    // 3. Merge Snaps
    for app in snap_res {
        let matching_key = if app_map.contains_key(&app.id) {
            Some(app.id.clone())
        } else {
            app_map
                .keys()
                .find(|k| {
                    k.starts_with(&format!("{}_", app.id))
                        || (app_map.get(*k).map_or(false, |a| {
                            a.install_method == InstallMethod::Snap
                                && (a.id.contains(&app.id) || a.name.to_lowercase() == app.name.to_lowercase())
                        }))
                })
                .cloned()
        };

        if let Some(key) = matching_key {
            let mut existing = app_map.remove(&key).unwrap();
            existing.id = app.id.clone();
            existing.install_method = InstallMethod::Snap;
            existing.version = app.version.or(existing.version);
            existing.total_size_bytes += app.total_size_bytes;
            existing.artifacts.extend(app.artifacts);
            existing.is_system = app.is_system;
            if existing.description.is_none() {
                existing.description = app.description;
            }
            app_map.insert(app.id.clone(), existing);
        } else {
            app_map.insert(app.id.clone(), app);
        }
    }

    // 4. Merge Manual installations (/opt, AppImages)
    for app in manual_res {
        if !app_map.contains_key(&app.id) {
            app_map.insert(app.id.clone(), app);
        }
    }

    // 5. Merge Native packages (if matching desktop entry, enrich; otherwise add user packages)
    for app in native_apps {
        if let Some(existing) = app_map.get_mut(&app.id) {
            existing.install_method = app.install_method;
            existing.version = app.version.or(existing.version.clone());
            existing.total_size_bytes = existing.total_size_bytes.max(app.total_size_bytes);
            existing.description = existing.description.clone().or(app.description);
            existing.is_system = app.is_system;
        } else if !app.is_system {
            // Include non-system native packages
            app_map.insert(app.id.clone(), app);
        }
    }

    let mut result: Vec<Application> = app_map.into_values().collect();
    result.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    result
}
