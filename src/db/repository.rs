use super::schema::INIT_SCHEMA;
use crate::models::Application;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Option<i64>,
    pub app_id: String,
    pub app_name: String,
    pub install_method: String,
    pub timestamp: DateTime<Utc>,
    pub freed_bytes: u64,
    pub deleted_paths: Vec<PathBuf>,
    pub status: String,
    pub error_details: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let db_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sweepx");
        std::fs::create_dir_all(&db_dir).ok();
        let db_path = db_dir.join("sweepx.db");
        Self::open(&db_path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(INIT_SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(INIT_SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn save_apps(&mut self, apps: &[Application]) -> Result<()> {
        let tx = self.conn.transaction()?;
        let now = Utc::now().to_rfc3339();

        for app in apps {
            let method_str = serde_json::to_string(&app.install_method).unwrap_or_default();
            let exec_str = app.exec_path.as_ref().map(|p| p.to_string_lossy().to_string());
            let desktop_str = app.desktop_file.as_ref().map(|p| p.to_string_lossy().to_string());
            let install_date_str = app.install_date.map(|d| d.to_rfc3339());

            tx.execute(
                "INSERT INTO applications (id, name, display_name, version, install_method, exec_path, icon, desktop_file, total_size_bytes, install_date, last_scanned, is_system)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    display_name = excluded.display_name,
                    version = excluded.version,
                    install_method = excluded.install_method,
                    exec_path = excluded.exec_path,
                    icon = excluded.icon,
                    desktop_file = excluded.desktop_file,
                    total_size_bytes = excluded.total_size_bytes,
                    last_scanned = excluded.last_scanned,
                    is_system = excluded.is_system",
                params![
                    app.id,
                    app.name,
                    app.display_name,
                    app.version,
                    method_str,
                    exec_str,
                    app.icon,
                    desktop_str,
                    app.total_size_bytes as i64,
                    install_date_str,
                    now,
                    if app.is_system { 1 } else { 0 },
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn record_audit(&self, entry: &AuditLogEntry) -> Result<()> {
        let paths_json = serde_json::to_string(&entry.deleted_paths).unwrap_or_default();
        let time_str = entry.timestamp.to_rfc3339();

        self.conn.execute(
            "INSERT INTO audit_logs (app_id, app_name, install_method, timestamp, freed_bytes, deleted_paths, status, error_details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.app_id,
                entry.app_name,
                entry.install_method,
                time_str,
                entry.freed_bytes as i64,
                paths_json,
                entry.status,
                entry.error_details,
            ],
        )?;

        Ok(())
    }

    pub fn get_audit_history(&self) -> Result<Vec<AuditLogEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, app_id, app_name, install_method, timestamp, freed_bytes, deleted_paths, status, error_details
             FROM audit_logs ORDER BY id DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let app_id: String = row.get(1)?;
            let app_name: String = row.get(2)?;
            let install_method: String = row.get(3)?;
            let time_str: String = row.get(4)?;
            let freed_bytes: i64 = row.get(5)?;
            let paths_json: String = row.get(6)?;
            let status: String = row.get(7)?;
            let error_details: Option<String> = row.get(8)?;

            let timestamp = DateTime::parse_from_rfc3339(&time_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let deleted_paths: Vec<PathBuf> =
                serde_json::from_str(&paths_json).unwrap_or_default();

            Ok(AuditLogEntry {
                id: Some(id),
                app_id,
                app_name,
                install_method,
                timestamp,
                freed_bytes: freed_bytes as u64,
                deleted_paths,
                status,
                error_details,
            })
        })?;

        let mut logs = Vec::new();
        for r in rows {
            logs.push(r?);
        }
        Ok(logs)
    }
}
