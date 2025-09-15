## Agents-Workflow REST Service — Purpose and Specification

### Purpose

- **Central orchestration**: Provide a network API to create and manage isolated agent coding sessions on demand, aligned with the filesystem snapshot model described in [FS Snapshots Overview](FS%20Snapshots/FS%20Snapshots%20Overview.md).
- **On‑prem/private cloud ready**: Designed for enterprises running self‑managed clusters or single hosts.
- **UI consumers**: Back the WebUI and TUI dashboards; enable custom internal portals and automations.
- **Uniform abstraction**: Normalize differences between agents (Claude Code, OpenHands, Copilot, etc.), runtimes (devcontainer/local), and snapshot providers (ZFS/Btrfs/Overlay/copy).

### Non‑Goals

- Replace VCS or CI systems.
- Store long‑term artifacts or act as a package registry.
- Provide full IDE functionality; instead, expose launch hooks and connection info.

### Architecture Overview

- **API Server (stateless)**: Exposes REST + SSE/WebSocket endpoints to localhost only by default. Optionally persists state in a database.
- **Executors (runners)**: One or many worker processes/hosts that provision workspaces and run agents.
- **Workspace provisioning**: Uses ZFS/Btrfs snapshots when available; falls back to OverlayFS or copy; can orchestrate devcontainers.
- **Transport**: JSON over HTTPS; events over SSE (preferred) and WebSocket (optional).
- **Identity & Access**: API Keys, OIDC/JWT, optional mTLS; project‑scoped RBAC.
- **Observability**: Structured logs, metrics, traces; per‑session logs streaming.

### Core Concepts

- **Task**: A request to run an agent with a prompt and runtime parameters.
- **Session**: The running instance of a task with lifecycle and logs; owns a per‑task workspace.
- **Workspace**: Isolated filesystem mount realized by snapshot provider or copy, optionally inside a devcontainer.
- **Runtime**: The execution environment for the agent (devcontainer/local), plus resource limits.
- **Agent**: The tool/runner performing the coding task.

### Lifecycle States

`queued → provisioning → running → pausing → paused → resuming → stopping → stopped → completed | failed | cancelled`

### Security and Tenancy

- **AuthN**: API Keys, OIDC (Auth0/Keycloak), or JWT bearer tokens.
- **AuthZ (RBAC)**: Roles `admin`, `operator`, `viewer`; resources scoped by `tenantId` and optional `projectId`.
- **Network policy**: Egress restrictions and allowlists per session.
- **Secrets**: Per‑tenant secret stores mounted as env vars/files, never logged.

### API Conventions

- **Base URL**: `/api/v1`
- **Content type**: `application/json; charset=utf-8`
- **Idempotency**: Supported via `Idempotency-Key` header on POSTs.
- **Pagination**: `page`, `perPage` query params; responses include `nextPage` and `total`.
- **Filtering**: Standard filters via query params (e.g., `status`, `agent`, `projectId`).
- **Errors**: Problem+JSON style:

```json
{
  "type": "https://docs.example.com/errors/validation",
  "title": "Invalid request",
  "status": 400,
  "detail": "repo.url must be provided when repo.mode=git",
  "errors": { "repo.url": ["is required"] }
}
```

### Data Model (high‑level)

- **Session**
  - `id` (string, ULID/UUID)
  - `tenantId`, `projectId` (optional)
  - `task`: prompt, attachments, labels
  - `agent`: type, version/config
  - `runtime`: type, devcontainer config reference, resource limits
  - `workspace`: snapshot provider, mount path, host, devcontainer details
  - `vcs`: repo info and delivery policy (PR/branch/patch)
  - `status`, `startedAt`, `endedAt`
  - `links`: SSE stream, logs, IDE/TUI launch helpers

### Endpoints

#### Create Task / Session

- `POST /api/v1/tasks`

Request:

```json
{
  "tenantId": "acme",
  "projectId": "storefront",
  "prompt": "Fix flaky tests in checkout service and improve logging.",
  "repo": {
    "mode": "git",
    "url": "git@github.com:acme/storefront.git",
    "branch": "feature/agent-task",
    "commit": null
  },
  "runtime": {
    "type": "devcontainer",
    "devcontainerPath": ".devcontainer/devcontainer.json",
    "resources": { "cpu": 4, "memoryMiB": 8192 }
  },
  "workspace": {
    "snapshotPreference": ["zfs", "btrfs", "overlay", "copy"],
    "executionHostId": "runner-a"
  },
  "agent": {
    "type": "claude-code",
    "version": "latest",
    "settings": { "maxTokens": 8000 }
  },
  "delivery": {
    "mode": "pr",
    "targetBranch": "main"
  },
  "labels": { "priority": "p2" },
  "webhooks": [
    { "event": "session.completed", "url": "https://hooks.acme.dev/agents" }
  ]
}
```

