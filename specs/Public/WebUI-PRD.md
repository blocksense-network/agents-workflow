## WebUI — Product Requirements and UI Specification

### Product Summary

The WebUI provides a browser-based experience for creating, monitoring, and managing agent coding sessions backed by the Agents-Workflow REST Service. It targets:

- Engineering teams running on-prem/private cloud clusters.
- Individual developers preferring a graphical dashboard over CLI.

### Goals

- Zero-friction task creation with sensible defaults and policy-aware templates.
- Real-time visibility into active sessions: status, logs, and artifacts.
- One-click launch into preferred IDEs (VS Code, Cursor, Windsurf) pointing at the per-task workspace.
- Governance: tenancy, RBAC, audit trail.

### Non-Goals

- Full web IDE. The WebUI integrates with external IDEs.
- Replacing VCS flows. It assists delivery (PR/branch/patch) but does not host repos.

### User Roles

- Admin: Manage tenants, executors, policies.
- Operator: Create/monitor sessions, manage queues, pause/resume.
- Viewer: Read-only access to sessions and logs.

### Key Use Cases

1. Create a new task with repo, runtime, and agent settings.
2. Watch live logs and events, inspect workspace details.
3. Stop/pause/resume a running session.
4. Launch IDE connected to the workspace.
5. Browse history, filter by status/agent/project, and inspect outcomes (PR/branch/patch).

### IA and Navigation

- Three‑pane layout:
  - Left: Repositories list (projects). Each item has a + button to create a new task for that repo.
  - Center: Chronological task feed across selected repo(s) with live status.
  - Right: Task details pane (opens on click) showing live log or final report/diff.
- Collapsible panes: Repositories and Task feed can be collapsed to maximize space for the details pane.
- Top Nav: Dashboard, Sessions, Create Task, Agents, Runtimes, Hosts, Settings.
- Global search (sessions by id/labels/project).

#### Repositories Pane (Projects)

- Enterprise (server mode):
  - Admin configures a Workspace that contains one or more Projects; each Project has a curated list of repositories eligible for task creation. Repository URLs are specified in Workspace settings.
  - End-user add/remove of repositories is disabled by default (role-gated for admins/operators only).
  - Branch metadata and suggestions are derived via standard git protocol against the configured repository URLs.
- Local mode:
  - The pane auto-populates with repositories previously used in agents-workflow (both through the WebUI and CLI).
  - Add repository: A dedicated “Add repository” control in the pane header opens an input dialog to enter a git URL or select a local path.
  - Remove repository: Two-step confirmation (or long-press context menu) to avoid accidental removal.
  - Distinct icons to avoid confusion with per-repo task creation:
    - Per-repo “New Task” uses a simple plus icon next to the repository row.
    - “Add repository” uses a folder-plus (or link-plus) icon in the pane header with tooltip.
    - If Projects are used locally, “New Project” (optional) uses a collection-plus icon in the header menu.

### Pages and UI Specs

#### Dashboard

- Widgets: Running sessions (by status), Queue depth, Success rate, Recent activity.
- Quick actions: Create Task, View all sessions.

#### Sessions List / Task Feed

- Default view is the chronological task feed (center pane).
- Each task card includes:
  - Status badge and minimal metadata (repo, branch/PR, agent, startedAt, duration).
  - Live-updating single-line last action (e.g., "Running tests (42%)", "Opened PR #123").
  - Quick actions: Stop, Pause/Resume (where allowed).
- Filters: status (queued/provisioning/running/paused/completed/failed), agent type, projectId/repo, label key/values, date range.
- Bulk actions: Stop, Cancel (role-gated).

##### Inline Task Creation (New Task Card)

- Trigger: Clicking the + button next to a repository in the left pane inserts a new task card at the top of the center feed.
- Description input: Vertically resizable textarea with placeholder guidance; supports markdown; auto-saves to a draft immediately on change.
- Branch selector: Combo-box prepopulated with the repo’s default branch for task creation (e.g., `main`). Allows switching to any available branch; includes search.
- Branch selector: Combo-box prepopulated with the repo’s default branch for task creation (e.g., `main`). Allows switching to any available branch; includes search and live autocomplete.
  - Local mode: Suggestions are sourced directly from the filesystem repo using standard git commands (e.g., `git for-each-ref`), cached in-memory per repo with debounce refresh.
  - Server mode: Suggestions come from the REST service’s in-memory branch cache populated via the standard git protocol (e.g., `git ls-remote`/refs fetch) against the admin-configured repository URL; the UI queries `/api/v1/repos/{id}/branches?query=<prefix>&limit=<n>`.
- Agent selector: Dropdown to choose agent type/version; prefilled from last used defaults or repo policy; validation against `/api/v1/agents` capabilities.
- Concurrency: Numeric selector (where supported) for number of concurrent instances; disabled if agent does not support concurrency; show limits.
- Actions:
  - Right-aligned Start button: Validates inputs and creates the task via `POST /api/v1/tasks`; upon success, card transitions to running state with live status line.
  - Draft delete: Button to remove the draft card (with confirm). Multiple drafts across repos are supported and preserved between reloads.
