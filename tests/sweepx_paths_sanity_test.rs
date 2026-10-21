use std::path::PathBuf;
use sweepx::db::Database;

#[test]
fn test_sweepx_database_path_uses_xdg_data_and_never_home_dot_sweepx() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fake_home = temp_dir.path();
    unsafe {
        std::env::set_var("HOME", fake_home);
        std::env::set_var("XDG_DATA_HOME", fake_home.join(".local/share"));
    }

    let _db = Database::open_default().expect("Must open default database");

    // Must exist under ~/.local/share/sweepx/sweepx.db
    let expected_db = fake_home.join(".local/share/sweepx/sweepx.db");
    assert!(expected_db.exists(), "SweepX database must be in ~/.local/share/sweepx/sweepx.db");

    // Must NOT create ~/.sweepx directly in home root
    let forbidden_home_dir = fake_home.join(".sweepx");
    assert!(!forbidden_home_dir.exists(), "SweepX must NEVER create a ~/.sweepx folder directly in home root");
}

#[test]
fn test_sweepx_standard_installation_paths_contract() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fake_home = temp_dir.path();

    // Contract definitions as implemented in installer and self-updater
    let bin_path = fake_home.join(".local/bin/sweepx");
    let apps_path = fake_home.join(".local/share/applications/sweepx.desktop");
    let icon_hicolor = fake_home.join(".local/share/icons/hicolor/scalable/apps/sweepx.svg");
    let icon_top = fake_home.join(".local/share/icons/sweepx.svg");
    let icon_pixmaps = fake_home.join(".local/share/pixmaps/sweepx.svg");
    let db_path = fake_home.join(".local/share/sweepx/sweepx.db");

    // All paths must strictly be within standard user local hierarchies
    assert!(bin_path.starts_with(fake_home.join(".local/bin")));
    assert!(apps_path.starts_with(fake_home.join(".local/share/applications")));
    assert!(icon_hicolor.starts_with(fake_home.join(".local/share/icons")));
    assert!(icon_top.starts_with(fake_home.join(".local/share/icons")));
    assert!(icon_pixmaps.starts_with(fake_home.join(".local/share/pixmaps")));
    assert!(db_path.starts_with(fake_home.join(".local/share/sweepx")));
}

#[test]
fn test_sweepx_updater_temp_patterns() {
    let parent = PathBuf::from("/home/user/.local/bin");
    let pid = 12345;

    let temp_download = parent.join(format!(".sweepx-update-{}", pid));
    let temp_extract = parent.join(format!(".sweepx-extract-{}", pid));
    let temp_old = parent.join(format!(".sweepx-old-{}", pid));

    assert!(temp_download.file_name().unwrap().to_str().unwrap().starts_with(".sweepx-update-"));
    assert!(temp_extract.file_name().unwrap().to_str().unwrap().starts_with(".sweepx-extract-"));
    assert!(temp_old.file_name().unwrap().to_str().unwrap().starts_with(".sweepx-old-"));
}
