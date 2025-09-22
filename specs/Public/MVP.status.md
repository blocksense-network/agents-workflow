### Overview

Goal: Deliver a minimum viable product (MVP) version of the Agents-Workflow CLI that provides core functionality for Linux users, focusing on agent time-travel capabilities with ZFS snapshots, Claude Code and Codex integration, and local mode operation. The MVP will serve as a foundation for subsequent cross-platform expansion and feature additions.

Total estimated timeline: 12-16 months (broken into major phases with parallel development tracks)

### Milestone Completion & Outstanding Tasks

Each milestone maintains an **outstanding tasks list** that tracks specific deliverables, bugs, and improvements. When milestones are completed, their sections are expanded with:

- Implementation details and architectural decisions
- References to key source files for diving into the implementation
- Test coverage reports and known limitations
- Integration points with other milestones/tracks

### MVP Feature Set

The initial MVP focuses on these core capabilities:

- **Linux-only platform support** with ZFS filesystem snapshots
- **Local Mode** with SQLite state management for single-developer workflows:
  - Task/session lifecycle orchestration in `aw-core` crate
  - Filesystem snapshot management in `aw-fs-snapshots` crate hierarchy
  - Sandbox isolation using `aw-sandbox` crate hierarchy
- **Claude Code and Codex agents** as the primary supported agent types
- **Agent Time Travel** with session recording, timeline navigation, and branching
- **Basic CLI commands** for task creation, session management, and time-travel operations
- **Repository reorganization** according to the Rust migration layout
- **Rust reimplementation** of FS snapshots daemon and test suite

### Parallel Development After Bootstrapping

Once the Rust workspace bootstrap (M0.2) and core infrastructure (M0.3-M0.6) are complete, multiple development tracks can proceed in parallel:

- **FS Snapshots Track**: Complete ZFS/Btrfs snapshot providers and test suite (continues from M0.5-M0.6)
- **CLI Core Track**: Implement CLI commands with dry-run behavior validation (starts after M0.2, can proceed in parallel with other tracks)
- **Database Track**: Build `aw-local-db` crate with comprehensive unit tests (starts after M0.2, proceeds in parallel with FS snapshots)
- **TUI Track**: Develop TUI user journey with mocked agents for comprehensive UX testing (starts after M1 core completion, uses existing [TUI PRD](TUI-PRD.md) specifications)
- **WebUI Track**: Implement WebUI user journey with mocked agents for end-to-end validation (starts after M1 core completion, uses existing [WebUI PRD](WebUI-PRD.md) specifications)
- **Desktop Notifications Track**: Build cross-platform notification library per [Handling-AW-URL-Scheme.md](Handling-AW-URL-Scheme.md) specifications (starts after M0.2, enables URL scheme handling)

### Approach

- **Repository Reorganization**: Restructure the codebase according to [Repository-Layout.md](Repository-Layout.md) before implementing new features, ensuring existing codex-setup tests remain functional through path adjustments.
- **Subcrates Pattern**: Apply the [subcrates design pattern](Subcrates-Pattern.md) for modular crate organization, following the monolith + facades approach.
- **Incremental Rust Implementation**: Start with core Rust crates for local mode, ZFS snapshots, and Claude Code integration, building toward the full CLI surface area.
- **Reference Existing Ruby Code**: Use the existing Ruby implementation (`lib/`, `bin/`, `test/`) as reference for API design, behavior validation, and test patterns during Rust reimplementation.
- **Agent Time Travel Foundation**: Implement session recording with Claude Code hooks, transcript trimming, and ZFS-based filesystem snapshots as the cornerstone feature.
- **Strong Test Coverage**: Prioritize integration tests that validate end-to-end workflows, especially time-travel branching and session resumption.
- **Documentation Parity**: Ensure CLI help text and documentation remain synchronized through automated snapshot testing.

### Development Phases (with Parallel Tracks)

**Phase 0: Infrastructure Bootstrap** (2-3 weeks total, with parallel infrastructure tracks)

**0.1 Repository Structure Reorganization** (3-4 days)

- Deliverables:
  - Reorganize repository structure according to [Repository-Layout.md](Repository-Layout.md)
  - Move existing Ruby code to `legacy/ruby/` preserving all functionality
  - Update all import paths and references in moved files
  - Create basic Rust workspace directory structure (`crates/`, `bins/`, etc.)
  - Rename all existing just targets to have a `legacy-` prefix.

- Verification:
  - All Ruby files can be found at their new `legacy/ruby/` locations
  - `just legacy-test` passes completely with no path-related failures
  - `just legacy-test-codex-setup-integration` passes with Docker containers finding correct paths
  - `find . -name "*.rb" | grep -v legacy/ | wc -l` returns 0 (no Ruby files in root) (this is a manual test)

**0.2 Rust Workspace & Core Crates Bootstrap** (2-3 days)

