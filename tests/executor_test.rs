use std::fs::File;
use std::io::Write;
use sweepx::cleaner::{execute_purge_residuals, DeletionMode};
use sweepx::models::{ArtifactKind, ResidualCandidate};
use tempfile::tempdir;

#[tokio::test]
async fn test_permanent_deletion_of_test_residual() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let test_dir = tmp.path().join(".config").join("testapp");
    std::fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    let file_path = test_dir.join("settings.json");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "{{\"theme\":\"dark\"}}").expect("Failed to write");

    assert!(test_dir.exists());

    let candidate = ResidualCandidate::new(
        "testapp",
        "Test App",
        test_dir.clone(),
        ArtifactKind::ConfigDir,
        100,
        0.9,
        false,
    );

    let report = execute_purge_residuals(&[candidate], DeletionMode::Permanent).await;
    assert!(report.success);
    assert_eq!(report.deleted_paths.len(), 1);
    assert!(!test_dir.exists());
}

#[tokio::test]
async fn test_executor_blocks_unsafe_paths() {
    let candidate = ResidualCandidate::new(
        "root",
        "Root Directory",
        std::path::PathBuf::from("/"),
        ArtifactKind::Unknown,
        1000,
        1.0,
        false,
    );

    let report = execute_purge_residuals(&[candidate], DeletionMode::Permanent).await;
    assert!(!report.success);
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].1.contains("Blocked by safety validator"));
}
