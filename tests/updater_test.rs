use sweepx::updater::{is_newer_version, select_best_asset, GithubReleaseAsset};

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

#[test]
fn test_select_best_asset_prefers_compressed_archive() {
    let assets = vec![
        GithubReleaseAsset {
            name: "sweepx".to_string(),
            size: Some(15_748_776),
            browser_download_url: "https://example.com/sweepx".to_string(),
        },
        GithubReleaseAsset {
            name: "sweepx-x86_64-unknown-linux-gnu.tar.gz".to_string(),
            size: Some(5_783_973),
            browser_download_url: "https://example.com/sweepx-x86_64-unknown-linux-gnu.tar.gz".to_string(),
        },
    ];

    let selected = select_best_asset(&assets, "x86_64").expect("Asset should be selected");
    assert_eq!(selected.name, "sweepx-x86_64-unknown-linux-gnu.tar.gz");
    assert_eq!(selected.size, Some(5_783_973));
}

#[test]
fn test_select_best_asset_fallback_to_raw_binary() {
    let assets = vec![
        GithubReleaseAsset {
            name: "sweepx".to_string(),
            size: Some(15_748_776),
            browser_download_url: "https://example.com/sweepx".to_string(),
        },
    ];

    let selected = select_best_asset(&assets, "x86_64").expect("Asset should be selected");
    assert_eq!(selected.name, "sweepx");
}

