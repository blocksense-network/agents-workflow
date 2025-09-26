### Overview

Goal: Deliver a minimum viable product (MVP) version of the Agents-Workflow CLI that provides core functionality for Linux users, focusing on agent time-travel capabilities with ZFS snapshots, Claude Code and Codex integration, and local mode operation. The MVP will serve as a foundation for subsequent cross-platform expansion and feature additions.

Total estimated timeline: 10-14 months (optimized with proper parallel development tracks and dependency ordering)

**Timeline Breakdown**:

- **Foundation Layer**: Weeks 1-4 (parallel infrastructure development)
- **Core Task Layer**: Weeks 5-12 (aw task command with agent integration)
- **Advanced Features Layer**: Weeks 13-20 (time travel + advanced sandboxing)
- **Integration Layer**: Weeks 21-24 (UI polish and final integration)

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

**Phase 0: Infrastructure Bootstrap** (with parallel infrastructure tracks)

**0.1 Repository Structure Reorganization** COMPLETED

- Deliverables:

  - Reorganize repository structure according to [Repository-Layout.md](Repository-Layout.md)
  - Move existing Ruby code to `legacy/ruby/` preserving all functionality
  - Update all import paths and references in moved files
  - Create basic Rust workspace directory structure (`crates/`, `bins/`, etc.)
  - Rename all existing just targets to have a `legacy-` prefix.

- Implementation Details:

  - Core Ruby library code moved to `legacy/ruby/lib/` and `legacy/ruby/test/`
  - Executable scripts remain in `bin/` and `scripts/` for functionality but reference legacy paths
  - Import paths updated (e.g., `bin/agent-task` now requires `../legacy/ruby/lib/agent_task/cli`)
  - Justfile targets renamed with `legacy-` prefix (`legacy-test`, `legacy-lint`, etc.)
  - Repository structure follows [Repository-Layout.md](Repository-Layout.md) with `crates/`, `bins/`, etc.

- Verification Results:
  - [x] Core Ruby library code moved to `legacy/ruby/` locations
  - [x] `just legacy-test` passes completely with no path-related failures
  - [x] `just legacy-test-codex-setup-integration` passes with Docker containers finding correct paths
  - [x] Executable scripts in `bin/` and `scripts/` remain functional with updated import paths

**0.2 Rust Workspace & Core Crates Bootstrap** COMPLETED

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

- Implementation Details:

  - Cargo.toml workspace configured with 25+ crates including core crates (`aw-core`, `aw-local-db`, `aw-fs-snapshots`), filesystem providers (`aw-fs-snapshots-zfs`, `aw-fs-snapshots-btrfs`), sandboxing (`aw-sandbox`, `aw-sandbox-linux`), and supporting crates
  - All crates implement proper module structure and dependencies
  - Essential dependencies configured in workspace: tokio, serde, clap, rusqlite, tracing, nix, async-trait, etc.
  - CI pipeline configured with GitHub Actions for automated Rust compilation and testing
  - Sandbox crates follow subcrates pattern with platform-specific implementations

- Key Source Files:

  - `Cargo.toml` - Workspace configuration with all crate members and shared dependencies
  - `crates/aw-core/src/lib.rs` - Task/session lifecycle orchestration skeleton
  - `crates/aw-local-db/src/lib.rs` - SQLite database management skeleton
  - `crates/aw-fs-snapshots/src/lib.rs` - Filesystem snapshot provider abstractions
  - `crates/aw-sandbox/src/lib.rs` - Core sandbox API with namespace orchestration
  - `crates/aw-sandbox-linux/src/lib.rs` - Linux-specific sandbox implementations
  - `.github/workflows/ci.yml` - CI pipeline configuration

- Verification Results:
  - [x] `cargo check --workspace` (`just check`) succeeds for all crates
  - [x] `cargo test --workspace` (`just test`) runs successfully
  - [x] CI pipeline configured and functional on push/PR
  - [x] Workspace structure matches [Repository-Layout.md](Repository-Layout.md)
  - [x] Essential dependencies (tokio, serde, clap, rusqlite, etc.) properly configured

**0.3 Privileged FS Operations Daemon** COMPLETED

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
  - Implemented proper SSZ union types for type-safe daemon communication (using `ethereum-ssz` with union behavior)
  - Added comprehensive ZFS and Btrfs operations (snapshot, clone, delete) with sudo privilege escalation and full validation
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

- **Future Enhancements** (non-blocking for MVP):

  - Consider alternatives to sudo requirement for privileged operations

- **Verification Results**:
  - [x] Daemon starts and listens on Unix socket at expected path
  - [x] Length-prefixed SSZ ping request returns success response via Unix socket
  - [x] Length-prefixed SSZ ping request returns success response via stdin mode
  - [x] Daemon handles invalid SSZ data gracefully with error responses
  - [x] Daemon shuts down cleanly on SIGINT/SIGTERM
  - [x] Comprehensive Rust integration tests that verify daemon communication via Unix socket, check daemon liveness via ping, and test both ZFS and Btrfs filesystem operations (similar to `legacy-test-snapshot` but implemented as proper Rust unit tests); available via `just test-daemon-integration`
  - [ ] Legacy tests still pass, using the legacy daemon from its new location

**0.4 FS Snapshots Core API** COMPLETED (parallel with 0.5-0.6)

- **Deliverables**:

  - Complete `aw-fs-snapshots` crate with `FsSnapshotProvider` trait matching FS-Snapshots-Overview.md specification
  - `prepare_writable_workspace()`, `snapshot_now()`, `mount_readonly()`, `branch_from_snapshot()`, and `cleanup()` method implementations
  - Provider auto-detection logic (`provider_for(path)`) with capability scoring
  - Basic error handling and path validation rejecting system directories (/dev, /proc, /sys, /run)
  - Integration with daemon for privileged operations (ZFS/Btrfs providers communicate with aw-fs-snapshots-daemon)

- **Implementation Details**:

  - Implemented complete `FsSnapshotProvider` trait with all methods specified in FS-Snapshots-Overview.md
  - Added `ProviderCapabilities`, `PreparedWorkspace`, `SnapshotRef` structs for type-safe API
  - ZFS provider supports CoW overlay mode with snapshot + clone operations via daemon
  - Btrfs provider supports CoW overlay mode with subvolume snapshots
  - Comprehensive path validation prevents workspace creation in system directories
  - Provider auto-detection scores capabilities (ZFS: 90, Btrfs: 80, Copy: 10)
  - Robust cleanup token system for idempotent resource teardown

- **Key Source Files**:

  - `crates/aw-fs-snapshots/src/lib.rs` - Core trait definition and provider selection logic
  - `crates/aw-fs-snapshots-zfs/src/lib.rs` - ZFS provider implementation with daemon integration
  - `crates/aw-fs-snapshots-btrfs/src/lib.rs` - Btrfs provider implementation
  - `crates/aw-fs-snapshots/src/error.rs` - Error types for filesystem operations

- **Verification Results**:
  - [x] Provider trait compiles and can be implemented by concrete providers
  - [x] Auto-detection returns correct provider with capability scoring
  - [x] Path validation rejects invalid destinations (system directories, root, etc.)
  - [x] Unit tests for provider selection logic pass (5/5 tests passing)
  - [x] All providers implement the complete trait specification

**0.5 ZFS Snapshot Provider** COMPLETED (parallel with 0.3-0.4, 0.6)

- **Deliverables**:

  - Complete `aw-fs-snapshots-zfs` crate with ZFS dataset operations
  - Dataset detection and mount point resolution
  - Snapshot creation, clone mounting, and cleanup via daemon communication
  - Proper error handling for missing datasets, permissions, daemon unavailability
  - Comprehensive unit tests covering all provider functionality

- **Implementation Details**:

  - Created separate `aw-fs-snapshots-traits` crate to avoid circular dependencies
  - ZFS provider uses daemon client for all privileged ZFS operations (snapshot, clone, destroy)
  - Supports CoW overlay mode with automatic dataset detection and mount point resolution
  - In-place mode supported for non-snapshot operations
  - Worktree mode not implemented (falls back to CoW overlay or fails)
  - Proper cleanup token system for idempotent resource management

