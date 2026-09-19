use super::artifact::AppArtifact;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Origin packaging method for an installed Linux application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstallMethod {
    NativeApt,
    NativeDnf,
    NativePacman,
    Flatpak,
    Snap,
    AppImage,
    ManualOpt,
    CustomDesktop,
}

impl InstallMethod {
    pub fn badge_label(&self) -> &'static str {
        match self {
            InstallMethod::NativeApt => "APT",
            InstallMethod::NativeDnf => "DNF",
            InstallMethod::NativePacman => "Pacman",
            InstallMethod::Flatpak => "Flatpak",
            InstallMethod::Snap => "Snap",
            InstallMethod::AppImage => "AppImage",
            InstallMethod::ManualOpt => "Manual (/opt)",
            InstallMethod::CustomDesktop => "Desktop Entry",
        }
    }

    pub fn is_native_system(&self) -> bool {
        matches!(
            self,
            InstallMethod::NativeApt | InstallMethod::NativeDnf | InstallMethod::NativePacman
        )
    }

    pub fn is_container(&self) -> bool {
        matches!(
            self,
            InstallMethod::Flatpak | InstallMethod::Snap | InstallMethod::AppImage
        )
    }
}

/// Unified representation of any installed application on the Linux host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub version: Option<String>,
    pub install_method: InstallMethod,
    pub exec_path: Option<PathBuf>,
    pub icon: Option<String>,
    pub desktop_file: Option<PathBuf>,
    pub install_date: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
    pub total_size_bytes: u64,
    pub artifacts: Vec<AppArtifact>,
    pub dependencies: Vec<String>,
    pub is_system: bool,
    pub description: Option<String>,
}

impl Application {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        install_method: InstallMethod,
    ) -> Self {
        let name_str = name.into();
        Self {
            id: id.into(),
            display_name: name_str.clone(),
            name: name_str,
            version: None,
            install_method,
            exec_path: None,
            icon: None,
            desktop_file: None,
            install_date: None,
            last_updated: None,
            total_size_bytes: 0,
            artifacts: Vec::new(),
            dependencies: Vec::new(),
            is_system: false,
            description: None,
        }
    }

    pub fn formatted_size(&self) -> String {
        format_size(self.total_size_bytes)
    }
}

/// Utility to format bytes into human-readable strings (KB, MB, GB).
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
