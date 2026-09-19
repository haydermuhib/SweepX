pub mod executor;
pub mod heuristic;
pub mod privilege;
pub mod safety;
pub mod signatures;
pub mod system_sweep;

pub use executor::{execute_purge_package, execute_purge_residuals, DeletionMode, PurgeReport};
pub use heuristic::discover_residuals_for_app;
pub use privilege::run_elevated_command;
pub use safety::{normalize_path, SafetyValidator, SafetyViolation};
pub use signatures::{find_signature, AppSignature, KNOWN_SIGNATURES};
pub use system_sweep::{
    execute_system_sweep, parse_snap_disabled_revisions, scan_all_sweep_items,
    scan_native_package_cache, SweepCategory, SystemSweepItem, SystemSweepReport,
};
