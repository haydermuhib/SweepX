use std::path::PathBuf;
use sweepx::db::Database;
use sweepx::tracker::FilesystemSnapshot;

#[test]
fn test_filesystem_snapshot_diff_and_manifest_storage() {
    let mut pre = FilesystemSnapshot::default();
    pre.file_entries.insert(PathBuf::from("/usr/local/bin/existing"), (100, 1000));

    let mut post = FilesystemSnapshot::default();
    post.file_entries.insert(PathBuf::from("/usr/local/bin/existing"), (100, 1000));
    post.file_entries.insert(PathBuf::from("/usr/local/bin/new_tool"), (5000, 2000));
    post.file_entries.insert(PathBuf::from("/usr/local/share/new_tool/config"), (200, 2000));

    let manifest = pre.diff(&post, "new_tool", "New Tool", "make install");

    assert_eq!(manifest.app_id, "new_tool");
    assert_eq!(manifest.created_files.len(), 2);
    assert!(manifest.created_files.contains(&PathBuf::from("/usr/local/bin/new_tool")));
    assert!(manifest.created_files.contains(&PathBuf::from("/usr/local/share/new_tool/config")));
    assert_eq!(manifest.total_size_bytes, 5200);

    // Test database persistence
    let db = Database::open_in_memory().unwrap();
    db.save_install_manifest(&manifest).unwrap();

    let loaded = db.get_install_manifest("new_tool").unwrap().expect("manifest should be found");
    assert_eq!(loaded.app_id, "new_tool");
    assert_eq!(loaded.created_files.len(), 2);
    assert_eq!(loaded.total_size_bytes, 5200);

    // Test deletion
    db.delete_install_manifest("new_tool").unwrap();
    assert!(db.get_install_manifest("new_tool").unwrap().is_none());
}
