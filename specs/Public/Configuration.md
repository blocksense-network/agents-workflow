## AH Configuration

### Overview

- `ah config` subcommand with Git-like interface for reading and updating configuration.
- Schema validation on both config file loading and CLI-based modification.
- Precedence for `~/.config` over `%APPDATA%` on Windows only when both are present.
- Motivation and support for tracking the origin of each configuration value, with use cases such as: debug-level log reporting, enforced setting explanation, and editor pre-fill mes
  sages.

Layered configuration supports system, user, repo, and repo-user scopes. Values can also be supplied via environment variables and CLI flags. See [CLI](CLI.md) for flag mappings.

### Locations (by scope)

- System (OS‑level):
  - Linux: `/etc/agent-harbor/config.toml`
  - macOS: `/Library/Application Support/agent-harbor/config.toml`
  - Windows: `%ProgramData%/agent-harbor/config.toml`
- User:
  - Linux: `$XDG_CONFIG_HOME/agent-harbor/config.toml` or `$HOME/.config/agent-harbor/config.toml`
  - macOS: `$HOME/Library/Application Support/agent-harbor/config.toml`
  - Windows: `%APPDATA%/agent-harbor/config.toml` (precedence is given to `~/.config` when both exist as noted below)
  - Custom (when `AH_HOME` is set): `$AH_HOME/config.toml`
- Repo: `<repo>/.agents/config.toml`
- Repo‑user: `<repo>/.agents/config.user.toml` (ignored by VCS; add to `.gitignore`)

Paths are illustrative; the CLI prints the exact search order in `ah config --explain` and logs them at debug level.

The `AH_HOME` environment variable can override the default user configuration and data directory locations. When set, it changes the user configuration file to `$AH_HOME/config.toml` and the local SQLite database to `$AH_HOME/state.db` (see [State-Persistence.md](State-Persistence.md)).

### Admin‑enforced values

Enterprise deployments may enforce specific keys at the System scope. Enforced values are read‑only to lower scopes. The CLI surfaces enforcement in `ah config <key> --explain` output and prevents writes with a clear error. See the initial rationale in [Configuration](../Initial-Developer-Input/Configuration.md).

Use a single key `ui` (not `ui.default`) to control the default UI.

### Mapping Rules (Flags ↔ Config ↔ ENV/JSON)

To keep things mechanical and predictable:

- TOML sections correspond to subcommand groups (e.g., `[repo]` for `ah repo ...`).
- CLI option keys preserve dashes in TOML (e.g., `default-mode`, `task-runner`). The name of the options should be chosen to read well both on the command-line and inside a configuration file.
- There are options that are available only within configuration files (e.g. `[[fleet]]` as described below).
- JSON and environment variables replace dashes with underscores. ENV vars keep the `AH_` prefix.

Examples:

- Flag `--remote-server` ↔ TOML `remote-server` ↔ ENV `AH_REMOTE_SERVER`
- Per-server URLs are defined under `[[server]]` entries; `remote-server` may refer to a server `name` or be a raw URL.
- WebUI-only: key `service-base-url` selects the REST base URL used by the browser client when the WebUI is hosted persistently at a fixed origin.
- Flag `--task-runner` ↔ TOML `repo.task-runner` ↔ ENV `AH_REPO_TASK_RUNNER`

### Keys

- `ui`: string — default UI to launch with bare `ah` (values: `"tui"` | `"webui"`).
- `browser-automation`: `boolean` — enable/disable site automation.
- `browser-profile`: string — preferred agent browser profile name.
- `chatgpt-username`: string — optional default ChatGPT username used for profile discovery.
- `codex-workspace`: string — default Codex workspace to select before pressing "Code".
- `remote-server`: string — either a known server `name` (from `[[server]]`) or a raw URL. If set, AH uses REST; otherwise it uses local SQLite state.
- `tui-font-style`: string — TUI symbol style (values: `"nerdfont"` | `"unicode"` | `"ascii"`). Auto-detected based on terminal capabilities.
- `tui-font`: string — TUI font name for advanced terminal font customization.

### Behavior

- CLI flags override environment, which override repo-user, repo, user, then system scope.
- On Windows, `~/.config` takes precedence over `%APPDATA%` only when both are present.
- The CLI can read, write, and explain config values via `ah config`.
- Backend selection: if `remote-server` is set (by flag/env/config), AH uses the REST API; otherwise it uses the local SQLite database.
- Repo detection: when `--repo` is not specified, AH walks parent directories to find a VCS root among supported systems; commands requiring a repo fail with a clear error when none is found.

### Validation

- The configuration file format is TOML, validated against a single holistic JSON Schema:
  - Schema: `specs/schemas/config.schema.json` (draft 2020-12)
  - Method: parse TOML → convert to a JSON data model → validate against the schema
  - Editors: tools like Taplo can use the JSON Schema to provide completions and diagnostics