- **Key Source Files**:

  - `crates/aw-fs-snapshots-traits/src/lib.rs` - Common traits and types
  - `crates/aw-fs-snapshots-zfs/src/lib.rs` - ZFS provider implementation
  - `crates/aw-fs-snapshots-daemon/src/client.rs` - Daemon client library

- **Verification Results**:
  - [x] ZFS provider detects available ZFS datasets correctly
  - [x] Daemon communication works for privileged ZFS operations
  - [x] CoW overlay mode creates snapshots and clones via daemon
  - [x] Error handling for missing datasets, permissions, daemon unavailability
  - [x] Comprehensive unit tests pass (8/8 tests passing)
  - [x] Cleanup tokens properly encoded and parsed for resource management

**0.6 FS Snapshots Test Infrastructure** COMPLETED (parallel with 0.3-0.5)

- **Deliverables**:

  - Port filesystem test helpers (`filesystem_test_helper.rb`) to Rust with `ZfsTestEnvironment` struct (focused on ZFS/Btrfs)
  - ZFS and Btrfs pool/subvolume creation for CI/testing environments with automatic cleanup (no loop device filesystems)
  - Port provider behavior tests (`provider_shared_behavior.rb`) with trait-based test organization
  - Port quota and performance tests to Rust equivalents with configurable expectations
  - Space measurement utilities for different filesystem types (ZFS, Btrfs, generic df)
  - **Reference existing Ruby test suite** (`test/snapshot/`) for test patterns and edge cases

- **Implementation Details**:

  - Created `ZfsTestEnvironment` struct for managing ZFS test pools (removed loop device filesystem support per requirements)
  - Added ZFS pool creation on file-based devices with dataset setup and mounting
  - Implemented full Btrfs provider support with subvolume snapshots and CoW operations
  - Fixed dependency cycles by having Btrfs provider depend on `aw-fs-snapshots-traits` instead of `aw-fs-snapshots`
  - Ported provider test traits: `ProviderCoreTestBehavior`, `ProviderPerformanceTestBehavior`, `ProviderQuotaTestBehavior`
  - Created space measurement utilities in `space_utils.rs` for parsing size strings and measuring filesystem usage
  - Added comprehensive integration tests for ZFS and Btrfs providers
  - Updated Justfile with new test targets: `test-fs-snapshots` and `test-fs-snapshots-unit`
  - Enabled Btrfs support in default feature set alongside ZFS

- **Key Source Files**:

  - `crates/aw-fs-snapshots/tests/filesystem_test_helpers.rs` - ZFS test pool management (no loop devices)
  - `crates/aw-fs-snapshots/tests/space_utils.rs` - Space measurement utilities
  - `crates/aw-fs-snapshots/tests/provider_core_behavior.rs` - Core provider test behaviors
  - `crates/aw-fs-snapshots/tests/zfs_integration_tests.rs` - ZFS-specific integration tests
  - `crates/aw-fs-snapshots/tests/integration.rs` - ZFS/Btrfs provider integration tests
  - `crates/aw-fs-snapshots-btrfs/src/lib.rs` - Full Btrfs provider implementation
  - `Justfile` - Added `test-fs-snapshots` and `test-fs-snapshots-unit` targets

- **Verification Results**:

  - [x] Rust test infrastructure compiles and provides ZFS/Btrfs test management
  - [x] ZFS and Btrfs providers compile and integrate correctly
  - [x] Provider auto-detection selects best provider (ZFS > Btrfs)
  - [x] Provider behavior traits ported from Ruby with equivalent functionality
  - [x] Space measurement utilities handle ZFS, Btrfs, and generic filesystems
  - [x] Integration tests created for ZFS and Btrfs provider validation
  - [x] Test targets added to Justfile for CI integration
  - [x] Loop device filesystem support removed per requirements
  - [x] Btrfs support enabled in default feature set

- **Outstanding Tasks** - Git Filesystem Snapshot Provider

The Git-based filesystem snapshot provider provides a portable fallback for environments without native CoW filesystems (ZFS/Btrfs). Implementation is tracked separately as it enables cross-platform time travel capabilities.

#### **Git Provider Implementation Requirements** ([Git-Based-Snapshots.md](../../specs/Public/FS%20Snapshots/Git-Based-Snapshots.md))

- **Deliverables**:

  - Create `aw-fs-snapshots-git` crate implementing `FsSnapshotProvider` trait
  - Shadow repository management with alternates for object sharing
  - Session-indexed snapshots using `git commit-tree` with temporary indexes
  - Git worktree support for writable workspaces and read-only mounting
  - Proper cleanup of refs, worktrees, and shadow repositories

- **Shadow Repository Management**:

  - Create bare shadow repository with alternates to primary repo
  - Manage per-session index files for incremental snapshots
  - Handle repository configuration (gc.auto=0, receive.denyCurrentBranch=ignore)

- **Snapshot Creation**:

  - Implement staged+unstaged changes capture using temporary index
  - Support untracked files inclusion (opt-in via config)
  - Create commits parented to primary HEAD with proper metadata
  - Store snapshots under namespaced refs `refs/aw/sessions/<sid>/snapshots/<n>`

- **Workspace Management**:

  - Restore writable working copies an worktrees from snapshot commits by copying
    the involved files.
  - Support read-only mounting for time travel inspection
  - Handle worktree cleanup and ref management

- **Configuration Integration**:

  - Add `git.includeUntracked`, `git.worktreesDir`, `git.shadowRepoDir` config options
  - Integrate with existing provider selection logic

- **Testing Requirements**:

  - Unit tests for shadow repository setup and snapshot creation
  - Integration tests with real git repositories, covering shadow repo setup, creation of snapshots and restoration of snapshots to a working tree.
  - Cross-platform compatibility testing (Linux/macOS/Windows)

- **Implementation Status**:
  - [ ] Create `aw-fs-snapshots-git` crate skeleton
  - [ ] Implement shadow repository management
  - [ ] Add session index file handling
  - [ ] Implement snapshot creation with temporary index
  - [ ] Add git worktree support for workspaces
  - [ ] Integrate with provider selection in `aw-fs-snapshots` crate
  - [ ] Add configuration options and CLI integration
  - [ ] Comprehensive testing and documentation

**Phase 1: Core Functionality** (with parallel VCS/task implementation tracks)

Phase 1 focuses on implementing the core `aw task` command in local mode with Codex support, recreating the behaviors of the legacy Ruby `agent-task` command. The CLI will be implemented in the `aw-cli` crate (per Repository-Layout.md) with Clap subcommands and TUI glue. Development proceeds through 10 granular milestones with automated testing at each step, starting with VCS abstraction and task file management, then building up to full CLI integration, sandboxing, and agent execution.

**Phase 1 Dependencies and Parallel Tracks**: Phase 1 implements core `aw task` functionality with proper dependency ordering. Components are organized in parallel tracks where possible:

- **VCS Track**: 1.1 VCS Repository Abstraction (foundation)
  - **Task Management Track**: 1.2 Task File Management (depends on 1.1), 1.3 Editor Integration (parallel), 1.4 Devshell Integration (parallel)
  - **Remote Operations Track**: 1.5 Push Operations (depends on 1.1)
  - **CLI Integration Track**: 1.6 AW Task CLI (depends on 1.1-1.5), 1.7 AW CLI Sandbox Integration (depends on 1.6), 1.8 Task State Persistence (depends on 1.6)
- **Agent Integration Track**: 1.9 Basic Codex Integration (depends on 1.6), 1.10 E2E Tests (depends on 1.6-1.9)

Parallel development enables faster progress while maintaining clean dependency boundaries.

**1.1 VCS Repository Abstraction** COMPLETED

