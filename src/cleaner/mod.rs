pub mod executor;
pub mod heuristic;
pub mod privilege;
pub mod safety;
pub mod signatures;

pub use executor::{execute_purge_package, execute_purge_residuals, DeletionMode, PurgeReport};
pub use heuristic::discover_residuals_for_app;
pub use privilege::run_elevated_command;
pub use safety::{normalize_path, SafetyValidator, SafetyViolation};
pub use signatures::{find_signature, AppSignature, KNOWN_SIGNATURES};
