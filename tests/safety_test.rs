use std::path::PathBuf;
use sweepx::cleaner::{SafetyValidator, SafetyViolation};

#[test]
fn test_rejects_root_and_system_directories() {
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("/")),
        Err(SafetyViolation::SystemRoot)
    );
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("/usr")),
        Err(SafetyViolation::SystemDirectory("/usr".to_string()))
    );
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("/etc")),
        Err(SafetyViolation::SystemDirectory("/etc".to_string()))
    );
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("/var/log")),
        Err(SafetyViolation::SystemDirectory("/var/log".to_string()))
    );
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("/opt")),
        Err(SafetyViolation::SystemDirectory("/opt".to_string()))
    );
}

#[test]
fn test_rejects_user_home_and_xdg_base_roots() {
    if let Some(home) = dirs::home_dir() {
        assert_eq!(
            SafetyValidator::is_path_safe_to_delete(&home),
            Err(SafetyViolation::UserHomeRoot)
        );
        assert_eq!(
            SafetyValidator::is_path_safe_to_delete(&home.join(".config")),
            Err(SafetyViolation::ProtectedXdgBase(
                home.join(".config").to_string_lossy().to_string()
            ))
        );
        assert_eq!(
            SafetyValidator::is_path_safe_to_delete(&home.join(".cache")),
            Err(SafetyViolation::ProtectedXdgBase(
                home.join(".cache").to_string_lossy().to_string()
            ))
        );
        assert_eq!(
            SafetyValidator::is_path_safe_to_delete(&home.join(".local/share")),
            Err(SafetyViolation::ProtectedXdgBase(
                home.join(".local/share").to_string_lossy().to_string()
            ))
        );
    }
}

#[test]
fn test_rejects_relative_and_empty_paths() {
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("")),
        Err(SafetyViolation::EmptyPath)
    );
    assert_eq!(
        SafetyValidator::is_path_safe_to_delete(&PathBuf::from("relative/path")),
        Err(SafetyViolation::RelativePath)
    );
}

#[test]
fn test_allows_valid_application_paths() {
    let valid_paths = [
        PathBuf::from("/opt/google/chrome"),
        PathBuf::from("/opt/Postman"),
        PathBuf::from("/home/user/.config/slack"),
        PathBuf::from("/home/user/.cache/spotify"),
        PathBuf::from("/home/user/.local/share/Trash"), // Though trash, it is a subfolder
        PathBuf::from("/home/user/.var/app/com.valvesoftware.Steam"),
        PathBuf::from("/etc/nginx/sites-available/myapp"),
    ];

    for path in &valid_paths {
        assert!(
            SafetyValidator::is_path_safe_to_delete(path).is_ok(),
            "Expected {:?} to be safe",
            path
        );
    }
}

#[test]
fn test_sanitize_deletion_targets() {
    let targets = vec![
        PathBuf::from("/"),
        PathBuf::from("/opt/myapp"),
        PathBuf::from("relative/path"),
        PathBuf::from("/home/test/.config/myapp"),
    ];

    let (safe, rejected) = SafetyValidator::sanitize_deletion_targets(&targets);
    assert_eq!(safe.len(), 2);
    assert!(safe.contains(&PathBuf::from("/opt/myapp")));
    assert!(safe.contains(&PathBuf::from("/home/test/.config/myapp")));
    assert_eq!(rejected.len(), 2);
}
