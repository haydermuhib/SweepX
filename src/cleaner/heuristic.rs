use super::safety::SafetyValidator;
use super::signatures::find_signature;
use crate::models::{Application, ArtifactKind, ResidualCandidate};
use crate::scanner::manual::calculate_dir_size;
use std::collections::HashSet;
use std::path::PathBuf;

/// Discovers leftover or associated configuration, cache, and data directories for an application.
pub async fn discover_residuals_for_app(app: &Application) -> Vec<ResidualCandidate> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let mut candidates = Vec::new();
        let mut seen_paths = HashSet::new();

        let home_dir = match dirs::home_dir() {
            Some(h) => h,
            None => return candidates,
        };

        let config_base = home_dir.join(".config");
        let cache_base = home_dir.join(".cache");
        let data_base = home_dir.join(".local/share");
        let state_base = home_dir.join(".local/state");
        let local_bin_base = home_dir.join(".local/bin");
        let desktop_base = home_dir.join(".local/share/applications");

        // 1. Signature-based matching
        let search_terms = vec![
            app_clone.id.as_str(),
            app_clone.name.as_str(),
            app_clone.display_name.as_str(),
        ];

        for term in &search_terms {
            if let Some(sig) = find_signature(term) {
                for &dir in sig.config_dirs {
                    let p = config_base.join(dir);
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        p,
                        ArtifactKind::ConfigDir,
                        0.98,
                    );
                }

                for &dir in sig.cache_dirs {
                    let p = cache_base.join(dir);
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        p,
                        ArtifactKind::CacheDir,
                        0.98,
                    );
                }

                for &dir in sig.data_dirs {
                    let p = data_base.join(dir);
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        p,
                        ArtifactKind::DataDir,
                        0.95,
                    );
                }

                for &rel in sig.custom_user_paths {
                    let p = home_dir.join(rel);
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        p,
                        ArtifactKind::DataDir,
                        0.92,
                    );
                }
            }
        }

        // 2. Heuristic name, reverse-DNS, and slug matching across all user directories
        let slugs = extract_slug_variations(&app_clone);
        let base_dirs = [
            (&config_base, ArtifactKind::ConfigDir),
            (&cache_base, ArtifactKind::CacheDir),
            (&data_base, ArtifactKind::DataDir),
            (&state_base, ArtifactKind::StateDir),
            (&local_bin_base, ArtifactKind::Binary),
        ];

        for (base_dir, kind) in &base_dirs {
            if !base_dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = entry.file_name().to_string_lossy().to_lowercase();

                    for slug in &slugs {
                        if file_name == *slug
                            || file_name.replace('-', "") == slug.replace('-', "")
                            || file_name.replace('_', "") == slug.replace('_', "")
                            || file_name.starts_with(&format!("{}-", slug))
                            || file_name.starts_with(&format!("{}_", slug))
                            || slug.starts_with(&format!("{}-", file_name))
                            || slug.starts_with(&format!("{}_", file_name))
                        {
                            check_and_add_candidate(
                                &mut candidates,
                                &mut seen_paths,
                                &app_clone,
                                path.clone(),
                                *kind,
                                0.88,
                            );
                            break;
                        }
                    }
                }
            }
        }

        // 3. Desktop Entry check in ~/.local/share/applications/
        if desktop_base.exists() {
            if let Ok(entries) = std::fs::read_dir(&desktop_base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    for slug in &slugs {
                        if name == format!("{}.desktop", slug) || name.contains(slug) {
                            check_and_add_candidate(
                                &mut candidates,
                                &mut seen_paths,
                                &app_clone,
                                path.clone(),
                                ArtifactKind::DesktopEntry,
                                0.90,
                            );
                            break;
                        }
                    }
                }
            }
        }

        // 4. Flatpak sandbox check: ~/.var/app/<app.id>
        let flatpak_sandbox = home_dir.join(".var/app").join(&app_clone.id);
        if flatpak_sandbox.exists() {
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                flatpak_sandbox,
                ArtifactKind::SandboxDir,
                0.99,
            );
        }

        // 5. Snap sandbox check: ~/snap/<app.name>
        let snap_dir = home_dir.join("snap").join(&app_clone.id);
        if snap_dir.exists() {
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                snap_dir,
                ArtifactKind::SandboxDir,
                0.99,
            );
        }

        candidates
    })
    .await
    .unwrap_or_default()
}

