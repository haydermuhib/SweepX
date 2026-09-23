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
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates an animated high-contrast terminal spinner with a steady tick.
fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

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
            let sp = create_spinner("Scanning host applications across all packaging tiers...");
            let apps = scan_all_applications().await;
            sp.finish_and_clear();

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

            println!("✓ Discovered {} applications", filtered.len());
            println!("{:-<88}", "");
            println!(
                "{:<35} {:<15} {:<15} {:<12}",
                "APPLICATION", "ORIGIN", "VERSION", "SIZE"
            );
            println!("{:-<88}", "");

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
            let sp = create_spinner(&format!("Searching for application '{}'...", app_id));
            let apps = scan_all_applications().await;
            let target = apps
                .iter()
                .find(|a| a.id.eq_ignore_ascii_case(&app_id) || a.name.eq_ignore_ascii_case(&app_id));

            let app = match target {
                Some(a) => a,
                None => {
                    sp.finish_with_message(format!("✗ Application '{}' not found.", app_id));
                    return Ok(());
                }
            };

            sp.set_message(format!("Crawling residual paths and sandboxes for '{}'...", app.display_name));
            let residuals = discover_residuals_for_app(app).await;
            sp.finish_and_clear();

            if json {
                let report = serde_json::json!({
                    "application": app,
                    "residuals": residuals,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            let package_size = app.total_size_bytes;
            let residuals_size: u64 = residuals.iter().map(|r| r.size_bytes).sum();
            let total_combined = package_size + residuals_size;

            println!("✓ Inspection complete for: {}", app.display_name);
            println!("  • ID:               {}", app.id);
            println!("  • Packaging:        {}", app.install_method.badge_label());
            println!("  • Version:          {}", app.version.as_deref().unwrap_or("N/A"));
            println!("  • Package Payload:  {}", format_size(package_size));
            println!("  • User Files Size:  {}", format_size(residuals_size));
            println!("  • Total Footprint:  {}", format_size(total_combined));
            if let Some(ref exec) = app.exec_path {
                println!("  • Binary:           {}", exec.display());
            }
            if let Some(ref desk) = app.desktop_file {
                println!("  • Desktop Launcher: {}", desk.display());
            }
            if let Some(ref desc) = app.description {
                println!("  • Description:      {}", desc);
            }

            println!("\n📂 Discovered User Artifacts & Caches ({} found):", residuals.len());
            if residuals.is_empty() {
                println!("  (No additional user configs or caches found)");
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
            let sp = create_spinner(&format!("Locating '{}' for deep uninstallation...", app_id));
            let apps = scan_all_applications().await;
            let target = apps
                .iter()
                .find(|a| a.id.eq_ignore_ascii_case(&app_id) || a.name.eq_ignore_ascii_case(&app_id));

            let app = match target {
                Some(a) => a,
                None => {
                    sp.finish_with_message(format!("✗ Application '{}' not found.", app_id));
                    return Ok(());
                }
            };

            sp.set_message(format!("Discovering residual files for '{}'...", app.display_name));
            let residuals = discover_residuals_for_app(app).await;
            sp.finish_and_clear();

            println!("📋 Deep Cleaning Plan for '{}':", app.display_name);
            println!("  • Package Tier:   {}", app.install_method.badge_label());
            println!("  • Deletion Mode:  {}", if permanent { "Permanent Unlink" } else { "Freedesktop Trash" });

            if residuals.is_empty() {
                println!("  • Residual Items: None");
            } else {
                println!("  • Residual Items to Remove ({}):", residuals.len());
                for r in &residuals {
                    println!("    - {} ({})", r.path.display(), format_size(r.size_bytes));
                }
            }

            if dry_run {
                println!("\n✓ [DRY RUN] No files or packages were modified.");
                return Ok(());
            }

            let purge_sp = create_spinner(if permanent {
                "Permanently unlinking application files..."
            } else {
                "Moving application files to system Trash..."
            });

            let _pkg_res = execute_purge_package(app).await;
            let mode = if permanent {
                DeletionMode::Permanent
            } else {
                DeletionMode::Trash
            };

            let report = execute_purge_residuals(&residuals, mode).await;
            purge_sp.finish_and_clear();

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
                "✓ Purge completed for '{}'! Freed ~{}.",
                app.display_name,
                format_size(report.freed_bytes + app.total_size_bytes)
            );
            if !report.errors.is_empty() {
                println!("⚠️ Notice: {} items encountered notices during cleanup:", report.errors.len());
                for (p, err) in report.errors {
                    println!("  - {}: {}", p.display(), err);
                }
            }
        }
        Commands::History => {
            let db = Database::open_default()?;
            let logs = db.get_audit_history()?;

            if logs.is_empty() {
                println!("✓ No uninstallation history recorded yet.");
                return Ok(());
            }

            println!("📜 Uninstallation & Purge History ({} entries):", logs.len());
            println!("{:-<85}", "");
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
            let sp = create_spinner("Scanning system for obsolete container revisions, caches, and unused runtimes...");
            let items = scan_all_sweep_items().await;
            sp.finish_and_clear();

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
                println!("✓ System is fully optimized. No obsolete revisions or package caches found.");
                return Ok(());
            }

            let total_reclaimable: u64 = items.iter().map(|i| i.reclaimable_bytes).sum();
            println!("✓ Discovered Reclaimable System Bloat ({} items, ~{}):", items.len(), format_size(total_reclaimable));
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
                println!("\n✓ [DRY RUN] No files, package caches, or container revisions were modified.");
                return Ok(());
            }

            let sweep_sp = create_spinner("Executing system optimization routines...");
            let report = execute_system_sweep(&items).await;
            sweep_sp.finish_and_clear();

            println!(
                "✓ System sweep completed! Cleaned {} items and reclaimed ~{}.",
                report.items_cleaned,
                format_size(report.freed_bytes)
            );

            if !report.errors.is_empty() {
                println!("⚠️ Notice: {} items encountered notices during cleanup:", report.errors.len());
                for err in report.errors {
                    println!("  - {}", err);
                }
            }
        }
        Commands::Watch { name, command } => {
            if command.is_empty() {
                eprintln!("✗ No installation command specified.");
                return Ok(());
            }

            let full_cmd_str = command.join(" ");
            let snap_sp = create_spinner("Capturing pre-installation filesystem snapshot across system roots...");
            let pre_snapshot = crate::tracker::FilesystemSnapshot::capture();
            snap_sp.finish_and_clear();

            println!("🚀 Executing installation command: '{}'...\n", full_cmd_str);
            let program = &command[0];
            let args = &command[1..];

            let status = tokio::process::Command::new(program)
                .args(args)
                .status()
                .await?;

            if !status.success() {
                eprintln!("\n✗ Installation command failed with exit code: {:?}", status.code());
                return Ok(());
            }

            let post_sp = create_spinner("Capturing post-installation filesystem snapshot and computing diff...");
            let post_snapshot = crate::tracker::FilesystemSnapshot::capture();

            let app_id = name.to_lowercase().replace(' ', "-");
            let manifest = pre_snapshot.diff(&post_snapshot, &app_id, &name, &full_cmd_str);

            if let Ok(db) = Database::open_default() {
                let _ = db.save_install_manifest(&manifest);
            }
            post_sp.finish_and_clear();

            println!("✓ Install Watch Complete for '{}'!", name);
            println!("  • Created Files ({}):", manifest.created_files.len());
            for f in &manifest.created_files {
                println!("    + {}", f.display());
            }
            if !manifest.modified_files.is_empty() {
                println!("  • Modified Files ({}):", manifest.modified_files.len());
                for f in &manifest.modified_files {
                    println!("    ~ {}", f.display());
                }
            }
            println!("  • Total Footprint: {}", format_size(manifest.total_size_bytes));
            println!("✓ Application manifest saved. Cleanly purgeable anytime via 'sweepx purge {}'.", app_id);
        }
        Commands::Update { check } => {
            let sp = create_spinner("Checking for updates on GitHub (haydermuhib/SweepX)...");
            match crate::updater::check_for_updates().await {
                Ok(update_info) => {
                    sp.finish_and_clear();
                    println!("  • Current version: v{}", update_info.current_version);
                    println!("  • Latest version:  v{}", update_info.latest_version);

                    if !update_info.has_update {
                        println!("✓ SweepX is up to date (v{}).", update_info.current_version);
                        return Ok(());
                    }

                    println!(
                        "\n🚀 New update available: v{} → v{}!",
                        update_info.current_version, update_info.latest_version
                    );
                    if !update_info.release_notes.is_empty() {
                        println!("\n📋 Release Notes:\n{}", update_info.release_notes.trim());
                    }
                    println!("🔗 Release URL: {}", update_info.html_url);

                    if check {
                        println!("\n💡 Run 'sweepx update' to install the latest release automatically.");
                        return Ok(());
                    }

                    if let Some(ref download_url) = update_info.download_url {
                        let dl_sp = create_spinner("Downloading and atomically replacing executable binary...");
                        match crate::updater::perform_self_update(download_url).await {
                            Ok(updated_path) => {
                                dl_sp.finish_with_message(format!(
                                    "✓ Successfully updated SweepX to v{} at {}!",
                                    update_info.latest_version,
                                    updated_path.display()
                                ));
                            }
                            Err(e) => {
                                dl_sp.finish_with_message(format!("✗ Update installation failed: {}", e));
                                eprintln!("💡 Fallback: Run curl -fsSL https://raw.githubusercontent.com/haydermuhib/SweepX/main/install.sh | bash");
                            }
                        }
                    } else {
                        println!("\n⚠️ No matching precompiled binary asset found for this architecture in release.");
                    }
                }
                Err(e) => {
                    sp.finish_with_message(format!("✗ Failed to check for updates: {}", e));
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
