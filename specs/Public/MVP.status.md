### Overview

Goal: Deliver a minimum viable product (MVP) version of the Agents-Workflow CLI that provides core functionality for Linux users, focusing on agent time-travel capabilities with ZFS snapshots, Claude Code and Codex integration, and local mode operation. The MVP will serve as a foundation for subsequent cross-platform expansion and feature additions.

### MVP Feature Set

The initial MVP focuses on these core capabilities:

- **Linux-only platform support** with ZFS filesystem snapshots
- **Local Mode** with SQLite state management for single-developer workflows
- **Claude Code and Codex agents** as the primary supported agent types
- **Agent Time Travel** with session recording, timeline navigation, and branching
- **Basic CLI commands** for task creation, session management, and time-travel operations
- **Repository reorganization** according to the Rust migration layout
- **Rust reimplementation** of FS snapshots daemon and test suite

### Approach

- **Repository Reorganization**: Restructure the codebase according to `Repository Layout.md` before implementing new features, ensuring existing codex-setup tests remain functional through path adjustments.
- **Incremental Rust Implementation**: Start with core Rust crates for local mode, ZFS snapshots, and Claude Code integration, building toward the full CLI surface area.
- **Agent Time Travel Foundation**: Implement session recording with Claude Code hooks, transcript trimming, and ZFS-based filesystem snapshots as the cornerstone feature.
- **Strong Test Coverage**: Prioritize integration tests that validate end-to-end workflows, especially time-travel branching and session resumption.
- **Documentation Parity**: Ensure CLI help text and documentation remain synchronized through automated snapshot testing.

### Milestones (automated verification)

**M0. Repository Reorganization & Bootstrap (1-2 weeks)**

- Deliverables:
  - Reorganize repository structure according to `Repository Layout.md`
  - Move existing Ruby code to `legacy/ruby/` preserving all functionality
  - Ensure `test-codex-setup-integration` tests pass with path adjustments only
  - Create initial Rust workspace with core crate structure
  - Basic CI pipeline for Rust crates
  - **Rust FS snapshots daemon reimplementation** with Unix socket protocol and privileged operations
  - **Port FS snapshots tests to Rust** with `agentfs-zfs` crate providing similar API to Ruby code

- **FS Snapshots Rust Port Details**:
  - **Daemon Reimplementation**: Port the Ruby `aw-fs-snapshots-daemon` (Unix socket server) to Rust with tokio async runtime. Maintain JSON protocol for clone/snapshot/delete operations on ZFS/Btrfs. Support privileged operations via sudo in development, container-based privilege escalation in production.
  - **Provider API Port**: Create `agentfs-core` crate with `FsSnapshotProvider` trait matching Ruby `Snapshot::Provider` API. Implement `create_workspace(dest)` and `cleanup_workspace(dest)` methods for isolated agent execution environments.
  - **Test Suite Port**: Port Ruby test suite (`test/snapshot/`) to Rust integration tests. Recreate loopback filesystem creation, provider behavior testing, and concurrent execution safety. Use `rstest` for parameterized tests and golden file snapshots for verification.
  - **ZFS Provider**: Implement `agentfs-zfs` crate with dataset detection, snapshot creation, clone mounting, and cleanup. Handle daemon communication for privileged operations.
  - **Test Infrastructure**: Port filesystem test helpers (`filesystem_test_helper.rb`) to Rust. Support loopback ZFS/Btrfs pool creation and mounting for CI environments.

- Verification:
  - `just legacy-tests` passes for existing Ruby components (renamed from `just test`)
  - `cargo check` succeeds for new Rust workspace
  - `test-codex-setup-integration` passes without functional changes
  - Rust daemon handles ZFS clone/snapshot/delete operations correctly
  - Ported FS snapshot tests pass with same coverage as Ruby tests

**M1. Core Local Mode & ZFS Snapshots (2-3 weeks)**

- Deliverables:
  - `aw-core` crate with local SQLite state management
  - ZFS snapshot provider implementation (`agentfs-zfs` crate)
  - Basic workspace preparation and cleanup
  - Local mode task creation and session tracking
  - `aw task` and `aw session list/logs` CLI commands

- Verification:
  - ZFS snapshot creation/cleanup works on test systems
  - SQLite database operations tested with property-based testing
  - Integration tests with temporary Git repos and ZFS volumes
  - `aw task` creates proper branch/task file structure locally

**M2. Claude Code & Codex Integration & Session Recording (4-5 weeks)**

- Deliverables:
  - Claude Code agent wrapper with hook-based session recording (PostToolUse events)
  - Codex agent wrapper with rollout file parsing (JSONL format from `Codex-Session-File-Format.md`)
  - `aw-agent-runner` binary for asciinema recording of both agent types
  - Session timeline creation with SessionMoments for both agents
  - Basic session resumption via `--resume` flag for both agents
  - Codex rollout file parsing and trimming for time travel