- DRY definitions: the schema uses `$defs` for shared `enums` and shapes reused across the CLI (e.g., `Mode`, `Multiplexer`, `Vcs`, `DevEnv`, `TaskRunner`, `AgentName`, `SupportedAgents`).

Tools in the dev shell:

- `taplo` (taplo-cli): TOML validation with JSON Schema mapping
- `ajv` (ajv-cli): JSON Schema `validator` for JSON instances
- `docson` (via shell function): local schema viewer using `npx` (no global install)

Examples (use Just targets inside the Nix dev shell):

```bash
# Validate all JSON Schemas (meta-schema compile)
just conf-schema-validate

# Check TOML files with Taplo
just conf-schema-taplo-check

# Preview the schemas with Docson (serves http://localhost:3000)
just conf-schema-docs
```

Tip: from the host without entering the shell explicitly, you can run any target via:

```bash
nix develop --command just conf-schema-validate
```

### Servers, Fleets, and Sandboxes

AH supports declaring remote servers, fleets (multi-environment presets), and sandbox profiles.

```toml
remote-server = "office-1"  # optional; can be a name from [[server]] or a raw URL

[[server]]
name = "office-1"
url  = "https:/ah.office-1.corp/api"

[[server]]
name = "office-2"
url  = "https://ah.office-2.corp/api"

# Fleets define a combination of local testing strategies and remote servers
# to be used as presets in multi-OS or multi-environment tasks.

[[fleet]]
name = "default"  # chosen when no other fleet is provided

  [[fleet.member]]
  type = "container"   # refers to a sandbox profile by name (see [[sandbox]] below)
  profile = "container"

  [[fleet.member]]
  type = "remote"      # special value; not a sandbox profile
  url  = "https://ah.office-1.corp/api"  # or `server = "office-1"`

[[sandbox]]
name = "container"
type = "container"      # predefined types with their own options

# Examples (type-specific options are illustrative and optional):
# [sandbox.options]
# engine = "docker"           # docker|podman
# image  = "ghcr.io/ah/agents-base:latest"
# user   = "1000:1000"        # uid:gid inside the container
# network = "isolated"         # bridge|host|none|isolated
```

Flags and mapping:

- `--remote-server <NAME|URL>` selects a server (overrides `remote-server` in config).
- `--fleet <NAME>` selects a fleet; default is the fleet named `default`.
- Bare `ah` uses `ui` to decide between TUI and WebUI (defaults to `tui`).

### Filesystem Snapshots

Control snapshotting and working‑copy strategy. Defaults are `auto`.

TOML keys (top‑level):

```toml
fs-snapshots = "auto"        # auto|zfs|btrfs|agentfs|git|disable
working-copy = "auto"        # auto|cow-overlay|worktree|in-place

# Provider‑specific (optional; may be organized under a [snapshots] section in the future)
# git.includeUntracked = false
# git.worktreesDir = "/var/tmp/ah-worktrees"
# git.shadowRepoDir = "/var/cache/ah/shadow-repos"
```

Flag and ENV mapping:

- Flags: `--fs-snapshots`, `--working-copy`
- ENV: `AH_FS_SNAPSHOTS`, `AH_WORKING_COPY`

Behavior:

- `auto` selects the highest‑score provider for the repo and platform. Users can pin to `git` (or any provider) even if CoW is available.
- `cow-overlay` requests isolation at the original repo path (Linux: namespaces/binds; macOS/Windows: AgentFS). When impossible, the system falls back to `worktree` with a diagnostic.
- `in-place` runs the agent directly on the original working copy. Isolation is disabled, but FsSnapshots may still be available when the chosen provider supports in‑place capture (e.g., Git shadow commits, ZFS/Btrfs snapshots). Use `fs-snapshots = "disable"` to turn snapshots off entirely.

### Example TOML (partial)

```toml
log-level = "info"

terminal-multiplexer = "tmux"

editor = "nvim"

service-base-url = "https://ah.office-1.corp/api"  # WebUI fetch base; browser calls this URL

# Browser automation (no subcommand section; single keys match CLI flags)
browser-automation = true
browser-profile = "work-codex"
chatgpt-username = "alice@example.com"

# Codex workspace (single key)
codex-workspace = "main"

[repo]
supported-agents = "all" # or ["codex","claude","cursor"]

  [repo.init]
  vcs = "git"
  devenv = "nix"
  devcontainer = true
  direnv = true
  task-runner = "just"
```

Notes:

- `supportedAgents` accepts "all" or an explicit array of agent names; the CLI may normalize this value internally.
- `devenv` accepts values like `nix`, `spack`, `bazel`, `none`/`no`, or `custom`.

ENV examples:

```
AH_REMOTE_SERVER=office-1
AH_REPO_SUPPORTED_AGENTS=all
```
