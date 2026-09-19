pub mod appimage;
pub mod desktop_entry;
pub mod flatpak;
pub mod manual;
pub mod native;
pub mod orchestrator;
pub mod snap;
pub mod symlink_graph;

pub use appimage::{integrate_appimage, is_appimage_integrated, AppImageInfo};
pub use desktop_entry::{
    extract_binary_from_exec, get_desktop_entry_directories, parse_desktop_entry_content,
    resolve_binary_path, scan_desktop_entries,
};
pub use flatpak::{parse_flatpak_list_output, parse_flatpak_size_string, scan_flatpaks};
pub use manual::{calculate_dir_size, scan_manual_installations};
pub use native::{
    detect_available_package_managers, AptManager, DnfManager, NativeBackend, PacmanManager,
};
pub use orchestrator::{apply_stage_batch, scan_all_applications, ScanStageBatch};
pub use snap::{parse_snap_list_output, scan_snaps};
pub use symlink_graph::{
    find_broken_symlinks, find_symlinks_pointing_to_app, scan_symlink_graph, SymlinkRecord,
};
