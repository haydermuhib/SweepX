use crate::models::{Application, InstallMethod};
use crate::scanner::desktop_entry::scan_desktop_entries;
use crate::scanner::flatpak::scan_flatpaks;
use crate::scanner::manual::scan_manual_installations;
use crate::scanner::native::detect_available_package_managers;
use crate::scanner::snap::scan_snaps;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ScanStageBatch {
    Desktop(Vec<Application>),
    Manual(Vec<Application>),
    Snap(Vec<Application>),
    Flatpak(Vec<Application>),
    Native(Vec<Application>),
}

impl ScanStageBatch {
    pub fn stage_name(&self) -> &'static str {
        match self {
            Self::Desktop(_) => "Desktop Launchers",
            Self::Manual(_) => "Manual Apps & AppImages",
            Self::Snap(_) => "Snap Packages",
            Self::Flatpak(_) => "Flatpaks",
            Self::Native(_) => "Native Packages",
        }
    }

    pub fn apply_to(self, app_map: &mut HashMap<String, Application>) {
        apply_stage_batch(app_map, self);
    }
}

pub fn apply_stage_batch(app_map: &mut HashMap<String, Application>, batch: ScanStageBatch) {
    match batch {
        ScanStageBatch::Desktop(apps) => {
            for app in apps {
                app_map.insert(app.id.clone(), app);
            }
        }
        ScanStageBatch::Flatpak(apps) => {
            for app in apps {
                let matching_key = if app_map.contains_key(&app.id) {
                    Some(app.id.clone())
                } else {
                    app_map.keys().find(|k| {
                        let existing = &app_map[*k];
                        existing.id == app.id
                            || (app.id.contains('.') && existing.id == app.id.split('.').last().unwrap_or(""))
                    }).cloned()
                };

                if let Some(key) = matching_key {
                    let mut existing = app_map.remove(&key).unwrap();
                    existing.id = app.id.clone();
                    existing.install_method = InstallMethod::Flatpak;
                    existing.version = app.version.or(existing.version);
                    existing.total_size_bytes = app.total_size_bytes.max(existing.total_size_bytes);
                    for art in app.artifacts {
                        if !existing.artifacts.iter().any(|a| a.path == art.path) {
                            existing.artifacts.push(art);
                        }
                    }
                    app_map.insert(app.id.clone(), existing);
                } else {
                    app_map.insert(app.id.clone(), app);
                }
            }
        }
        ScanStageBatch::Snap(apps) => {
            for app in apps {
                let matching_key = if app_map.contains_key(&app.id) {
                    Some(app.id.clone())
                } else {
                    app_map
                        .keys()
                        .find(|k| {
                            let existing = &app_map[*k];
                            k.starts_with(&format!("{}_", app.id))
                                || existing.name.to_lowercase() == app.name.to_lowercase()
                                || existing.id == app.id
                        })
                        .cloned()
                };

                if let Some(key) = matching_key {
                    let mut existing = app_map.remove(&key).unwrap();
                    existing.id = app.id.clone();
                    existing.install_method = InstallMethod::Snap;
                    existing.version = app.version.or(existing.version);
                    existing.total_size_bytes = app.total_size_bytes.max(existing.total_size_bytes);
                    for art in app.artifacts {
                        if !existing.artifacts.iter().any(|a| a.path == art.path) {
                            existing.artifacts.push(art);
                        }
                    }
                    existing.is_system = app.is_system;
                    if existing.description.is_none() {
                        existing.description = app.description;
                    }
                    app_map.insert(app.id.clone(), existing);
                } else {
                    app_map.insert(app.id.clone(), app);
                }
            }
        }
        ScanStageBatch::Manual(apps) => {
            for app in apps {
                // Prevent duplicate entries where an existing desktop or container app already represents this program
                let app_name_clean = clean_normalized(&app.name);
                let app_id_clean = clean_normalized(&app.id);

                let already_exists = app_map.values().any(|existing| {
                    if existing.id == app.id {
                        return true;
                    }
                    if existing.exec_path.is_some() && existing.exec_path == app.exec_path {
                        return true;
                    }
                    let ex_id_short = existing.id.trim_end_matches(".desktop").rsplit('.').next().unwrap_or(&existing.id);
                    let ex_id_clean = clean_normalized(ex_id_short);
                    let ex_name_clean = clean_normalized(&existing.name);

                    (!app_id_clean.is_empty() && (app_id_clean == ex_id_clean || app_id_clean == ex_name_clean))
                        || (!app_name_clean.is_empty() && (app_name_clean == ex_name_clean || app_name_clean == ex_id_clean))
                });
                if !already_exists {
                    app_map.insert(app.id.clone(), app);
                }
            }
        }
        ScanStageBatch::Native(apps) => {
            // Index existing desktop, flatpak, snap, and manual applications before processing native packages.
            // This prevents the quadratic O(N * M) explosion and millions of allocations against system packages.
            let mut lookup_index: HashMap<String, String> = HashMap::new();
            let mut pre_existing_keys: Vec<String> = app_map.keys().cloned().collect();
            pre_existing_keys.sort();

            for key in &pre_existing_keys {
                if let Some(existing) = app_map.get(key) {
                    // Only index Desktop entries (CustomDesktop) that need native package enrichment.
                    // Container (Flatpak, Snap, Bottles) and manual apps must not be hijacked by native packages.
                    if existing.install_method != InstallMethod::CustomDesktop {
                        continue;
                    }

                    let key_clone = key.clone();
                    lookup_index.insert(existing.id.to_lowercase(), key_clone.clone());

                    let short_id = existing.id.rsplit('.').next().unwrap_or(&existing.id).to_lowercase();
                    lookup_index.insert(short_id.clone(), key_clone.clone());
                    lookup_index.insert(clean_normalized(&short_id), key_clone.clone());

                    if let Some(bin) = existing.exec_path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
                        let bin_lower = bin.to_lowercase();
                        lookup_index.insert(bin_lower.clone(), key_clone.clone());
                        lookup_index.insert(clean_normalized(&bin_lower), key_clone.clone());
                    }

                    let name_lower = existing.name.to_lowercase();
                    lookup_index.insert(name_lower.clone(), key_clone.clone());
                    lookup_index.insert(clean_normalized(&name_lower), key_clone.clone());
                }
            }

            for app in apps {
                let app_id_lower = app.id.to_lowercase();
                let app_id_clean = clean_normalized(&app_id_lower);
                let app_name_clean = clean_normalized(&app.name);

                let matching_key = if let Some(existing) = app_map.get(&app.id) {
                    if existing.install_method == InstallMethod::CustomDesktop {
                        Some(app.id.clone())
                    } else {
                        None
                    }
                } else if let Some(target) = lookup_index.get(&app_id_lower) {
                    Some(target.clone())
                } else if let Some(target) = lookup_index.get(&app_id_clean) {
                    Some(target.clone())
                } else if let Some(target) = lookup_index.get(&app_name_clean) {
                    Some(target.clone())
                } else {
                    // Fast prefix check against ONLY the pre-existing desktop entries
                    pre_existing_keys.iter().find(|k| {
                        if let Some(existing) = app_map.get(*k) {
                            if existing.install_method != InstallMethod::CustomDesktop {
                                return false;
                            }
                            let ex_short = existing.id.rsplit('.').next().unwrap_or(&existing.id);
                            let ex_clean = clean_normalized(ex_short);
                            if !ex_clean.is_empty() && (ex_clean.starts_with(&app_id_clean) || app_id_clean.starts_with(&ex_clean)) {
                                let diff = (ex_clean.len() as isize - app_id_clean.len() as isize).abs();
                                return diff <= 7;
                            }
                        }
                        false
                    }).cloned()
                };

                if let Some(key) = matching_key {
                    if let Some(mut existing) = app_map.remove(&key) {
                        existing.id = app.id.clone();
                        existing.install_method = app.install_method;
                        existing.version = app.version.or(existing.version);
                        existing.total_size_bytes = app.total_size_bytes.max(existing.total_size_bytes);
                        existing.description = existing.description.or(app.description);
                        existing.is_system = false;
                        if existing.exec_path.is_none() {
                            existing.exec_path = app.exec_path;
                        }
                        app_map.insert(app.id.clone(), existing);
                    }
                } else {
                    let mut sys_app = app;
                    sys_app.is_system = true;
                    let target_key = if app_map.contains_key(&sys_app.id) {
                        format!("native-{}", sys_app.id)
                    } else {
                        sys_app.id.clone()
                    };
                    app_map.insert(target_key, sys_app);
                }
            }
        }
    }
}

