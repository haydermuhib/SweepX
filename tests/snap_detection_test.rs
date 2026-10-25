use std::path::PathBuf;
use sweepx::cleaner::heuristic::discover_residuals_for_app;
use sweepx::models::{Application, ArtifactKind, InstallMethod};
use sweepx::scanner::desktop_entry::{extract_binary_from_exec, parse_desktop_entry_content};
use sweepx::scanner::snap::is_snap_runtime_or_dependency;

#[test]
fn test_snap_exec_binary_extraction_with_env_prefixes() {
    let raw_exec = "env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/notepad-plus-plus_notepad-plus-plus.desktop /var/lib/snapd/snap/bin/notepad-plus-plus %F";
    let extracted = extract_binary_from_exec(raw_exec);
    assert_eq!(extracted, "/var/lib/snapd/snap/bin/notepad-plus-plus");

    let simple_exec = "/snap/bin/spotify %U";
    assert_eq!(extract_binary_from_exec(simple_exec), "/snap/bin/spotify");

    let quoted_exec = "\"/snap/bin/custom app\" --flag";
    assert_eq!(extract_binary_from_exec(quoted_exec), "/snap/bin/custom app");
}

#[test]
fn test_snap_desktop_entry_normalization_notepad_plus_plus() {
    let desktop_content = r#"[Desktop Entry]
X-SnapInstanceName=notepad-plus-plus
Name=Notepad++
GenericName=Text Editor
Comment=Simple and free source code editor.
Version=8.9.6.4
Type=Application
Terminal=false
Icon=/var/lib/snapd/snap/notepad-plus-plus/current/usr/share/pixmaps/notepad-plus-plus.png
X-SnapAppName=notepad-plus-plus
Exec=/var/lib/snapd/snap/bin/notepad-plus-plus %F
StartupWMClass=notepad-plus-plus.exe
Categories=Utility;TextEditor
"#;

    let dummy_path = PathBuf::from("/var/lib/snapd/desktop/applications/notepad-plus-plus_notepad-plus-plus.desktop");
    let app = parse_desktop_entry_content(desktop_content, &dummy_path).expect("Must parse desktop entry");

    assert_eq!(app.id, "notepad-plus-plus");
    assert_eq!(app.name, "Notepad++");
    assert_eq!(app.install_method, InstallMethod::Snap);
    assert_eq!(app.version.as_deref(), Some("8.9.6.4"));
    assert_eq!(app.exec_path.as_ref().unwrap().to_str().unwrap(), "/var/lib/snapd/snap/bin/notepad-plus-plus");
}

#[test]
fn test_parse_snap_list_version_and_runtime_classification() {
    use sweepx::scanner::snap::parse_snap_list_output;

    let snap_list_sample = "Name               Version          Rev    Tracking       Publisher          Notes\n\
notepad-plus-plus  8.9.6.4          444    latest/stable  notepad-plus-plus  -\n\
core22             20240111         1122   latest/stable  canonical✓         base\n";

    let apps = parse_snap_list_output(snap_list_sample);
    assert_eq!(apps.len(), 2);

    let npp = apps.iter().find(|a| a.id == "notepad-plus-plus").expect("Notepad++ must be parsed");
    assert_eq!(npp.version.as_deref(), Some("8.9.6.4"));
    assert_eq!(npp.install_method, InstallMethod::Snap);
    assert!(!npp.is_system);

    let core = apps.iter().find(|a| a.id == "core22").expect("core22 must be parsed");
    assert_eq!(core.version.as_deref(), Some("20240111"));
    assert!(core.is_system);
}

#[tokio::test]
async fn test_snap_residual_discovery_finds_user_sandbox() {
    let app = Application::new("notepad-plus-plus", "Notepad++", InstallMethod::Snap);
    let residuals = discover_residuals_for_app(&app).await;

    // If ~/snap/notepad-plus-plus exists on this host, it must be discovered
    if let Some(home) = dirs::home_dir() {
        let expected = home.join("snap/notepad-plus-plus");
        if expected.exists() {
            let found = residuals.iter().find(|r| r.kind == ArtifactKind::SandboxDir);
            assert!(found.is_some(), "Must discover ~/snap/notepad-plus-plus");
            assert_eq!(found.unwrap().path, expected);
            assert!(found.unwrap().size_bytes > 0);
        }
    }
}

#[test]
fn test_snap_runtime_classification() {
    assert!(is_snap_runtime_or_dependency("core22"));
    assert!(is_snap_runtime_or_dependency("core24"));
    assert!(is_snap_runtime_or_dependency("bare"));
    assert!(is_snap_runtime_or_dependency("snapd"));
    assert!(is_snap_runtime_or_dependency("gnome-46-2404"));
    assert!(is_snap_runtime_or_dependency("mesa-2404"));
    assert!(is_snap_runtime_or_dependency("wine-platform-runtime-core24"));
    assert!(!is_snap_runtime_or_dependency("notepad-plus-plus"));
    assert!(!is_snap_runtime_or_dependency("spotify"));
    assert!(!is_snap_runtime_or_dependency("firefox"));
}

#[test]
fn test_snapd_preserved_in_snap_batch_and_not_hijacked_by_native_rpm() {
    use std::collections::HashMap;
    use sweepx::scanner::orchestrator::ScanStageBatch;

    let mut app_map = HashMap::new();

    // 1. Snap scan discovers snapd package
    let snap_snapd = Application::new("snapd", "snapd", InstallMethod::Snap);
    let snap_batch = ScanStageBatch::Snap(vec![snap_snapd]);
    snap_batch.apply_to(&mut app_map);

    assert!(app_map.contains_key("snapd"));
    assert_eq!(app_map["snapd"].install_method, InstallMethod::Snap);

    // 2. Native RPM package scan discovers host snapd package
    let mut rpm_snapd = Application::new("snapd", "snapd", InstallMethod::NativeDnf);
    rpm_snapd.total_size_bytes = 45_000_000;
    rpm_snapd.is_system = true;

    let native_batch = ScanStageBatch::Native(vec![rpm_snapd]);
    native_batch.apply_to(&mut app_map);

    // Contract: The snapd Snap package MUST remain in Snap category,
    // and the native package must be inserted under a distinct key!
    assert!(app_map.contains_key("snapd"));
    assert_eq!(app_map["snapd"].install_method, InstallMethod::Snap);

    assert!(app_map.contains_key("native-snapd"));
    assert_eq!(app_map["native-snapd"].install_method, InstallMethod::NativeDnf);
    assert!(app_map["native-snapd"].is_system);
}
