use clap::Parser;
use sweepx::cli::{run_cli_command, Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => {
            sweepx::ui::run_gui()?;
        }
        Some(cmd) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_cli_command(cmd))?;
        }
    }

    Ok(())
}
