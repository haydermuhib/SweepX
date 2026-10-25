use super::privilege::run_elevated_command;
use super::safety::SafetyValidator;
use crate::models::{Application, InstallMethod, ResidualCandidate};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Deletion strategy for removing leftover artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionMode {
    Trash,
    Permanent,
}

/// Helper to delete a target path safely using Trash or Permanent mode, falling back to elevated rm if needed.
pub async fn delete_target_path(path: &Path, mode: DeletionMode) -> Result<(), String> {
    if !path.exists() && !path.is_symlink() {
        return Ok(());
    }

    if let Err(violation) = SafetyValidator::is_path_safe_to_delete(path) {
        return Err(format!("Blocked by safety validator: {}", violation));
    }

    match mode {
        DeletionMode::Trash => match trash::delete(path) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to move {} to trash: {}", path.display(), e)),
        },
        DeletionMode::Permanent => {
            let res = if !path.is_symlink() && path.is_dir() {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            };

            match res {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    let path_str = path.to_string_lossy().to_string();
                    let args = if !path.is_symlink() && path.is_dir() {
                        vec!["-rf", path_str.as_str()]
                    } else {
                        vec!["-f", path_str.as_str()]
                    };
                    run_elevated_command("rm", &args)
                        .await
                        .map(|_| ())
                        .map_err(|err| {
                            format!(
                                "Permission denied deleting {}. Elevated removal (pkexec/sudo) failed: {}. Verify polkit authentication agent is active or adjust target permissions.",
                                path.display(),
                                err
                            )
                        })
                }
                Err(e) => Err(format!("Failed to delete {}: {}", path.display(), e)),
            }
        }
    }
}

/// Detailed audit report of an uninstallation / purge action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PurgeReport {
    pub success: bool,
    pub package_purged: bool,
    pub package_message: Option<String>,
    pub deleted_paths: Vec<PathBuf>,
    pub trashed_paths: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
    pub freed_bytes: u64,
}

/// Executes the removal of the primary package based on its installation method.
pub async fn execute_purge_package(app: &Application, mode: DeletionMode) -> Result<String, String> {
    match app.install_method {
        InstallMethod::Flatpak => {
            let output = Command::new("flatpak")
                .args(["uninstall", "--delete-data", "-y", &app.id])
                .output()
                .await
                .map_err(|e| format!("Failed to run flatpak uninstall: {}", e))?;

            if output.status.success() {
                // Also clean up unused runtimes
                let _ = Command::new("flatpak")
                    .args(["uninstall", "--unused", "-y"])
                    .output()
                    .await;

                // Also clean up desktop file export if still present
                if let Some(ref desk) = app.desktop_file {
                    if desk.exists() || desk.is_symlink() {
                        let _ = delete_target_path(desk, mode).await;
                    }
                }

                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        InstallMethod::Snap => {
            let res = run_elevated_command("snap", &["remove", "--purge", &app.id]).await?;
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    let _ = delete_target_path(desk, mode).await;
                }
            }
            Ok(res)
        }
        InstallMethod::NativeApt => {
            let res = run_elevated_command("apt-get", &["purge", "-y", &app.id]).await?;
            let _ = run_elevated_command("apt-get", &["autoremove", "-y"]).await;
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    let _ = delete_target_path(desk, mode).await;
                }
            }
            Ok(res)
        }
        InstallMethod::NativeDnf => {
            let res = run_elevated_command("dnf", &["remove", "-y", &app.id]).await?;
            let _ = run_elevated_command("dnf", &["autoremove", "-y"]).await;
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    let _ = delete_target_path(desk, mode).await;
                }
            }
            Ok(res)
        }
        InstallMethod::NativePacman => {
            let res = run_elevated_command("pacman", &["-Rns", "--noconfirm", &app.id]).await?;
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    let _ = delete_target_path(desk, mode).await;
                }
            }
            Ok(res)
        }
        InstallMethod::Bottles => {
            let mut errors = Vec::new();
            for artifact in &app.artifacts {
                if artifact.path.exists() || artifact.path.is_symlink() {
                    if let Err(e) = delete_target_path(&artifact.path, mode).await {
                        errors.push(e);
                    }
                }
            }
            if let Some(ref exec) = app.exec_path {
                if exec.exists() || exec.is_symlink() {
                    if let Err(e) = delete_target_path(exec, mode).await {
                        errors.push(e);
                    }
                }
            }
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    if let Err(e) = delete_target_path(desk, mode).await {
                        errors.push(e);
                    }
                }
            }
            if let Ok(db) = crate::db::Database::open_default() {
                let _ = db.delete_install_manifest(&app.id);
            }
            if errors.is_empty() {
                Ok("Bottles application payload removed successfully".to_string())
            } else {
                Err(errors.join("; "))
            }
        }
        InstallMethod::ManualOpt
        | InstallMethod::AppImage
        | InstallMethod::CustomDesktop => {
            let mut errors = Vec::new();

            // 1. Delete associated package artifacts (e.g. /opt/appdir, standalone binary)
            for artifact in &app.artifacts {
                if artifact.path.exists() || artifact.path.is_symlink() {
                    if let Err(e) = delete_target_path(&artifact.path, mode).await {
                        errors.push(e);
                    }
                }
            }

            // 2. Delete primary executable binary if still present on disk
            if let Some(ref exec) = app.exec_path {
                if exec.exists() || exec.is_symlink() {
                    if let Err(e) = delete_target_path(exec, mode).await {
                        errors.push(e);
                    }
                }
            }

            // 3. Delete desktop launcher if present
            if let Some(ref desk) = app.desktop_file {
                if desk.exists() || desk.is_symlink() {
                    if let Err(e) = delete_target_path(desk, mode).await {
                        errors.push(e);
                    }
                }
            }

            // 4. Clean up SQLite install manifest if present
            if let Ok(db) = crate::db::Database::open_default() {
                let _ = db.delete_install_manifest(&app.id);
            }

            if errors.is_empty() {
                Ok("Application package payload and binaries removed successfully".to_string())
            } else {
                Err(errors.join("; "))
            }
        }
    }
}