- Draft behavior: Drafts are stored locally (and optionally server-side when authenticated); restored on reload; invalid fields highlighted; Start disabled until required fields are valid.

#### Session Details (Right Pane)

- Header: id, status badge, repo, agent, startedAt/endedAt, duration, owner.
- Tabs:
  - Overview: prompt, delivery mode, repo info, runtime and workspace summary, labels.
  - Live Log: real-time stream with tail controls, level filter, copy/download; auto-scroll with pause.
  - Events: SSE timeline (provisioned, tests passed, PR opened, etc.).
  - Report: final summary and created diff/patch with download and PR links.
  - Workspace: mount paths, snapshot provider, IDE launch helpers.
- Actions: Stop, Pause/Resume, Cancel; Open IDE (VS Code, Cursor, Windsurf).

#### Create Task Wizard

- Step 1: Prompt
  - Textarea with tips, optional prompt file upload.
  - Labels editor (key/value chips).
- Step 2: Repository
  - Modes: Git URL/branch/commit; Upload; None (template workspace).
  - Validation and repo reachability check.
- Step 3: Runtime
  - Type: devcontainer (path selector), local, disabled (policy-guarded).
  - Resources: CPU, Memory; Time limit; Egress policy.
- Step 4: Agent
  - Agent type/version; settings as schema-driven form.
- Step 5: Delivery
  - PR (target branch), Branch push, Patch artifact.
- Step 6: Review & Create
  - Summary, JSON preview, Idempotency key (auto-generated), Create button.

#### Agents Catalog

- List of supported agents with descriptions and configurable defaults.
- Detail page shows schema for settings and compatibility notes.

#### Runtimes

- List devcontainers and local runtime templates; show available resources.

#### Runners

- Show registered execution hosts, snapshot capabilities (zfs/btrfs/overlay/copy), capacity, and health.

#### Settings

- Tenant config (RBAC, quotas, API keys), IDE integration hints, webhook destinations.

### Real-Time Behavior

- Use SSE to subscribe to `/api/v1/sessions/{id}/events` for status/log updates.
- Reconnect with exponential backoff; buffer events during network blips.

### IDE Launch Integration

- Call `POST /api/v1/sessions/{id}/open/ide` and display returned commands.
- Provide copy-to-clipboard and "Try locally" hints.

TODO: This has to be re-thinked. How does it work in [Local-Mode](Local-Mode.md)? How does it work with a [Remote Server](Remote-Mode.md)? VS Code and Cursor have remote mode, accessible over the web, but we need to create a secure tunnel for this.

### Empty States and Errors

- Helpful guidance for no sessions, no hosts, or missing permissions.
- Problem+JSON errors rendered with field-level highlights.

### Accessibility and i18n

- WCAG AA color contrast; keyboard navigation; ARIA landmarks.
- Strings externalized for localization; LTR/RTL aware layouts.

### Telemetry and Audit

- Client events (navigation/actions) batched and sent to server metrics endpoint.
- Audit trail ties UI actions to user identity and session ids.

### Performance Targets

- TTI < 2s on 3G Fast; live log latency < 300ms; lists virtualized beyond 200 rows.

### Tech Notes (non-binding)

- SPA built with React + Vite + TypeScript (non‑binding default), SSE for events, OpenAPI client for REST.
- State normalized by session id; optimistic UI for pause/stop/resume.

### Local Mode (--local)

- Purpose: Provide a zero-setup, single-developer experience. The WebUI binds to `127.0.0.1` only and targets a locally running Agents-Workflow REST service.
- Invocation: `webui --local [--port <port>]` (command name illustrative). In this mode:
  - Network binding: HTTP server listens on localhost only.
  - Auth and tenancy: No RBAC/tenants; implicit single user. Admin pages are hidden (Agents/Runtimes/Hosts/Settings for multi-tenant ops).
  - Config discovery: API base URL resolved from config key `service-base-url` or env `AGENTS_WORKFLOW_SERVICE_BASE_URL`; otherwise defaults to `http://127.0.0.1:<default-port>`.
  - Intention: `service-base-url` is primarily for deployments hosting a persistent WebUI at a fixed URL (e.g., behind an ingress/LB) so the browser knows which REST origin to call. In local development, leave it unset and the WebUI will target localhost by default.
  - IDE integration: Unchanged; IDE launch helpers assume local filesystem access to the workspace mount.
  - Persistence: Uses browser local storage for UI preferences. No external DB required.
  - Security: No TLS in local mode; not intended for remote access.
- Service reachability:
  - If the local REST service is unreachable, show a blocking banner with retry and guidance (e.g., “Start the service, then retry”).
  - Optionally offer a copyable command to start the local service.
- Feature differences vs full mode:
  - Hidden sections: Agents, Runtimes, Hosts, multi-tenant Settings.
  - Sessions, Create Task, and basic Settings (local) remain.
  - Delivery flows (PR/branch/patch) are available; features gated by what the local service advertises via `/api/v1/*` capability endpoints.
