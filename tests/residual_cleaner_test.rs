use sweepx::cleaner::signatures::find_signature;

#[test]
fn test_find_signature_lookup() {
    assert!(find_signature("code").is_some());
    assert!(find_signature("visual-studio-code").is_some());
    assert!(find_signature("com.spotify.Client").is_some());
    assert!(find_signature("google-chrome-stable").is_some());
    assert!(find_signature("discord").is_some());
    assert!(find_signature("steam").is_some());
    assert!(find_signature("pycharm-community").is_some());

    assert!(find_signature("non_existent_app_12345").is_none());
}

#[test]
fn test_signature_paths_structure() {
    let sig = find_signature("vscode").expect("Should find vscode signature");
    assert_eq!(sig.id, "vscode");
    assert!(sig.config_dirs.contains(&"Code"));
    assert!(sig.custom_user_paths.contains(&".vscode"));
}

#[tokio::test]
async fn test_flatpak_app_does_not_leak_system_binary_or_shared_caches() {
    let mut flatpak_app = sweepx::models::Application::new(
        "com.brave.Browser",
        "Brave",
        sweepx::models::InstallMethod::Flatpak,
    );
    flatpak_app.exec_path = Some(std::path::PathBuf::from("/usr/bin/flatpak"));

    let residuals = sweepx::cleaner::discover_residuals_for_app(&flatpak_app).await;

    for r in &residuals {
        assert_ne!(r.path, std::path::PathBuf::from("/usr/bin/flatpak"));
        let path_str = r.path.to_string_lossy();
        assert!(!path_str.ends_with("/.cache/flatpak"));
        assert!(!path_str.ends_with("/.local/share/flatpak"));
    }
}
