pub mod apt;
pub mod dnf;
pub mod pacman;

pub use apt::AptManager;
pub use dnf::DnfManager;
pub use pacman::PacmanManager;

use crate::models::Application;
use std::path::PathBuf;

/// Unified enum dispatch for host native package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBackend {
    Dnf,
    Apt,
    Pacman,
}

impl NativeBackend {
    pub fn name(&self) -> &'static str {
        match self {
            NativeBackend::Dnf => DnfManager.name(),
            NativeBackend::Apt => AptManager.name(),
            NativeBackend::Pacman => PacmanManager.name(),
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            NativeBackend::Dnf => DnfManager.is_available(),
            NativeBackend::Apt => AptManager.is_available(),
            NativeBackend::Pacman => PacmanManager.is_available(),
        }
    }

    pub async fn list_installed(&self) -> Vec<Application> {
        match self {
            NativeBackend::Dnf => DnfManager.list_installed().await,
            NativeBackend::Apt => AptManager.list_installed().await,
            NativeBackend::Pacman => PacmanManager.list_installed().await,
        }
    }

    pub async fn get_package_files(&self, pkg: &str) -> Vec<PathBuf> {
        match self {
            NativeBackend::Dnf => DnfManager.get_package_files(pkg).await,
            NativeBackend::Apt => AptManager.get_package_files(pkg).await,
            NativeBackend::Pacman => PacmanManager.get_package_files(pkg).await,
        }
    }

    pub async fn get_dependencies(&self, pkg: &str) -> Vec<String> {
        match self {
            NativeBackend::Dnf => DnfManager.get_dependencies(pkg).await,
            NativeBackend::Apt => AptManager.get_dependencies(pkg).await,
            NativeBackend::Pacman => PacmanManager.get_dependencies(pkg).await,
        }
    }
}

/// Detects and returns all available native package managers on the current system.
pub fn detect_available_package_managers() -> Vec<NativeBackend> {
    let mut managers = Vec::new();

    if DnfManager.is_available() {
        managers.push(NativeBackend::Dnf);
    }
    if AptManager.is_available() {
        managers.push(NativeBackend::Apt);
    }
    if PacmanManager.is_available() {
        managers.push(NativeBackend::Pacman);
    }

    managers
}
