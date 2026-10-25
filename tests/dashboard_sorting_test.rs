use std::collections::HashMap;
use sweepx::models::{Application, InstallMethod, SizeUnitSystem};
use sweepx::ui::{
    sort_and_filter_applications, CategoryFilter, SortDirection, SortField, SweepXApp,
};

fn create_test_apps() -> Vec<Application> {
    let mut app_a = Application::new("code", "Visual Studio Code", InstallMethod::NativeDnf);
    app_a.display_name = "Visual Studio Code (Text Editor)".to_string();
    app_a.total_size_bytes = 900 * 1024 * 1024; // 900 MB
    app_a.is_system = false;

    let mut app_b = Application::new("calc", "Calculator", InstallMethod::NativeDnf);
    app_b.display_name = "Calculator".to_string();
    app_b.total_size_bytes = 10 * 1024 * 1024; // 10 MB
    app_b.is_system = false;

    let mut app_c = Application::new("com.spotify.Client", "Spotify", InstallMethod::Flatpak);
    app_c.display_name = "Spotify (Music Player)".to_string();
    app_c.total_size_bytes = 150 * 1024 * 1024; // 150 MB
    app_c.is_system = false;

    let mut app_d = Application::new("notepad-plus-plus", "Notepad++", InstallMethod::Snap);
    app_d.display_name = "Notepad++ (Text Editor)".to_string();
    app_d.total_size_bytes = 20 * 1024 * 1024; // 20 MB
    app_d.is_system = false;

    let mut app_sys = Application::new("glibc", "glibc", InstallMethod::NativeDnf);
    app_sys.display_name = "glibc (C Library)".to_string();
    app_sys.total_size_bytes = 35 * 1024 * 1024; // 35 MB
    app_sys.is_system = true;

    vec![app_a, app_b, app_c, app_d, app_sys]
}

#[test]
fn test_sort_by_name_ascending_and_descending() {
    let apps = create_test_apps();
    let cache = HashMap::new();

    // Ascending
    let sorted_asc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false, // hide system
        SortField::Name,
        SortDirection::Ascending,
        &cache,
    );

    let names_asc: Vec<&str> = sorted_asc.iter().map(|a| a.display_name.as_str()).collect();
    assert_eq!(
        names_asc,
        vec![
            "Calculator",
            "Notepad++ (Text Editor)",
            "Spotify (Music Player)",
            "Visual Studio Code (Text Editor)",
        ]
    );

    // Descending
    let sorted_desc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Name,
        SortDirection::Descending,
        &cache,
    );

    let names_desc: Vec<&str> = sorted_desc.iter().map(|a| a.display_name.as_str()).collect();
    assert_eq!(
        names_desc,
        vec![
            "Visual Studio Code (Text Editor)",
            "Spotify (Music Player)",
            "Notepad++ (Text Editor)",
            "Calculator",
        ]
    );
}

#[test]
fn test_sort_by_payload_size_ascending_and_descending() {
    let apps = create_test_apps();
    let cache = HashMap::new();

    // Ascending (smallest first)
    let sorted_asc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Size,
        SortDirection::Ascending,
        &cache,
    );

    let ids_asc: Vec<&str> = sorted_asc.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids_asc, vec!["calc", "notepad-plus-plus", "com.spotify.Client", "code"]);

    // Descending (largest first)
    let sorted_desc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Size,
        SortDirection::Descending,
        &cache,
    );

    let ids_desc: Vec<&str> = sorted_desc.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids_desc, vec!["code", "com.spotify.Client", "notepad-plus-plus", "calc"]);
}

#[test]
fn test_sort_by_footprint_with_and_without_residual_cache() {
    let apps = create_test_apps();
    let mut cache = HashMap::new();

    // Give Spotify a huge residual footprint (e.g. 2 GB of song cache & sandbox)
    cache.insert("com.spotify.Client".to_string(), 2 * 1024 * 1024 * 1024);

    let sorted_desc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Footprint,
        SortDirection::Descending,
        &cache,
    );

    // Spotify has payload 150MB + 2GB residuals = ~2.15GB, beating VS Code's 900MB!
    assert_eq!(sorted_desc[0].id, "com.spotify.Client");
    assert_eq!(sorted_desc[1].id, "code");
    assert_eq!(sorted_desc[2].id, "notepad-plus-plus");
    assert_eq!(sorted_desc[3].id, "calc");
}

#[test]
fn test_sort_by_origin_packaging_tier() {
    let apps = create_test_apps();
    let cache = HashMap::new();

    let sorted_asc = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Origin,
        SortDirection::Ascending,
        &cache,
    );

    // DNF < Flatpak < Snap
    let origins: Vec<&str> = sorted_asc.iter().map(|a| a.install_method.badge_label()).collect();
    assert_eq!(origins[0], "DNF");
    assert_eq!(origins[1], "DNF");
    assert_eq!(origins[2], "Flatpak");
    assert_eq!(origins[3], "Snap");
}

