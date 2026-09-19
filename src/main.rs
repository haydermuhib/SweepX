use clap::Parser;
use eframe::egui;
use sweepx::cli::{run_cli_command, Cli, Commands};
use sweepx::ui::SweepXApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => {
            // Launch eframe/egui GUI Desktop App
            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1100.0, 720.0])
                    .with_min_inner_size([850.0, 550.0])
                    .with_title("SweepX — Unified Linux App Tracker & Deep Purge")
                    .with_app_id("org.sweepx.SweepX"),
                ..Default::default()
            };

            eframe::run_native(
                "SweepX",
                native_options,
                Box::new(|cc| Ok(Box::new(SweepXApp::new(cc)))),
            )
            .map_err(|e| format!("GUI launch failed: {}", e))?;
        }
        Some(cmd) => {
            // Execute CLI Subcommand
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_cli_command(cmd))?;
        }
    }

    Ok(())
}
