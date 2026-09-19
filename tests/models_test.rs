use std::path::PathBuf;
use sweepx::models::*;

#[test]
fn test_application_creation_and_defaults() {
    let mut app = Application::new("org.mozilla.firefox", "Firefox", InstallMethod::Flatpak);
    assert_eq!(app.id, "org.mozilla.firefox");
    assert_eq!(app.name, "Firefox");
    assert_eq!(app.display_name, "Firefox");
    assert_eq!(app.install_method, InstallMethod::Flatpak);
    assert_eq!(app.total_size_bytes, 0);
    assert!(app.artifacts.is_empty());
    assert!(!app.is_system);

    app.total_size_bytes = 150 * 1024 * 1024; // 150 MB
    assert_eq!(app.formatted_size(), "150.0 MB");
}

#[test]
fn test_artifact_and_residual_models() {
    let path = PathBuf::from("/home/test/.config/slack");
    let artifact = AppArtifact::new(ArtifactKind::ConfigDir, path.clone(), 1024 * 1024, true);
    assert_eq!(artifact.kind.display_name(), "Configuration (~/.config)");
    assert_eq!(artifact.size_bytes, 1024 * 1024);
    assert!(!artifact.is_protected);

    let residual = ResidualCandidate::new(
        "slack",
        "Slack Desktop",
        path,
        ArtifactKind::ConfigDir,
        1024 * 1024,
        0.95,
        false,
    );
    assert_eq!(residual.app_id, "slack");
    assert_eq!(residual.confidence, 0.95);
    assert!(residual.selected_for_deletion);
}

#[test]
fn test_serialization_roundtrip() {
    let app = Application::new("code", "Visual Studio Code", InstallMethod::NativeApt);
    let serialized = serde_json::to_string(&app).expect("Serialization failed");
    let deserialized: Application = serde_json::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(app, deserialized);
}

#[test]
fn test_format_size_helper() {
    assert_eq!(format_size(500), "500 B");
    assert_eq!(format_size(2048), "2.0 KB");
    assert_eq!(format_size(15 * 1024 * 1024), "15.0 MB");
    assert_eq!(format_size(3 * 1024 * 1024 * 1024 + 500 * 1024 * 1024), "3.49 GB");
}
