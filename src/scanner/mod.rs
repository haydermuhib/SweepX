pub mod desktop_entry;
pub mod flatpak;
pub mod manual;
pub mod native;
pub mod orchestrator;
pub mod snap;

pub use desktop_entry::{
    extract_binary_from_exec, get_desktop_entry_directories, parse_desktop_entry_content,
    resolve_binary_path, scan_desktop_entries,
};
pub use flatpak::{parse_flatpak_list_output, parse_flatpak_size_string, scan_flatpaks};
pub use manual::{calculate_dir_size, scan_manual_installations};
pub use native::{
    detect_available_package_managers, AptManager, DnfManager, NativeBackend, PacmanManager,
};
pub use orchestrator::scan_all_applications;
pub use snap::{parse_snap_list_output, scan_snaps};
