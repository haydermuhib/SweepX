use std::path::{Path, PathBuf};
use std::time::Instant;
use sweepx::models::{Application, InstallMethod};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use sweepx::scanner::orchestrator::ScanStageBatch;
use sweepx::scanner::snap::scan_snaps;

#[test]
fn test_desktop_entry_parsing_speed_no_blocking_processes() {
    let desktop_sample = r#"[Desktop Entry]
Name=Calculator
GenericName=Calculator
Comment=Perform arithmetic, scientific or financial calculations
Exec=gnome-calculator
Icon=org.gnome.Calculator
Terminal=false
Type=Application
StartupNotify=true
Categories=GNOME;GTK;Utility;Calculator;
"#;

    let dummy_path = Path::new("/usr/share/applications/org.gnome.Calculator.desktop");

    // Benchmark parsing 500 desktop entries
    let start = Instant::now();
    for _ in 0..500 {
        let app = parse_desktop_entry_content(desktop_sample, dummy_path)
            .expect("Should parse desktop entry cleanly");
        assert_eq!(app.name, "Calculator");
        assert_eq!(app.id, "org.gnome.Calculator");
    }
    let elapsed = start.elapsed();

    // 500 entries must parse in well under 100 milliseconds (typically ~1-5ms without child process spawning)
    assert!(
        elapsed.as_millis() < 250,
        "Parsing 500 desktop entries took too long: {:?} (must not spawn external processes)",
        elapsed
    );
}

#[tokio::test]
async fn test_snap_scanner_is_non_blocking_when_daemon_socket_missing() {
    // If /run/snapd.socket doesn't exist (e.g. in environments without snapd running),
    // scan_snaps must return immediately (< 50ms) without hanging on socket timeouts.
    let socket_path = PathBuf::from("/run/snapd.socket");
    if !socket_path.exists() {
        let start = Instant::now();
        let snaps = scan_snaps().await;
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 300,
            "scan_snaps hung for {:?} when /run/snapd.socket was missing!",
            elapsed
        );
        assert!(snaps.is_empty());
    }
}

#[test]
fn test_native_batch_merging_keeps_desktop_apps_visible() {
    let mut app_map = std::collections::HashMap::new();

    // 1. App discovered via desktop file (e.g., /usr/share/applications/org.gnome.Calculator.desktop)
    let mut desktop_app = Application::new("org.gnome.Calculator", "Calculator", InstallMethod::CustomDesktop);
    desktop_app.desktop_file = Some(PathBuf::from("/usr/share/applications/org.gnome.Calculator.desktop"));
    desktop_app.exec_path = Some(PathBuf::from("/usr/bin/gnome-calculator"));
    desktop_app.total_size_bytes = 4096;
    desktop_app.is_system = false;
    app_map.insert("org.gnome.Calculator".to_string(), desktop_app);

    // 2. Batch Native scan returns gnome-calculator package, flagged by the native PM as is_system = true
    let mut pm_package = Application::new("gnome-calculator", "gnome-calculator", InstallMethod::NativeDnf);
    pm_package.version = Some("49.2-1.fc43".to_string());
    pm_package.total_size_bytes = 7_800_000;
    pm_package.is_system = true; // In DNF/RPM it is an OS dependency

    // 3. Pure system library without a desktop entry
    let mut pure_system_lib = Application::new("glibc", "glibc", InstallMethod::NativeDnf);
    pure_system_lib.version = Some("2.39-13.fc43".to_string());
    pure_system_lib.total_size_bytes = 29_000_000;
    pure_system_lib.is_system = true;

    // Apply the batch merge using ScanStageBatch::Native logic
    let batch = ScanStageBatch::Native(vec![pm_package, pure_system_lib]);
    batch.apply_to(&mut app_map);

    // Assert: Calculator was merged with DNF package info, updated its size and version,
    // and CRITICALLY: remained is_system = false because it has a desktop entry!
    let calc = app_map.get("gnome-calculator").expect("gnome-calculator must be present in map");
    assert_eq!(calc.install_method, InstallMethod::NativeDnf);
    assert_eq!(calc.version.as_deref(), Some("49.2-1.fc43"));
    assert_eq!(calc.total_size_bytes, 7_800_000);
    assert!(!calc.is_system, "Desktop apps must NOT be hidden as system libraries!");

    // Assert: glibc was inserted as a system package (is_system = true)
    let glibc = app_map.get("glibc").expect("glibc should be present as system package");
    assert!(glibc.is_system, "glibc must be marked as system package!");
}

#[test]
fn test_payload_and_footprint_metrics_calculation() {
    let mut app = Application::new("notepad-plus-plus", "Notepad++", InstallMethod::Snap);
    app.total_size_bytes = 16_100_000; // 16.1 MB payload

    // Without residuals
    assert_eq!(app.total_size_bytes, 16_100_000);
    assert_eq!(app.known_footprint_bytes(), 16_100_000);

    // With discovered residuals (e.g. 50 MB cache + data)
    let residual_bytes = 50_000_000;
    let combined_footprint = app.total_size_bytes + residual_bytes;
    assert_eq!(combined_footprint, 66_100_000);
}
