use chrono::Utc;
use std::path::PathBuf;
use sweepx::db::{AuditLogEntry, Database};
use sweepx::ui::{Message, NavigationTab, SweepXApp};

#[test]
fn test_audit_history_database_records_and_retrieves_deleted_paths() {
    let db = Database::open_in_memory().expect("Should open in-memory db");

    let deleted_paths = vec![
        PathBuf::from("/home/user/.var/app/com.spotify.Client/cache"),
        PathBuf::from("/home/user/.var/app/com.spotify.Client/config"),
        PathBuf::from("/home/user/.local/share/spotify"),
    ];

    let entry = AuditLogEntry {
        id: None,
        app_id: "com.spotify.Client".to_string(),
        app_name: "Spotify".to_string(),
        install_method: "Flatpak".to_string(),
        timestamp: Utc::now(),
        freed_bytes: 520 * 1024 * 1024,
        deleted_paths: deleted_paths.clone(),
        status: "SUCCESS".to_string(),
        error_details: None,
    };

    db.record_audit(&entry).expect("Should record audit entry");

    let history = db.get_audit_history().expect("Should retrieve audit history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].app_id, "com.spotify.Client");
    assert_eq!(history[0].app_name, "Spotify");
    assert_eq!(history[0].status, "SUCCESS");
    assert_eq!(history[0].freed_bytes, 520 * 1024 * 1024);
    assert_eq!(history[0].deleted_paths, deleted_paths);
}

#[test]
fn test_audit_inspector_modal_lifecycle_and_view() {
    let (mut app_state, _task) = SweepXApp::new();

    // 1. Initial state has no audit inspector active
    assert!(app_state.inspector_audit_entry.is_none());

    let deleted_paths = vec![
        PathBuf::from("/home/user/.cache/myapp"),
        PathBuf::from("/home/user/.config/myapp"),
    ];

    let entry = AuditLogEntry {
        id: Some(1),
        app_id: "org.example.MyApp".to_string(),
        app_name: "MyApp".to_string(),
        install_method: "Flatpak".to_string(),
        timestamp: Utc::now(),
        freed_bytes: 42 * 1024 * 1024,
        deleted_paths: deleted_paths.clone(),
        status: "SUCCESS".to_string(),
        error_details: None,
    };

    // 2. Load audit history into app
    let _ = app_state.update(Message::AuditHistoryLoaded(vec![entry.clone()]));
    assert_eq!(app_state.audit_history.len(), 1);

    // 3. Switch to Audit History tab and render base view without panic
    let _ = app_state.update(Message::SwitchTab(NavigationTab::AuditHistory));
    assert_eq!(app_state.active_tab, NavigationTab::AuditHistory);
    {
        let _view = app_state.view();
    }

    // 4. Open Audit Inspector
    let _ = app_state.update(Message::OpenAuditInspector(entry.clone()));
    assert!(app_state.inspector_audit_entry.is_some());
    let active_inspector = app_state.inspector_audit_entry.as_ref().unwrap();
    assert_eq!(active_inspector.app_id, "org.example.MyApp");
    assert_eq!(active_inspector.deleted_paths.len(), 2);

    // 5. Render view with overlay active
    {
        let _modal_view = app_state.view();
    }

    // 6. Close Audit Inspector
    let _ = app_state.update(Message::CloseAuditInspector);
    assert!(app_state.inspector_audit_entry.is_none());
}

#[test]
fn test_purge_modal_consistency_with_inspect_and_audit() {
    use sweepx::models::{Application, ArtifactKind, InstallMethod, ResidualCandidate};

    let (mut app_state, _task) = SweepXApp::new();

    let mut app = Application::new("ai.storyteller.cadcraft", "CADCraft", InstallMethod::Flatpak);
    app.display_name = "CADCraft (Computer-Aided Design)".to_string();
    app.exec_path = Some(PathBuf::from("/usr/bin/flatpak"));
    app.desktop_file = Some(PathBuf::from("/home/user/.local/share/flatpak/exports/share/applications/ai.storyteller.cadcraft.desktop"));
    app.total_size_bytes = 125 * 1024 * 1024; // 125 MiB base payload

    let residuals = vec![
        ResidualCandidate::new(
            &app.id,
            &app.name,
            PathBuf::from("/home/user/.var/app/ai.storyteller.cadcraft"),
            ArtifactKind::SandboxDir,
            2 * 1024 * 1024,
            1.0,
            true,
        ),
        ResidualCandidate::new(
            &app.id,
            &app.name,
            PathBuf::from("/home/user/.local/share/flatpak/exports/share/applications/ai.storyteller.cadcraft.desktop"),
            ArtifactKind::DesktopEntry,
            609,
            1.0,
            true,
        ),
    ];

    // 1. Open Clean Modal
    let _ = app_state.update(Message::OpenCleanModal(app.clone()));
    assert!(app_state.clean_target.is_some());
    assert!(app_state.purge_package, "Primary package purge must default to true");

    // 2. Load residuals
    let _ = app_state.update(Message::CleanResidualsLoaded(residuals.clone()));
    assert_eq!(app_state.clean_residuals.len(), 2);

    // 3. Render clean modal overlay without panic
    {
        let _clean_view = app_state.view();
    }

    // 4. Toggle purge_package off and on
    let _ = app_state.update(Message::TogglePurgePackage(false));
    assert!(!app_state.purge_package);

    let _ = app_state.update(Message::TogglePurgePackage(true));
    assert!(app_state.purge_package);

    // 5. Close Clean Modal
    let _ = app_state.update(Message::CloseCleanModal);
    assert!(app_state.clean_target.is_none());
    assert!(app_state.clean_residuals.is_empty());
}
