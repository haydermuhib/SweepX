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

    // 1. Create a stale legacy PNG icon and a truncated index.theme + cache that would break desktop apps
    let stale_png = home.join(".local/share/icons/sweepx.png");
    if let Some(parent) = stale_png.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&stale_png, "stale png mock bytes").unwrap();
    assert!(stale_png.exists());

    let hicolor_dir = home.join(".local/share/icons/hicolor");
    std::fs::create_dir_all(&hicolor_dir).unwrap();
    let broken_index = hicolor_dir.join("index.theme");
    let broken_cache = hicolor_dir.join("icon-theme.cache");
    std::fs::write(
        &broken_index,
        "[Icon Theme]\nName=Hicolor\nComment=Fallback theme\nDirectories=scalable/apps\n\n[scalable/apps]\nSize=48\nType=Scalable\nMinSize=16\nMaxSize=512\n",
    ).unwrap();
    std::fs::write(&broken_cache, "broken-cache").unwrap();
    assert!(broken_index.exists());
    assert!(broken_cache.exists());

    // 2. Run install_assets_to
    let res = install_assets_to(home);
    assert!(res.is_ok(), "install_assets_to should succeed: {:?}", res);

    // 3. Verify scalable SVG icon is written
    let svg_icon = home.join(".local/share/icons/hicolor/scalable/apps/sweepx.svg");
    assert!(svg_icon.exists(), "Scalable SVG icon must exist");
    let svg_content = std::fs::read_to_string(&svg_icon).unwrap();
    assert!(svg_content.contains("<svg"), "Must contain valid SVG XML");
    assert!(svg_content.contains("title-app-icon"), "Must contain brand SVG content");

    // 4. Verify truncated hicolor index.theme and cache were purged to repair desktop icons
    assert!(!broken_index.exists(), "Truncated hicolor/index.theme must be removed to avoid shadowing system PNG icons");
    assert!(!broken_cache.exists(), "Crippled icon-theme.cache must be removed");

    // 5. Verify stale legacy PNG was removed
    assert!(!stale_png.exists(), "Stale PNG icon must be deleted");

    // 6. Verify desktop entry was written
    let desktop_file = home.join(".local/share/applications/sweepx.desktop");
    assert!(desktop_file.exists(), "sweepx.desktop launcher must be created");
    let desktop_content = std::fs::read_to_string(&desktop_file).unwrap();
    assert!(desktop_content.contains("[Desktop Entry]"));
    assert!(desktop_content.contains("Name=SweepX"));
    assert!(desktop_content.contains("Icon="));
    assert!(desktop_content.contains("StartupWMClass=sweepx"));
    assert!(desktop_content.contains("StartupNotify=true"));
    assert!(desktop_content.contains("Terminal=false"));
    assert!(desktop_content.contains("Categories=System;Utility;Settings;"));
}


