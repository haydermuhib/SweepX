use chrono::Utc;
use std::fs::File;
use std::io::Write;
use sweepx::cleaner::{execute_purge_residuals, DeletionMode, SafetyValidator};
use sweepx::db::{AuditLogEntry, Database};
use sweepx::models::{ArtifactKind, InstallMethod, ResidualCandidate};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use tempfile::tempdir;

#[tokio::test]
async fn test_end_to_end_discovery_residual_purge_and_audit() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let base_path = tmp.path();

    let config_dir = base_path.join(".config").join("mock-ide");
    let cache_dir = base_path.join(".cache").join("mock-ide");
    std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    std::fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

    let config_file = config_dir.join("config.json");
    let mut f = File::create(&config_file).expect("Failed to create config file");
    writeln!(f, "{{\"theme\":\"dark\",\"plugins\":[\"rust\"]}}").expect("Failed to write");

    let desktop_content = r#"
[Desktop Entry]
Name=Mock IDE
GenericName=Integrated Development Environment
Exec=/opt/mock-ide/bin/mock-ide %F
Icon=mock-ide
Type=Application
"#;
    let desktop_file_path = base_path.join("mock-ide.desktop");
    let app = parse_desktop_entry_content(desktop_content, &desktop_file_path)
        .expect("Should parse desktop entry");

    assert_eq!(app.id, "mock-ide");
    assert_eq!(app.display_name, "Mock IDE (Integrated Development Environment)");
    assert_eq!(app.install_method, InstallMethod::ManualOpt);

    assert!(SafetyValidator::is_path_safe_to_delete(&config_dir).is_ok());
    assert!(SafetyValidator::is_path_safe_to_delete(&cache_dir).is_ok());

    let candidates = vec![
        ResidualCandidate::new(
            &app.id,
            &app.name,
            config_dir.clone(),
            ArtifactKind::ConfigDir,
            1024,
            0.95,
            false,
        ),
        ResidualCandidate::new(
            &app.id,
            &app.name,
            cache_dir.clone(),
            ArtifactKind::CacheDir,
            2048,
            0.95,
            false,
        ),
    ];

    let report = execute_purge_residuals(&candidates, DeletionMode::Permanent).await;
    assert!(report.success);
    assert_eq!(report.deleted_paths.len(), 2);
    assert_eq!(report.freed_bytes, 1024 + 2048);
    assert!(!config_dir.exists());
    assert!(!cache_dir.exists());

    let db_path = base_path.join("test_audit.db");
    let mut db = Database::open(&db_path).expect("Failed to open test db");

    db.save_apps(&[app.clone()]).expect("Failed to save app");

    let audit_entry = AuditLogEntry {
        id: None,
        app_id: app.id.clone(),
        app_name: app.display_name.clone(),
        install_method: app.install_method.badge_label().to_string(),
        timestamp: Utc::now(),
        freed_bytes: report.freed_bytes,
        deleted_paths: report.deleted_paths.clone(),
        status: "SUCCESS".to_string(),
        error_details: None,
    };

    db.record_audit(&audit_entry).expect("Failed to record audit");

    let history = db.get_audit_history().expect("Failed to fetch history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].app_id, "mock-ide");
    assert_eq!(history[0].status, "SUCCESS");
    assert_eq!(history[0].freed_bytes, 3072);
}
