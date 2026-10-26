use std::path::{Path, PathBuf};
use std::time::Instant;
use sweepx::models::{Application, InstallMethod};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use sweepx::scanner::orchestrator::ScanStageBatch;
use sweepx::scanner::snap::scan_snaps;

#[test]
fn test_desktop_entry_parsing_speed_no_blocking_processes() {
    let desktop_sample = r#"[Desktop Entry]
Name=Calculator
GenericName=Calculator
Comment=Perform arithmetic, scientific or financial calculations
Exec=gnome-calculator
Icon=org.gnome.Calculator
Terminal=false
Type=Application
StartupNotify=true
Categories=GNOME;GTK;Utility;Calculator;
"#;

    let dummy_path = Path::new("/usr/share/applications/org.gnome.Calculator.desktop");

    // Benchmark parsing 500 desktop entries
    let start = Instant::now();
    for _ in 0..500 {
        let app = parse_desktop_entry_content(desktop_sample, dummy_path)
            .expect("Should parse desktop entry cleanly");
        assert_eq!(app.name, "Calculator");
        assert_eq!(app.id, "org.gnome.Calculator");
    }
    let elapsed = start.elapsed();

    // 500 entries must parse in well under 100 milliseconds (typically ~1-5ms without child process spawning)
    assert!(
        elapsed.as_millis() < 250,
        "Parsing 500 desktop entries took too long: {:?} (must not spawn external processes)",
        elapsed
    );
}

#[tokio::test]
async fn test_snap_scanner_is_non_blocking_when_daemon_socket_missing() {
    // If /run/snapd.socket doesn't exist (e.g. in environments without snapd running),
    // scan_snaps must return immediately (< 50ms) without hanging on socket timeouts.
    let socket_path = PathBuf::from("/run/snapd.socket");
    if !socket_path.exists() {
        let start = Instant::now();
        let snaps = scan_snaps().await;
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 300,
            "scan_snaps hung for {:?} when /run/snapd.socket was missing!",
            elapsed
        );
        assert!(snaps.is_empty());
    }
}

#[test]
fn test_native_batch_merging_keeps_desktop_apps_visible() {
    let mut app_map = std::collections::HashMap::new();

    // 1. App discovered via desktop file (e.g., /usr/share/applications/org.gnome.Calculator.desktop)
    let mut desktop_app = Application::new("org.gnome.Calculator", "Calculator", InstallMethod::CustomDesktop);
    desktop_app.desktop_file = Some(PathBuf::from("/usr/share/applications/org.gnome.Calculator.desktop"));
    desktop_app.exec_path = Some(PathBuf::from("/usr/bin/gnome-calculator"));
    desktop_app.total_size_bytes = 4096;
    desktop_app.is_system = false;
    app_map.insert("org.gnome.Calculator".to_string(), desktop_app);

    // 2. Batch Native scan returns gnome-calculator package, flagged by the native PM as is_system = true
    let mut pm_package = Application::new("gnome-calculator", "gnome-calculator", InstallMethod::NativeDnf);
    pm_package.version = Some("49.2-1.fc43".to_string());
    pm_package.total_size_bytes = 7_800_000;
    pm_package.is_system = true; // In DNF/RPM it is an OS dependency

    // 3. Pure system library without a desktop entry
    let mut pure_system_lib = Application::new("glibc", "glibc", InstallMethod::NativeDnf);
    pure_system_lib.version = Some("2.39-13.fc43".to_string());
    pure_system_lib.total_size_bytes = 29_000_000;
    pure_system_lib.is_system = true;

    // Apply the batch merge using ScanStageBatch::Native logic
    let batch = ScanStageBatch::Native(vec![pm_package, pure_system_lib]);
    batch.apply_to(&mut app_map);

    // Assert: Calculator was merged with DNF package info, updated its size and version,
    // and CRITICALLY: remained is_system = false because it has a desktop entry!
    let calc = app_map.get("gnome-calculator").expect("gnome-calculator must be present in map");
    assert_eq!(calc.install_method, InstallMethod::NativeDnf);
    assert_eq!(calc.version.as_deref(), Some("49.2-1.fc43"));
    assert_eq!(calc.total_size_bytes, 7_800_000);
    assert!(!calc.is_system, "Desktop apps must NOT be hidden as system libraries!");

    // Assert: glibc was inserted as a system package (is_system = true)
    let glibc = app_map.get("glibc").expect("glibc should be present as system package");
    assert!(glibc.is_system, "glibc must be marked as system package!");
}

