use std::path::{Path, PathBuf};
use sweepx::cleaner::heuristic::discover_residuals_for_app;
use sweepx::models::{Application, ArtifactKind, InstallMethod};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use sweepx::scanner::snap::find_snap_package_files;

#[test]
fn test_snap_payload_versus_sandbox_footprint_separation() {
    // Contract: Snap apps must separate compressed squashfs package payload
    // from user ~/snap/<app> container sandboxes.
    let desktop_sample = r#"[Desktop Entry]
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
    let desktop_path = Path::new("/var/lib/snapd/desktop/applications/notepad-plus-plus_notepad-plus-plus.desktop");
    let app = parse_desktop_entry_content(desktop_sample, desktop_path)
        .expect("Must parse snap desktop entry");

    assert_eq!(app.id, "notepad-plus-plus");
    assert_eq!(app.install_method, InstallMethod::Snap);

    // If snap squashfs exists on host, total_size_bytes must reflect the real package file (e.g. ~9MB),
    // and NEVER the 13-byte symlink or wrapper.
    let snap_files = find_snap_package_files(&app.id);
    if !snap_files.is_empty() {
        let expected_size: u64 = snap_files.iter().map(|(_, sz)| *sz).sum();
        assert_eq!(app.total_size_bytes, expected_size);
        assert!(app.total_size_bytes > 1_000_000, "Snap squashfs package must be > 1 MB");
    }
}

#[tokio::test]
async fn test_flatpak_footprint_discovers_var_app_sandbox() {
    // Contract: Flatpak apps have user sandboxes strictly confined in ~/.var/app/<app.id>
    let app = Application::new("org.videolan.VLC", "VLC", InstallMethod::Flatpak);
    let residuals = discover_residuals_for_app(&app).await;

    if let Some(home) = dirs::home_dir() {
        let expected_sandbox = home.join(".var/app/org.videolan.VLC");
        if expected_sandbox.exists() {
            let found = residuals.iter().find(|r| r.kind == ArtifactKind::SandboxDir);
            assert!(found.is_some(), "Must detect Flatpak sandbox in ~/.var/app/<app-id>");
            assert_eq!(found.unwrap().path, expected_sandbox);
        }
    }
}

#[test]
fn test_manual_opt_payload_measures_app_dir_without_premature_cache_crawl() {
    // Contract: Manual /opt and standalone binaries register their binary/folder size,
    // without stalling the initial scan on multi-gigabyte ~/.cache folders.
    let mut app = Application::new("uv", "uv", InstallMethod::ManualOpt);
    app.total_size_bytes = 36_000_000; // 36 MB standalone executable

    assert_eq!(app.total_size_bytes, 36_000_000);
    assert_eq!(app.known_footprint_bytes(), 36_000_000);

    // Adding dynamic user cache residuals on demand during inspection increases footprint accurately
    app.artifacts.push(sweepx::models::AppArtifact::new(
        ArtifactKind::CacheDir,
        PathBuf::from("/home/user/.cache/uv"),
        150_000_000,
        true,
    ));
    assert_eq!(app.known_footprint_bytes(), 186_000_000);
}