- **Deliverables**:

  - Direct port of `legacy/ruby/lib/vcs_repo.rb` to Rust `aw-repo` crate (per Repository-Layout.md):
    - Multi-VCS support: Git, Mercurial, Bazaar, Fossil (same VCS types as Ruby implementation)
    - Repository root detection by walking parent directories (same logic as `find_repo_root`)
    - Current branch detection and validation (same commands as `current_branch`)
    - Branch name validation with regex `^[A-Za-z0-9._-]+$` (same as `valid_branch_name?`)
    - Main branch protection logic (refuse operations on `main`, `master`, `trunk`, `default`)
    - Basic branch creation and checkout operations (same as `start_branch`, `checkout_branch`)
    - Remote URL detection and SSH-to-HTTPS conversion (same as `default_remote_http_url`)
    - File operations: `commit_file`, `add_file` (same as Ruby implementation)
    - Push operations: `push_current_branch`, `force_push_current_branch` (same commands)
    - Query operations: `tip_commit`, `commit_count`, `branches`, `branch_exists?`
    - Commit operations: `first_commit_in_current_branch`, `latest_agent_branch_commit`
    - Setup operations: `setup_autopush` (same hook installation logic)
  - VCS abstraction traits and implementations for each supported VCS type
  - Error types for VCS operations with proper error handling
  - Port existing test patterns from `legacy/ruby/test/test_vcs_repo_methods.rb`

- **Reference Implementation**: Direct port of [legacy/ruby/lib/vcs_repo.rb](../../legacy/ruby/lib/vcs_repo.rb)
- **Reference Tests**: Port test patterns from [legacy/ruby/test/test_vcs_repo_methods.rb](../../legacy/ruby/test/test_vcs_repo_methods.rb) and [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb)

- **Implementation Details**:

  - Created `aw-repo` crate with synchronous API using std::process for all VCS operations
  - Implemented `VcsRepo` struct with methods matching Ruby implementation exactly
  - Added `VcsType` enum for Git, Hg, Bzr, Fossil support with per-VCS command builders
  - Environment isolation in tests: Set `HOME` to temp directory to prevent git authentication prompts
  - Command execution with proper environment variables (`GIT_CONFIG_NOSYSTEM`, `GIT_TERMINAL_PROMPT=0`, etc.)
  - Error handling with comprehensive `VcsError` enum for all failure scenarios
  - Branch parsing logic to strip git markers (`*`, spaces) from branch listings
  - SSH URL conversion from `git@github.com:user/repo.git` to `https://github.com/user/repo.git`

- **Verification Results**:
  - [x] Unit tests for repository detection in various directory structures (port `test_default_remote_http_url_*` tests)
  - [x] Branch name validation rejects invalid names and accepts valid ones (same regex as Ruby)
  - [x] Main branch protection prevents operations on `main`, `master`, `trunk`, `default` (same logic)
  - [x] Multi-VCS support tested for Git, Mercurial, Bazaar, Fossil repositories (same VCS commands)
  - [x] Branch creation and checkout operations work correctly across VCS types (same `git checkout -b`, etc.)
  - [x] Remote URL detection and SSH conversion works (same patterns as `test_ssh_url_variations`)
  - [x] Commit message retrieval works correctly (port `test_commit_message_retrieval`)
  - [x] Agent branch commit detection works (same `latest_agent_branch_commit` logic)
  - [x] Autopush setup installs hooks correctly (same hook installation as Ruby)

**1.2 Task File Management System** COMPLETED

- **Deliverables**:

  - Direct port of task file logic from `legacy/ruby/lib/agent_tasks.rb` to `aw-core` crate (per Repository-Layout.md task/session lifecycle orchestration):
    - Timestamped file naming: `.agents/tasks/YYYY/MM/DD-HHMM-branch_name` (same format as `record_initial_task`)
    - Task file format with follow-up delimiter `--- FOLLOW UP TASK ---` (same as `append_task`)
    - Initial task recording with branch name and timestamp (same logic as `record_initial_task`)
    - Follow-up task appending to existing task files (same logic as `append_task`)
    - Task file directory structure creation (`.agents/tasks/` hierarchy, same as Ruby)
    - Commit message generation: `Start-Agent-Branch: <branch>` for initial, `Follow-up task` for additional
  - Integration with VCS for committing task files (same as `commit_file` calls)

- **Reference Implementation**: Direct port of task file logic from [legacy/ruby/lib/agent_tasks.rb](../../legacy/ruby/lib/agent_tasks.rb) methods `record_initial_task` and `append_task`
- **Reference Tests**: Port test patterns from [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb) `assert_task_branch_created` helper and task file assertions

- **Implementation Details**:

  - Created `AgentTasks` struct with async API matching `aw-repo` requirements
  - Implemented `record_initial_task()` method with timestamped file naming and directory creation
  - Implemented `append_task()` method with proper delimiter handling
  - Added `agent_task_file_in_current_branch()` and `on_task_branch()` for task branch detection
  - Integrated `setup_autopush()` and `online()` connectivity check methods
  - Used ureq instead of reqwest for better Nix compatibility
  - Methods are async for HTTP operations and task file I/O

- **Key Source Files**:

  - `crates/aw-core/src/agent_tasks.rs` - AgentTasks struct and implementation
  - `crates/aw-core/tests/agent_tasks_tests.rs` - Comprehensive test suite (11 tests)
  - `crates/aw-core/Cargo.toml` - Updated with aw-repo and ureq dependencies

- **Verification Results**:
  - [x] Task files created with correct timestamped naming format (same as `record_initial_task`)
  - [x] Follow-up tasks appended with proper delimiter `--- FOLLOW UP TASK ---`
  - [x] Directory structure created automatically (`.agents/tasks/YYYY/MM/`)
  - [x] File content matches legacy Ruby implementation format
  - [x] Commit messages use correct format (`Start-Agent-Branch: <branch>` or `Follow-up task`)
  - [x] Integration tests with mock VCS operations (port `assert_task_branch_created` logic)
  - [x] All 11 unit tests pass covering file creation, appending, branch detection, and error cases

**1.3 Editor Integration** COMPLETED (depends on 1.1)

- **Deliverables**:

  - Direct port of editor logic from `legacy/ruby/lib/agent_task/cli.rb` to Rust:
    - Editor discovery chain: `$EDITOR` → nano → pico → micro → vim → helix → vi (same order as Ruby)
    - Temporary file creation with task template `EDITOR_HINT` (exact same text as Ruby)
    - Template processing: strip comments and normalize line endings (same logic as Ruby)
    - Empty task validation and user-friendly error messages (same "Aborted: empty task prompt." message)
    - Interactive vs non-interactive mode handling (same behavior as Ruby)

- **Reference Implementation**: Direct port of editor logic from [legacy/ruby/lib/agent_task/cli.rb](../../legacy/ruby/lib/agent_task/cli.rb) `start_task` method editor handling
- **Reference Tests**: Port test patterns from [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb) `test_editor_failure` and `test_empty_file` tests

- **Implementation Details**:

  - Created `editor.rs` module in `aw-core` crate with comprehensive editor functionality
  - Implemented `discover_editor()` function with same fallback chain as Ruby implementation
  - Created `edit_content_interactive()` function for full editing workflow with temporary files
  - Added `process_template()` function for comment stripping and line ending normalization
  - Integrated with existing error handling patterns using `thiserror`
  - Added `tempfile` dependency to `aw-core` for temporary file management

- **Key Source Files**:

  - `crates/aw-core/src/editor.rs` - Complete editor integration module
  - `crates/aw-core/src/lib.rs` - Updated exports for editor functionality
  - `crates/aw-core/Cargo.toml` - Added tempfile dependency

- **Verification Results**:
  - [x] Editor discovery finds correct editor in PATH (same chain as Ruby)
  - [x] Template file created with proper content and hints (exact same `EDITOR_HINT` text)
  - [x] Comment lines stripped correctly during processing (same logic as Ruby)
  - [x] Empty tasks rejected with clear error messages (same "Aborted: empty task prompt." message)
  - [x] Editor failure handling works correctly (same as `test_editor_failure`)
  - [x] Comprehensive unit tests covering all functionality (5/5 tests passing)
  - [x] Workspace compilation successful with no breaking changes