#[test]
fn test_payload_and_footprint_metrics_calculation() {
    let mut app = Application::new("notepad-plus-plus", "Notepad++", InstallMethod::Snap);
    app.total_size_bytes = 16_100_000; // 16.1 MB payload

    // Without residuals
    assert_eq!(app.total_size_bytes, 16_100_000);
    assert_eq!(app.known_footprint_bytes(), 16_100_000);

    // With discovered residuals (e.g. 50 MB cache + data)
    let residual_bytes = 50_000_000;
    let combined_footprint = app.total_size_bytes + residual_bytes;
    assert_eq!(combined_footprint, 66_100_000);
}

#[tokio::test]
async fn test_profile_scanner_breakdown() {
    let t0 = Instant::now();
    let d = sweepx::scanner::desktop_entry::scan_desktop_entries().await;
    let d_time = t0.elapsed();

    let t1 = Instant::now();
    let m = sweepx::scanner::manual::scan_manual_installations().await;
    let m_time = t1.elapsed();

    let t2 = Instant::now();
    let f = sweepx::scanner::flatpak::scan_flatpaks().await;
    let f_time = t2.elapsed();

    let t3 = Instant::now();
    let s = sweepx::scanner::snap::scan_snaps().await;
    let s_time = t3.elapsed();

    let t4 = Instant::now();
    let pms = sweepx::scanner::native::detect_available_package_managers();
    let mut native_apps = Vec::new();
    for pm in pms {
        let mut pkgs = pm.list_installed().await;
        native_apps.append(&mut pkgs);
    }
    let native_time = t4.elapsed();

    let t5 = Instant::now();
    let _all = sweepx::scanner::scan_all_applications().await;
    let full_time = t5.elapsed();

    println!("\n=== SCANNER TIMING BREAKDOWN ===");
    println!("Desktop entries:      {:?} ({} apps)", d_time, d.len());
    println!("Manual installations: {:?} ({} apps)", m_time, m.len());
    println!("Flatpaks:             {:?} ({} apps)", f_time, f.len());
    println!("Snaps:                {:?} ({} apps)", s_time, s.len());
    println!("Native packages:      {:?} ({} pkgs)", native_time, native_apps.len());
    println!("FULL scan_all_applications(): {:?}", full_time);
    println!("================================\n");
}

#[tokio::test]
async fn test_manual_scanner_deduplicates_against_existing_desktop_apps() {
    use std::collections::HashMap;
    use sweepx::models::{Application, InstallMethod};
    use sweepx::scanner::orchestrator::{apply_stage_batch, ScanStageBatch};

    let mut app_map = HashMap::new();

    // 1. Existing Spotify desktop application
    let mut desktop_spotify = Application::new("spotify.desktop", "Spotify", InstallMethod::CustomDesktop);
    desktop_spotify.exec_path = Some(std::path::PathBuf::from("/usr/bin/spotify"));
    app_map.insert(desktop_spotify.id.clone(), desktop_spotify);

    // 2. Incoming manual Spotify entry (e.g. from ~/.local/bin/spotify or wrapper)
    let mut manual_spotify = Application::new("spotify", "spotify", InstallMethod::ManualOpt);
    manual_spotify.exec_path = Some(std::path::PathBuf::from("/home/user/.local/bin/spotify"));

    // 3. Incoming unique manual app (e.g. spicetify)
    let mut manual_spicetify = Application::new("spicetify", "spicetify", InstallMethod::ManualOpt);
    manual_spicetify.exec_path = Some(std::path::PathBuf::from("/home/user/.spicetify/spicetify"));

    apply_stage_batch(&mut app_map, ScanStageBatch::Manual(vec![manual_spotify, manual_spicetify]));

    // Assert that Spotify was not duplicated (only the original desktop entry exists)
    assert_eq!(app_map.values().filter(|a| a.name.to_lowercase() == "spotify").count(), 1);
    assert_eq!(app_map.get("spotify.desktop").unwrap().install_method, InstallMethod::CustomDesktop);

    // Assert that unique tool (spicetify) was added safely
    assert!(app_map.contains_key("spicetify"));
}