- Deliverables:
  - Create initial `Cargo.toml` workspace configuration
  - Implement `aw-core` crate skeleton with task/session lifecycle orchestration
  - Set up `aw-local-db` crate skeleton for SQLite database management
  - Set up `aw-fs-snapshots` crate with snapshot provider abstractions
  - Create `aw-fs-snapshots-zfs` and `aw-fs-snapshots-btrfs` sub-crates
  - Set up `aw-sandbox` crate following [subcrates pattern](Subcrates-Pattern.md):
    - Core sandbox API with namespace orchestration and lifecycle management
    - Create `aw-sandbox-linux` sub-crate for Linux-specific implementations
    - Placeholder sub-crates for future platforms (macOS, Windows)
  - Configure basic CI pipeline (GitHub Actions) for Rust crates
  - Add essential dependencies: tokio, serde, clap, rusqlite, etc.

- Verification:
  - `cargo check --workspace` (`just check`) succeeds for all crates
  - `cargo test --workspace` (`just test`) runs (may have empty test suites)
  - CI pipeline runs successfully on push/PR
  - Workspace structure matches [Repository-Layout.md](Repository-Layout.md)

**0.3 Privileged FS Operations Daemon** IN PROGRESS (5 days)

- **Deliverables**:
  - Rust daemon binary (`bins/aw-fs-snapshots-daemon`) with Unix socket server (the implementation should operate similarly to the reference implementation `bin/aw-fs-snapshots-daemon` which should be moved to the legacy/ruby folder; The new implementation should be made production-ready)
  - Length-prefixed SSZ marshaling format for communication (see [Using-SSZ.md](../../Research/Using-SSZ.md) for implementation reference)
  - Basic ZFS operations (snapshot, clone, delete) with sudo privilege escalation
  - Async tokio runtime for concurrent request handling
  - Tracing library for structured logging
  - Proper signal handling and cleanup
  - **Daemon should not assume presence of any ZFS datasets or subvolumes** - all filesystem operations should be validated dynamically
  - **Stdin-driven mode**: daemon should provide option to accept SSZ-encoded commands from stdin as alternative to Unix socket communication

- **Implementation Details**:
  - Created new Rust crate `aw-fs-snapshots-daemon` with async Tokio-based Unix socket server
  - Implemented length-prefixed JSON marshaling (temporary solution until proper SSZ support is available)
  - Added comprehensive ZFS and Btrfs operations (snapshot, clone, delete) with sudo privilege escalation
  - Dynamic validation ensures ZFS datasets/snapshots and Btrfs subvolumes exist before operations
  - Proper signal handling (SIGINT/SIGTERM) with graceful shutdown and socket cleanup
  - Stdin-driven mode for testing and integration with existing scripts
  - Structured logging with tracing library for production monitoring
  - Concurrent request handling with async/await patterns

- **Key Source Files**:
  - `crates/aw-fs-snapshots-daemon/src/main.rs` - Main binary entry point
  - `crates/aw-fs-snapshots-daemon/src/server.rs` - Unix socket server implementation
  - `crates/aw-fs-snapshots-daemon/src/operations.rs` - Filesystem operations with validation
  - `crates/aw-fs-snapshots-daemon/src/types.rs` - Request/response type definitions
  - `Justfile` - Added `start-aw-fs-snapshots-daemon`, `stop-aw-fs-snapshots-daemon`, `check-aw-fs-snapshots-daemon` targets

- **Outstanding Tasks**:
  - Implement proper SSZ marshaling format (currently uses length-prefixed JSON)
  - Fix SSZ derive macros compatibility issues
  - Add comprehensive Btrfs snapshot testing (currently ZFS-focused)
  - Consider alternatives to sudo requirement for privileged operations

- **Verification Results**:
  - [x] Daemon starts and listens on Unix socket at expected path
  - [ ] Length-prefixed SSZ ping request returns success response via Unix socket
  - [ ] Length-prefixed SSZ ping request returns success response via stdin mode
  - [ ] Daemon handles invalid SSZ data gracefully with error responses
  - [x] Daemon shuts down cleanly on SIGINT/SIGTERM
  - [ ] Integration test: daemon processes basic ZFS snapshot request using file-backed test filesystems (see `scripts/create-test-filesystems.sh`, `scripts/check-test-filesystems.sh`)
  - [ ] Legacy tests still pass, using the legacy daemon from its new location

**0.4 FS Snapshots Core API** (3-4 days, parallel with 0.3, 0.5-0.6)

- Deliverables:
  - Complete `aw-fs-snapshots` crate with `FsSnapshotProvider` trait
  - `create_workspace(dest)` and `cleanup_workspace(dest)` method implementations
  - Provider auto-detection logic (`provider_for(path)`)
  - Basic error handling and path validation
  - Integration with daemon for privileged operations

- Verification:
  - Provider trait compiles and can be implemented by concrete providers
  - Auto-detection returns correct provider for ZFS/Btrfs paths
  - Path validation rejects invalid destinations (system directories, etc.)
  - Unit tests for provider selection logic pass

**0.5 ZFS Snapshot Provider** (4-5 days, parallel with 0.3-0.4, 0.6)

