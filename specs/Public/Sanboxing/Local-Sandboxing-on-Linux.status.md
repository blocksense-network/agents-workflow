### Overview

This document tracks the implementation status of the [Local-Sandboxing-on-Linux](Local-Sandboxing-on-Linux.md) functionality.

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

**M1. Project bootstrap** (1–2d)

- Deliverables:
  - Scaffolding for crates, workspace, linting (clippy), formatting (rustfmt), CI (GitHub Actions, Ubuntu runners).
  - Smoke test: cargo build + unit test skeletons.

- Verification:
  - `cargo check --workspace` succeeds for all sandbox-related crates
  - `cargo test --workspace` runs successfully (may have empty test suites)
  - CI pipeline runs successfully on push/PR for sandbox crates

**M2. Minimal sandbox run (namespaces + RO sealing)** (3–5d)

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

**M3. Cgroups v2 limits** (2–3d)

- Deliverables:
  - Create per‑session subtree; set pids.max, memory.high/max, cpu.max.
  - Metrics sampling (read files under cgroupfs).

- Verification:
  - E2E test: fork-bomb process is contained within sandbox (cannot create unlimited processes)
  - E2E test: memory allocation beyond limit triggers OOM kill within sandbox
  - Unit test: CPU throttling applied correctly when limits exceeded
  - Integration test: cgroup subtree created and cleaned up properly

**Phase 2: Advanced Features** (4-5 weeks total, with parallel feature tracks)

**M4. Overlays + static mode** (3–4d)

- Deliverables:
  - Overlay planner for selected paths; upper/work dirs under session state dir.
  - Static mode switch: blacklist + overlays without dynamic prompts.

- Verification:
  - E2E test: modifying a blacklisted path fails with appropriate error
  - E2E test: overlay path persists changes to upperdir across sandbox restarts
  - E2E test: clean teardown removes overlay upper/work directories
  - Unit tests for overlay path planning and validation

**M5. Dynamic read allow‑list (seccomp notify)** (5–8d)

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