**1.4 Devshell Integration** COMPLETED (depends on 1.1)

- **Deliverables**:

  - Direct port of devshell logic from `legacy/ruby/lib/agent_task/cli.rb` to Rust:
    - Nix flake detection and devShell parsing (same `devshell_names` function logic)
    - Devshell name validation against `flake.nix` devShells (same validation as Ruby)
    - Multi-system devShell resolution (current system preferred, same fallback logic)
    - Devshell information recording in commit messages (`Dev-Shell: <name>`)
    - Graceful fallback when Nix/flake.nix not available (same error handling)

- **Reference Implementation**: Direct port of devshell logic from [legacy/ruby/lib/agent_task/cli.rb](../../legacy/ruby/lib/agent_task/cli.rb) `devshell_names` method and devshell validation in `start_task`
- **Reference Tests**: Port test patterns from [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb) `test_devshell_option`, `test_devshell_option_invalid`, and `test_devshell_without_flake` tests

- **Implementation Details**:

  - Created `devshell.rs` module in `aw-core` crate with async `devshell_names()` function
  - Implemented three-tier fallback: nix eval for current system → nix eval for all systems → regex parsing
  - Added comprehensive test suite covering all scenarios from Ruby tests
  - Integrated devshell functionality into `aw-core` lib.rs exports

- **Key Source Files**:

  - `crates/aw-core/src/devshell.rs` - Complete devshell parsing implementation with nix eval and regex fallbacks
  - `crates/aw-core/src/lib.rs` - Updated to export `devshell_names` function
  - `crates/aw-core/Cargo.toml` - Added regex dependency for fallback parsing

- **Verification Results**:
  - [x] Devshell names extracted correctly from flake.nix (same nix eval commands as Ruby)
  - [x] Validation rejects non-existent devshell names (same error messages)
  - [x] Multi-system flake support (current system prioritized, same logic)
  - [x] Commit message includes `Dev-Shell: <name>` when specified (same format)
  - [x] Graceful degradation when Nix not available (same error handling)
  - [x] Devshell validation works for new branch creation only (same restriction)
  - [x] All 6 unit tests pass covering parsing, validation, and error cases
  - [x] Full workspace compilation and test suite passes

**1.5 Push Operations & Remote Management** COMPLETED (depends on 1.1)

- **Deliverables**:

  - Direct port of push logic from `legacy/ruby/lib/agent_task/cli.rb` to Rust:
    - Remote URL detection from VCS configuration (same as `default_remote_http_url`)
    - SSH-to-HTTPS URL conversion for Git remotes (same conversion logic)
    - Interactive push prompts: "Push to default remote? [Y/n]:" (exact same prompt)
    - `--push-to-remote` flag for automated/non-interactive mode (same boolean parsing)
    - Push operation execution with proper VCS-specific commands (same as `push_current_branch`)
    - Commit message generation with remote URL tracking (`Target-Remote: <url>`)

- **Reference Implementation**: Direct port of push logic from [legacy/ruby/lib/agent_task/cli.rb](../../legacy/ruby/lib/agent_task/cli.rb) `start_task` method push handling
- **Reference Tests**: Port test patterns from VCS repo tests and task creation tests for push operations

- **Implementation Details**:

  - Created `push.rs` module in `aw-core` crate with `PushHandler` and `PushOptions` structs
  - Implemented boolean parsing for `--push-to-remote` flag with same truthy/falsy values as Ruby (`1`, `true`, `yes`, `y` / `0`, `false`, `no`, `n`)
  - Added interactive prompt logic with exact same prompt text: "Push to default remote? [Y/n]:"
  - Integrated with existing `aw-repo` crate for VCS operations and remote URL detection
  - Proper error handling for non-interactive environments (same exit behavior as Ruby)

- **Key Source Files**:

  - `crates/aw-core/src/push.rs` - Complete push handling implementation with interactive prompts and VCS integration
  - `crates/aw-core/src/lib.rs` - Updated to export push functionality (`PushHandler`, `PushOptions`, `parse_push_to_remote_flag`)

- **Verification Results**:
  - [x] Remote URLs detected correctly from VCS configuration (same as `default_remote_http_url`)
  - [x] SSH URLs converted to HTTPS format for authentication (same conversion patterns)
  - [x] Interactive prompts work correctly with stdin handling (same "Push to default remote? [Y/n]:" prompt)
  - [x] `--push-to-remote` flag bypasses interactive prompts (same boolean logic as Ruby)
  - [x] Push operations execute correctly for each VCS type (same VCS commands)
  - [x] Commit messages include `Target-Remote: <url>` when applicable (same format)
  - [x] Non-interactive mode validation works (same exit code 10 behavior)
  - [x] All unit tests pass covering boolean parsing, options builder, and error cases
  - [x] Full workspace compilation and test suite passes

**1.6 AW Task CLI Implementation** COMPLETED (1 week, depends on 1.1-1.5)

- **Deliverables**:

  - Complete `aw task` command implementation in `aw-cli` crate (per Repository-Layout.md) with Clap derive API
  - Direct port of CLI argument parsing from `legacy/ruby/lib/agent_task/cli.rb` `start_task`:
    - `--prompt <TEXT>`: Direct task content (same as Ruby `--prompt` option)
    - `--prompt-file <FILE>`: Read from file (same as Ruby `--prompt-file` option)
    - `--branch <NAME>`: Branch name for new tasks (same as positional branch argument)
    - `--devshell <NAME>`: Devshell specification (same as Ruby `--devshell` option)
    - `--push-to-remote <BOOL>`: Push automation (same as Ruby `--push-to-remote` option)
    - `--non-interactive`: Non-interactive mode (new flag for Rust implementation)
  - Integration with all subsystems (VCS, task files, editor, devshell, push)
  - Error handling and user-friendly messages (same error messages as Ruby)
  - Branch name validation and main branch protection (same logic as Ruby)

- **Reference Implementation**: Direct port of CLI logic from [legacy/ruby/lib/agent_task/cli.rb](../../legacy/ruby/lib/agent_task/cli.rb) `start_task` method and option parsing
- **Reference Tests**: Port comprehensive test patterns from [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb) including all test cases for different VCS types

- **Implementation Details**:

  - Created `task.rs` module in `aw-cli` crate with Clap derive API and complete workflow implementation
  - Implemented `TaskCommands` and `TaskCreateArgs` structs with all Ruby-compatible options
  - Integrated all core components: VCS repo abstraction, task file management, editor integration, devshell validation, and push operations
  - Added comprehensive error handling with exact Ruby error messages and behavior
  - Implemented branch validation and main branch protection logic matching Ruby implementation
  - Added non-interactive mode support for CI/CD environments
  - Updated CLI structure to include `aw task` subcommand
  - Made VcsRepo synchronous with no async interfaces for cleaner integration testing

- **Key Source Files**:

  - `crates/aw-cli/src/task.rs` - Complete task CLI implementation with argument parsing and workflow orchestration
  - `crates/aw-cli/src/lib.rs` - Updated to include task module and CLI structure
  - `crates/aw-cli/src/main.rs` - Updated to handle task subcommands
  - `crates/aw-cli/Cargo.toml` - Added aw-core and aw-repo dependencies

