//! Database models and persistence operations.

use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Database model for tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<String>,
}

impl TaskRecord {
    /// Convert to an aw-core Task.
    pub fn to_core_task(&self) -> crate::Result<aw_core::Task> {
        let status = match self.status.as_str() {
            "Pending" => aw_core::TaskStatus::Pending,
            "Running" => aw_core::TaskStatus::Running,
            "Completed" => aw_core::TaskStatus::Completed,
            "Failed" => aw_core::TaskStatus::Failed,
            "Cancelled" => aw_core::TaskStatus::Cancelled,
            _ => {
                return Err(crate::Error::generic(format!(
                    "Unknown task status: {}",
                    self.status
                )))
            }
        };

        let metadata = self
            .metadata
            .as_ref()
            .map(|m| serde_json::from_str(m))
            .transpose()?
            .unwrap_or_default();

        Ok(aw_core::Task {
            id: aw_core::TaskId(self.id as u64),
            name: self.name.clone(),
            description: self.description.clone(),
            status,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| crate::Error::generic(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| crate::Error::generic(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            metadata,
        })
    }

    /// Create from an aw-core Task.
    pub fn from_core_task(task: &aw_core::Task) -> Self {
        let status = match task.status {
            aw_core::TaskStatus::Pending => "Pending",
            aw_core::TaskStatus::Running => "Running",
            aw_core::TaskStatus::Completed => "Completed",
            aw_core::TaskStatus::Failed => "Failed",
            aw_core::TaskStatus::Cancelled => "Cancelled",
        };

        let metadata = if task.metadata.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&task.metadata).unwrap())
        };

        Self {
            id: task.id.0 as i64,
            name: task.name.clone(),
            description: task.description.clone(),
            status: status.to_string(),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            metadata,
        }
    }
}

/// Database operations for tasks.
pub struct TaskStore<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> TaskStore<'a> {
    /// Create a new task store.
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Insert a new task.
    pub fn insert(&self, task: &aw_core::Task) -> crate::Result<i64> {
        let record = TaskRecord::from_core_task(task);
        self.conn.execute(
            r#"
            INSERT INTO tasks (name, description, status, created_at, updated_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                record.name,
                record.description,
                record.status,
                record.created_at,
                record.updated_at,
                record.metadata
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get a task by ID.
    pub fn get(&self, id: aw_core::TaskId) -> crate::Result<Option<aw_core::Task>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, description, status, created_at, updated_at, metadata
            FROM tasks WHERE id = ?
            "#,
        )?;

        let mut rows = stmt.query_map(params![id.0 as i64], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                metadata: row.get(6)?,
            })
        })?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record.to_core_task()?)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List all tasks.
    pub fn list(&self) -> crate::Result<Vec<aw_core::Task>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, description, status, created_at, updated_at, metadata
            FROM tasks ORDER BY created_at DESC
            "#,
        )?;

        let records = stmt.query_map(params![], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                metadata: row.get(6)?,
            })
        })?;

        let mut tasks = Vec::new();
        for record in records {
            tasks.push(record?.to_core_task()?);
        }
        Ok(tasks)
    }

    /// Update a task.
    pub fn update(&self, task: &aw_core::Task) -> crate::Result<()> {
        let record = TaskRecord::from_core_task(task);
        self.conn.execute(
            r#"
            UPDATE tasks
            SET name = ?, description = ?, status = ?, updated_at = ?, metadata = ?
            WHERE id = ?
            "#,
            params![
                record.name,
                record.description,
                record.status,
                record.updated_at,
                record.metadata,
                record.id
            ],
        )?;
        Ok(())
    }
}

/// Database model for filesystem snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsSnapshotRecord {
    pub id: i64,
    pub session_id: String,
    pub ts: String,
    pub provider: String,
    pub ref_: String, // `ref` is a keyword in Rust, so use `ref_`
    pub path: Option<String>,
    pub parent_id: Option<i64>,
    pub metadata: Option<String>,
}

/// Database operations for filesystem snapshots.
pub struct FsSnapshotStore<'a> {
    conn: &'a rusqlite::Connection,
}

impl<'a> FsSnapshotStore<'a> {
    /// Create a new fs snapshot store.
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Insert a new filesystem snapshot.
    pub fn insert(&self, record: &FsSnapshotRecord) -> crate::Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO fs_snapshots (session_id, ts, provider, ref, path, parent_id, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                record.session_id,
                record.ts,
                record.provider,
                record.ref_,
                record.path,
                record.parent_id,
                record.metadata
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get a snapshot by ID.
    pub fn get(&self, id: i64) -> crate::Result<Option<FsSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, ts, provider, ref, path, parent_id, metadata
            FROM fs_snapshots WHERE id = ?
            "#,
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(FsSnapshotRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                ts: row.get(2)?,
                provider: row.get(3)?,
                ref_: row.get(4)?,
                path: row.get(5)?,
                parent_id: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List snapshots for a session.
    pub fn list_by_session(&self, session_id: &str) -> crate::Result<Vec<FsSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, ts, provider, ref, path, parent_id, metadata
            FROM fs_snapshots
            WHERE session_id = ?
            ORDER BY ts ASC
            "#,
        )?;

        let records = stmt.query_map(params![session_id], |row| {
            Ok(FsSnapshotRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                ts: row.get(2)?,
                provider: row.get(3)?,
                ref_: row.get(4)?,
                path: row.get(5)?,
                parent_id: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        let mut snapshots = Vec::new();
        for record in records {
            snapshots.push(record?);
        }
        Ok(snapshots)
    }

    /// List all snapshots.
    pub fn list_all(&self) -> crate::Result<Vec<FsSnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, ts, provider, ref, path, parent_id, metadata
            FROM fs_snapshots
            ORDER BY ts DESC
            "#,
        )?;

        let records = stmt.query_map(params![], |row| {
            Ok(FsSnapshotRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                ts: row.get(2)?,
                provider: row.get(3)?,
                ref_: row.get(4)?,
                path: row.get(5)?,
                parent_id: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        let mut snapshots = Vec::new();
        for record in records {
            snapshots.push(record?);
        }
        Ok(snapshots)
    }

    /// Update a snapshot.
    pub fn update(&self, record: &FsSnapshotRecord) -> crate::Result<()> {
        self.conn.execute(
            r#"
            UPDATE fs_snapshots
            SET session_id = ?, ts = ?, provider = ?, ref = ?, path = ?, parent_id = ?, metadata = ?
            WHERE id = ?
            "#,
            params![
                record.session_id,
                record.ts,
                record.provider,
                record.ref_,
                record.path,
                record.parent_id,
                record.metadata,
                record.id
            ],
        )?;
        Ok(())
    }

    /// Delete a snapshot.
    pub fn delete(&self, id: i64) -> crate::Result<()> {
        self.conn.execute(
            "DELETE FROM fs_snapshots WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }
}