#[test]
fn test_interactive_sort_toggle_behavior() {
    let (mut app_state, _task) = SweepXApp::new();

    // Default is Name Ascending
    assert_eq!(app_state.sort_field, SortField::Name);
    assert_eq!(app_state.sort_direction, SortDirection::Ascending);

    // Clicking Name again toggles to Descending
    app_state.select_sort(SortField::Name);
    assert_eq!(app_state.sort_field, SortField::Name);
    assert_eq!(app_state.sort_direction, SortDirection::Descending);

    // Clicking Name again toggles back to Ascending
    app_state.select_sort(SortField::Name);
    assert_eq!(app_state.sort_direction, SortDirection::Ascending);

    // Switching to Size defaults to Descending (show heaviest apps first)
    app_state.select_sort(SortField::Size);
    assert_eq!(app_state.sort_field, SortField::Size);
    assert_eq!(app_state.sort_direction, SortDirection::Descending);

    // Clicking Size again toggles to Ascending
    app_state.select_sort(SortField::Size);
    assert_eq!(app_state.sort_direction, SortDirection::Ascending);

    // Switching to Footprint defaults to Descending
    app_state.select_sort(SortField::Footprint);
    assert_eq!(app_state.sort_field, SortField::Footprint);
    assert_eq!(app_state.sort_direction, SortDirection::Descending);

    // Switching to Origin defaults to Ascending
    app_state.select_sort(SortField::Origin);
    assert_eq!(app_state.sort_field, SortField::Origin);
    assert_eq!(app_state.sort_direction, SortDirection::Ascending);
}

#[test]
fn test_category_and_system_filtering_integrated_with_sorting() {
    let apps = create_test_apps();
    let cache = HashMap::new();

    // 1. Without system libraries, glibc is excluded
    let user_apps = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        false,
        SortField::Name,
        SortDirection::Ascending,
        &cache,
    );
    assert_eq!(user_apps.len(), 4);
    assert!(user_apps.iter().all(|a| !a.is_system));

    // 2. With show_system_libraries enabled, glibc is included
    let all_apps = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::All,
        true,
        SortField::Name,
        SortDirection::Ascending,
        &cache,
    );
    assert_eq!(all_apps.len(), 5);
    assert!(all_apps.iter().any(|a| a.id == "glibc"));

    // 3. Flatpak category only
    let flatpaks = sort_and_filter_applications(
        &apps,
        "",
        CategoryFilter::Flatpak,
        false,
        SortField::Name,
        SortDirection::Ascending,
        &cache,
    );
    assert_eq!(flatpaks.len(), 1);
    assert_eq!(flatpaks[0].id, "com.spotify.Client");

    // 4. Search query filter
    let searched = sort_and_filter_applications(
        &apps,
        "editor",
        CategoryFilter::All,
        false,
        SortField::Name,
        SortDirection::Ascending,
        &cache,
    );
    assert_eq!(searched.len(), 2);
    let searched_ids: Vec<&str> = searched.iter().map(|a| a.id.as_str()).collect();
    assert!(searched_ids.contains(&"code"));
    assert!(searched_ids.contains(&"notepad-plus-plus"));
}

#[test]
fn test_footprint_consistency_across_search_and_inspect_cache() {
    let (mut app_state, _task) = SweepXApp::new();
    let apps = create_test_apps();

    // 1. Initial scan completed updates apps list
    let _ = app_state.update(sweepx::ui::Message::ScanCompleted(apps));

    // 2. Residual batch loaded populates cache
    let _ = app_state.update(sweepx::ui::Message::ResidualsBatchLoaded(vec![
        ("code".to_string(), 500 * 1024 * 1024), // 500 MB residuals
    ]));

    // 3. When searching for "code", cache must match inspected calculation
    let _ = app_state.update(sweepx::ui::Message::SearchChanged("code".to_string()));

    let cached = app_state.app_residuals_cache.get("code").copied();
    assert_eq!(cached, Some(500 * 1024 * 1024));

    let code_app = app_state.applications.iter().find(|a| a.id == "code").unwrap();
    let total_footprint = code_app.total_size_bytes + cached.unwrap();
    assert_eq!(total_footprint, 1400 * 1024 * 1024); // 900 MB payload + 500 MB residuals = 1400 MB
}

#[test]
fn test_unit_system_toggle_and_formatting() {
    let (mut app_state, _task) = SweepXApp::new();

    // 1. Defaults to GiB system (IEC binary base 1024)
    assert_eq!(app_state.unit_system, SizeUnitSystem::GiB);
    assert_eq!(app_state.format_bytes(1024 * 1024 * 1024), "1.00 GiB");
    assert_eq!(app_state.format_bytes(50 * 1024 * 1024 * 1024), "50.00 GiB");

    // 2. Toggle unit system to GB (SI decimal base 1000)
    let _ = app_state.update(sweepx::ui::Message::ToggleUnitSystem);
    assert_eq!(app_state.unit_system, SizeUnitSystem::GB);
    assert_eq!(app_state.format_bytes(1000 * 1000 * 1000), "1.00 GB");
    // 50 GiB in bytes formatted as GB decimal: 50 * 1024^3 = 53,687,091,200 bytes => 53.69 GB
    assert_eq!(app_state.format_bytes(50 * 1024 * 1024 * 1024), "53.69 GB");

    // 3. Toggle back to GiB
    let _ = app_state.update(sweepx::ui::Message::ToggleUnitSystem);
    assert_eq!(app_state.unit_system, SizeUnitSystem::GiB);
    assert_eq!(app_state.format_bytes(50 * 1024 * 1024 * 1024), "50.00 GiB");
}


