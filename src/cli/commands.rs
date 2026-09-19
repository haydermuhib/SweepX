use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "sweepx")]
#[command(author = "SweepX Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Unified Linux Application Tracker & Deep Uninstaller", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the SweepX graphical desktop interface (Default)
    Gui,

    /// List all discovered applications on the host
    List {
        /// Filter by packaging category
        #[arg(short, long, value_enum, default_value = "all")]
        category: CategoryFilter,

        /// Output results in formatted JSON
        #[arg(long)]
        json: bool,
    },

    /// Inspect detailed artifacts, dependencies, and residual files for an application
    Inspect {
        /// Application ID or name
        app_id: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Deep purge an application and its associated residual files
    Purge {
        /// Application ID or name to purge
        app_id: String,

        /// Perform a dry-run without deleting files
        #[arg(long)]
        dry_run: bool,

        /// Permanently delete files instead of moving them to desktop trash
        #[arg(long)]
        permanent: bool,
    },

    /// Scan system for orphaned residual directories and caches
    Residuals {
        /// Automatically clean detected orphaned residuals
        #[arg(long)]
        clean: bool,

        /// Permanently delete instead of moving to trash
        #[arg(long)]
        permanent: bool,
    },

    /// Show uninstallation audit log history
    History,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CategoryFilter {
    All,
    Native,
    Flatpak,
    Snap,
    Appimage,
    Manual,
}