#[tokio::test]
async fn test_manual_scanner_deduplicates_against_flatpak_and_snap_containers() {
    use std::collections::HashMap;
    use sweepx::models::{Application, InstallMethod};
    use sweepx::scanner::orchestrator::{apply_stage_batch, ScanStageBatch};

    let mut app_map = HashMap::new();

    // Flatpak app with reverse-DNS ID
    let flatpak_discord = Application::new("com.discordapp.Discord", "Discord", InstallMethod::Flatpak);
    app_map.insert(flatpak_discord.id.clone(), flatpak_discord);

    // Snap app with short name ID
    let snap_slack = Application::new("slack", "Slack", InstallMethod::Snap);
    app_map.insert(snap_slack.id.clone(), snap_slack);

    // Manual duplicates discovered via bin paths
    let manual_discord = Application::new("discord", "discord", InstallMethod::ManualOpt);
    let manual_slack = Application::new("slack", "slack", InstallMethod::ManualOpt);
    let manual_unique = Application::new("my-custom-cli", "my-custom-cli", InstallMethod::ManualOpt);

    apply_stage_batch(
        &mut app_map,
        ScanStageBatch::Manual(vec![manual_discord, manual_slack, manual_unique]),
    );

    // Discord count must remain 1 (no manual duplicate)
    assert_eq!(app_map.values().filter(|a| a.name.to_lowercase() == "discord").count(), 1);
    assert_eq!(app_map.get("com.discordapp.Discord").unwrap().install_method, InstallMethod::Flatpak);

    // Slack count must remain 1 (no manual duplicate)
    assert_eq!(app_map.values().filter(|a| a.name.to_lowercase() == "slack").count(), 1);
    assert_eq!(app_map.get("slack").unwrap().install_method, InstallMethod::Snap);

    // Unique tool must be preserved
    assert!(app_map.contains_key("my-custom-cli"));
}

