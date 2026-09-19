use sweepx::scanner::appimage::{integrate_appimage, is_appimage_integrated};
use tempfile::tempdir;

#[test]
fn test_appimage_integration_desktop_file_generation() {
    let temp_dir = tempdir().unwrap();
    let appimage_path = temp_dir.path().join("Obsidian.AppImage");
    std::fs::File::create(&appimage_path).unwrap();

    let display_name = "Obsidian";
    let res = integrate_appimage(&appimage_path, display_name);
    assert!(res.is_ok());

    let desktop_file = res.unwrap();
    assert!(desktop_file.exists());
    let content = std::fs::read_to_string(&desktop_file).unwrap();
    assert!(content.contains("[Desktop Entry]"));
    assert!(content.contains("Name=Obsidian"));
    assert!(content.contains("Obsidian.AppImage"));

    let is_integrated = is_appimage_integrated(&appimage_path);
    assert!(is_integrated.is_some());

    // Clean up created desktop file
    let _ = std::fs::remove_file(desktop_file);
}