- Deliverables:
  - Complete `aw-fs-snapshots-zfs` crate with ZFS dataset operations
  - Dataset detection and mount point resolution
  - Snapshot creation, clone mounting, and cleanup
  - Daemon communication for privileged ZFS commands
  - Error handling for missing datasets, permissions, etc.
- Verification:
  - ZFS provider detects available ZFS datasets correctly
  - `create_workspace()` creates valid symlinks to ZFS clone mount points
  - `cleanup_workspace()` destroys ZFS clones and removes symlinks
  - Integration test: full workspace lifecycle on test ZFS pool
  - Error cases handled: missing datasets, permission denied, etc.

**0.6 FS Snapshots Test Infrastructure** (4-5 days, parallel with 0.3-0.5)

- Deliverables:
  - Port filesystem test helpers (`filesystem_test_helper.rb`) to Rust
  - Loopback ZFS pool creation for CI/testing environments
  - Port provider behavior tests (`provider_shared_behavior.rb`)
  - Port quota and performance tests to Rust equivalents
  - Integration tests using `rstest` and golden file snapshots
  - **Reference existing Ruby test suite** (`test/snapshot/`) for test patterns and edge cases

- Verification:
  - Rust test suite creates and manages test ZFS pools automatically
  - All provider behaviors (shared, quota, performance) ported and passing
  - Concurrent execution tests pass without race conditions
  - Golden file snapshots match expected outputs
  - Test coverage equivalent to original Ruby test suite

**Phase 1: Core Functionality** (2-3 weeks total, with parallel implementation tracks)

**1.1 Local Mode & Database Management** (2-3 weeks, with parallel CLI/Database tracks)

- Deliverables:
  - Create `aw-local-db` crate for SQLite database management:
    - SQLite schema definitions and models (tasks, sessions, agent recordings, etc.)
    - Database connection management and pooling
    - Schema migration system with version tracking
    - CRUD operations for all entities with proper error handling
    - Unit tests for database operations and migrations
  - Complete `aw-core` crate with task/session lifecycle orchestration:
    - Task creation, execution tracking, and completion handling
    - Session state management (delegated to `aw-local-db`)
    - Agent runner coordination and monitoring
    - Integration with `aw-fs-snapshots` for workspace isolation
    - Configuration management and validation
  - Task and session state persistence with migrations
  - Local mode configuration and workspace management
  - Basic `aw task` and `aw session list/logs` CLI commands
  - Integration with ZFS snapshots for workspace operations

- Schema/Migration Management:
  - Define migration framework supporting up/down migrations
  - Versioned schema files with automatic application on startup
  - Migration testing framework to ensure compatibility
  - Schema validation and integrity checks

- Verification:
  - `aw-local-db` crate has comprehensive unit tests for all database operations
  - Schema migrations work correctly (upgrade/downgrade paths)
  - SQLite database operations tested with property-based testing
  - Task creation and session tracking work correctly
  - `aw task` creates proper branch/task file structure locally
  - State persists across process restarts
  - Integration tests with temporary Git repos and ZFS snapshots

**Phase 2: Agent Integration & Session Management** (4-5 weeks)

**2.1 Claude Code & Codex Agent Integration** (4-5 weeks)

- Deliverables:
  - Claude Code agent wrapper with hook-based session recording (PostToolUse events)
  - Codex agent wrapper with rollout file parsing (JSONL format from [Codex-Session-File-Format.md](../Research/Codex-Session-File-Format.md))
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

**Phase 3: Agent Time Travel** (4-5 weeks)

**3.1 Time Travel Core Implementation** (4-5 weeks)

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

**Phase 4: Sandboxing & Isolation** (6-8 weeks)

**4.1 Sandbox Integration** (6-8 weeks)

- Deliverables:
  - Complete Linux sandboxing implementation (see [Local-Sandboxing-on-Linux.status.md](Sanboxing/Local-Sandboxing-on-Linux.status.md))
  - Dynamic read allow-list with seccomp notify
  - Resource limits and audit logging
  - `aw session audit` command integration
  - Sandboxed agent execution with time travel

- Verification:
  - All sandbox milestones from [Local-Sandboxing-on-Linux.status.md](Sanboxing/Local-Sandboxing-on-Linux.status.md)
  - Agents run in isolated namespaces with proper resource limits
  - Audit logs capture file access decisions and sandbox events
  - Time travel works within sandboxed environments

**Phase 5: User Interface Development** (4-6 weeks, with parallel TUI/WebUI tracks)

**5.1 TUI Dashboard Implementation** (4-6 weeks)

- Deliverables:
  - Ratatui-based TUI implementation following [TUI-PRD.md](TUI-PRD.md)
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

**Phase 6: MVP Completion & Polish** (2-3 weeks)

**6.1 Final Integration & Documentation** (2-3 weeks)

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
- **Sandbox Complexity**: Mitigated by following the detailed milestone plan in [Local-Sandboxing-on-Linux.status.md](Sanboxing/Local-Sandboxing-on-Linux.status.md); each component tested in isolation before integration.
