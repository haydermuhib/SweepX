pub const INIT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS applications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    version TEXT,
    install_method TEXT NOT NULL,
    exec_path TEXT,
    icon TEXT,
    desktop_file TEXT,
    total_size_bytes INTEGER NOT NULL DEFAULT 0,
    install_date TEXT,
    last_scanned TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    install_method TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    freed_bytes INTEGER NOT NULL DEFAULT 0,
    deleted_paths TEXT NOT NULL,
    status TEXT NOT NULL,
    error_details TEXT
);
"#;
