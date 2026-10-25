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

#[tokio::test]
async fn test_purge_package_deletes_manual_binary_and_artifacts() {
    use sweepx::cleaner::execute_purge_package;
    use sweepx::models::{AppArtifact, Application, InstallMethod};

    let tmp = tempdir().expect("Failed to create tempdir");
    let bin_dir = tmp.path().join("local_bin");
    std::fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");

    let bin_path = bin_dir.join("feynman");
    {
        let mut f = File::create(&bin_path).expect("Failed to create binary");
        writeln!(f, "#!/bin/sh\necho 'hello'").expect("write binary");
    }
    assert!(bin_path.exists());

    let mut app = Application::new("feynman", "Feynman", InstallMethod::ManualOpt);
    app.exec_path = Some(bin_path.clone());
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::Binary,
        bin_path.clone(),
        50,
        true,
    ));

    let res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(res.is_ok(), "Purge should succeed: {:?}", res);
    assert!(!bin_path.exists(), "Binary file should be removed from disk");
}

#[tokio::test]
async fn test_purge_package_deletes_custom_desktop_and_launcher() {
    use sweepx::cleaner::execute_purge_package;
    use sweepx::models::{AppArtifact, Application, InstallMethod};

    let tmp = tempdir().expect("Failed to create tempdir");
    let app_dir = tmp.path().join("opt_testapp");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app dir");

    let bin_path = app_dir.join("testapp_bin");
    {
        let mut f = File::create(&bin_path).expect("Failed to create binary");
        writeln!(f, "ELF binary mock").expect("write binary");
    }

    let desktop_dir = tmp.path().join("applications");
    std::fs::create_dir_all(&desktop_dir).expect("Failed to create desktop dir");
    let desktop_path = desktop_dir.join("testapp.desktop");
    {
        let mut f = File::create(&desktop_path).expect("Failed to create desktop entry");
        writeln!(f, "[Desktop Entry]\nName=TestApp\nExec={}", bin_path.display()).expect("write desktop");
    }

    assert!(bin_path.exists());
    assert!(desktop_path.exists());

    let mut app = Application::new("testapp", "TestApp", InstallMethod::CustomDesktop);
    app.exec_path = Some(bin_path.clone());
    app.desktop_file = Some(desktop_path.clone());
    app.artifacts.push(AppArtifact::new(
        ArtifactKind::AppDir,
        app_dir.clone(),
        200,
        true,
    ));

    let res = execute_purge_package(&app, DeletionMode::Permanent).await;
    assert!(res.is_ok(), "Purge should succeed: {:?}", res);
    assert!(!bin_path.exists(), "Binary file should be removed from disk");
    assert!(!desktop_path.exists(), "Desktop file should be removed from disk");
    assert!(!app_dir.exists(), "App directory should be removed from disk");
}

