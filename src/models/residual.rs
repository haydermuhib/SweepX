use super::artifact::ArtifactKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A residual or leftover directory/file discovered through heuristic or pattern scanning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidualCandidate {
    pub app_id: String,
    pub app_name: String,
    pub path: PathBuf,
    pub kind: ArtifactKind,
    pub size_bytes: u64,
    pub confidence: f32, // 0.0 to 1.0 confidence that this belongs to the app
    pub is_orphaned: bool, // true if the parent app binary is already uninstalled
    pub selected_for_deletion: bool,
}

impl ResidualCandidate {
    pub fn new(
        app_id: impl Into<String>,
        app_name: impl Into<String>,
        path: PathBuf,
        kind: ArtifactKind,
        size_bytes: u64,
        confidence: f32,
        is_orphaned: bool,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            app_name: app_name.into(),
            path,
            kind,
            size_bytes,
            confidence,
            is_orphaned,
            selected_for_deletion: true,
        }
    }
}