Response `201 Created`:

```json
{
  "id": "01HVZ6K9T1N8S6M3V3Q3F0X5B7",
  "status": "queued",
  "links": {
    "self": "/api/v1/sessions/01HVZ6K9T1...",
    "events": "/api/v1/sessions/01HVZ6K9T1.../events",
    "logs": "/api/v1/sessions/01HVZ6K9T1.../logs"
  }
}
```

Notes:

- `repo.mode` may also be `upload` (use pre‑signed upload flow) or `none` (operate on previously provisioned workspace template).
- `runtime.type` may be `local` (no container) or `disabled` (explicitly allowed by policy).

#### List Sessions

- `GET /api/v1/sessions?status=running&projectId=storefront&page=1&perPage=20`

Response `200 OK` includes array of sessions and pagination metadata.

#### Get Session

- `GET /api/v1/sessions/{id}` → session details including current status and workspace summary.

#### Stop / Cancel

- `POST /api/v1/sessions/{id}/stop` → graceful stop (agent asked to wrap up).
- `DELETE /api/v1/sessions/{id}` → force terminate and cleanup.

#### Pause / Resume

- `POST /api/v1/sessions/{id}/pause`
- `POST /api/v1/sessions/{id}/resume`

#### Logs and Events

- `GET /api/v1/sessions/{id}/logs?tail=1000` → historical logs.
- `GET /api/v1/sessions/{id}/events` (SSE) → live status, logs, and milestones.

Event payload (SSE `data:` line):

```json
{
  "type": "log",
  "level": "info",
  "message": "Running tests...",
  "ts": "2025-01-01T12:00:00Z"
}
```

#### Event Ingestion (leader → server)

The server does not initiate any connections to followers. Multi‑OS execution (sync‑fence, run‑everywhere) is performed by the leader over SSH (or SSH via rendezvous SOCKS). To keep the UI and automations informed, the leader pushes timeline events to the server.

- `POST /api/v1/sessions/{id}/events/ingest`
  - Purpose: Accepts session timeline events emitted by the leader for observability (and later rebroadcast over the session SSE stream).
  - AuthZ: Same as other session write operations; scoped to the session.
  - Idempotency and ordering: Clients may include `seq` per event and an optional `Event-Sequence` header to enable deduplication and ordered processing.
  - Formats:
    - Batch: `application/json` with a JSON array of events.
    - Streaming: `application/x-ndjson` with one JSON event per line.
  - Schema: see `specs/Public/Schemas/session-events.schema.json` and `specs/Public/Schemas/session-events.ingest.schema.json`.

Accepted event types (minimum set):

```json
{ "type": "followersCatalog", "hosts": [{"name":"win-01","os":"windows","tags":["os=windows"]}] }
{ "type": "fenceStarted",  "snapshotId": "snap-01H...", "ts": "...", "origin": "leader", "transport": "ssh" }
{ "type": "fenceResult",   "snapshotId": "snap-01H...", "hosts": {"win-01": {"state": "consistent", "tookMs": 842}}, "ts": "..." }
{ "type": "hostStarted",    "host": "mac-02", "ts": "..." }
{ "type": "hostLog",        "host": "win-01", "stream": "stdout", "message": "Running tests...", "ts": "..." }
{ "type": "hostExited",     "host": "mac-02", "code": 0, "ts": "..." }
{ "type": "summary",        "passed": ["mac-02","lin-03"], "failed": ["win-01"], "ts": "..." }
{ "type": "note",           "message": "optional annotation", "ts": "..." }
```

#### Workspace and IDE/TUI Launch Helpers

- `GET /api/v1/sessions/{id}/workspace` → mount paths (host/container), snapshot provider, devcontainer info.
- `POST /api/v1/sessions/{id}/open/ide` with body `{ "ide": "vscode" | "cursor" | "windsurf" }`

Response example:

```json
{
  "ide": "vscode",
  "commands": [
    "devcontainer open --workspace-folder /workspaces/agent-01HVZ6K9",
    "code --folder-uri file:///workspaces/agent-01HVZ6K9"
  ],
  "notes": "Run locally on a machine with access to the workspace mount."
}
```

#### Capability Discovery

