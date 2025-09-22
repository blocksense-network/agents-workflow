//! Database schema definitions and constants.

// Current schema version
pub const SCHEMA_VERSION: u32 = 1;

// Table names
pub const TABLE_SCHEMA_MIGRATIONS: &str = "schema_migrations";
pub const TABLE_TASKS: &str = "tasks";
pub const TABLE_SESSIONS: &str = "sessions";

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
