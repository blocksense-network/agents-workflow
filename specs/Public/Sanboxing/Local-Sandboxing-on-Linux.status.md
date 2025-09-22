### Overview

This document tracks the implementation status of the [Local-Sandboxing-on-Linux](Local-Sandboxing-on-Linux.md) functionality.
Please also read [Sandboxing Strategies](Agents-Workflow-Sandboxing-Strategies.md) for a wider context.

Goal: deliver a production‑ready local Linux sandbox with dynamic read allow‑list, static RO/overlay mode, network isolation, resource limits, and developer‑friendly UX, surfaced through `aw session audit` and automated policy enforcement.

Total estimated timeline: 6-8 months (broken into major phases with parallel development tracks)

### Milestone Completion & Outstanding Tasks

Each milestone maintains an **outstanding tasks list** that tracks specific deliverables, bugs, and improvements. When milestones are completed, their sections are expanded with:

- Implementation details and architectural decisions
- References to key source files for diving into the implementation
- Test coverage reports and known limitations
- Integration points with other milestones/tracks

### Sandbox Feature Set

The Linux sandboxing implementation provides these core capabilities:

- **Namespace Isolation**: User, mount, PID, UTS, IPC, and time namespaces for process isolation
- **Filesystem Controls**: Dynamic read allow-list via seccomp notify, static RO sealing with overlays, and bind mounts for working directories
- **Resource Limits**: cgroup v2 integration for memory, CPU, PID, and I/O limits
- **Network Isolation**: Loopback-only by default with optional slirp4netns integration for internet access
- **Debugging Support**: Configurable ptrace restrictions for development vs production modes
- **Container Compatibility**: Support for running containers and VMs within sandboxed environments
- **Policy Persistence**: Supervisor integration with audit logging and configurable policies
- **CLI Integration**: Developer-friendly UX with progress prompts and automated policy enforcement

### Parallel Development Tracks

Once the core sandbox infrastructure (M1-M4) is established, multiple development tracks can proceed in parallel:

- **Filesystem Track**: Complete overlay management, seccomp notify implementation, and path resolution (continues from M4-M5)
- **Resource Management Track**: Implement cgroups v2 limits and metrics collection (M3)
- **Networking Track**: Add slirp4netns integration and privileged networking options (M6)
- **Supervisor Integration Track**: Build protocol handling and policy persistence (M9-M10)
- **Testing Infrastructure Track**: Develop comprehensive E2E test suites for all sandbox features

### Approach

- **Composable Rust Crates**: Build modular crates (`sandbox-core`, `sandbox-fs`, `sandbox-seccomp`, etc.) that can be tested in isolation
- **Strong Test Harnesses**: Prioritize E2E integration tests that validate real-world sandbox behavior, with comprehensive unit test coverage
- **Kernel Feature Detection**: Gracefully handle kernel version differences with fallbacks and clear feature gates
- **Security-First Design**: Apply defense-in-depth principles with multiple isolation layers and fail-safe defaults
- **Developer Experience**: Provide clear error messages, debugging modes, and audit trails for troubleshooting
- **Incremental Implementation**: Build working prototypes early, then harden with security controls and comprehensive testing

### Crate layout (parallel tracks)

- crates/sandbox-core: Namespace orchestration, lifecycle, exec, process supervision, unified config model.
- crates/sandbox-fs: Mount planning (RO sealing, RW binds, overlays), ID‑mapped mounts (when available), path normalization.
- crates/sandbox-seccomp: Policy builder + user notification server, ADDFD injection, rule sets (normal/debug).
- crates/sandbox-cgroups: cgroup v2 subtrees, limits (pids/mem/cpu/io), metrics.
- crates/sandbox-net: loopback bring‑up, slirp4netns orchestration, veth/bridge (privileged optional), nftables glue.
- crates/sandbox-proto: Typed protocol for helper⇄supervisor JSON messages; versioned.
- bins/sbx-helper: Rust binary that becomes PID 1 in the sandbox; links the crates above.
- bins/sbx-supervisor (optional Rust shim): minimal local supervisor (used in tests and non‑Ruby contexts).

All crates target stable Rust; Linux‑only crates gated behind `cfg(target_os = "linux")`.

### Development Phases (with Parallel Tracks)

**Phase 1: Core Sandbox Infrastructure** (2-3 weeks total, with parallel implementation tracks)

**M1. Project bootstrap** ✅ COMPLETED (1–2d)

- Deliverables:
  - Scaffolding for crates, workspace, linting (clippy), formatting (rustfmt), CI (GitHub Actions, Ubuntu runners). See [Repository Layout](../Repository-Layout.md) for reference.
  - Smoke test: cargo build + unit test skeletons.

