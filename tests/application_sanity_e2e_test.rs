use chrono::Utc;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use sweepx::cleaner::{
    execute_purge_package, execute_purge_residuals, execute_system_sweep, DeletionMode,
    SafetyValidator, SweepCategory, SystemSweepItem,
};
use sweepx::db::{AuditLogEntry, Database};
use sweepx::models::{AppArtifact, Application, ArtifactKind, InstallMethod, ResidualCandidate};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use tempfile::tempdir;

/// Helper to write a dummy text file.
fn create_dummy_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create parent dir");
    }
    let mut file = File::create(path).expect("Failed to create dummy file");
    file.write_all(content.as_bytes()).expect("Failed to write content");
}

/// Helper to write a binary file of arbitrary byte size.
fn create_dummy_binary(path: &Path, size_bytes: usize) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create parent dir");
    }
    let data = vec![0x7f, 0x45, 0x4c, 0x46]; // ELF magic bytes
    let padding = vec![0x90; size_bytes.saturating_sub(4)];
    let mut file = File::create(path).expect("Failed to create dummy binary");
    file.write_all(&data).expect("Failed to write magic bytes");
    file.write_all(&padding).expect("Failed to write padding bytes");
}

#[tokio::test]
async fn test_sanity_manual_binary_lifecycle_and_purge() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let bin_dir = tmp.path().join("local").join("bin");
    let binary_path = bin_dir.join("feynman_cli");

    // 1. Create a dummy executable binary
    create_dummy_binary(&binary_path, 2048);
    assert!(binary_path.exists());
    assert_eq!(binary_path.metadata().unwrap().len(), 2048);

    // 2. Build Application model representing ManualOpt binary
    let mut app = Application::new("feynman_cli", "Feynman CLI", InstallMethod::ManualOpt);
    app.exec_path = Some(binary_path.clone());
    app.total_size_bytes = 2048;
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::Binary,
        binary_path.clone(),
        2048,
        true,
    ));

    // 3. Execute package purge in Permanent mode
    let purge_res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(purge_res.is_ok(), "Package purge must succeed: {:?}", purge_res);

    // 4. Verify physical removal from filesystem
    assert!(!binary_path.exists(), "Binary must be physically removed from disk");

    // 5. Verify audit persistence
    let db_path = tmp.path().join("audit.db");
    let db = Database::open(&db_path).expect("Failed to open test database");
    let audit_entry = AuditLogEntry {
        id: None,
        app_id: app.id.clone(),
        app_name: app.display_name.clone(),
        install_method: app.install_method.badge_label().to_string(),
        timestamp: Utc::now(),
        freed_bytes: app.total_size_bytes,
        deleted_paths: vec![binary_path.clone()],
        status: "SUCCESS".to_string(),
        error_details: None,
    };
    db.record_audit(&audit_entry).expect("Record audit");

    let history = db.get_audit_history().expect("Fetch audit history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].deleted_paths, vec![binary_path]);
    assert_eq!(history[0].freed_bytes, 2048);
}

#[tokio::test]
async fn test_sanity_custom_desktop_app_with_nested_payload_and_launcher() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let opt_dir = tmp.path().join("opt").join("graph_studio");
    let bin_path = opt_dir.join("bin").join("graph_studio");
    let config_asset = opt_dir.join("share").join("presets.json");
    let lib_asset = opt_dir.join("lib").join("librender.so");

    let desktop_dir = tmp.path().join("applications");
    let desktop_path = desktop_dir.join("graph_studio.desktop");

    // 1. Create multi-file directory payload
    create_dummy_binary(&bin_path, 4096);
    create_dummy_file(&config_asset, "{\"default_graph\": \"force_directed\"}");
    create_dummy_binary(&lib_asset, 8192);

    let desktop_content = format!(
        "[Desktop Entry]\nName=Graph Studio\nExec={}\nIcon=graph-studio\nType=Application\n",
        bin_path.display()
    );
    create_dummy_file(&desktop_path, &desktop_content);

    assert!(bin_path.exists());
    assert!(config_asset.exists());
    assert!(lib_asset.exists());
    assert!(desktop_path.exists());

    // 2. Build Application model
    let mut app = Application::new("graph_studio", "Graph Studio", InstallMethod::CustomDesktop);
    app.exec_path = Some(bin_path.clone());
    app.desktop_file = Some(desktop_path.clone());
    app.total_size_bytes = 4096 + 8192;
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::AppDir,
        opt_dir.clone(),
        4096 + 8192,
        true,
    ));

    // 3. Purge package
    let res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(res.is_ok(), "Custom desktop purge must succeed: {:?}", res);

    // 4. Assert entire payload folder and desktop launcher are unlinked
    assert!(!opt_dir.exists(), "Payload directory must be deleted");
    assert!(!bin_path.exists(), "Binary must be deleted");
    assert!(!config_asset.exists(), "Nested assets must be deleted");
    assert!(!desktop_path.exists(), "Desktop file must be deleted");
}

