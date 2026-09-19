use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Categorization of application-related filesystem artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactKind {
    Binary,
    DesktopEntry,
    Icon,
    ConfigDir,
    CacheDir,
    DataDir,
    StateDir,
    LogDir,
    SystemdService,
    SandboxDir,
    AppDir,
    Unknown,
}

impl ArtifactKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ArtifactKind::Binary => "Executable Binary",
            ArtifactKind::DesktopEntry => "Desktop Launcher (.desktop)",
            ArtifactKind::Icon => "Application Icon",
            ArtifactKind::ConfigDir => "Configuration (~/.config)",
            ArtifactKind::CacheDir => "Cache Data (~/.cache)",
            ArtifactKind::DataDir => "Application Data (~/.local/share)",
            ArtifactKind::StateDir => "Application State (~/.local/state)",
            ArtifactKind::LogDir => "System / User Logs",
            ArtifactKind::SystemdService => "Systemd Unit / Service",
            ArtifactKind::SandboxDir => "Container Sandbox (~/.var / ~/snap)",
            ArtifactKind::AppDir => "Application Directory (/opt or local)",
            ArtifactKind::Unknown => "Other Artifact",
        }
    }
}

/// Represents a concrete filesystem artifact associated with an application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppArtifact {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub exists: bool,
    pub is_protected: bool,
}

impl AppArtifact {
    pub fn new(kind: ArtifactKind, path: PathBuf, size_bytes: u64, exists: bool) -> Self {
        Self {
            kind,
            path,
            size_bytes,
            exists,
            is_protected: false,
        }
    }
}
