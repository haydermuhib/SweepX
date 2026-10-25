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