- **Verification Results**:
  - [x] All command-line flags parsed correctly (same options as Ruby)
  - [x] CLI help displays correctly with all options (`aw task --help`)
  - [x] New branch creation works end-to-end (same flow as Ruby `start_task`)
  - [x] Follow-up tasks on existing branches work correctly (same logic as Ruby)
  - [x] Integration with editor for interactive input (same editor chain)
  - [x] Integration with file-based input (`--prompt-file`) (same file reading)
  - [x] Error messages match legacy Ruby behavior (same error texts)
  - [x] Non-interactive mode validation works correctly (same exit code 10 behavior)
  - [x] Branch validation works (same regex and error messages)
  - [x] Main branch protection works (same primary branch names)
  - [x] Boolean parsing for `--push-to-remote` works with same truthy/falsy values
  - [x] All unit tests pass covering argument parsing, flag validation, and logic components
  - [x] Complete integration test suite ported from Ruby test_start_task.rb (13 tests total):
    - [x] test_clean_repo - Clean repository task creation with real git repos
    - [x] test_prompt_option - Direct prompt input (--prompt flag)
    - [x] test_prompt_file_option - File-based prompt input (--prompt-file flag)
    - [x] test_editor_failure - Editor exit failure handling (exit code 1)
    - [x] test_empty_file - Empty task content rejection (editor returns empty)
    - [x] test_dirty_repo_staged - Staged changes preservation
    - [x] test_dirty_repo_unstaged - Unstaged changes preservation
    - [x] test_devshell_option - Valid devshell validation (flake.nix required)
    - [x] test_devshell_option_invalid - Invalid devshell rejection
    - [x] test_devshell_without_flake - Missing flake.nix handling
    - [x] test_prompt_option_empty - Empty/whitespace prompt rejection
    - [x] test_prompt_file_empty - Empty file rejection
    - [x] test_invalid_branch - Invalid branch name rejection (no editor call)
  - [x] Integration tests run in CI (require git + binary, fully synchronous and reliable)
  - [x] Editor-based tests use --prompt fallback for test stability
  - [x] All tests replicate exact Ruby test_start_task.rb behavior and assertions
  - [x] Manual testing confirms CLI works correctly in real git repositories
  - [x] Full workspace compilation and test suite passes
  - [x] Integration tests use synchronous VcsRepo directly for VCS operations
  - [x] VcsRepo made synchronous with no async interfaces as requested

**1.7 AW CLI Sandbox Integration** COMPLETED (2–3d, depends on 1.6 + Local-Sandboxing-on-Linux.md M1-M8)

- **Deliverables**:

  - **AW CLI Parameters**: Initial `aw agent sandbox` parameter set matching current capabilities:
    - `--type local`: Enable basic process isolation (namespaces + filesystem sealing)
    - `--allow-network <yes|no>`: Allow internet access via slirp4netns (default: no)
    - `--allow-containers <yes|no>`: Enable container device access (/dev/fuse, storage dirs) (default: no)
    - `--allow-kvm <yes|no>`: Enable KVM device access for VMs (/dev/kvm) (default: no)
    - `--seccomp <yes|no>`: Enable dynamic filesystem access control (default: no)
    - `--seccomp-debug <yes|no>`: Enable debugging operations in sandbox (default: no)
    - `--mount-rw <PATH>...`: Additional writable paths to bind mount
    - `--overlay <PATH>...`: Paths to promote to copy-on-write overlays
  - **FS Snapshot Pre-cloning**: Snapshot cloning operations performed before sandbox creation, returning path pairs for bind mounting
  - **AW Task Integration**: Sandbox parameters added to `aw task` command for agent execution in isolated environments

- **Implementation Details**:

  - **Pre-sandbox Workflow**: FS snapshot provider clones workspace to temporary location before sandbox launch, providing source→destination path pairs for bind mounting
  - **Sandbox Launch Protocol**: Sandbox receives list of path pairs (host_path→sandbox_path) and performs bind mounts during initialization
  - **Sudo-less Snapshots**: The `aw-fs-snapshots-daemon` ([`crates/aw-fs-snapshots-daemon/`](../../crates/aw-fs-snapshots-daemon/)) provides privileged filesystem operations (ZFS/Btrfs snapshots) without requiring sudo in user applications; the same daemon used for testing will enable snapshot operations for `aw agent sandbox`.
  - **Integration Points**: Combines MVP FS snapshots (Phase 0.4-0.6) with sandboxing ([Local-Sandboxing-on-Linux.status.md](../../specs/Public/Sanboxing/Local-Sandboxing-on-Linux.status.md) M1-M8)

- **Verification Results**:

  - [x] AW CLI Parameters: `aw agent sandbox` subcommand implemented with all specified CLI flags (`--type local`, `--allow-network`, `--allow-containers`, `--allow-kvm`, `--seccomp`, `--seccomp-debug`, `--mount-rw`, `--overlay`)
  - [x] FS Snapshot Pre-cloning: Implemented workspace preparation with ZFS/Btrfs logic using `prepare_workspace_with_fallback()`
  - [x] AW Task Integration: Sandbox parameters added to `aw task` command with proper argument parsing and validation
  - [x] Basic Sandbox Configuration Mapping: `create_sandbox_from_args()` function maps CLI parameters to sandbox-core configuration
  - [x] E2E test: Basic sandbox integration test (`integration_test_sandbox_basic`) validates task creation with sandbox parameters
  - [ ] E2E test: Full sandbox execution with network/device access control (requires additional sandbox-core implementation)
  - [ ] E2E test: Dynamic filesystem access via seccomp (requires additional sandbox-core implementation)
  - [ ] All sandbox integration tests use custom `AW_HOME` for environment isolation from user configuration

- **Key Source Files**:

  - `crates/aw-cli/src/task.rs` - AW task command with sandbox parameter integration
  - `crates/aw-core/src/sandbox.rs` - Sandbox configuration mapping from CLI parameters
  - `crates/aw-fs-snapshots/src/lib.rs` - Pre-sandbox snapshot cloning interface
  - `tests/integration/sandbox_cli_integration.rs` - E2E tests for AW CLI sandbox integration

- **Cross-Spec Dependencies**:

  - **[Local-Sandboxing-on-Linux.status.md](../../specs/Public/Sanboxing/Local-Sandboxing-on-Linux.status.md) M1-M8**: Provides the sandbox implementation this milestone integrates
  - **FS-Snapshots-Overview.md**: Defines snapshot cloning operations performed before sandbox creation
  - **CLI.md**: Defines the parameter interface this milestone implements

**1.8 AW Agent FS Commands Implementation** COMPLETED

- **Deliverables**:

  - **Filesystem Detection Command**: `aw agent fs status` - Run filesystem detection and report capabilities, provider selection, and mount point information
  - **Session Snapshot Management**: `aw agent snapshot` - Create snapshots for agent sessions using standard repository and provider selection
  - **Snapshot Listing**: `aw agent fs snapshots <SESSION_ID>` - List snapshots created in agent coding sessions
  - **Branch Creation**: `aw agent fs branch create <SNAPSHOT_ID>` - Create writable branches from snapshots
  - **Branch Binding**: `aw agent fs branch bind <BRANCH_ID>` - Bind processes to specific branch views
  - **Branch Execution**: `aw agent fs branch exec <BRANCH_ID>` - Execute commands within branch contexts
  - **Integration with AW Task**: Automatic snapshot creation during task execution for supported filesystems
  - **State Persistence**: Recording of snapshot and branch metadata in local SQLite database

- **Test Filesystem Details** (created by `just create-test-filesystems`):

  - **ZFS Filesystem**: Pool `agents_workflow_test_zfs`, dataset `test_dataset`
    - Linux mount point: `/agents_workflow_test_zfs/test_dataset`
    - macOS mount point: `/Volumes/agents_workflow_test_zfs/test_dataset`
    - Permissions: User delegated for snapshot, create, destroy, mount operations
  - **Btrfs Filesystem**: Mounted at `$HOME/.cache/agents-workflow/btrfs_mount`, subvolume `test_subvol`
    - Full path: `$HOME/.cache/agents-workflow/btrfs_mount/test_subvol`
    - Features: user_subvol_rm_allowed mount option enabled
  - **Setup Requirements**: Run `just create-test-filesystems` before E2E tests (requires sudo for ZFS/Btrfs setup)
  - **Status Check**: Use `just check-test-filesystems` to verify if test filesystems are already created and properly mounted

- **Implementation Details**:

  - **Filesystem Detection**: Implemented `aw agent fs status` with JSON and verbose output modes, integrating with `aw_fs_snapshots::provider_for()` logic
  - **Command Structure**: Complete Clap-based CLI implementation for all agent FS commands with proper help text and argument parsing
  - **Note**: Snapshot metadata is authoritative in the filesystem providers (ZFS/Btrfs/Git/AgentFS). The CLI does not duplicate snapshot state in SQLite.
  - **Task Integration**: Added automatic snapshot creation placeholder in AW task workflow (awaiting AgentFS implementation)
  - **Branch Operations**: Command structures implemented for all branch operations (awaiting AgentFS integration)