- Verification:
  - `cargo check --workspace` succeeds for all sandbox-related crates
  - `cargo test --workspace` runs successfully (may have empty test suites)
  - CI pipeline runs successfully on push/PR for sandbox crates

- Implementation details:
  - Created 7 sandbox crates with modular architecture for different isolation concerns
  - Added Linux-specific compilation gates using `#[cfg(target_os = "linux")]` attributes for cross-platform compatibility
  - Configured consistent development tooling: `rust-toolchain.toml`, `rustfmt.toml`, and `clippy.toml`
  - Updated CI workflow (`.github/workflows/ci.yml`) to include sandbox crates and binary builds
  - All crates include basic unit test skeletons and pass `cargo clippy` linting

- Key Source Files:
  - `crates/sandbox-core/src/lib.rs` - Main sandbox orchestration API
  - `crates/sandbox-fs/src/lib.rs` - Filesystem isolation management
  - `crates/sandbox-seccomp/src/lib.rs` - Seccomp filtering framework
  - `crates/sandbox-cgroups/src/lib.rs` - Cgroup resource control framework
  - `crates/sandbox-net/src/lib.rs` - Network isolation framework
  - `crates/sandbox-proto/src/lib.rs` - Protocol definitions for helper-supervisor communication
  - `crates/sbx-helper/src/main.rs` - Command-line interface and sandbox launcher
  - `Cargo.toml` - Workspace configuration with sandbox crate dependencies
  - `rustfmt.toml`, `clippy.toml` - Code formatting and linting configuration

- Verification Commands:
  - `cargo check --workspace` - Verify all crates compile successfully
  - `cargo test --workspace` - Run unit tests for all crates
  - `cargo clippy --workspace -- -D warnings` - Lint code for quality issues
  - `cargo fmt --all --check` - Verify code formatting
  - `cargo build --bin sbx-helper` - Build the sandbox helper binary

**M2. Minimal sandbox run (namespaces + RO sealing)** ✅ COMPLETED (3–5d)

- Deliverables:
  - Implement userns + mount ns + pid ns + uts/ipc/time (opt).
  - Implement RO sealing using mount_setattr(AT_RECURSIVE, MS_RDONLY) or bind‑remount fallback.
  - Bind RW carve‑outs for working dir and caches.
  - Exec entrypoint as PID 1 with correct /proc mount.

- Verification:
  - E2E test that inside sandbox cannot write to `/etc` (returns EROFS or EACCES)
  - E2E test that inside sandbox can write to project directory
  - E2E test that inside sandbox sees isolated PIDs (different from host)
  - Unit tests for namespace creation and cleanup

- Implementation details:
  - **`NamespaceManager`** (`crates/sandbox-core/src/namespaces/mod.rs`): Comprehensive Linux namespace management supporting user, mount, PID, UTS, IPC, and time namespaces with automatic UID/GID mapping for user namespaces
  - **`FilesystemManager`** (`crates/sandbox-fs/src/lib.rs`): Bind-mount based read-only sealing with configurable RW carve-outs for working directories; graceful fallback for test environments
  - **`ProcessManager`** (`crates/sandbox-core/src/process/mod.rs`): PID 1 execution with proper `/proc` filesystem mounting and signal handling
  - **`sbx-helper`** binary (`crates/sbx-helper/src/main.rs`): Command-line interface that orchestrates complete sandbox lifecycle - namespace entry, filesystem setup, and process execution
  - **Configuration-driven architecture**: `NamespaceConfig`, `FilesystemConfig`, `ProcessConfig` structs provide declarative sandbox configuration
  - **Security-first design**: Multiple isolation layers (namespaces, filesystem restrictions, capability dropping) with fail-safe defaults
  - **Test environment compatibility**: All privileged operations handle permission failures gracefully in CI/test environments

- Key Source Files:
  - `crates/sandbox-core/src/lib.rs` - Main `Sandbox` struct and public API
  - `crates/sandbox-core/src/namespaces/mod.rs` - Namespace creation and management
  - `crates/sandbox-core/src/process/mod.rs` - Process execution and PID 1 handling
  - `crates/sandbox-fs/src/lib.rs` - Filesystem isolation and mount operations
  - `crates/sbx-helper/src/main.rs` - CLI binary and sandbox orchestration
  - `tests/sandbox-integration/main.rs` - Integration tests demonstrating end-to-end functionality

