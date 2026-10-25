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
    Bottles,
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
            InstallMethod::Bottles => "Bottles",
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
            InstallMethod::Flatpak | InstallMethod::Snap | InstallMethod::AppImage | InstallMethod::Bottles
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

    pub fn formatted_size_with_unit(&self, system: SizeUnitSystem) -> String {
        format_size_with_unit(self.total_size_bytes, system)
    }

    pub fn total_footprint_bytes(&self, residuals: &[super::residual::ResidualCandidate]) -> u64 {
        let residuals_sum: u64 = residuals.iter().map(|r| r.size_bytes).sum();
        self.total_size_bytes + residuals_sum
    }

    pub fn formatted_total_footprint(&self, residuals: &[super::residual::ResidualCandidate]) -> String {
        format_size(self.total_footprint_bytes(residuals))
    }

    pub fn formatted_total_footprint_with_unit(
        &self,
        residuals: &[super::residual::ResidualCandidate],
        system: SizeUnitSystem,
    ) -> String {
        format_size_with_unit(self.total_footprint_bytes(residuals), system)
    }

    /// Calculates known on-disk footprint combining the package payload with known discovered artifacts (e.g. desktop entry, sandboxes, appdirs).
    pub fn known_footprint_bytes(&self) -> u64 {
        let non_binary_artifacts: u64 = self
            .artifacts
            .iter()
            .filter(|a| {
                a.kind != super::artifact::ArtifactKind::Binary
                    && a.kind != super::artifact::ArtifactKind::AppDir
                    && !(a.kind == super::artifact::ArtifactKind::SandboxDir && a.size_bytes == self.total_size_bytes)
            })
            .map(|a| a.size_bytes)
            .sum();
        self.total_size_bytes + non_binary_artifacts
    }

    pub fn formatted_known_footprint(&self) -> String {
        format_size(self.known_footprint_bytes())
    }

    pub fn formatted_known_footprint_with_unit(&self, system: SizeUnitSystem) -> String {
        format_size_with_unit(self.known_footprint_bytes(), system)
    }
}

/// Units for displaying storage metrics: GiB (binary IEC, base 1024) or GB (decimal SI, base 1000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SizeUnitSystem {
    #[default]
    GiB, // IEC binary: B, KiB, MiB, GiB, TiB
    GB,  // SI decimal: B, KB, MB, GB, TB
}

impl SizeUnitSystem {
    pub fn label(&self) -> &'static str {
        match self {
            SizeUnitSystem::GiB => "GiB (1024)",
            SizeUnitSystem::GB => "GB (1000)",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            SizeUnitSystem::GiB => SizeUnitSystem::GB,
            SizeUnitSystem::GB => SizeUnitSystem::GiB,
        }
    }
}

/// Formats bytes according to the specified size unit system (GiB default binary base 1024, or GB decimal base 1000).
pub fn format_size_with_unit(bytes: u64, system: SizeUnitSystem) -> String {
    match system {
        SizeUnitSystem::GiB => {
            const KIB: u64 = 1024;
            const MIB: u64 = 1024 * KIB;
            const GIB: u64 = 1024 * MIB;
            const TIB: u64 = 1024 * GIB;

            if bytes >= TIB {
                format!("{:.2} TiB", bytes as f64 / TIB as f64)
            } else if bytes >= GIB {
                format!("{:.2} GiB", bytes as f64 / GIB as f64)
            } else if bytes >= MIB {
                format!("{:.1} MiB", bytes as f64 / MIB as f64)
            } else if bytes >= KIB {
                format!("{:.1} KiB", bytes as f64 / KIB as f64)
            } else {
                format!("{} B", bytes)
            }
        }
        SizeUnitSystem::GB => {
            const KB: u64 = 1000;
            const MB: u64 = 1000 * KB;
            const GB: u64 = 1000 * MB;
            const TB: u64 = 1000 * GB;

            if bytes >= TB {
                format!("{:.2} TB", bytes as f64 / TB as f64)
            } else if bytes >= GB {
                format!("{:.2} GB", bytes as f64 / GB as f64)
            } else if bytes >= MB {
                format!("{:.1} MB", bytes as f64 / MB as f64)
            } else if bytes >= KB {
                format!("{:.1} KB", bytes as f64 / KB as f64)
            } else {
                format!("{} B", bytes)
            }
        }
    }
}

/// Utility to format bytes into human-readable strings (defaults to GiB binary system).
pub fn format_size(bytes: u64) -> String {
    format_size_with_unit(bytes, SizeUnitSystem::GiB)
}
