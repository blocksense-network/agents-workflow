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