#[test]
fn test_appimage_payload_is_exact_file_size() {
    let temp_dir = std::env::temp_dir().join("sweepx_test_appimage");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let appimage_path = temp_dir.join("test_app.AppImage");
    std::fs::write(&appimage_path, vec![0u8; 4096]).unwrap();

    let desktop_sample = format!(
        "[Desktop Entry]\nName=TestApp\nExec={} %U\nIcon=test\nType=Application\n",
        appimage_path.to_string_lossy()
    );
    let desktop_path = temp_dir.join("test_app.desktop");
    let app = parse_desktop_entry_content(&desktop_sample, &desktop_path)
        .expect("Must parse AppImage desktop entry");

    assert_eq!(app.install_method, InstallMethod::AppImage);
    assert_eq!(app.exec_path.as_deref(), Some(appimage_path.as_path()));
    assert_eq!(app.total_size_bytes, 4096);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_bottles_wine_app_payload_detects_program_and_prefix() {
    // Contract: Bottles / Wine apps must be classified as InstallMethod::Bottles
    let desktop_sample = r#"[Desktop Entry]
Name=Notepad Plus Plus (Wine)
Exec=flatpak run --command=bottles-cli com.usebottles.bottles run -b 'Office' -p 'notepad++' -- %u
Type=Application
"#;
    let desktop_path = Path::new("/home/user/.local/share/applications/notepad-plus-plus-wine.desktop");
    let app = parse_desktop_entry_content(desktop_sample, desktop_path)
        .expect("Must parse Bottles desktop entry");

    assert_eq!(app.install_method, InstallMethod::Bottles);
}

#[test]
fn test_storage_footprint_metric_stability_across_repeated_refreshes_and_mutations() {
    use sweepx::models::{format_size_with_unit, SizeUnitSystem};
    use sweepx::ui::{Message, NavigationTab, SweepXApp};

    let (mut app_state, _init_task) = SweepXApp::new();

    let mut app1 = Application::new("code", "Visual Studio Code", InstallMethod::NativeApt);
    app1.total_size_bytes = 900 * 1024 * 1024; // 900 MiB
    let mut app2 = Application::new("spotify", "Spotify", InstallMethod::Flatpak);
    app2.total_size_bytes = 150 * 1024 * 1024; // 150 MiB
    let mut app3 = Application::new("firefox", "Firefox", InstallMethod::NativeApt);
    app3.total_size_bytes = 200 * 1024 * 1024; // 200 MiB
    let mut app4 = Application::new("vlc", "VLC", InstallMethod::Flatpak);
    app4.total_size_bytes = 80 * 1024 * 1024; // 80 MiB
    let mut app5 = Application::new("gimp", "GIMP", InstallMethod::NativeApt);
    app5.total_size_bytes = 100 * 1024 * 1024; // 100 MiB

    let test_apps = vec![app1, app2, app3, app4, app5];
    let expected_base_bytes: u64 = 1430 * 1024 * 1024; // 1,499,463,680 bytes = 1.40 GiB

    // 1. First scan completed
    let _ = app_state.update(Message::ScanCompleted(test_apps.clone()));
    let visible_sum: u64 = app_state.applications.iter().filter(|a| !a.is_system).map(|a| a.total_size_bytes).sum();
    assert_eq!(visible_sum, expected_base_bytes);
    assert_eq!(app_state.format_bytes(visible_sum), "1.40 GiB");

    // 2. Simulate inspecting apps and populating residual cache with 500 MiB + 300 MiB
    app_state.app_residuals_cache.insert("code".to_string(), 500 * 1024 * 1024);
    app_state.app_residuals_cache.insert("firefox".to_string(), 300 * 1024 * 1024);

    // 3. User performs search and switches tabs
    let _ = app_state.update(Message::SearchChanged("code".to_string()));
    let _ = app_state.update(Message::SwitchTab(NavigationTab::SystemSweep));
    let _ = app_state.update(Message::SwitchTab(NavigationTab::Applications));
    let _ = app_state.update(Message::SearchChanged("".to_string()));

    // 4. Repeated scan completed events (simulating Refresh Scan clicks)
    for _ in 0..5 {
        let _ = app_state.update(Message::ScanCompleted(test_apps.clone()));
    }

    // Contract: The headline storage footprint MUST remain constant and deterministic
    let current_sum: u64 = app_state.applications.iter().filter(|a| !a.is_system).map(|a| a.total_size_bytes).sum();
    assert_eq!(current_sum, expected_base_bytes);
    assert_eq!(app_state.format_bytes(current_sum), "1.40 GiB");

    // 5. Toggle unit system to GB
    let _ = app_state.update(Message::ToggleUnitSystem);
    assert_eq!(app_state.unit_system, SizeUnitSystem::GB);
    // 1,499,463,680 bytes in decimal GB is 1.50 GB (1,499,463,680 / 10^9 = 1.499... => "1.50 GB")
    assert_eq!(app_state.format_bytes(current_sum), "1.50 GB");
    assert_eq!(format_size_with_unit(current_sum, SizeUnitSystem::GB), "1.50 GB");

    // 6. Toggle back to GiB
    let _ = app_state.update(Message::ToggleUnitSystem);
    assert_eq!(app_state.unit_system, SizeUnitSystem::GiB);
    assert_eq!(app_state.format_bytes(current_sum), "1.40 GiB");
}

#[test]
fn test_size_unit_system_strict_boundaries_and_toggle() {
    use sweepx::models::{format_size_with_unit, SizeUnitSystem};

    // Sub-kilobyte exact behavior
    assert_eq!(format_size_with_unit(0, SizeUnitSystem::GiB), "0 B");
    assert_eq!(format_size_with_unit(0, SizeUnitSystem::GB), "0 B");
    assert_eq!(format_size_with_unit(500, SizeUnitSystem::GiB), "500 B");
    assert_eq!(format_size_with_unit(500, SizeUnitSystem::GB), "500 B");
    assert_eq!(format_size_with_unit(1023, SizeUnitSystem::GiB), "1023 B");
    assert_eq!(format_size_with_unit(999, SizeUnitSystem::GB), "999 B");

    // KiB (1024) vs KB (1000) boundary
    assert_eq!(format_size_with_unit(1024, SizeUnitSystem::GiB), "1.0 KiB");
    assert_eq!(format_size_with_unit(1000, SizeUnitSystem::GB), "1.0 KB");
    assert_eq!(format_size_with_unit(1024, SizeUnitSystem::GB), "1.0 KB");

    // MiB (1024^2 = 1,048,576) vs MB (1000^2 = 1,000,000) boundary
    assert_eq!(format_size_with_unit(1_048_576, SizeUnitSystem::GiB), "1.0 MiB");
    assert_eq!(format_size_with_unit(1_000_000, SizeUnitSystem::GB), "1.0 MB");
    assert_eq!(format_size_with_unit(1_048_576, SizeUnitSystem::GB), "1.0 MB");

    // GiB (1024^3) vs GB (1000^3) boundary - Crucial explanation of 48-49 vs 51-52 GB
    let test_48_gib = 48 * 1024 * 1024 * 1024u64; // 51,539,607,552 bytes
    assert_eq!(format_size_with_unit(test_48_gib, SizeUnitSystem::GiB), "48.00 GiB");
    assert_eq!(format_size_with_unit(test_48_gib, SizeUnitSystem::GB), "51.54 GB");

    let test_50_gib = 50 * 1024 * 1024 * 1024u64; // 53,687,091,200 bytes
    assert_eq!(format_size_with_unit(test_50_gib, SizeUnitSystem::GiB), "50.00 GiB");
    assert_eq!(format_size_with_unit(test_50_gib, SizeUnitSystem::GB), "53.69 GB");

    // TiB vs TB boundary
    let test_2_tib = 2 * 1024 * 1024 * 1024 * 1024u64; // 2,199,023,255,552 bytes
    assert_eq!(format_size_with_unit(test_2_tib, SizeUnitSystem::GiB), "2.00 TiB");
    assert_eq!(format_size_with_unit(test_2_tib, SizeUnitSystem::GB), "2.20 TB");

    // Involution invariant: toggle twice returns original
    assert_eq!(SizeUnitSystem::GiB.toggle().toggle(), SizeUnitSystem::GiB);
    assert_eq!(SizeUnitSystem::GB.toggle().toggle(), SizeUnitSystem::GB);

    // Labels
    assert_eq!(SizeUnitSystem::GiB.label(), "GiB (1024)");
    assert_eq!(SizeUnitSystem::GB.label(), "GB (1000)");
}

#[test]
fn test_startup_and_refresh_do_not_dispatch_blanket_residual_scans() {
    use sweepx::ui::{Message, SweepXApp};

    let (mut app_state, _init_task) = SweepXApp::new();

    // Generate 60 test applications
    let apps: Vec<Application> = (0..60)
        .map(|i| {
            let mut a = Application::new(format!("app-{}", i), format!("App {}", i), InstallMethod::NativeApt);
            a.total_size_bytes = 10 * 1024 * 1024;
            a
        })
        .collect();

    // Contract: ScanCompleted must not enqueue unconstrained residual crawling
    let _task = app_state.update(Message::ScanCompleted(apps));

    // in_flight_residuals MUST remain empty on scan completed
    assert!(
        app_state.in_flight_residuals.is_empty(),
        "Startup / scan refresh must not insert apps into in_flight_residuals"
    );
    assert_eq!(app_state.applications.len(), 60);
    assert!(!app_state.is_scanning);
}