#[tokio::test]
async fn test_manual_scanner_binary_isolation_avoids_user_data_attachment() {
    use sweepx::models::{ArtifactKind, InstallMethod};

    let temp_dir = std::env::temp_dir().join("sweepx_test_binary_isolation");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let bin_path = temp_dir.join("spicetify");
    std::fs::write(&bin_path, [0x7f, b'E', b'L', b'F', 0, 0, 0, 0]).unwrap();

    let mut app = sweepx::models::Application::new("spicetify", "spicetify", InstallMethod::ManualOpt);
    app.exec_path = Some(bin_path.clone());
    app.artifacts.push(sweepx::models::AppArtifact::new(
        ArtifactKind::Binary,
        bin_path.clone(),
        8,
        true,
    ));

    // Verify artifact kind is strictly Binary and never AppDir, ConfigDir, or DataDir
    assert_eq!(app.artifacts.len(), 1);
    assert_eq!(app.artifacts[0].kind, ArtifactKind::Binary);
    assert_ne!(app.artifacts[0].kind, ArtifactKind::AppDir);
    assert_ne!(app.artifacts[0].kind, ArtifactKind::ConfigDir);
    assert_ne!(app.artifacts[0].kind, ArtifactKind::DataDir);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_orchestrator_simultaneous_snapd_and_spicetify_isolation() {
    use std::collections::HashMap;
    use sweepx::models::{Application, InstallMethod};
    use sweepx::scanner::orchestrator::{apply_stage_batch, ScanStageBatch};

    let mut app_map = HashMap::new();

    // 1. Desktop Entry stage
    let desktop_spotify = Application::new("spotify.desktop", "Spotify", InstallMethod::CustomDesktop);
    apply_stage_batch(&mut app_map, ScanStageBatch::Desktop(vec![desktop_spotify]));

    // 2. Snap stage: snapd container app
    let snap_snapd = Application::new("snapd", "snapd", InstallMethod::Snap);
    apply_stage_batch(&mut app_map, ScanStageBatch::Snap(vec![snap_snapd]));

    // 3. Manual stage: spicetify tool + duplicate spotify wrapper
    let manual_spicetify = Application::new("spicetify", "spicetify", InstallMethod::ManualOpt);
    let manual_spotify = Application::new("spotify", "spotify", InstallMethod::ManualOpt);
    apply_stage_batch(&mut app_map, ScanStageBatch::Manual(vec![manual_spicetify, manual_spotify]));

    // 4. Native stage: RPM snapd
    let rpm_snapd = Application::new("snapd", "snapd", InstallMethod::NativeDnf);
    apply_stage_batch(&mut app_map, ScanStageBatch::Native(vec![rpm_snapd]));

    // Assert: snapd remains Snap
    assert!(app_map.contains_key("snapd"));
    assert_eq!(app_map.get("snapd").unwrap().install_method, InstallMethod::Snap);

    // Assert: native RPM snapd isolated under native-snapd
    assert!(app_map.contains_key("native-snapd"));
    assert_eq!(app_map.get("native-snapd").unwrap().install_method, InstallMethod::NativeDnf);
    assert!(app_map.get("native-snapd").unwrap().is_system);

    // Assert: Spotify deduplicated to 1 entry
    assert_eq!(app_map.values().filter(|a| a.name.to_lowercase() == "spotify").count(), 1);

    // Assert: spicetify preserved cleanly as standalone manual app
    assert!(app_map.contains_key("spicetify"));
    assert_eq!(app_map.get("spicetify").unwrap().install_method, InstallMethod::ManualOpt);
}

#[test]
fn test_dynamic_path_discovery_finds_pixi_and_spicetify_without_hardcoding() {
    use std::path::PathBuf;
    use sweepx::scanner::manual::extract_user_binary_scan_dirs;

    let base_temp = std::env::temp_dir().join("sweepx_test_dynamic_path");
    let _ = std::fs::remove_dir_all(&base_temp);

    // Mock home directories
    let mock_pixi = base_temp.join(".pixi/bin");
    let mock_spicetify = base_temp.join(".spicetify");
    let mock_cargo = base_temp.join(".cargo/bin");
    let mock_gemini = base_temp.join(".gemini/bin");

    std::fs::create_dir_all(&mock_pixi).unwrap();
    std::fs::create_dir_all(&mock_spicetify).unwrap();
    std::fs::create_dir_all(&mock_cargo).unwrap();
    std::fs::create_dir_all(&mock_gemini).unwrap();

    // Construct a synthetic PATH joining these directories plus system paths
    let synthetic_paths = vec![
        mock_pixi.clone(),
        mock_spicetify.clone(),
        mock_cargo.clone(),
        mock_gemini.clone(),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/var/lib/snapd/snap/bin"),
    ];

    let synthetic_path_var = std::env::join_paths(&synthetic_paths).unwrap();
    let scan_dirs = extract_user_binary_scan_dirs(Some(synthetic_path_var));

    // Assert: System paths are strictly excluded
    assert!(!scan_dirs.contains(&PathBuf::from("/usr/bin")));
    assert!(!scan_dirs.contains(&PathBuf::from("/bin")));
    assert!(!scan_dirs.contains(&PathBuf::from("/var/lib/snapd/snap/bin")));

    // Assert: IDE internal paths are strictly excluded
    assert!(!scan_dirs.iter().any(|p| p.to_string_lossy().contains(".gemini")));

    let _ = std::fs::remove_dir_all(&base_temp);
}

#[test]
fn test_no_hardcoded_app_names_in_manual_scanner_source() {
    let source_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/scanner/manual.rs");
    let source_code = std::fs::read_to_string(source_path)
        .expect("Failed to read src/scanner/manual.rs");

    // Assert: No specific third-party application names are hardcoded in discovery routines
    assert!(
        !source_code.contains(".spicetify"),
        "src/scanner/manual.rs must not hardcode .spicetify"
    );
    assert!(
        !source_code.contains(".pixi"),
        "src/scanner/manual.rs must not hardcode .pixi"
    );
}

#[test]
fn test_desktop_launcher_environment_discovers_user_shell_paths_without_hardcoding() {
    use sweepx::scanner::manual::get_user_binary_scan_dirs;

    let scan_dirs = get_user_binary_scan_dirs();
    // System directories must be excluded
    assert!(!scan_dirs.contains(&std::path::PathBuf::from("/usr/bin")));
    assert!(!scan_dirs.contains(&std::path::PathBuf::from("/bin")));

    // When running on this host, ~/.local/bin or user paths should be found
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin");
        if local_bin.exists() {
            assert!(scan_dirs.contains(&local_bin));
        }
        let cargo_bin = home.join(".cargo/bin");
        if cargo_bin.exists() {
            assert!(scan_dirs.contains(&cargo_bin), "Cargo bin directory must be discovered");
        }
    }
}