/// Safely executes deletion of selected residual directories and files.
pub async fn execute_purge_residuals(
    residuals: &[ResidualCandidate],
    mode: DeletionMode,
) -> PurgeReport {
    let mut report = PurgeReport {
        success: true,
        ..Default::default()
    };

    for candidate in residuals {
        if !candidate.selected_for_deletion || !candidate.path.exists() {
            continue;
        }

        // Validate path safety
        if let Err(violation) = SafetyValidator::is_path_safe_to_delete(&candidate.path) {
            report.errors.push((
                candidate.path.clone(),
                format!("Blocked by safety validator: {}", violation),
            ));
            report.success = false;
            continue;
        }

        match mode {
            DeletionMode::Trash => {
                // FreeDesktop Trash deletion
                match trash::delete(&candidate.path) {
                    Ok(()) => {
                        report.trashed_paths.push(candidate.path.clone());
                        report.freed_bytes += candidate.size_bytes;
                    }
                    Err(e) => {
                        // If trash fails (e.g. on root partition or permission), report error
                        report.errors.push((
                            candidate.path.clone(),
                            format!("Failed to move to trash: {}", e),
                        ));
                        report.success = false;
                    }
                }
            }
            DeletionMode::Permanent => {
                let delete_res = if candidate.path.is_dir() {
                    std::fs::remove_dir_all(&candidate.path)
                } else {
                    std::fs::remove_file(&candidate.path)
                };

                match delete_res {
                    Ok(()) => {
                        report.deleted_paths.push(candidate.path.clone());
                        report.freed_bytes += candidate.size_bytes;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        // Attempt elevated deletion for protected files (e.g. in /opt or /etc)
                        let path_str = candidate.path.to_string_lossy();
                        match run_elevated_command("rm", &["-rf", &path_str]).await {
                            Ok(_) => {
                                report.deleted_paths.push(candidate.path.clone());
                                report.freed_bytes += candidate.size_bytes;
                            }
                            Err(elevated_err) => {
                                report.errors.push((candidate.path.clone(), elevated_err));
                                report.success = false;
                            }
                        }
                    }
                    Err(e) => {
                        report.errors.push((
                            candidate.path.clone(),
                            format!("Failed to delete: {}", e),
                        ));
                        report.success = false;
                    }
                }
            }
        }
    }

    report
}