- **Verification Results**:

  - [x] Command structure: All `aw agent fs` commands implemented with complete CLI argument parsing and help text
  - [x] Filesystem status: `aw agent fs status` command works with provider detection and capability reporting
  - [x] Database models: `FsSnapshotRecord` and `FsSnapshotStore` implemented in aw-local-db crate
  - [x] Task integration: Placeholder for automatic snapshot creation added to AW task workflow
  - [x] Compilation: All code compiles successfully and integrates with existing codebase
  - [ ] E2E functionality: Commands show informative messages (awaiting AgentFS and database persistence implementation)
  - [ ] Full E2E tests: Require AgentFS integration and database persistence to be fully testable
  - [ ] All agent FS integration tests use custom `AW_HOME` for environment isolation from user configuration

- **Key Source Files**:

  - `crates/aw-cli/src/agent/fs.rs` - Complete agent FS command implementations with Clap argument parsing (status, snapshots, branch ops) and `aw agent snapshot`
  - `crates/aw-local-db/src/models.rs` - FsSnapshotRecord and FsSnapshotStore database models and operations
  - `crates/aw-local-db/src/schema.rs` - Database schema definitions
  - `crates/aw-local-db/src/migrations.rs` - Database migration scripts
  - `crates/aw-cli/src/task.rs` - Task execution workflow with snapshot integration placeholder

- **Cross-Spec Dependencies**:

  - **FS-Snapshots-Overview.md**: Defines snapshot and branch operations implemented by these commands
  - **Agent-Time-Travel.md**: Provides the time travel use cases that drive FS branch operations
  - **Local-Mode.md**: Defines session lifecycle integration points
  - **State-Persistence.md**: Defines the SQL schema used for snapshot metadata storage

- **Implementation Notes**:

  - Command structures and CLI interfaces are complete and ready for AgentFS integration
  - Database models and schema are implemented (awaiting state persistence milestone activation)
  - All commands currently show informative messages about future functionality when AgentFS and database persistence are implemented
  - Task integration placeholder is positioned correctly in the workflow for automatic snapshot creation

- **Outstanding Tasks**:
  - Wire `aw agent fs status` to `aw_fs_snapshots::provider_for()` end‑to‑end and surface real filesystem type and mount point (platform‑specific detection).
  - Implement `aw agent snapshot` repository discovery (walk to VCS root), provider selection, and snapshot creation for ZFS/Btrfs/Git/AgentFS.
  - Do not persist snapshot rows in SQLite; rely on provider state. Implement human-friendly text and machine‑readable JSON output formats (`{ provider, ref, path }`).
  - Implement `aw agent fs snapshots <SESSION_ID>` to list snapshots using the correct provider with JSON/text modes.
  - Implement `aw agent fs branch create|bind|exec` behaviors backed by provider APIs and record branches in state.
  - Add automated tests: unit tests for provider wiring; integration tests that exercise status/init/snapshots using temporary repos and AW_HOME‑scoped DB.
  - Ensure all Agent FS tests run with custom `AW_HOME` to isolate environment.

**1.9 Task State Persistence** (parallel with 1.6)

- **Deliverables**:

  - Integration with `aw-local-db` crate for task state persistence (per State-Persistence.md specification)
  - Task metadata storage (branch, repository, timestamps, status) following State-Persistence.md schema
  - Session lifecycle tracking tied to task execution using SQLite database
  - Migration support for task-related schema changes with proper versioning
  - Query APIs for task listing and status retrieval matching State-Persistence.md tables
  - Support for `AW_HOME` environment variable to customize user configuration and database location

- **Verification**:

  - [ ] Tasks recorded in SQLite database on creation following State-Persistence.md schema
  - [ ] Task metadata includes all required fields from State-Persistence.md tables
  - [ ] Database migrations handle schema evolution per State-Persistence.md versioning
  - [ ] Task queries work correctly for listing operations using State-Persistence.md APIs
  - [ ] `AW_HOME` environment variable correctly overrides default configuration and database paths
  - [ ] Unit tests verify high-level API provided by aw-local-db crate works as expected
  - [ ] Integration tests with temporary databases validate State-Persistence.md compliance
  - [ ] All state persistence integration tests use custom `AW_HOME` for environment isolation from user configuration

- **Outstanding Tasks**:

  - Add unit tests for `aw-local-db` stores (Repo/Agent/Runtime/Session/Task/FsSnapshot/Kv) covering inserts, queries, and update paths.
  - Add integration tests in `aw-core`/`aw-cli` that verify session + task records are written on `aw task create`, honoring `AW_HOME` override.
  - Add migration tests to assert `schema_migrations` handling and idempotent re‑runs.
  - Implement session status transitions and tests (created → running → completed/failed/cancelled) and timestamps.
  - Implement automatic initial snapshot persistence hook in `aw task` when provider supports it, with tests.

- **Cross-Spec Dependencies**:

  - **[State-Persistence.md](../../specs/Public/State-Persistence.md)**: Defines the complete SQL schema, backend selection rules, and data model used for task state persistence

**1.10 Basic Codex Agent Integration** (1 week, depends on 1.6)

- **Deliverables**:

  - Codex agent detection and validation
  - Direct asciinema recording integration for session capture
  - Task execution orchestration with agent process management
  - Session file format compatibility ([Codex-Session-File-Format.md](../Research/Codex-Session-File-Format.md))
  - Mock agent fallback for testing environments

- **Verification**:
  - [ ] Codex CLI availability detected correctly
  - [ ] Asciinema recording captures agent execution directly
  - [ ] Session files written in correct JSONL format
  - [ ] Task execution manages agent processes with proper cleanup
  - [ ] Session resumption works for interrupted Codex sessions

**1.11 AW Task E2E Integration Tests** (1 week, depends on 1.6-1.11)

- **Deliverables**:

  - Comprehensive end-to-end test suite for `aw task` workflows
  - Direct port of test infrastructure from `legacy/ruby/test/test_helper.rb`:
    - Temporary Git repository test fixtures (same `setup_repo` function)
    - VCS helper functions (`git`, `hg`, `fossil`, `capture`) (same implementations)
    - Test runner utilities (`run_agent_task`, `run_get_task`, etc.) (adapted for Rust)
  - Integration tests covering all scenarios from Ruby tests:
    - New task creation with branch setup (port `test_clean_repo`)
    - Dirty repo handling (port `test_dirty_repo_staged`, `test_dirty_repo_unstaged`)
    - Follow-up tasks on existing branches (port `assert_task_branch_created`)
    - Editor integration in test environments (port `test_editor_failure`, `test_empty_file`)
    - Push operations with mock remotes (port push-related test logic)
    - Prompt options testing (port `test_prompt_option`, `test_prompt_file_option`)
    - Devshell integration (port `test_devshell_option*` tests)
    - Branch validation (port `test_invalid_branch`)
  - Property-based testing for edge cases
  - CI integration with automated test execution

- **Reference Implementation**: Port test infrastructure from [legacy/ruby/test/test_helper.rb](../../legacy/ruby/test/test_helper.rb) and test patterns from [legacy/ruby/test/test_start_task.rb](../../legacy/ruby/test/test_start_task.rb)
- **Reference Tests**: All test cases from the Ruby StartTaskCases module for Git, Hg, and Fossil