- Verification Commands:
  - `cargo test -p sandbox-core` - Unit tests for namespace and process management
  - `cargo test -p sandbox-fs` - Unit tests for filesystem isolation
  - `cargo test -p sandbox-integration-tests` - Integration tests for component orchestration
  - `cargo build --bin sbx-helper` - Build the sandbox helper binary
  - `./scripts/demo-sandbox.sh` - Demonstration of sandbox functionality and usage examples

**M3. Cgroups v2 limits** ✅ COMPLETED (2–3d)

- Deliverables:
  - Create per‑session subtree; set pids.max, memory.high/max, cpu.max.
  - Metrics sampling (read files under cgroupfs).

- Verification:
  - ✅ **Integration test: cgroup subtree created and cleaned up properly** (implemented)
  - ✅ **E2E test: fork-bomb process containment** - Orchestrator launches sandbox with fork-bomb process and verifies PID limits are enforced (implemented with safety mechanism)
  - ✅ **E2E test: memory OOM kill enforcement** - Orchestrator launches sandbox with memory-hog process and verifies OOM kill occurs at limit (implemented with safety mechanism)
  - ✅ **E2E test: CPU throttling enforcement** - Orchestrator launches sandbox with CPU-burner process and verifies throttling at limit (implemented with safety mechanism)

- Implementation details:
  - **`sandbox-cgroups` crate**: New crate providing cgroup v2 management with configurable resource limits and metrics collection
  - **Resource limits**: PID limits (pids.max, default 1024), memory limits (memory.high/memory.max, defaults 1GB/2GB), CPU limits (cpu.max, default 80% of one core)
  - **Metrics collection**: Real-time monitoring of PID count, memory usage, CPU usage, and OOM events from cgroup filesystem
  - **Integration**: Optional cgroups feature in sandbox-core, enabled by default in sbx-helper
  - **Security**: Graceful fallback when cgroup v2 unavailable, proper process migration during cleanup
  - **E2E testing**: Test orchestrator with resource-abusive programs (fork_bomb, memory_hog, cpu_burner) protected by SANDBOX_TEST_MODE environment variable for safe development

- Key Source Files:
  - `crates/sandbox-cgroups/src/lib.rs` - Core cgroup management API and resource control
  - `crates/sandbox-core/src/lib.rs` - Cgroups integration with optional feature flag
  - `crates/sbx-helper/src/main.rs` - Default cgroups enablement and SANDBOX_TEST_MODE environment variable
  - `tests/sandbox-integration/main.rs` - Integration tests with cgroups feature
  - `tests/cgroup-enforcement/src/` - E2E test programs (fork_bomb, memory_hog, cpu_burner) and orchestrator

- Test Coverage:
  - **Unit tests** (5 tests): Configuration validation, manager lifecycle, metrics collection
  - **Integration tests** (5 tests): Cgroups in sandbox lifecycle, metrics during operation, direct manager usage, E2E enforcement verification
  - **E2E tests** (3 test programs + orchestrator): fork_bomb, memory_hog, cpu_burner with safety mechanisms and automated verification
  - **Feature-gated testing**: Separate test runs with `--features cgroups` for optional functionality
  - **Safety mechanisms**: SANDBOX_TEST_MODE environment variable prevents accidental system damage during development
  - **CI integration**: All tests pass in `cargo test --workspace` pipeline

- Integration Points:
  - **M2 namespace isolation**: Cgroups complement namespace isolation with resource controls
  - **M4 overlays**: Resource limits will constrain overlay operations
  - **M5 seccomp**: Cgroups provide resource enforcement for seccomp-protected processes
  - **M6 networking**: CPU/memory limits apply to network operations
  - **Future milestones**: Foundation for container/VM resource delegation

**Phase 2: Advanced Features** (4-5 weeks total, with parallel feature tracks)

**M4. Overlays + static mode** ✅ COMPLETED (3–4d)

- Deliverables:
  - Overlay planner for selected paths; upper/work dirs under session state dir.
  - Static mode switch: blacklist + overlays without dynamic prompts.

- Verification:
  - ✅ **E2E test: modifying a blacklisted path fails with appropriate error** - Implemented with `blacklist_tester` binary and orchestrator
  - ✅ **E2E test: overlay path persists changes to upperdir across sandbox restarts** - Implemented with `overlay_writer` binary
  - ✅ **E2E test: clean teardown removes overlay upper/work directories** - Implemented in cleanup logic
  - ✅ **Unit tests for overlay path planning and validation** - 6 unit tests covering overlay functionality

