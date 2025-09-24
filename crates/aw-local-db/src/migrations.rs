//! Database migration management.

use rusqlite::{params, Connection};

/// Database migration manager.
pub struct MigrationManager;

impl MigrationManager {
    /// Apply all pending migrations to the database.
    pub fn migrate(conn: &Connection) -> crate::Result<()> {
        // For now, just ensure the basic schema exists
        // Future migrations would be applied here based on version tracking

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Insert version 1 if not exists
            INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);

            -- Create tables for version 1
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                repo_id INTEGER,
                workspace_id INTEGER,
                agent_id INTEGER,
                runtime_id INTEGER,
                multiplexer_kind TEXT,
                mux_session TEXT,
                mux_window INTEGER,
                pane_left TEXT,
                pane_right TEXT,
                pid_agent INTEGER,
                status TEXT NOT NULL,
                log_path TEXT,
                workspace_path TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT
            );

            CREATE TABLE IF NOT EXISTS fs_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                ts TEXT NOT NULL,
                provider TEXT NOT NULL,
                ref TEXT,
                path TEXT,
                parent_id INTEGER,
                metadata TEXT,
                FOREIGN KEY (parent_id) REFERENCES fs_snapshots(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_fs_snapshots_session_ts ON fs_snapshots(session_id, ts);
            "#,
        )?;

        Ok(())
    }

    /// Get the current schema version.
    pub fn current_version(conn: &Connection) -> crate::Result<Option<u32>> {
        let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations")?;

        let version: Option<u32> = stmt.query_row(params![], |row| row.get(0)).ok();

        Ok(version)
    }
}