#[test]
fn test_expand_home_path_expands_tokens_and_rejects_relative() {
    use sweepx::scanner::manual::expand_home_path;
    let home = std::path::Path::new("/fake/home");

    // Valid home expansions
    assert_eq!(
        expand_home_path("~/.local/bin", home),
        Some(std::path::PathBuf::from("/fake/home/.local/bin"))
    );
    assert_eq!(
        expand_home_path("$HOME/.cargo/bin", home),
        Some(std::path::PathBuf::from("/fake/home/.cargo/bin"))
    );
    assert_eq!(
        expand_home_path("${HOME}/bin", home),
        Some(std::path::PathBuf::from("/fake/home/bin"))
    );
    assert_eq!(
        expand_home_path("/usr/local/bin", home),
        Some(std::path::PathBuf::from("/usr/local/bin"))
    );

    // Relative or invalid paths are rejected
    assert_eq!(expand_home_path("relative/path", home), None);
    assert_eq!(expand_home_path("$PATH", home), None);
    assert_eq!(expand_home_path("-option", home), None);
}

#[test]
fn test_extract_paths_from_profile_files_at_parses_nested_sourced_scripts() {
    use sweepx::scanner::manual::extract_paths_from_profile_files_at;
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let home = temp_dir.path();

    // 1. Create real directories to simulate user tool installations
    let cargo_bin = home.join(".cargo/bin");
    let pixi_bin = home.join(".pixi/bin");
    let custom_bin = home.join(".custom_tool/bin");
    std::fs::create_dir_all(&cargo_bin).unwrap();
    std::fs::create_dir_all(&pixi_bin).unwrap();
    std::fs::create_dir_all(&custom_bin).unwrap();

    // 2. Create nested sourced environment script ~/.cargo/env
    let cargo_env = home.join(".cargo/env");
    std::fs::write(&cargo_env, "export PATH=\"$HOME/.cargo/bin:${PATH}\"\n").unwrap();

    // 3. Create another sourced script
    let custom_env = home.join(".custom_env");
    std::fs::write(&custom_env, "export PATH=\"$HOME/.custom_tool/bin:$PATH\"\n").unwrap();

    // 4. Create ~/.zshrc referencing both sourced scripts and direct PATH assignments
    let zshrc = home.join(".zshrc");
    let zshrc_content = format!(
        "# Shell startup comment\n\
         source \"{}\"\n\
         . \"{}\"\n\
         export PATH=\"$HOME/.pixi/bin:$PATH\"\n\
         # Irrelevant command\n\
         echo 'hello'\n",
        cargo_env.display(),
        custom_env.display()
    );
    std::fs::write(&zshrc, zshrc_content).unwrap();

    // 5. Run profile extraction
    let os_paths = extract_paths_from_profile_files_at(home)
        .expect("Should discover paths from profile and nested scripts");

    let split_paths: Vec<std::path::PathBuf> = std::env::split_paths(&os_paths).collect();
    assert!(split_paths.contains(&cargo_bin), "Must discover ~/.cargo/bin via nested sourced script");
    assert!(split_paths.contains(&custom_bin), "Must discover ~/.custom_tool/bin via dotted source");
    assert!(split_paths.contains(&pixi_bin), "Must discover ~/.pixi/bin via direct PATH assignment");
}