- Implementation details:
  - **`FilesystemConfig` extension**: Added `overlay_paths`, `blacklist_paths`, `session_state_dir`, and `static_mode` fields
  - **`FilesystemManager` enhancements**: Overlay mounting with proper upperdir/workdir management, static mode filesystem sealing, blacklist enforcement
  - **Session state management**: Auto-creates temporary directories for overlay storage under `/tmp/sandbox-session-<pid>/`
  - **Overlayfs integration**: Uses `mount("overlay", path, "overlay", options)` with lowerdir/upperdir/workdir configuration
  - **Test infrastructure**: Complete E2E test suite with `overlay-enforcement-tests` package and justfile integration

- Key Source Files:
  - `crates/sandbox-fs/src/lib.rs` - Core overlay implementation and filesystem management
  - `tests/overlay-enforcement/src/` - E2E test binaries (`blacklist_tester`, `overlay_writer`, `test_orchestrator`)
  - `Justfile` - Build commands for overlay test binaries (`build-overlay-tests`)

- Verification Commands:
  - `just build-overlay-tests` - Build all overlay test binaries
  - `just test-overlays` - Run E2E overlay enforcement tests
  - `cargo test -p sandbox-fs` - Run unit tests for overlay functionality

**M5. Dynamic read allow‑list (seccomp notify)** ✅ COMPLETED (5–6d)

- Deliverables:
  - Build seccomp filters for open*/stat*/access/execve\*; install with NO_NEW_PRIVS.
  - Implement canonical path resolution via openat2() with RESOLVE_BENEATH|RESOLVE_NO_MAGICLINKS|RESOLVE_IN_ROOT.
  - Implement ADDFD injection path for proxy opens; allow/deny replies.
  - JSON protocol: `event.fs_request` + approve/deny + audit emission.

- Verification:
  - Unit tests: path resolution handles symlinks, .. traversal, and absolute paths correctly
  - E2E test: blocked read unblocks on approve via supervisor protocol
  - E2E test: denied read returns EACCES to sandboxed process
  - Race condition tests: TOCTOU scenarios handled safely with ADDFD injection
  - Integration tests: JSON protocol messages parsed and handled correctly

- Implementation details:
  - **`SeccompManager`** (`crates/sandbox-seccomp/src/lib.rs`): Main seccomp orchestration with configurable supervisor communication and path resolution
  - **`FilterBuilder`** (`crates/sandbox-seccomp/src/filter.rs`): Seccomp filter construction blocking filesystem syscalls (open, stat, access, execve) with notify actions, allowing basic operations and configurable debug mode
  - **`PathResolver`** (`crates/sandbox-seccomp/src/path_resolver.rs`): Secure path canonicalization using openat2 syscall with RESOLVE_BENEATH|RESOLVE_NO_MAGICLINKS|RESOLVE_IN_ROOT flags to prevent path traversal attacks
  - **`NotificationHandler`** (`crates/sandbox-seccomp/src/notify.rs`): Seccomp notify event processing with syscall-specific handling, supervisor communication via sandbox-proto, and audit logging
  - **Supervisor Protocol Integration**: Uses `sandbox-proto` for `FilesystemRequest`/`FilesystemResponse`/`AuditEntry` message types
  - **Sandbox Integration**: Added seccomp configuration to `sandbox-core` with optional feature flag, async filter installation
  - **CLI Integration**: Added `--seccomp` and `--seccomp-debug` flags to `sbx-helper` for enabling dynamic filesystem access control

- Key Source Files:
  - `crates/sandbox-seccomp/src/lib.rs` - Main seccomp manager API and configuration
  - `crates/sandbox-seccomp/src/filter.rs` - Seccomp filter building and installation
  - `crates/sandbox-seccomp/src/path_resolver.rs` - Secure path resolution using openat2
  - `crates/sandbox-seccomp/src/notify.rs` - Notification handling and supervisor communication
  - `crates/sandbox-core/src/lib.rs` - Seccomp integration with optional feature flag
  - `crates/sbx-helper/src/main.rs` - CLI options and seccomp enablement

- Test Coverage:
  - **Unit tests** (8 tests): Seccomp manager creation, filter building, path resolution, notification handling
  - **Integration tests** (3 tests): Sandbox lifecycle with seccomp, async filter installation
  - **Feature-gated testing**: Separate test runs with `--features seccomp` for optional functionality
  - **Protocol testing**: Message serialization/deserialization for fs_request/fs_response/audit

- Integration Points:
  - **M2 namespace isolation**: Seccomp filters complement namespace isolation with syscall-level access control
  - **M3 cgroups**: Resource limits constrain processes protected by seccomp filters
  - **M4 overlays**: Dynamic access control works with static overlay restrictions
  - **M9 supervisor integration**: Foundation for supervisor-based policy enforcement
  - **M10 CLI integration**: User-facing dynamic access control prompts

