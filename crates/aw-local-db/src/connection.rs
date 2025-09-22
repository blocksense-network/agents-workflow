//! Database connection management.

use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;

/// Database connection wrapper with connection pooling support.
#[derive(Debug, Clone)]
pub struct Database {
    connection: Arc<std::sync::Mutex<Connection>>,
}

impl Database {
    /// Open a new database connection at the specified path.
    ///
    /// If the path doesn't exist, the database will be created.
    pub fn open<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let conn = Connection::open(path)?;
        Self::initialize_schema(&conn)?;
        Ok(Self {
            connection: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Open an in-memory database for testing.
    pub fn open_in_memory() -> crate::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::initialize_schema(&conn)?;
        Ok(Self {
            connection: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Initialize the database schema.
    fn initialize_schema(conn: &Connection) -> crate::Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                task_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                started_at DATETIME,
                finished_at DATETIME,
                metadata TEXT,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );
            "#,
        )?;
        Ok(())
    }

    /// Get a reference to the underlying connection.
    ///
    /// This method provides access to the connection for executing queries.
    /// The caller must ensure proper locking if used concurrently.
    pub fn connection(&self) -> &std::sync::Mutex<Connection> {
        &self.connection
    }

    /// Execute a transaction with automatic rollback on error.
    pub fn transaction<F, T>(&self, f: F) -> crate::Result<T>
    where
        F: FnOnce(&Connection) -> crate::Result<T>,
    {
        let conn = self.connection.lock().map_err(|e| {
            crate::Error::generic(format!("Failed to acquire database lock: {}", e))
        })?;

        let tx = conn.unchecked_transaction()?;
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()?;
                Err(e)
            }
        }
    }
}
