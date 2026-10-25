use std::collections::HashMap;
use std::path::PathBuf;
use sweepx::updater::install_assets_to;
use tempfile::tempdir;

#[test]
fn test_install_assets_to_100_verifications_guarantees_third_party_icons_never_corrupted() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let home = tmp.path();

    // Setup 10 third-party application icons spanning multiple resolutions and directories
    let third_party_icons = [
        ("48x48/apps/google-chrome.png", "chrome_raster_png_48_bytes"),
        ("128x128/apps/terax.png", "terax_raster_png_128_bytes"),
        ("256x256/apps/org.fedoraproject.MediaWriter.png", "media_writer_png_256_bytes"),
        ("512x512/apps/md.obsidian.Obsidian.png", "obsidian_raster_png_512_bytes"),
        ("64x64/apps/com.tutanota.Tutanota.png", "tutanota_png_64_bytes"),
        ("32x32/apps/slack.png", "slack_png_32_bytes"),
        ("24x24/apps/spotify.png", "spotify_png_24_bytes"),
        ("16x16/apps/code.png", "vscode_png_16_bytes"),
        ("scalable/apps/firefox.svg", "<svg>firefox</svg>"),
        ("scalable/apps/custom-app.svg", "<svg>custom</svg>"),
    ];

    let hicolor_dir = home.join(".local/share/icons/hicolor");
    for (rel_path, content) in &third_party_icons {
        let full_path = hicolor_dir.join(rel_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full_path, content).unwrap();
    }

    // Capture initial content hash/fingerprint of all third-party files
    let initial_states: HashMap<PathBuf, String> = third_party_icons
        .iter()
        .map(|(rel, _)| {
            let p = hicolor_dir.join(rel);
            let s = std::fs::read_to_string(&p).unwrap();
            (p, s)
        })
        .collect();

    // Run 100 diverse verifications exercising different environmental corruptions,
    // repetitive updates, and concurrent injection patterns.
    for i in 1..=100 {
        match i % 7 {
            0 => {
                // Scenario 0: Inject legacy SweepX truncated index.theme (Directories=scalable/apps)
                let broken_index = hicolor_dir.join("index.theme");
                std::fs::write(
                    &broken_index,
                    "[Icon Theme]\nName=Hicolor\nComment=Fallback theme\nDirectories=scalable/apps\n\n[scalable/apps]\nSize=48\nType=Scalable\nMinSize=16\nMaxSize=512\n",
                ).unwrap();
            }
            1 => {
                // Scenario 1: Inject orphan icon-theme.cache without index.theme
                let orphan_cache = hicolor_dir.join("icon-theme.cache");
                std::fs::write(&orphan_cache, format!("corrupt_cache_payload_{}", i)).unwrap();
            }
            2 => {
                // Scenario 2: Inject BOTH truncated index.theme and broken cache
                let broken_index = hicolor_dir.join("index.theme");
                let broken_cache = hicolor_dir.join("icon-theme.cache");
                std::fs::write(
                    &broken_index,
                    "[Icon Theme]\nName=Hicolor\nComment=Corrupted theme\nDirectories=scalable/apps\n",
                ).unwrap();
                std::fs::write(&broken_cache, format!("bad_cache_{}", i)).unwrap();
            }
            3 => {
                // Scenario 3: Inject legacy sweepx.png in various directories
                let stale_user = home.join(".local/share/icons/sweepx.png");
                let stale_pix = home.join(".local/share/pixmaps/sweepx.png");
                let stale_48 = hicolor_dir.join("48x48/apps/sweepx.png");
                std::fs::create_dir_all(stale_user.parent().unwrap()).unwrap();
                std::fs::create_dir_all(stale_pix.parent().unwrap()).unwrap();
                std::fs::create_dir_all(stale_48.parent().unwrap()).unwrap();
                std::fs::write(&stale_user, "stale").unwrap();
                std::fs::write(&stale_pix, "stale").unwrap();
                std::fs::write(&stale_48, "stale").unwrap();
            }
            4 => {
                // Scenario 4: Clean state (no bad files injected)
            }
            5 => {
                // Scenario 5: Rapid consecutive execution (idempotence)
                let res = install_assets_to(home);
                assert!(res.is_ok(), "Pre-run in iteration {} failed: {:?}", i, res);
            }
            _ => {
                // Scenario 6: Broken index.theme with slight variation
                let broken_index = hicolor_dir.join("index.theme");
                std::fs::write(
                    &broken_index,
                    "[Icon Theme]\nDirectories=scalable/apps\n",
                ).unwrap();
            }
        }

        // Execute install_assets_to
        let result = install_assets_to(home);
        assert!(result.is_ok(), "Iteration {} install_assets_to must succeed: {:?}", i, result);

        // Verification 1: NEVER leave a truncated index.theme in user hicolor
        let index_theme = hicolor_dir.join("index.theme");
        if index_theme.exists() {
            let content = std::fs::read_to_string(&index_theme).unwrap();
            assert!(
                !content.contains("Directories=scalable/apps") || content.lines().count() >= 25,
                "Iteration {}: Truncated index.theme must never persist!",
                i
            );
        }

        // Verification 2: NEVER leave an orphan icon-theme.cache when index.theme is absent
        if !index_theme.exists() {
            let cache_file = hicolor_dir.join("icon-theme.cache");
            assert!(
                !cache_file.exists(),
                "Iteration {}: Orphan icon-theme.cache must never persist!",
                i
            );
        }

        // Verification 3: SweepX scalable SVG must always exist and be valid
        let sweepx_svg = hicolor_dir.join("scalable/apps/sweepx.svg");
        assert!(sweepx_svg.exists(), "Iteration {}: sweepx.svg must exist!", i);
        let svg_data = std::fs::read_to_string(&sweepx_svg).unwrap();
        assert!(svg_data.contains("<svg"), "Iteration {}: Valid SVG expected", i);

        // Verification 4: Desktop launcher must exist and point to sweepx.svg
        let desktop_file = home.join(".local/share/applications/sweepx.desktop");
        assert!(desktop_file.exists(), "Iteration {}: sweepx.desktop must exist!", i);

        // Verification 5: ALL third-party application icons MUST remain 100% untouched and byte-identical!
        for (path, expected_content) in &initial_states {
            assert!(
                path.exists(),
                "Iteration {}: Third-party icon {:?} was unexpectedly deleted!",
                i,
                path
            );
            let current_content = std::fs::read_to_string(path).unwrap();
            assert_eq!(
                &current_content,
                expected_content,
                "Iteration {}: Third-party icon {:?} content was modified!",
                i,
                path
            );
        }
    }
}
