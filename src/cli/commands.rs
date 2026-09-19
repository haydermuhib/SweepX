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

        /// Include system libraries, base frameworks, and shared runtimes
        #[arg(short = 's', long)]
        all_system: bool,

        /// Output results in formatted JSON
        #[arg(long)]
        json: bool,
    },

    /// Inspect detailed artifacts, dependencies, and residual files for a specific application
    Inspect {
        /// Application ID or name
        app_id: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Deep purge an application and its associated configuration and cache files
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

    /// Show uninstallation audit log history
    History,

    /// Scan and clean system bloat, obsolete container revisions, unused runtimes, and package caches
    Sweep {
        /// Perform dry-run without deleting files or removing revisions
        #[arg(long)]
        dry_run: bool,

        /// Output findings in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Watch and record exact filesystem changes during an unmanaged installation (CheckInstall-style)
    Watch {
        /// Application name or identifier
        #[arg(short, long)]
        name: String,

        /// The installation command and arguments to execute
        #[arg(required = true, num_args = 1..)]
        command: Vec<String>,
    },
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
