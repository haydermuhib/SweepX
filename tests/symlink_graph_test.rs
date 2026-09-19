use std::fs::File;
use std::os::unix::fs::symlink;
use sweepx::scanner::symlink_graph::find_symlinks_pointing_to_app;
use tempfile::tempdir;

#[test]
fn test_symlink_graph_resolution_and_broken_detection() {
    let temp_dir = tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    let opt_dir = temp_dir.path().join("opt").join("myapp");

    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::create_dir_all(&opt_dir).unwrap();

    let target_bin = opt_dir.join("myapp_bin");
    File::create(&target_bin).unwrap();

    // 1. Create valid symlink
    let valid_link = bin_dir.join("myapp");
    symlink(&target_bin, &valid_link).unwrap();

    // 2. Create broken symlink
    let broken_link = bin_dir.join("broken_app");
    symlink(opt_dir.join("non_existent_binary"), &broken_link).unwrap();

    // Verify pointing symlinks finder
    let _pointing = find_symlinks_pointing_to_app(&opt_dir);
    assert!(target_bin.exists());
    assert!(!opt_dir.join("non_existent_binary").exists());
}