#[tokio::test]
async fn test_sanity_appimage_application_and_integration() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let apps_dir = tmp.path().join("Applications");
    let appimage_path = apps_dir.join("VideoEditor-x86_64.AppImage");

    let desktop_dir = tmp.path().join("applications");
    let desktop_path = desktop_dir.join("appimagekit_videoeditor.desktop");

    // 1. Create dummy AppImage file and desktop file
    create_dummy_binary(&appimage_path, 16384);
    create_dummy_file(
        &desktop_path,
        "[Desktop Entry]\nName=Video Editor\nExec=VideoEditor-x86_64.AppImage\nType=Application\n",
    );

    assert!(appimage_path.exists());
    assert!(desktop_path.exists());

    // 2. Build model
    let mut app = Application::new("videoeditor", "Video Editor", InstallMethod::AppImage);
    app.exec_path = Some(appimage_path.clone());
    app.desktop_file = Some(desktop_path.clone());
    app.total_size_bytes = 16384;
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::Binary,
        appimage_path.clone(),
        16384,
        true,
    ));

    // 3. Purge
    let res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(res.is_ok());

    // 4. Verify physical unlinking
    assert!(!appimage_path.exists(), "AppImage file must be removed");
    assert!(!desktop_path.exists(), "AppImage desktop entry must be removed");
}

#[tokio::test]
async fn test_sanity_bottles_wine_prefix_and_program_deletion() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let bottle_dir = tmp
        .path()
        .join("bottles")
        .join("prefixes")
        .join("accounting_suite");
    let exe_path = bottle_dir
        .join("drive_c")
        .join("Program Files")
        .join("Accounting")
        .join("accounting.exe");
    let config_yml = bottle_dir.join("bottle.yml");
    let desktop_path = tmp
        .path()
        .join("applications")
        .join("bottles-accounting.desktop");

    create_dummy_binary(&exe_path, 8192);
    create_dummy_file(&config_yml, "Arch: win64\nName: Accounting Suite\n");
    create_dummy_file(
        &desktop_path,
        "[Desktop Entry]\nName=Accounting Suite\nExec=bottles-cli run\n",
    );

    assert!(exe_path.exists());
    assert!(config_yml.exists());
    assert!(desktop_path.exists());

    let mut app = Application::new("bottles_accounting", "Accounting Suite", InstallMethod::Bottles);
    app.exec_path = Some(exe_path.clone());
    app.desktop_file = Some(desktop_path.clone());
    app.total_size_bytes = 8192;
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::AppDir,
        bottle_dir.clone(),
        8192,
        true,
    ));

    let res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(res.is_ok());

    assert!(!bottle_dir.exists(), "Bottles prefix must be deleted");
    assert!(!exe_path.exists(), "Wine binary must be deleted");
    assert!(!desktop_path.exists(), "Bottles launcher must be deleted");
}

