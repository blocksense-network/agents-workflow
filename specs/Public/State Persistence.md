# State Persistence

This document specifies how Agents‑Workflow (AW) persists CLI/TUI state
locally and how it aligns with a remote server. It defines selection rules,
storage locations, and the canonical SQL schema used by the local state
database and mirrored (logically) by the server.

## Overview

- AW operates against one of two backends:
  - **Local SQLite**: the CLI performs state mutations directly against a
    per‑user SQLite database. Multiple `aw` processes may concurrently read/write
    this DB.
  - **Remote REST**: the CLI talks to a remote server which implements the same
    logical schema and API endpoints.

Both backends share the same logical data model so behavior is consistent.

## Backend Selection

- If a `remote-server` is provided via the configuration system (or via an
  equivalent CLI flag), AW uses the REST API of that server.
- Otherwise, AW uses the local SQLite database.

All behavior follows standard configuration layering (CLI flags > env >
project‑user > project > user > system).

## DB Locations

- Local SQLite DB:
  - Linux: `${XDG_STATE_HOME:-~/.local/state}/agents-workflow/state.db`
  - macOS: `~/Library/Application Support/Agents-Workflow/state.db`
  - Windows: `%LOCALAPPDATA%\Agents-Workflow\state.db`

SQLite is opened in WAL mode. The CLI manages `PRAGMA user_version` for
migrations (see Schema Versioning).

## Relationship to Prior Drafts

Earlier drafts described PID‑like JSON session records and a local daemon.
These are no longer part of the design. The SQLite database is the sole local
source of truth; the CLI talks directly to it.

## SQL Schema (SQLite dialect)

This schema models repositories, workspaces, tasks, sessions, runtimes, agents,
events, and filesystem snapshots. It is intentionally normalized for
portability to server backends.

```sql
-- Schema versioning
PRAGMA user_version = 1;

-- Repositories known to the system (local path and/or remote URL)
CREATE TABLE IF NOT EXISTS repos (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  vcs           TEXT NOT NULL,                 -- git|hg|pijul|...
  root_path     TEXT,                          -- local filesystem root
(nullable in REST)
  remote_url    TEXT,                          -- canonical remote URL
(nullable in local)
  default_branch TEXT,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(root_path),
  UNIQUE(remote_url)
);

-- Workspaces are named logical groupings on some servers. Optional locally.
CREATE TABLE IF NOT EXISTS workspaces (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT NOT NULL,
  external_id  TEXT,                           -- server-provided ID (REST)
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE(name)
);

-- Agents catalog (type + version descriptor)
CREATE TABLE IF NOT EXISTS agents (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT NOT NULL,                  -- e.g., 'openhands',
'claude-code'
  version      TEXT NOT NULL,                  -- 'latest' or semver-like
  metadata     TEXT,                           -- JSON string for extra
capabilities
  UNIQUE(name, version)
);

-- Runtime definitions (devcontainer, local, nosandbox, etc.)
CREATE TABLE IF NOT EXISTS runtimes (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  type         TEXT NOT NULL,                  -- devcontainer|local|nosandbox
  devcontainer_path TEXT,                      -- when type=devcontainer
  metadata     TEXT                            -- JSON string
);

-- Sessions are concrete agent runs bound to a repo (and optionally a workspace)
CREATE TABLE IF NOT EXISTS sessions (
  id           TEXT PRIMARY KEY,               -- stable ULID/UUID string
  repo_id      INTEGER NOT NULL REFERENCES repos(id) ON DELETE RESTRICT,
  workspace_id INTEGER REFERENCES workspaces(id) ON DELETE SET NULL,
  agent_id     INTEGER NOT NULL REFERENCES agents(id) ON DELETE RESTRICT,
  runtime_id   INTEGER NOT NULL REFERENCES runtimes(id) ON DELETE RESTRICT,
  multiplexer_kind TEXT,                       -- tmux|zellij|screen
  mux_session  TEXT,
  mux_window   INTEGER,
  pane_left    TEXT,
  pane_right   TEXT,
  pid_agent    INTEGER,
  status       TEXT NOT NULL,                  --
created|running|failed|succeeded|cancelled
  log_path     TEXT,
  workspace_path TEXT,                         -- per-task filesystem workspace
  started_at   TEXT NOT NULL,
  ended_at     TEXT
);

-- Tasks capture user intent and parameters used to launch a session
CREATE TABLE IF NOT EXISTS tasks (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id   TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  prompt       TEXT NOT NULL,
  branch       TEXT,
  delivery     TEXT,                           -- pr|branch|patch
  instances    INTEGER DEFAULT 1,
  labels       TEXT,                           -- JSON object k=v
  browser_automation INTEGER NOT NULL DEFAULT 1, -- 1=true, 0=false
  browser_profile  TEXT,
  chatgpt_username TEXT,
  codex_workspace  TEXT
);

-- Event log per session for diagnostics and incremental state
CREATE TABLE IF NOT EXISTS events (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id   TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  ts           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  type         TEXT NOT NULL,
  data         TEXT                             -- JSON payload
);

CREATE INDEX IF NOT EXISTS idx_events_session_ts ON events(session_id, ts);

-- Filesystem snapshots associated with a session (see docs/fs-snapshots)
CREATE TABLE IF NOT EXISTS fs_snapshots (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id   TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  ts           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  provider     TEXT NOT NULL,                  -- zfs|btrfs|overlay|copy
  ref          TEXT,                           -- dataset/subvolume id, overlay
dir, etc.
  path         TEXT,                           -- mount or copy path
  parent_id    INTEGER REFERENCES fs_snapshots(id) ON DELETE SET NULL,
  metadata     TEXT                            -- JSON payload
);

-- Key/value subsystem for small, fast lookups (scoped configuration, caches)
CREATE TABLE IF NOT EXISTS kv (
  scope        TEXT NOT NULL,                  -- user|project|repo|runtime|...
  k            TEXT NOT NULL,
  v            TEXT,
  PRIMARY KEY (scope, k)
);
```

### Schema Versioning

- The database uses `PRAGMA user_version` for migrations. Increment the version
  for any backwards‑incompatible change. A simple `migrations/` folder with
  `N__description.sql` files can be applied in order.

### Concurrency and Locking

- SQLite operates in WAL mode to minimize writer contention. Multiple `aw`
  processes can write concurrently; all writes use transactions with retry on
  `SQLITE_BUSY`.

### Security and Privacy

- Secrets are never stored in plain text in this DB. Authentication with remote
  services uses OS‑level keychains or scoped token stores managed by the CLI
  and/or OS keychain helpers.

## Repo Detection

When `--repo` is not supplied, AW detects a repository by walking up from the
current directory until it finds a VCS root. All supported VCS are checked
(git, hg, etc.). If none is found, commands requiring a repository fail with a
clear error.

## Workspaces

`--workspace` is only meaningful when speaking to a server that supports named
workspaces. Local SQLite mode does not define workspaces by default. Commands
that specify `--workspace` while the active backend does not support workspaces
MUST fail with a clear message.