- `GET /api/v1/agents` → List supported agent types and configurable options.
  - Response:

  ```json
  {
    "items": [
      {
        "type": "openhands",
        "versions": ["latest"],
        "settingsSchemaRef": "/api/v1/schemas/agents/openhands.json"
      },
      {
        "type": "claude-code",
        "versions": ["latest"],
        "settingsSchemaRef": "/api/v1/schemas/agents/claude-code.json"
      }
    ]
  }
  ```

- `GET /api/v1/runtimes` → Available runtime kinds and images/templates.
  - Response:

  ```json
  {
    "items": [
      {
        "type": "devcontainer",
        "images": ["ghcr.io/acme/base:latest"],
        "paths": [".devcontainer/devcontainer.json"]
      },
      { "type": "local", "sandboxProfiles": ["default", "disabled"] }
    ]
  }
  ```

- `GET /api/v1/runners` → Execution runner hosts (terminology aligned with CLI.md).
  - Response entries include: `id`, `os`, `arch`, `snapshotCapabilities` (e.g., `zfs`, `btrfs`, `overlay`, `copy`), and health.

- Optional helper endpoints used by CLI completions and WebUI forms:
  - `GET /api/v1/git/refs?url=<git_url>` → Cached branch/ref suggestions for `--target-branch` UX.
  - `GET /api/v1/projects` → List known projects per tenant for filtering.

#### Followers and Multi‑OS Execution

- `GET /api/v1/followers` → List follower hosts with metadata (os, tags, status). The server’s view is populated from the `followersCatalog` events ingested for active sessions or via configured inventories.

Notes:

- Sync‑fence and run‑everywhere are leader‑executed actions over SSH. They are not exposed as server‑triggered REST methods. The server observes progress via events ingested at `/api/v1/sessions/{id}/events/ingest` and rebroadcasts them on the session SSE stream.

#### Connectivity (Overlay Keys, Handshake, Relay)

Note: The REST service never connects directly to followers. The leader initiates all data‑plane communication to followers over SSH (or SSH routed via the session SOCKS5 rendezvous when overlays/TUNs are unavailable). The server’s role is limited to control‑plane coordination and event ingestion/broadcast.

- `POST /api/v1/connect/keys` → Request session‑scoped connectivity credentials.
  - Body: `{ providers: ["netbird","tailscale"], tags?: [string] }`
  - Response: `{ provider: "netbird"|"tailscale"|"none", credentials?: {...} }`
    - For `netbird`: `{ setupKey: string, reusable: bool, ephemeral: bool, autoGroups: [string] }`
    - For `tailscale`: `{ authKey: string, ephemeral: bool, aclTags: [string] }`

- `POST /api/v1/connect/handshake` → Initiate follower connectivity check.
  - Body: `{ sessionId: string, hosts: [string], timeoutSec?: number }`
  - Response: `{ statuses: { [host: string]: { overlay: "ok"|"fail"|"skip", relay: "ok"|"fail"|"skip", ssh: "ok"|"fail" } } }`

- `POST /api/v1/connect/handshake/ack` → Follower ack upon readiness.
  - Body: `{ sessionId: string, host: string, overlayReady?: bool, relayReady?: bool, sshOk?: bool }`

- Relay endpoints (fallback):
  - `GET /api/v1/relay/{sessionId}/{host}/control` (SSE) — control stream to follower
  - `GET /api/v1/relay/{sessionId}/{host}/stdin` (SSE) — stdin stream to follower (optional)
  - `POST /api/v1/relay/{sessionId}/{host}/stdout`
  - `POST /api/v1/relay/{sessionId}/{host}/stderr`
  - `POST /api/v1/relay/{sessionId}/{host}/status`

#### Session SOCKS5 Rendezvous (fallback)

- `GET /api/v1/connect/socks` → Session‑scoped SOCKS5 front‑end (TCP only). Auth via session token; maps logical hostnames to registered peers.
- `GET /api/v1/connect/socks/register` (WebSocket) → Peer registers local targets.
  - Query: `?sessionId=...&peerId=...&role=leader|follower`
  - On connect, peer sends JSON: `{ "targets": { "ssh": "127.0.0.1:22" } }`
  - Server binds logical names (e.g., `follower-01:22`) to that WS stream.
- SOCKS name resolution: leader’s SSH connects to `follower-01:22`; server forwards over WS to peer’s target (e.g., `127.0.0.1:22`).

Client‑hosted rendezvous: The `aw` client may alternatively host a session‑scoped SOCKS5 front‑end and a WS hub, using the same register protocol. In this mode, peers connect their WS to the client, and SSH/Mutagen use the client’s local SOCKS5.