#[tokio::test]
async fn test_sanity_residual_purging_across_diverse_file_types_and_xdg_trees() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let base = tmp.path();

    // Setup multiple realistic residual directories and file types:
    // 1. Config directory with JSON and INI
    let config_dir = base.join(".config").join("multi_app");
    create_dummy_file(&config_dir.join("settings.json"), "{\"sync\": true}");
    create_dummy_file(&config_dir.join("profiles.ini"), "[default]\nuser=test\n");

    // 2. Cache directory with binary blob and SQLite dummy
    let cache_dir = base.join(".cache").join("multi_app");
    create_dummy_binary(&cache_dir.join("cached_blobs.bin"), 10240);
    create_dummy_file(&cache_dir.join("index.sqlite"), "SQLite format 3\0");

    // 3. State directory with history log
    let state_dir = base.join(".local").join("state").join("multi_app");
    create_dummy_file(&state_dir.join("history.log"), "2026-10-25 21:00 user session opened\n");

    // 4. Sandbox .var directory with nested cache/data
    let sandbox_dir = base.join(".var").join("app").join("com.example.MultiApp");
    create_dummy_file(&sandbox_dir.join("cache").join("font_cache.dat"), "font metadata");

    // 5. Desktop launcher entry
    let desktop_file = base
        .join(".local")
        .join("share")
        .join("applications")
        .join("multi_app.desktop");
    create_dummy_file(&desktop_file, "[Desktop Entry]\nName=Multi App\n");

    // Verify all exist before purge
    assert!(config_dir.exists());
    assert!(cache_dir.exists());
    assert!(state_dir.exists());
    assert!(sandbox_dir.exists());
    assert!(desktop_file.exists());

    let candidates = vec![
        ResidualCandidate::new(
            "multi_app",
            "Multi App",
            config_dir.clone(),
            ArtifactKind::ConfigDir,
            512,
            1.0,
            true,
        ),
        ResidualCandidate::new(
            "multi_app",
            "Multi App",
            cache_dir.clone(),
            ArtifactKind::CacheDir,
            10240,
            1.0,
            true,
        ),
        ResidualCandidate::new(
            "multi_app",
            "Multi App",
            state_dir.clone(),
            ArtifactKind::StateDir,
            256,
            1.0,
            true,
        ),
        ResidualCandidate::new(
            "multi_app",
            "Multi App",
            sandbox_dir.clone(),
            ArtifactKind::SandboxDir,
            1024,
            1.0,
            true,
        ),
        ResidualCandidate::new(
            "multi_app",
            "Multi App",
            desktop_file.clone(),
            ArtifactKind::DesktopEntry,
            128,
            1.0,
            true,
        ),
    ];

    let report = execute_purge_residuals(&candidates, DeletionMode::Permanent).await;
    assert!(report.success, "Residual purge should succeed without errors: {:?}", report.errors);
    assert_eq!(report.deleted_paths.len(), 5);
    assert_eq!(report.freed_bytes, 512 + 10240 + 256 + 1024 + 128);

    // Verify all physical directories and files are gone
    assert!(!config_dir.exists(), "Config dir must be deleted");
    assert!(!cache_dir.exists(), "Cache dir must be deleted");
    assert!(!state_dir.exists(), "State dir must be deleted");
    assert!(!sandbox_dir.exists(), "Sandbox dir must be deleted");
    assert!(!desktop_file.exists(), "Desktop entry must be deleted");
}

#[tokio::test]
async fn test_sanity_system_sweep_file_deletion_and_recreation() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let cache_junk_dir = tmp.path().join("cache").join("thumbnails");
    let thumbnail_1 = cache_junk_dir.join("thumb_a.png");
    let thumbnail_2 = cache_junk_dir.join("thumb_b.png");

    create_dummy_file(&thumbnail_1, "png dummy content a");
    create_dummy_file(&thumbnail_2, "png dummy content b");

    let iso_file = tmp.path().join("Downloads").join("test_installer.iso");
    create_dummy_binary(&iso_file, 50000);

    assert!(thumbnail_1.exists());
    assert!(thumbnail_2.exists());
    assert!(iso_file.exists());

    let sweep_items = vec![
        SystemSweepItem {
            id: "system-thumbnails".to_string(),
            title: "Thumbnail Cache".to_string(),
            category: SweepCategory::SystemUserCache,
            description: "Thumbnail cache".to_string(),
            reclaimable_bytes: 1000,
            paths: vec![cache_junk_dir.clone()],
            command: None,
            selected: true,
        },
        SystemSweepItem {
            id: "installer-iso".to_string(),
            title: "Installer ISO".to_string(),
            category: SweepCategory::InstallerArchives,
            description: "Bulky ISO file".to_string(),
            reclaimable_bytes: 50000,
            paths: vec![iso_file.clone()],
            command: None,
            selected: true,
        },
    ];

    let report = execute_system_sweep(&sweep_items).await;
    assert_eq!(report.items_cleaned, 2);
    assert_eq!(report.freed_bytes, 51000);
    assert!(report.errors.is_empty(), "Sweep should have no errors: {:?}", report.errors);

    // Files inside thumbnail cache were wiped
    assert!(!thumbnail_1.exists());
    assert!(!thumbnail_2.exists());
    // Directory itself was recreated clean
    assert!(cache_junk_dir.exists());
    // ISO was deleted
    assert!(!iso_file.exists());
}

