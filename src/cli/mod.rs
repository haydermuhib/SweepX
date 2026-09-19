pub mod commands;

pub use commands::{CategoryFilter, Cli, Commands};

use crate::cleaner::{
    discover_residuals_for_app, execute_purge_package, execute_purge_residuals,
    execute_system_sweep, scan_all_sweep_items, DeletionMode,
};
use crate::db::{AuditLogEntry, Database};
use crate::models::{format_size, Application, InstallMethod};
use crate::scanner::scan_all_applications;
use chrono::Utc;

/// Executes the selected CLI command asynchronously.
pub async fn run_cli_command(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Gui => {
            // Handled in main.rs
        }
        Commands::List {
            category,
            all_system,
            json,
        } => {
            println!("🔍 Scanning host applications across all packaging tiers...");
            let apps = scan_all_applications().await;

            let filtered: Vec<Application> = apps
                .into_iter()
                .filter(|app| {
                    if !all_system && app.is_system {
                        return false;
                    }
                    match category {
                        CategoryFilter::All => true,
                        CategoryFilter::Native => app.install_method.is_native_system(),
                        CategoryFilter::Flatpak => app.install_method == InstallMethod::Flatpak,
                        CategoryFilter::Snap => app.install_method == InstallMethod::Snap,
                        CategoryFilter::Appimage => app.install_method == InstallMethod::AppImage,
                        CategoryFilter::Manual => {
                            app.install_method == InstallMethod::ManualOpt
                                || app.install_method == InstallMethod::CustomDesktop
                        }
                    }
                })
                .collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&filtered)?);
                return Ok(());
            }

            println!("\n📦 Discovered Applications ({} found):", filtered.len());
            println!(
                "{:<35} {:<15} {:<15} {:<12}",
                "APPLICATION", "ORIGIN", "VERSION", "SIZE"
            );
            println!("{:-<80}", "");

            for app in &filtered {
                let ver = app.version.as_deref().unwrap_or("-");
                println!(
                    "{:<35} {:<15} {:<15} {:<12}",
                    truncate_str(&app.display_name, 34),
                    app.install_method.badge_label(),
                    truncate_str(ver, 14),
                    app.formatted_size()
                );
            }
        }
        Commands::Inspect { app_id, json } => {
            println!("🔍 Inspecting application '{}'...", app_id);
            let apps = scan_all_applications().await;
            let target = apps
                .iter()
                .find(|a| a.id.eq_ignore_ascii_case(&app_id) || a.name.eq_ignore_ascii_case(&app_id));

            let app = match target {
                Some(a) => a,
                None => {
                    eprintln!("❌ Application '{}' not found.", app_id);
                    return Ok(());
                }
            };

            let residuals = discover_residuals_for_app(app).await;

            if json {
                let report = serde_json::json!({
                    "application": app,
                    "residuals": residuals,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n📋 Application Details: {}", app.display_name);
            println!("  • ID: {}", app.id);
            println!("  • Packaging: {}", app.install_method.badge_label());
            println!("  • Version: {}", app.version.as_deref().unwrap_or("N/A"));
            println!("  • Total Size: {}", app.formatted_size());
            if let Some(ref exec) = app.exec_path {
                println!("  • Binary: {}", exec.display());
            }
            if let Some(ref desk) = app.desktop_file {
                println!("  • Desktop Launcher: {}", desk.display());
            }
            if let Some(ref desc) = app.description {
                println!("  • Description: {}", desc);
            }

            println!("\n📂 Associated Artifacts & Config Files ({} found):", residuals.len());
            if residuals.is_empty() {
                println!("  (No user configs or caches found)");
            } else {
                for r in &residuals {
                    println!(
                        "  [{}] {} ({})",
                        r.kind.display_name(),
                        r.path.display(),
                        format_size(r.size_bytes),
                    );
                }
            }
        }
        Commands::Purge {
            app_id,
            dry_run,
            permanent,
        } => {
            println!("🔍 Locating '{}' for deep uninstallation...", app_id);
            let apps = scan_all_applications().await;
            let target = apps
                .iter()
                .find(|a| a.id.eq_ignore_ascii_case(&app_id) || a.name.eq_ignore_ascii_case(&app_id));

            let app = match target {
                Some(a) => a,
                None => {
                    eprintln!("❌ Application '{}' not found.", app_id);
                    return Ok(());
                }
            };

            let residuals = discover_residuals_for_app(app).await;

            println!("\n⚠️ Deep Cleaning Plan for '{}':", app.display_name);
            println!("  • Package Tier: {}", app.install_method.badge_label());
            println!("  • Deletion Mode: {}", if permanent { "Permanent Delete" } else { "FreeDesktop Trash" });

            if residuals.is_empty() {
                println!("  • Residual Items: None");
            } else {
                println!("  • Residual Items to Remove ({}):", residuals.len());
                for r in &residuals {
                    println!("    - {} ({})", r.path.display(), format_size(r.size_bytes));
                }
            }

            if dry_run {
                println!("\n✅ [DRY RUN] No files or packages were modified.");
                return Ok(());
            }

            println!("\n🚀 Executing purge...");
            let _pkg_res = execute_purge_package(app).await;
            let mode = if permanent {
                DeletionMode::Permanent
            } else {
                DeletionMode::Trash
            };

            let report = execute_purge_residuals(&residuals, mode).await;

            if let Ok(db) = Database::open_default() {
                let audit = AuditLogEntry {
                    id: None,
                    app_id: app.id.clone(),
                    app_name: app.display_name.clone(),
                    install_method: app.install_method.badge_label().to_string(),
                    timestamp: Utc::now(),
                    freed_bytes: report.freed_bytes + app.total_size_bytes,
                    deleted_paths: report
                        .deleted_paths
                        .iter()
                        .chain(report.trashed_paths.iter())
                        .cloned()
                        .collect(),
                    status: if report.success { "SUCCESS".to_string() } else { "PARTIAL".to_string() },
                    error_details: if report.errors.is_empty() {
                        None
                    } else {
                        Some(format!("{:?}", report.errors))
                    },
                };
                let _ = db.record_audit(&audit);
            }

            println!(
                "🎉 Purge completed! Freed ~{}.",
                format_size(report.freed_bytes + app.total_size_bytes)
            );
            if !report.errors.is_empty() {
                println!("⚠️ Notice: {} items encountered errors:", report.errors.len());
                for (p, err) in report.errors {
                    println!("  - {}: {}", p.display(), err);
                }
            }
        }
        Commands::History => {
            let db = Database::open_default()?;
            let logs = db.get_audit_history()?;

            println!("\n📜 Uninstallation & Purge History ({} entries):", logs.len());
            if logs.is_empty() {
                println!("  (No uninstallation history recorded yet)");
                return Ok(());
            }

            println!(
                "{:<25} {:<30} {:<15} {:<12}",
                "TIMESTAMP", "APPLICATION", "METHOD", "FREED"
            );
            println!("{:-<85}", "");

            for log in logs {
                let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                println!(
                    "{:<25} {:<30} {:<15} {:<12}",
                    time_str,
                    truncate_str(&log.app_name, 29),
                    log.install_method,
                    format_size(log.freed_bytes)
                );
            }
        }
        Commands::Sweep { dry_run, json } => {
            println!("🔍 Scanning system for obsolete container revisions, caches, and unused runtimes...");
            let items = scan_all_sweep_items().await;

            if json {
                println!("{}", serde_json::to_string_pretty(&items.iter().map(|i| {
                    serde_json::json!({
                        "id": i.id,
                        "title": i.title,
                        "category": i.category.display_name(),
                        "description": i.description,
                        "reclaimable_bytes": i.reclaimable_bytes,
                        "command": i.command,
                    })
                }).collect::<Vec<_>>())?);
                return Ok(());
            }

            if items.is_empty() {
                println!("✨ System is fully optimized! No obsolete revisions or package caches found.");
                return Ok(());
            }

            let total_reclaimable: u64 = items.iter().map(|i| i.reclaimable_bytes).sum();
            println!("\n🧹 Discovered Reclaimable System Bloat ({} items, ~{}):", items.len(), format_size(total_reclaimable));
            println!(
                "{:<38} {:<28} {:<12}",
                "ITEM", "CATEGORY", "RECLAIMABLE"
            );
            println!("{:-<80}", "");

            for item in &items {
                println!(
                    "{:<38} {:<28} {:<12}",
                    truncate_str(&item.title, 37),
                    truncate_str(item.category.display_name(), 27),
                    if item.reclaimable_bytes > 0 { format_size(item.reclaimable_bytes) } else { "Dynamic".to_string() }
                );
            }

            if dry_run {
                println!("\n✅ [DRY RUN] No files, package caches, or snap revisions were modified.");
                return Ok(());
            }

            println!("\n🚀 Executing system optimization...");
            let report = execute_system_sweep(&items).await;

            println!(
                "🎉 System sweep completed! Cleaned {} items and reclaimed ~{}.",
                report.items_cleaned,
                format_size(report.freed_bytes)
            );

            if !report.errors.is_empty() {
                println!("⚠️ Notice: {} items encountered errors during cleanup:", report.errors.len());
                for err in report.errors {
                    println!("  - {}", err);
                }
            }
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