- **Verification**:
  - [ ] E2E test: Complete task creation workflow (new branch) - port `test_clean_repo`
  - [ ] E2E test: Follow-up task workflow (existing branch) - port `assert_task_branch_created`
  - [ ] E2E test: Editor integration with template processing - port `test_editor_failure`, `test_empty_file`
  - [ ] E2E test: Push operations with remote interaction - port push logic from tests
  - [ ] E2E test: Codex agent integration end-to-end - new tests for Rust implementation
  - [ ] CLI integration test: Sandbox command validation - `test_sandbox_filesystem_isolation_cli_integration` in `sandbox.rs` validates `aw agent sandbox` command parameter parsing and execution attempts
  - [ ] E2E test: Agent FS commands integration - validate automatic snapshot creation
  - [ ] Property tests for branch name validation and file naming - same regex validation
  - [ ] CI pipeline includes E2E test execution with proper cleanup (same temp dir handling)
  - [ ] All VCS types tested (Git, Hg, Fossil) with same test patterns as Ruby
  - [ ] All integration tests use custom `AW_HOME` for environment isolation from user configuration

**Phase 6: TUI Dashboard Implementation** (with sophisticated E2E testing)

**6.1 TUI Core Infrastructure**

- **Deliverables**:

  - Create `aw-tui` crate with Ratatui-based TUI framework (per Repository-Layout.md)
  - Implement basic terminal event loop and rendering pipeline
  - Set up crossterm for input handling and screen management
  - Create TUI application skeleton with screen management and navigation
  - Add theme system with high-contrast accessibility option

- **Reference Implementation**: Basic Ratatui application structure with event loop
- **Verification**:
  - [ ] TUI application compiles and displays basic screen
  - [ ] Terminal input events are captured and processed
  - [ ] Screen rendering works with proper ANSI escape sequences
  - [ ] High-contrast theme can be toggled

**6.2 Multiplexer Integration**

- **Deliverables**:

  - Implement multiplexer detection and auto-attachment logic (tmux > zellij > screen)
  - Create multiplexer session management with window/pane creation
  - Add window creation for new tasks with split panes (right=agent activity, left=editor/workspace)
  - Implement remote multiplexer attachment for REST backend sessions
  - Handle devcontainer pane execution within container context

- **Reference Implementation**: Use existing `aw-mux` crate from Repository-Layout.md for multiplexer operations
- **Verification**:
  - [ ] Auto-attaches to existing multiplexer session or creates new one
  - [ ] New task launches create proper window with split panes
  - [ ] Remote sessions use SSH details for multiplexer attachment
  - [ ] Devcontainer execution works within container panes

**6.3 Dashboard Layout and Widgets**

- **Deliverables**:

  - Implement main dashboard layout with top selectors and bottom task editor
  - Create fixed-height list widgets for Project, Branch, Agent selectors
  - Add multiline task description editor with resizable height (Ctrl+Up/Down)
  - Implement Start action button and hotkey (Ctrl+Enter)
  - Add status bar showing backend (local/rest), multiplexer, and operation results

- **Reference Implementation**: Ratatui widgets for lists, text input, and layout management
- **Verification**:
  - [ ] Dashboard displays proper layout with all widgets visible
  - [ ] Task description editor resizes with Ctrl+Up/Down
  - [ ] Start action launches task and creates multiplexer window
  - [ ] Status bar shows correct backend and multiplexer information

**6.4 Selector Components and Filtering**

- **Deliverables**:

  - Implement filtering input for each selector (prefix/substring matching)
  - Add keyboard navigation (arrows, PageUp/Down, Home/End) within fixed-height viewports
  - Connect Branch selector to VCS data (git commands in local mode, REST API in remote mode)
  - Connect Agent selector to local config or REST `/api/v1/agents`
  - Connect Project selector to local usage history or remote workspaces

- **Reference Implementation**: Ratatui List widget with custom filtering and navigation logic
- **Verification**:
  - [ ] Each selector filters entries as user types
  - [ ] Arrow key navigation works within viewport bounds
  - [ ] Branch selector shows correct VCS branches for current repo
  - [ ] Agent selector displays available agents from backend
  - [ ] Project selector shows accessible repositories/workspaces

**6.5 Dynamic Footer and Hotkeys**

- **Deliverables**:

  - Implement context-sensitive footer with actionable shortcuts
  - Add hotkey handling: Tab/Shift+Tab cycling, Ctrl+F for filters, navigation keys
  - Create help overlay (F1) showing complete keymap
  - Implement Esc for back navigation and Ctrl+C for safe abort
  - Add double Ctrl+C for quit from dashboard

- **Reference Implementation**: Crossterm key event handling with context-aware shortcut display
- **Verification**:
  - [ ] Footer shows relevant shortcuts for current context
  - [ ] All hotkeys work as specified (Tab cycling, Ctrl+F, etc.)
  - [ ] Help overlay displays complete keymap on F1
  - [ ] Esc and Ctrl+C handle navigation and abort correctly
  - [ ] Double Ctrl+C quits from dashboard

**6.6 Error Handling and Validation**

- **Deliverables**:

  - Add inline validation messages under selectors (branch not found, agent unsupported)
  - Implement error handling for failed operations with user-friendly messages
  - Add validation for task launch (required fields, valid selections)
  - Handle network errors and backend unavailability gracefully
  - Add retry logic for transient failures

- **Reference Implementation**: Error state management with user feedback display
- **Verification**:
  - [ ] Invalid selections show clear error messages
  - [ ] Network failures display helpful error states
  - [ ] Task validation prevents launch with incomplete information
  - [ ] Backend errors are handled with retry options where appropriate

**6.7 Persistence and Configuration**

- **Deliverables**:

  - Implement persistence of last selections (project, agent, branch) per repo/user scope
  - Add configuration integration for TUI preferences and defaults
  - Save/restore window layout and splitter positions
  - Remember multiplexer preferences and session settings
  - Support user-level and repo-level configuration overrides

- **Reference Implementation**: Integration with `aw-config` crate for persistence
- **Verification**:
  - [ ] Last selections are restored on TUI restart
  - [ ] Configuration changes affect TUI behavior
  - [ ] Window layout preferences are preserved
  - [ ] Per-repo configurations override user defaults

**6.8 TUI Sophisticated E2E Testing Infrastructure**

- **Deliverables**:

  - Set up comprehensive E2E testing framework using expectrl + portable-pty + insta
  - Create PTY-based test harness for simulating real terminal interactions
  - Implement snapshot testing for UI regression detection
  - Add scenario-based E2E tests covering complete user workflows
  - Create test utilities for keyboard input simulation and screen content verification

- **Reference Implementation**: expectrl for PTY control, portable-pty for terminal simulation, insta for snapshot testing
- **Verification**:
  - [ ] Test framework can launch TUI in PTY environment
  - [ ] Keyboard inputs are properly simulated and processed
  - [ ] Screen snapshots capture UI state for regression testing
  - [ ] Complete workflows (selector navigation → task launch) work end-to-end

**6.9 TUI Scenario-Based E2E Tests**

- **Deliverables**:

  - Implement E2E test scenarios for all major TUI workflows
  - Add tests for selector filtering and navigation
  - Create tests for task launch with multiplexer window creation
  - Add tests for error handling and validation feedback
  - Implement tests for remote session handling and SSH multiplexer attachment
  - Create cross-platform compatibility tests (Linux/macOS/Windows where applicable)

- **Reference Implementation**: expectrl-based scenario testing similar to mock-agent integration tests
- **Verification**:
  - [ ] All selector interactions work correctly (filtering, navigation, selection)
  - [ ] Task launch creates proper multiplexer windows with correct pane layout
  - [ ] Error states display appropriate messages and recovery options
  - [ ] Remote session workflows complete successfully
  - [ ] Accessibility features work (high-contrast theme, keyboard navigation)

**Cross-Spec Dependencies and Implementation Order**

The MVP implementation must coordinate across multiple specifications with proper dependency ordering:

**Foundation Layer (Weeks 1-4)**:

- **Agent-Time-Travel.md Phase 0**: Mock Agent + Mock API Server (test harness foundation)
- **Local-Sandboxing-on-Linux.md M1-M2**: Core sandbox infrastructure (namespaces, basic FS isolation)
- **Phase 1.1**: VCS Repository Abstraction (shared foundation for all components)

**Core Task Layer (Weeks 5-12)**:

- **Phase 1.2-1.10**: Complete `aw task` command implementation with all features
- **Agent-Time-Travel.md Phase 1**: Codex agent integration (adapted from Claude Code phases)
- **Local-Sandboxing-on-Linux.md M3-M4**: Cgroups and overlay support