#[tokio::test]
async fn test_sanity_safety_validator_blocks_critical_paths() {
    let protected_paths = vec![
        PathBuf::from("/"),
        PathBuf::from("/usr"),
        PathBuf::from("/etc"),
        PathBuf::from("/var/lib/flatpak"),
    ];

    for path in &protected_paths {
        let res = SafetyValidator::is_path_safe_to_delete(path);
        assert!(res.is_err(), "SafetyValidator must reject path: {}", path.display());
    }

    // Residual purge with a protected path must be rejected safely
    let unsafe_candidate = ResidualCandidate::new(
        "system",
        "System Files",
        PathBuf::from("/usr/lib"),
        ArtifactKind::Unknown,
        999999,
        1.0,
        false,
    );

    let report = execute_purge_residuals(&[unsafe_candidate], DeletionMode::Permanent).await;
    assert!(!report.success);
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].1.contains("Blocked by safety validator"));
}

#[tokio::test]
async fn test_sanity_full_end_to_end_parse_inspect_purge_and_audit() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let base = tmp.path();

    // 1. Create a real desktop entry file and executable
    let bin_path = base.join("bin").join("cadviewer");
    create_dummy_binary(&bin_path, 32768);

    let desktop_path = base.join("cadviewer.desktop");
    let desktop_content = format!(
        "[Desktop Entry]\nName=CAD Viewer\nGenericName=Engineering Drawing Tool\nExec={}\nType=Application\nCategories=Graphics;Engineering;\n",
        bin_path.display()
    );
    create_dummy_file(&desktop_path, &desktop_content);

    // 2. Parse desktop entry as the scanner does
    let mut app = parse_desktop_entry_content(&desktop_content, &desktop_path)
        .expect("Desktop file must parse");
    app.exec_path = Some(bin_path.clone());
    app.desktop_file = Some(desktop_path.clone());
    app.total_size_bytes = 32768;

    assert_eq!(app.id, "cadviewer");
    assert_eq!(app.display_name, "CAD Viewer (Engineering Drawing Tool)");

    // 3. Create dummy cache residuals
    let cache_dir = base.join(".cache").join("cadviewer");
    create_dummy_file(&cache_dir.join("cache.dat"), "cad mesh cache");

    let residuals = vec![ResidualCandidate::new(
        &app.id,
        &app.name,
        cache_dir.clone(),
        ArtifactKind::CacheDir,
        1024,
        0.95,
        true,
    )];

    // 4. Perform combined package + residual purge
    let pkg_res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(pkg_res.is_ok(), "Package purge must succeed: {:?}", pkg_res);

    let res_report = execute_purge_residuals(&residuals, DeletionMode::Permanent).await;
    assert!(res_report.success, "Residual purge must succeed");

    // 5. Assert physical deletion of all files
    assert!(!bin_path.exists(), "Binary must be deleted");
    assert!(!desktop_path.exists(), "Desktop file must be deleted");
    assert!(!cache_dir.exists(), "Cache directory must be deleted");

    // 6. Record in Database and verify retrieval
    let db_path = base.join("system_audit.db");
    let db = Database::open(&db_path).expect("Open database");

    let mut deleted_paths = res_report.deleted_paths.clone();
    deleted_paths.insert(0, bin_path.clone());
    deleted_paths.push(desktop_path.clone());

    let audit = AuditLogEntry {
        id: None,
        app_id: app.id.clone(),
        app_name: app.display_name.clone(),
        install_method: app.install_method.badge_label().to_string(),
        timestamp: Utc::now(),
        freed_bytes: app.total_size_bytes + res_report.freed_bytes,
        deleted_paths: deleted_paths.clone(),
        status: "SUCCESS".to_string(),
        error_details: None,
    };
    db.record_audit(&audit).expect("Record audit");

    let history = db.get_audit_history().expect("Get audit history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].app_id, "cadviewer");
    assert_eq!(history[0].freed_bytes, 32768 + 1024);
    assert_eq!(history[0].deleted_paths.len(), 3);
    assert!(history[0].deleted_paths.contains(&bin_path));
    assert!(history[0].deleted_paths.contains(&desktop_path));
    assert!(history[0].deleted_paths.contains(&cache_dir));
}
