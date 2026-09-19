use sweepx::updater::is_newer_version;

#[test]
fn test_semver_comparison() {
    assert!(is_newer_version("0.2.0", "0.1.0"));
    assert!(is_newer_version("1.0.0", "0.9.9"));
    assert!(is_newer_version("0.1.1", "0.1.0"));
    assert!(is_newer_version("0.1.0-alpha.2", "0.1.0-alpha.1"));

    assert!(!is_newer_version("0.1.0", "0.1.0"));
    assert!(!is_newer_version("0.1.0", "0.2.0"));
    assert!(!is_newer_version("0.0.9", "0.1.0"));
}