**Advanced Features Layer (Weeks 13-20)**:

- **Agent-Time-Travel.md Phase 2-3**: Full time travel features (seek, branch, checkpointing)
- **Local-Sandboxing-on-Linux.md M5-M8**: Dynamic allow-list, networking, debugging, containers/VMs

**Integration Layer (Weeks 21-24)**:

- **Agent-Time-Travel.md Phase 4**: Cross-platform workspace binding
- **Local-Sandboxing-on-Linux.md M9-M10**: Supervisor integration and CLI acceptance
- **Phase 2-5**: Agent integrations, time travel UI, sandboxing polish

**Key Dependency Insights**:

- Mock agents must be available before real agent integration testing
- Basic sandboxing (M1-M2) enables safe agent execution during development
- VCS abstraction is required by task files, push operations, and devshell validation
- Agent integration depends on working `aw task` CLI but can develop mock-first
- Time travel features build on agent integration and session recording
- Advanced sandboxing features can be added incrementally without blocking core functionality

**Phase 2: Agent Integration & Session Management** (with parallel agent tracks, can start after foundation layer)

**2.1 Codex Agent Integration** (depends on Phase 1.6 + Agent-Time-Travel.md Phase 0, parallel with 2.2)

- Deliverables:

  - Codex agent wrapper with rollout file parsing (JSONL format from [Codex-Session-File-Format.md](../../Research/Codex-Session-File-Format.md))
  - Integrated asciinema recording in task execution flow
  - Session timeline creation with SessionMoments for Codex
  - Basic session resumption via `--resume` flag for Codex
  - Codex rollout file parsing and trimming for time travel

- Verification:
  - Codex rollout files parsed correctly from session directories
  - Asciinema recording integrated into task execution
  - Session recordings captured and stored in SQLite for Codex
  - Rollout path detection and session ID mapping works
  - Codex resumes from interrupted sessions correctly
  - Rollout files can be trimmed to specific moments for time travel

**2.2.5 Claude Code Mock Agent Support** (depends on Agent-Time-Travel.md Phase 0)

- Deliverables:

  - Extend mock agent (`tests/tools/mock-agent/`) to support Claude Code session format
  - Implement Claude session file creation in `~/.claude/projects/<encoded-workspace-path>/<uuid>.jsonl`
  - Add Claude-compatible API server responses for tool execution and conversation threading
  - Create scenario-based automation for Claude Code interactive testing
  - Validate Claude session format with proper parent-child UUID relationships and tool metadata

- Reference Implementation: Extend existing mock agent architecture from [tests/tools/mock-agent/README.md](../../tests/tools/mock-agent/README.md)
- Verification:
  - Mock agent can drive Claude Code with API key confirmation workflows
  - Claude format session files created with correct metadata and threading
  - Interactive testing scenarios work for Claude Code API interactions
  - Session files match Claude format specifications
  - Tool execution properly recorded in Claude session format

**2.2 Claude Code Agent Integration** (depends on Phase 1.6 + 2.2.5, parallel with 2.1)

- Deliverables:

  - Claude Code agent wrapper with hook-based session recording (PostToolUse events)
  - Integrated asciinema recording in task execution flow
  - Session timeline creation with SessionMoments for Claude Code
  - Basic session resumption via `--resume` flag for Claude Code
  - Claude transcript parsing and trimming for time travel

- Verification:
  - Claude Code hooks emit SessionMoments at tool boundaries
  - Asciinema recording integrated into task execution
  - Session recordings captured and stored in SQLite for Claude Code
  - Transcript path detection and session ID mapping works
  - Claude Code resumes from interrupted sessions correctly
  - Transcript files can be trimmed to specific moments for time travel

**2.3 Agent Runner & Session Management** (depends on 2.1 & 2.2)

- Deliverables:

  - Agent execution coordination within main CLI for both Claude Code and Codex
  - Session management coordination between different agent types
  - Integration with mock agent for testing (`tests/tools/mock-agent/`)
  - Agent process monitoring and lifecycle management in task execution flow

- Verification:
  - Both Claude Code and Codex work with integrated agent execution
  - Session management handles multiple concurrent agent types
  - Mock agent integration enables deterministic testing
  - Agent process monitoring detects completion/failure correctly

**Phase 3: Agent Time Travel** (depends on Phase 2 agent integration, with incremental implementation)

**3.1 Session Timeline Infrastructure**

- Deliverables:

  - Session timeline data structures and storage in SQLite
  - SessionMoment creation and indexing for both Claude Code and Codex
  - Timeline navigation and seeking APIs
  - FsSnapshot integration for timestamp-to-filesystem mapping
  - Basic `aw session seek` command for timeline inspection

- Verification:
  - Session timelines build correctly from agent recordings
  - SessionMoments indexed and searchable by timestamp
  - Timeline navigation works for both agent types
  - FsSnapshot references correctly link moments to filesystem state

**3.2 Time Travel Commands & UI** (depends on 3.1)

- Deliverables:

  - Read-only snapshot mounting for inspection at specific moments
  - Session branching with injected messages
  - Transcript/rollout trimming for precise time travel resumption
  - `aw session branch` command with message injection
  - Time travel UI components for timeline visualization

- Verification:
  - ZFS snapshots mount read-only at specific timestamps
  - Transcript/rollout trimming preserves conversation up to target moment
  - Branched sessions start agents with trimmed context
  - End-to-end time travel: seek → inspect → branch → resume
  - UI shows clear timeline with branching points

**Phase 4: Sandboxing & Isolation** (can start parallel with Phase 2, depends on Local-Sandboxing-on-Linux.md M1-M4)

**4.1 Sandbox Integration**

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

**Phase 5: User Interface Development** (depends on Phase 1-4 completion, with parallel TUI/WebUI tracks)

**5.1 TUI Dashboard Implementation**

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

**Phase 6: MVP Completion & Polish**

**6.1 Final Integration & Documentation**

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
- **Mock Agent Testing**: Use the mock agent implementation (`tests/tools/mock-agent/`) for deterministic testing of agent integrations, session recording, and time travel functionality without external API dependencies. The mock agent simulates Claude Code/Codex behavior by editing files, streaming thinking traces, and writing session files in the correct [Codex Session File Format](../../specs/Research/Codex-Session-File-Format.md).
- **Time Travel E2E Tests**: Automated tests that create sessions with mock agents, seek to specific moments, create branches, and verify resumed agents have correct context.
- **Snapshot Testing**: Use `cargo insta` for CLI help text and generated documentation to ensure spec parity.
- **CI Pipeline**: Maintain separate pipelines for `just legacy-tests` (Ruby), Rust MVP development, and integration tests requiring ZFS/sandboxes. Ensure `test-codex-setup-integration` continues to pass during reorganization.
- **Environment Isolation**: All integration tests must use custom `AW_HOME` environment variable to isolate test execution from user configuration and state. This prevents test interference with user data and ensures reproducible test results.

### Risks & mitigations

- **ZFS Dependency**: Mitigated by providing alternative Git-based snapshot fallback in development; ZFS becomes optional for basic functionality but required for full time-travel features.
- **Agent Evolution**: Mitigated by comprehensive hook testing and version compatibility checks for both Claude Code and Codex; maintain fallback to basic session resumption if hooks/API change.
- **Codex Rollout Complexity**: Mitigated by thorough testing of JSONL parsing and trimming logic; the rollout file format specification provides clear parsing rules to follow.
- **Repository Reorganization**: Mitigated by preserving all existing functionality in `legacy/` during transition; `test-codex-setup-integration` tests must pass unchanged.
- **Complex Time Travel Logic**: Mitigated by building extensive integration tests from day one; both transcript and rollout trimming logic will be thoroughly tested with synthetic session files.
- **Sandbox Complexity**: Mitigated by following the detailed milestone plan in [Local-Sandboxing-on-Linux.status.md](Sanboxing/Local-Sandboxing-on-Linux.status.md); each component tested in isolation before integration.
