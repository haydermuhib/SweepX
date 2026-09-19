use super::safety::SafetyValidator;
use super::signatures::find_signature;
use crate::models::{Application, ArtifactKind, ResidualCandidate};
use crate::scanner::manual::calculate_dir_size;
use std::collections::HashSet;
use std::path::PathBuf;

/// Strictly discovers associated configuration, cache, data, and state paths for a specific application.
/// Anchored strictly to the application's exact ID, binary name, desktop launcher, and container ID.
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
        let desktop_base = home_dir.join(".local/share/applications");

        // 0. Check SQLite Install Watcher Manifest (Exact recorded files at install time)
        if let Ok(db) = crate::db::Database::open_default() {
            if let Ok(Some(manifest)) = db.get_install_manifest(&app_clone.id) {
                for created_file in manifest.created_files {
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        created_file,
                        ArtifactKind::Binary,
                        1.0,
                    );
                }
            }
        }

        // 1. Direct Anchor Identifiers (Exact App ID, Executable Stem, and Package Name)
        let mut exact_anchors: Vec<String> = Vec::new();
        exact_anchors.push(app_clone.id.to_lowercase());
        exact_anchors.push(app_clone.name.to_lowercase());

        if let Some(ref exec) = app_clone.exec_path {
            if let Some(stem) = exec.file_stem() {
                let stem_lower = stem.to_string_lossy().to_lowercase();
                if !exact_anchors.contains(&stem_lower) {
                    exact_anchors.push(stem_lower);
                }
            }
        }

        // If ID is reverse-DNS (e.g. org.mozilla.firefox), also include the trailing component (e.g. firefox)
        if app_clone.id.contains('.') {
            if let Some(last) = app_clone.id.split('.').last() {
                let last_lower = last.to_lowercase();
                if !exact_anchors.contains(&last_lower) && last_lower.len() > 1 {
                    exact_anchors.push(last_lower);
                }
            }
        }

        // 2. Exact Path Target Checks in Standard XDG Bases and Home Root Dotdirs
        for anchor in &exact_anchors {
            // Direct home root dot-directory (e.g. ~/.feynman, ~/.docker, ~/.rustup)
            if !anchor.is_empty() && anchor != "bash" && anchor != "profile" && anchor != "zsh" && anchor != "config" && anchor != "cache" && anchor != "local" {
                let dotdir = home_dir.join(format!(".{}", anchor));
                check_and_add_candidate(
                    &mut candidates,
                    &mut seen_paths,
                    &app_clone,
                    dotdir,
                    ArtifactKind::ConfigDir,
                    0.95,
                );
            }
            // ~/.config/<anchor>
            let cfg = config_base.join(anchor);
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                cfg,
                ArtifactKind::ConfigDir,
                1.0,
            );

            // ~/.cache/<anchor>
            let ca = cache_base.join(anchor);
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                ca,
                ArtifactKind::CacheDir,
                1.0,
            );

            // ~/.local/share/<anchor>
            let dat = data_base.join(anchor);
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                dat,
                ArtifactKind::DataDir,
                1.0,
            );

            // ~/.local/state/<anchor>
            let st = state_base.join(anchor);
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                st,
                ArtifactKind::StateDir,
                1.0,
            );

            // ~/.local/share/applications/<anchor>.desktop
            let desk = desktop_base.join(format!("{}.desktop", anchor));
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                desk,
                ArtifactKind::DesktopEntry,
                1.0,
            );
        }

        // 3. Known Verified Signatures (For multi-directory apps like VS Code or Firefox)
        for term in [&app_clone.id, &app_clone.name] {
            if let Some(sig) = find_signature(term) {
                for &dir in sig.config_dirs {
                    let p = config_base.join(dir);
                    check_and_add_candidate(
                        &mut candidates,
                        &mut seen_paths,
                        &app_clone,
                        p,
                        ArtifactKind::ConfigDir,
                        1.0,
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
                        1.0,
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
                        1.0,
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
                        1.0,
                    );
                }
            }
        }

        // 4. Container Sandboxes
        // Flatpak: ~/.var/app/<app.id>
        let flatpak_sandbox = home_dir.join(".var/app").join(&app_clone.id);
        check_and_add_candidate(
            &mut candidates,
            &mut seen_paths,
            &app_clone,
            flatpak_sandbox,
            ArtifactKind::SandboxDir,
            1.0,
        );

        // Snap: ~/snap/<app.name>
        let snap_dir = home_dir.join("snap").join(&app_clone.name);
        check_and_add_candidate(
            &mut candidates,
            &mut seen_paths,
            &app_clone,
            snap_dir,
            ArtifactKind::SandboxDir,
            1.0,
        );

        // 5. Stow-Style Symlink Graph: Find any symlinks in ~/.local/bin, /usr/local/bin pointing to this app
        if let Some(ref exec_path) = app_clone.exec_path {
            let pointing_symlinks = crate::scanner::symlink_graph::find_symlinks_pointing_to_app(exec_path);
            for symlink in pointing_symlinks {
                check_and_add_candidate(
                    &mut candidates,
                    &mut seen_paths,
                    &app_clone,
                    symlink,
                    ArtifactKind::Binary,
                    1.0,
                );
            }
        }

        // 5. Binary Executable Path
        if let Some(ref bin) = app_clone.exec_path {
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                bin.clone(),
                ArtifactKind::Binary,
                1.0,
            );
        }

        // 6. Desktop File Path
        if let Some(ref desk) = app_clone.desktop_file {
            check_and_add_candidate(
                &mut candidates,
                &mut seen_paths,
                &app_clone,
                desk.clone(),
                ArtifactKind::DesktopEntry,
                1.0,
            );
        }

        candidates
    })
    .await
    .unwrap_or_default()
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

    // Must pass strict safety path validator
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