/// Helper to extract clean slug variations for heuristic matching.
fn extract_slug_variations(app: &Application) -> Vec<String> {
    let mut slugs = HashSet::new();

    let id_lower = app.id.to_lowercase();
    let name_lower = app.name.to_lowercase();
    slugs.insert(id_lower.clone());
    slugs.insert(name_lower.clone());

    // Strip prefixes like "opt-" or "appimage-"
    if let Some(stripped) = id_lower.strip_prefix("opt-") {
        slugs.insert(stripped.to_string());
    }
    if let Some(stripped) = id_lower.strip_prefix("appimage-") {
        slugs.insert(stripped.to_string());
    }

    // Handle dashes & suffixes (e.g. moviebox-tui -> moviebox, moviebox_tui)
    if name_lower.contains('-') {
        if let Some(first) = name_lower.split('-').next() {
            if first.len() > 2 {
                slugs.insert(first.to_string());
            }
        }
    }
    if name_lower.contains('_') {
        if let Some(first) = name_lower.split('_').next() {
            if first.len() > 2 {
                slugs.insert(first.to_string());
            }
        }
    }

    if id_lower.contains('.') {
        if let Some(last) = id_lower.split('.').last() {
            if !last.is_empty() {
                slugs.insert(last.to_string());
            }
        }
    }

    if let Some(ref exec) = app.exec_path {
        if let Some(stem) = exec.file_stem() {
            let s = stem.to_string_lossy().to_lowercase();
            slugs.insert(s.clone());
            if s.contains('-') {
                if let Some(first) = s.split('-').next() {
                    slugs.insert(first.to_string());
                }
            }
        }
    }

    slugs.into_iter().filter(|s| s.len() > 1).collect()
}

fn check_and_add_candidate(
    candidates: &mut Vec<ResidualCandidate>,
    seen_paths: &mut HashSet<PathBuf>,
    app: &Application,
    path: PathBuf,
    kind: ArtifactKind,
    confidence: f32,
) {
    if !path.exists() {
        return;
    }

    if SafetyValidator::is_path_safe_to_delete(&path).is_err() {
        return;
    }

    if seen_paths.insert(path.clone()) {
        let size = if path.is_dir() {
            calculate_dir_size(&path)
        } else {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };

        candidates.push(ResidualCandidate::new(
            &app.id,
            &app.name,
            path,
            kind,
            size,
            confidence,
            false,
        ));
    }
}

pub async fn scan_all_orphaned_residuals(
    installed_apps: &[Application],
) -> Vec<ResidualCandidate> {
    let active_slugs: HashSet<String> = installed_apps
        .iter()
        .flat_map(extract_slug_variations)
        .collect();

    tokio::task::spawn_blocking(move || {
        let mut orphans = Vec::new();
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return orphans,
        };

        let scan_roots = [
            (home.join(".config"), ArtifactKind::ConfigDir),
            (home.join(".cache"), ArtifactKind::CacheDir),
            (home.join(".local/share"), ArtifactKind::DataDir),
            (home.join(".local/state"), ArtifactKind::StateDir),
        ];

        for (root, kind) in &scan_roots {
            if !root.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    if name.starts_with('.') {
                        continue;
                    }

                    let is_active = active_slugs.iter().any(|slug| {
                        name_lower == *slug
                            || name_lower.contains(slug)
                            || slug.contains(&name_lower)
                    });

                    if !is_active && SafetyValidator::is_path_safe_to_delete(&path).is_ok() {
                        let size = if path.is_dir() {
                            calculate_dir_size(&path)
                        } else {
                            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
                        };

                        orphans.push(ResidualCandidate::new(
                            format!("orphan-{}", name_lower),
                            &name,
                            path,
                            *kind,
                            size,
                            0.75,
                            true,
                        ));
                    }
                }
            }
        }

        orphans
    })
    .await
    .unwrap_or_default()
}