- Verification:
  - Claude Code hooks emit SessionMoments at tool boundaries
  - Codex rollout files parsed correctly from `~/.codex/sessions/` directory structure
  - Session recordings captured and stored in SQLite for both agents
  - Transcript/rollout path detection and session ID mapping
  - Both agents resume from interrupted sessions correctly
  - Codex rollout files can be trimmed to specific moments for time travel

**M3. Agent Time Travel Foundation (4-5 weeks)**

- Deliverables:
  - Session timeline navigation and seeking
  - Read-only snapshot mounting for inspection
  - Session branching with injected messages
  - Claude Code transcript trimming for precise time travel
  - `aw session seek` and `aw session branch` commands

- Verification:
  - Timeline navigation shows correct SessionMoments/FsSnapshots
  - ZFS snapshots mount read-only at specific timestamps
  - Transcript trimming preserves conversation up to target moment
  - Branched sessions start Claude Code with trimmed context
  - End-to-end time travel: seek → inspect → branch → resume

**M4. Local Sandboxing Integration (6-8 weeks)**

- Deliverables:
  - Complete Linux sandboxing implementation (see `Local Sandboxing on Linux.status.md`)
  - Dynamic read allow-list with seccomp notify
  - Resource limits and audit logging
  - `aw session audit` command integration
  - Sandboxed agent execution with time travel

- Verification:
  - All sandbox milestones from `Local Sandboxing on Linux.status.md`
  - Agents run in isolated namespaces with proper resource limits
  - Audit logs capture file access decisions and sandbox events
  - Time travel works within sandboxed environments

**M5. TUI Dashboard (4-6 weeks)**

- Deliverables:
  - Ratatui-based TUI implementation following `TUI PRD.md`
  - Project/Branch/Agent selectors with filtering
  - Task description editor and launch workflow
  - Time travel timeline viewer and controls
  - Multiplexer integration (tmux/zellij/screen)

- Verification:
  - TUI launches and auto-attaches to multiplexer sessions
  - All keyboard navigation and hotkeys work as specified
  - Time travel scrubbing shows correct terminal playback
  - Task launch creates proper multiplexer windows
  - Footer shows context-appropriate shortcuts

**M6. MVP Polish & Documentation (2-3 weeks)**

- Deliverables:
  - Complete CLI command surface for MVP features
  - Man pages and shell completions
  - User documentation and examples
  - Performance optimization and error handling
  - Release packaging for Linux

- Verification:
  - All MVP commands work end-to-end
  - Generated help/man pages match spec documentation
  - Performance benchmarks meet targets (snapshot creation <1s)
  - Error messages are clear and actionable

### Test & QA strategy

- **MVP-Focused Testing**: Prioritize end-to-end integration tests that validate complete user workflows (task creation → agent execution → time travel → branching) over comprehensive unit test coverage in early milestones.
- **ZFS Integration Testing**: Use loopback ZFS pools in CI for snapshot testing; provide developer setup scripts for local ZFS testing environments.
- **Agent Mock Testing**: Develop mock Claude Code and Codex servers for deterministic testing of hook-based session recording and transcript/rollout trimming without external API dependencies.
- **Time Travel E2E Tests**: Automated tests that create sessions, seek to specific moments, create branches, and verify resumed agents have correct context.
- **Snapshot Testing**: Use `cargo insta` for CLI help text and generated documentation to ensure spec parity.
- **CI Pipeline**: Maintain separate pipelines for `just legacy-tests` (Ruby), Rust MVP development, and integration tests requiring ZFS/sandboxes. Ensure `test-codex-setup-integration` continues to pass during reorganization.

### Risks & mitigations

- **ZFS Dependency**: Mitigated by providing alternative Git-based snapshot fallback in development; ZFS becomes optional for basic functionality but required for full time-travel features.
- **Agent Evolution**: Mitigated by comprehensive hook testing and version compatibility checks for both Claude Code and Codex; maintain fallback to basic session resumption if hooks/API change.
- **Codex Rollout Complexity**: Mitigated by thorough testing of JSONL parsing and trimming logic; the rollout file format specification provides clear parsing rules to follow.
- **Repository Reorganization**: Mitigated by preserving all existing functionality in `legacy/` during transition; `test-codex-setup-integration` tests must pass unchanged.
- **Complex Time Travel Logic**: Mitigated by building extensive integration tests from day one; both transcript and rollout trimming logic will be thoroughly tested with synthetic session files.
- **Sandbox Complexity**: Mitigated by following the detailed milestone plan in `Local Sandboxing on Linux.status.md`; each component tested in isolation before integration.
