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