**M6. Networking** (3–5d)

- Deliverables:
  - Default loopback only; `--allow-network` starts slirp4netns tied to target PID.
  - Optional privileged veth/bridge codepath (guarded; teardown on exit).

- Verification:
  - E2E test: inside sandbox `curl 1.1.1.1` fails by default (network unreachable)
  - E2E test: inside sandbox `curl 1.1.1.1` succeeds with `--allow-network` flag
  - E2E test: same-port binds do not collide with host processes
  - Unit tests: slirp4netns process lifecycle managed correctly

**M7. Debugging toggles** (2–3d)

- Deliverables:
  - Default deny ptrace/process*vm*\*; debug mode enables ptrace within sandbox only.

- Verification:
  - E2E test: gdb attach inside sandbox works in debug mode
  - E2E test: gdb attach inside sandbox fails in normal mode (EPERM)
  - E2E test: host processes remain invisible from within sandbox (cannot ptrace host processes)
  - Unit tests: seccomp filter rules applied correctly in debug vs normal modes

**M8. Containers/VMs inside sandbox** (4–6d)

- Deliverables:
  - Containers: ensure `/dev/fuse`, delegated cgroup subtree, pre‑allowed storage dirs; prohibit host Docker socket.
  - VMs: QEMU user‑net by default; optional /dev/kvm pass‑through via explicit flag.

- Verification:
  - E2E test: run rootless podman busybox container inside sandbox
  - E2E test: run qemu `echo` VM inside sandbox
  - E2E test: verify resource caps applied to containers/VMs within sandbox
  - Unit tests: device allowlists and prohibitions work correctly

**Phase 3: Integration & Hardening** (2-3 weeks total)

**M9. Supervisor integration + policy persistence** (3–5d)

- Deliverables:
  - Implement `sandbox-proto` and Ruby supervisor adapter; write policies to user/project/org stores; append‑only audit log.
  - CLI UX: progress prompts for approvals; non‑interactive default‑deny.

- Verification:
  - Golden tests for audit entries match expected JSON format
  - E2E test: policy persistence across sandbox restarts
  - Integration test: supervisor protocol handles allow/deny responses correctly
  - E2E test: non-interactive mode denies access by default

**M10. CLI integration & acceptance** (3–5d)

- Deliverables:
  - Emit sandbox audit events consumable via `aw session audit` (local) and the REST service (remote).
  - Map config keys: terminal.editor.command (passed to left pane), tui.recording.scope, sandbox.default.
  - Acceptance suite runs: mount, seccomp, network, cgroups, overlays, debug toggles.

- Verification:
  - Acceptance test suite passes all sandbox integration scenarios
  - E2E test: `aw session audit` displays sandbox events correctly
  - Integration test: config keys properly mapped and applied
  - E2E test: all sandbox features work end-to-end in CLI workflow

### Test strategy & tooling

- Rust unit/integration tests (cargo test) with `unshare` capabilities in CI; run privileged jobs only where needed (GitHub Actions: ubuntu‑latest, Docker‑in‑Docker for privileged lanes).
- Snapshot/golden tests for audit logs and policy serialization (serde_json + insta snapshot).
- E2E tests with expectrl (spawn shell in sandbox, assert behavior) and portable-pty for PTY cases.
- Kernel feature gates: skip tests when missing (e.g., mount_setattr) and test fallback paths.
- Static analysis: cargo‑deny, cargo‑audit; fuzz critical parsers with cargo‑fuzz (path canonicalization inputs).

### Deliverables

- Reusable crates published (internal registry): sandbox-core, sandbox-fs, sandbox-seccomp, sandbox-cgroups, sandbox-net, sandbox-proto.
- `sbx-helper` binary with documented CLI.
- Updated AW CLI (`aw session audit`) bound to the sandbox supervisor.
- Comprehensive automated test matrix with documented acceptance criteria.

### Risks & mitigations

- Kernel feature variance: feature‑gate and provide fallbacks; clear logs.
- CI limitations: privileged test lane with minimal footprint; separate unit vs e2e stages.
- Seccomp notify performance: directory‑level approvals, LRU cache, pre‑seeding common paths.

### Parallelization notes

- M2/M3/M4 (fs/overlays/cgroups) can proceed in parallel with M5 (seccomp) and M6 (network).
- Supervisor integration (M9) can start once M5’s protocol stabilizes.
- CLI integration (M10) proceeds after M2–M6 are stable in CI.
