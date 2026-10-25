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

#[test]
fn test_install_assets_to_refreshes_branding_and_cleans_legacy_pngs() {
    use sweepx::updater::install_assets_to;
    use tempfile::tempdir;

    let tmp = tempdir().expect("Failed to create tempdir");
    let home = tmp.path();

    // 1. Create a stale legacy PNG icon that should be cleaned up
    let stale_png = home.join(".local/share/icons/sweepx.png");
    if let Some(parent) = stale_png.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&stale_png, "stale png mock bytes").unwrap();
    assert!(stale_png.exists());

    // 2. Run install_assets_to
    let res = install_assets_to(home);
    assert!(res.is_ok(), "install_assets_to should succeed: {:?}", res);

    // 3. Verify scalable SVG icon is written
    let svg_icon = home.join(".local/share/icons/hicolor/scalable/apps/sweepx.svg");
    assert!(svg_icon.exists(), "Scalable SVG icon must exist");
    let svg_content = std::fs::read_to_string(&svg_icon).unwrap();
    assert!(svg_content.contains("<svg"), "Must contain valid SVG XML");
    assert!(svg_content.contains("title-app-icon"), "Must contain brand SVG content");

    // 4. Verify hicolor index.theme was created
    let index_theme = home.join(".local/share/icons/hicolor/index.theme");
    assert!(index_theme.exists(), "hicolor/index.theme must exist for icon cache tools");

    // 5. Verify stale legacy PNG was removed
    assert!(!stale_png.exists(), "Stale PNG icon must be deleted");

    // 6. Verify desktop entry was written
    let desktop_file = home.join(".local/share/applications/sweepx.desktop");
    assert!(desktop_file.exists(), "sweepx.desktop launcher must be created");
    let desktop_content = std::fs::read_to_string(&desktop_file).unwrap();
    assert!(desktop_content.contains("[Desktop Entry]"));
    assert!(desktop_content.contains("Name=SweepX"));
    assert!(desktop_content.contains("Icon="));
}


