use super::privilege::run_elevated_command;
use super::safety::SafetyValidator;
use crate::models::{Application, InstallMethod, ResidualCandidate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

/// Deletion strategy for removing leftover artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionMode {
    Trash,
    Permanent,
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
pub async fn execute_purge_package(app: &Application) -> Result<String, String> {
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
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        InstallMethod::Snap => {
            run_elevated_command("snap", &["remove", "--purge", &app.id]).await
        }
        InstallMethod::NativeApt => {
            let res = run_elevated_command("apt-get", &["purge", "-y", &app.id]).await?;
            let _ = run_elevated_command("apt-get", &["autoremove", "-y"]).await;
            Ok(res)
        }
        InstallMethod::NativeDnf => {
            let res = run_elevated_command("dnf", &["remove", "-y", &app.id]).await?;
            let _ = run_elevated_command("dnf", &["autoremove", "-y"]).await;
            Ok(res)
        }
        InstallMethod::NativePacman => {
            run_elevated_command("pacman", &["-Rns", "--noconfirm", &app.id]).await
        }
        InstallMethod::ManualOpt | InstallMethod::AppImage | InstallMethod::CustomDesktop => {
            // For manual / AppImage, package removal is handled through artifact/directory deletion
            Ok("Manual installation ready for artifact cleanup".to_string())
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