- `GET /api/v1/agents` → supported agent types and configurable options.
- `GET /api/v1/runtimes` → available runtime images/devcontainers.
- `GET /api/v1/runners` → execution runner hosts and their snapshot capabilities.

#### Uploads (optional flow)

- `POST /api/v1/uploads` → returns pre‑signed URL and upload token; use with `repo.mode=upload`.

#### Health and Metadata

- `GET /api/v1/healthz` → liveness.
- `GET /api/v1/readyz` → readiness.
- `GET /api/v1/version` → server version/build.

### Snapshot and Workspace Behavior

- Implements the snapshot priority described in [FS Snapshots Overview](FS%20Snapshots/FS%20Snapshots%20Overview.md).
- When `runtime.type=devcontainer`, the snapshot is mounted as the container workspace path; otherwise, mounted directly on host.
- On non‑CoW filesystems, OverlayFS or efficient copy (`cp --reflink=auto`) is used; the original working tree remains untouched.

### Delivery Modes

- **PR**: Create PR against `targetBranch`.
- **Branch push**: Push to a designated branch.
- **Patch**: Provide a downloadable patch artifact.

### Authentication Examples

- API Key: `Authorization: ApiKey <token>`
- OIDC/JWT: `Authorization: Bearer <jwt>`

### Rate Limiting and Quotas

- Configurable per tenant/project/user; `429` responses include `Retry-After`.

### Observability

- Metrics: per‑session counts, durations, success rates.
- Tracing: provision → run → delivery spans with session id.

### Versioning and Compatibility

- Semantic API versioning via URL prefix (`/api/v1`).
- OpenAPI spec served at `/api/v1/openapi.json`.

### Deployment Topologies

- Single host: API + runner in one process.
- Scaled cluster: API behind LB; multiple runners with shared DB/queue; shared snapshot‑capable storage or local snapshots per host.

### Security Considerations

- Egress controls; per‑session network policies.
- Least‑privilege mounts; readonly base layers with CoW upper layers.
- Secret redaction in logs/events.

### Example: Minimal Task Creation

```bash
curl -X POST "$BASE/api/v1/tasks" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Refactor build pipeline to reduce flakiness.",
    "repo": {"mode": "git", "url": "git@github.com:acme/storefront.git", "branch": "agent/refactor"},
    "runtime": {"type": "devcontainer"},
    "agent": {"type": "openhands"}
  }'
```

### Alignment with CLI.md (current)

- `aw task` → `POST /api/v1/tasks` (returns `sessionId` usable for polling and SSE).
- `aw session list|get|logs|events` → `GET /api/v1/sessions[/{id}]`, `GET /api/v1/sessions/{id}/logs`, `GET /api/v1/sessions/{id}/events`.
- `aw session run <SESSION_ID> <IDE>` → `POST /api/v1/sessions/{id}/open/ide`.
- `aw remote agents|runtimes|runners` → `GET /api/v1/agents`, `GET /api/v1/runtimes`, `GET /api/v1/runners`.
- `aw agent followers list` → `GET /api/v1/followers` (optional; server view populated from ingested `followersCatalog`).
- `aw agent sync-fence|run-everywhere` → leader‑executed over SSH; server observes via `POST /api/v1/sessions/{id}/events/ingest` and rebroadcasts on session SSE.

SSE event taxonomy for sessions:

```json
{ "type": "status",  "status": "provisioning", "ts": "..." }
{ "type": "log",     "level": "info", "message": "Running tests...", "ts": "..." }
{ "type": "moment",  "snapshotId": "snap-01H...", "note": "post-fence", "ts": "..." }
{ "type": "delivery", "mode": "pr", "url": "https://github.com/.../pull/123", "ts": "..." }
{ "type": "fenceStarted",  "snapshotId": "snap-01H...", "ts": "..." }
{ "type": "fenceResult",   "snapshotId": "snap-01H...", "hosts": {"...": {"state": "consistent", "tookMs": 842}}, "ts": "..." }
{ "type": "hostStarted",   "host": "...", "ts": "..." }
{ "type": "hostLog",       "host": "...", "stream": "stdout", "message": "...", "ts": "..." }
{ "type": "hostExited",    "host": "...", "code": 0, "ts": "..." }
{ "type": "summary",       "passed": ["..."], "failed": ["..."], "ts": "..." }
```

### Implementation and Testing Plan

Planning and status tracking for this spec live in `REST Service.status.md`. That document defines milestones, success criteria, and a precise, automated test plan per specs/AGENTS.md.
