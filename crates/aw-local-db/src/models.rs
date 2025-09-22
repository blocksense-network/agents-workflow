//! Database models and persistence operations.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
