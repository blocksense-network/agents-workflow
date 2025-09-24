//! Database schema definitions and constants.

// Current schema version
pub const SCHEMA_VERSION: u32 = 1;

// Table names
pub const TABLE_SCHEMA_MIGRATIONS: &str = "schema_migrations";
pub const TABLE_TASKS: &str = "tasks";
pub const TABLE_SESSIONS: &str = "sessions";
pub const TABLE_FS_SNAPSHOTS: &str = "fs_snapshots";

// Column names for tasks table
pub mod tasks {
    pub const ID: &str = "id";
    pub const NAME: &str = "name";
    pub const DESCRIPTION: &str = "description";
    pub const STATUS: &str = "status";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
    pub const METADATA: &str = "metadata";
}

// Column names for sessions table
pub mod sessions {
    pub const ID: &str = "id";
    pub const TASK_ID: &str = "task_id";
    pub const NAME: &str = "name";
    pub const STATUS: &str = "status";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
    pub const STARTED_AT: &str = "started_at";
    pub const FINISHED_AT: &str = "finished_at";
    pub const METADATA: &str = "metadata";
}

// Column names for fs_snapshots table
pub mod fs_snapshots {
    pub const ID: &str = "id";
    pub const SESSION_ID: &str = "session_id";
    pub const TS: &str = "ts";
    pub const PROVIDER: &str = "provider";
    pub const REF: &str = "ref";
    pub const PATH: &str = "path";
    pub const PARENT_ID: &str = "parent_id";
    pub const METADATA: &str = "metadata";
}