/// Normalizes a string by filtering out non-alphanumeric characters and converting to lowercase.
#[inline]
fn clean_normalized(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Scans all native package managers installed on the system.
pub async fn scan_native_packages() -> Vec<Application> {
    let native_managers = detect_available_package_managers();
    let mut native_apps = Vec::new();
    for pm in native_managers {
        let mut apps = pm.list_installed().await;
        native_apps.append(&mut apps);
    }
    native_apps
}

/// Concurrently scans and aggregates applications across all system packaging tiers.
pub async fn scan_all_applications() -> Vec<Application> {
    let (desktop_res, manual_res, flatpak_res, snap_res, native_apps) = tokio::join!(
        scan_desktop_entries(),
        scan_manual_installations(),
        scan_flatpaks(),
        scan_snaps(),
        scan_native_packages()
    );

    let mut app_map: HashMap<String, Application> = HashMap::new();
    apply_stage_batch(&mut app_map, ScanStageBatch::Desktop(desktop_res));
    apply_stage_batch(&mut app_map, ScanStageBatch::Flatpak(flatpak_res));
    apply_stage_batch(&mut app_map, ScanStageBatch::Snap(snap_res));
    apply_stage_batch(&mut app_map, ScanStageBatch::Manual(manual_res));
    apply_stage_batch(&mut app_map, ScanStageBatch::Native(native_apps));

    let mut result: Vec<Application> = app_map.into_values().collect();
    result.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    result
}
