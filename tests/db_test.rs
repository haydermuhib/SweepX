use chrono::Utc;
use std::path::PathBuf;
use sweepx::db::{AuditLogEntry, Database};
use sweepx::models::{Application, InstallMethod};

#[test]
fn test_database_in_memory_save_and_audit() {
    let mut db = Database::open_in_memory().expect("Should open in-memory db");

    let mut app = Application::new("org.gimp.GIMP", "GIMP", InstallMethod::Flatpak);
    app.version = Some("2.10.36".to_string());
    app.total_size_bytes = 250 * 1024 * 1024;

    db.save_apps(&[app]).expect("Should save app");

    let audit = AuditLogEntry {
        id: None,
        app_id: "org.gimp.GIMP".to_string(),
        app_name: "GIMP".to_string(),
        install_method: "Flatpak".to_string(),
        timestamp: Utc::now(),
        freed_bytes: 250 * 1024 * 1024,
        deleted_paths: vec![PathBuf::from("/home/user/.var/app/org.gimp.GIMP")],
        status: "SUCCESS".to_string(),
        error_details: None,
    };

    db.record_audit(&audit).expect("Should record audit");

    let history = db.get_audit_history().expect("Should get history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].app_id, "org.gimp.GIMP");
    assert_eq!(history[0].freed_bytes, 250 * 1024 * 1024);
    assert_eq!(history[0].deleted_paths.len(), 1);
}
