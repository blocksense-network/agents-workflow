### Overview

This document tracks the implementation status of the [Local Sandboxing on Linux](./Local%20Sandboxing%20on%20Linux.md) functionality.

Goal: deliver a production‑ready local Linux sandbox with dynamic read allow‑list, static RO/overlay mode, network isolation, resource limits, and developer‑friendly UX, surfaced through `aw session audit` and automated policy enforcement.

Approach: Build composable Rust crates with strong test harnesses. Wire them into the AW CLI and the (Ruby) supervisor via a small JSON‑over‑UDS protocol. Execute in phases with parallelizable tracks.

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

### Milestones and tasks (with automated tests)

M1. Project bootstrap (1–2d)

- Scaffolding for crates, workspace, linting (clippy), formatting (rustfmt), CI (GitHub Actions, Ubuntu runners).
- Smoke test: cargo build + unit test skeletons.

M2. Minimal sandbox run (namespaces + RO sealing) (3–5d)

- Implement userns + mount ns + pid ns + uts/ipc/time (opt).
- Implement RO sealing using mount_setattr(AT_RECURSIVE, MS_RDONLY) or bind‑remount fallback.
- Bind RW carve‑outs for working dir and caches.
- Exec entrypoint as PID 1 with correct /proc mount.
- Tests: e2e test that inside sandbox cannot write to `/etc`, can write to project dir, and sees isolated PIDs.

M3. Cgroups v2 limits (2–3d)

- Create per‑session subtree; set pids.max, memory.high/max, cpu.max.
- Metrics sampling (read files under cgroupfs).
- Tests: fork‑bomb containment, memory cap enforced (trigger OOM), cpu throttling basic check.

M4. Overlays + static mode (3–4d)

- Overlay planner for selected paths; upper/work dirs under session state dir.
- Static mode switch: blacklist + overlays without dynamic prompts.
- Tests: modifying a blacklisted path fails; overlay path persists changes to upperdir; clean teardown.

M5. Dynamic read allow‑list (seccomp notify) (5–8d)

- Build seccomp filters for open*/stat*/access/execve\*; install with NO_NEW_PRIVS.
- Implement canonical path resolution via openat2() with RESOLVE_BENEATH|RESOLVE_NO_MAGICLINKS|RESOLVE_IN_ROOT.
- Implement ADDFD injection path for proxy opens; allow/deny replies.
- JSON protocol: `event.fs_request` + approve/deny + audit emission.
- Tests: unit (path resolution), e2e (blocked read unblocks on approve; deny returns EACCES), race/TOCTOU.

M6. Networking (3–5d)

- Default loopback only; `--allow-network` starts slirp4netns tied to target PID.
- Optional privileged veth/bridge codepath (guarded; teardown on exit).
- Tests: inside sandbox `curl 1.1.1.1` fails by default; succeeds with allow‑network; same‑port binds do not collide with host.

M7. Debugging toggles (2–3d)

- Default deny ptrace/process*vm*\*; debug mode enables ptrace within sandbox only.
- Tests: gdb attach inside sandbox works in debug mode and fails in normal mode; host processes remain invisible.

M8. Containers/VMs inside sandbox (4–6d)

- Containers: ensure `/dev/fuse`, delegated cgroup subtree, pre‑allowed storage dirs; prohibit host Docker socket.
- VMs: QEMU user‑net by default; optional /dev/kvm pass‑through via explicit flag.
- Tests: run rootless podman busybox; run qemu `echo` VM; verify resource caps applied.

M9. Supervisor integration + policy persistence (3–5d)

- Implement `sandbox-proto` and Ruby supervisor adapter; write policies to user/project/org stores; append‑only audit log.
- CLI UX: progress prompts for approvals; non‑interactive default‑deny.
- Tests: golden tests for audit entries; policy persistence across runs.

M10. CLI integration & acceptance (3–5d)

- Emit sandbox audit events consumable via `aw session audit` (local) and the REST service (remote).
- Map config keys: terminal.editor.command (passed to left pane), tui.recording.scope, sandbox.default.
- Acceptance suite runs: mount, seccomp, network, cgroups, overlays, debug toggles.

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
