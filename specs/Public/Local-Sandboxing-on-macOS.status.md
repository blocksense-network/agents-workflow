### Overview

This document tracks the implementation status and plan for Local Sandboxing on macOS and serves as the single source of truth for milestones, automated success criteria, and cross‑team integration points.

Goal: deliver a production‑ready macOS sandbox for agents with an FSKit‑backed AgentFS overlay filesystem, process‑scoped branch binding, chroot+Seatbelt hardening, and Endpoint Security (ES)‑based interactive file/network approvals, surfaced via the AW CLI and supervisor.

Total estimated timeline: 6–8 months (phased with parallel tracks)

### Components

- FSKit adapter and XPC control service: `adapters/macos/xcode/AgentFSKitExtension/` (filesystem and control plane)
- Host app for extension registration: `apps/macos/AgentsWorkflow/`
- AgentFS Rust core and FFI: `crates/agentfs-core/`, `crates/agentfs-ffi/`, `crates/agentfs-proto/`
- Sandbox launcher (macOS): new target to orchestrate FSKit mount → chroot → Seatbelt → exec
- Endpoint Security system extension: new target providing file/process/network authorization
- Supervisor UI/daemon: prompts, policy store, and audit, integrated with AW CLI

### Key design clarifications (macOS)

- FSKit‑based AgentFS provides a per‑process mount overlay covering the entire system view. AgentFS enforces read/write policy and branch isolation. ES is used to block the agent thread while awaiting user approval for out‑of‑policy file accesses (dynamic allow‑list).
- Entry into AgentFS is performed via a chroot. Seatbelt (custom SBPL profile) is applied to ensure the launched process cannot touch anything outside the AgentFS view. Seatbelt rules remain in effect inside the AgentFS mount namespace.
- Endpoint Security is the primary enforcement hook for dynamic approvals (filesystem reads/exec, signals, network connects), auditing, and defense‑in‑depth where it helps.

### Parallel development tracks

- AgentFS/FSKit track: FSKit adapter, XPC control, per‑process binding; chroot handoff flows.
- Endpoint Security track: authorization clients for file open/exec, signal, and connect; supervisor prompt; policy cache.
- Seatbelt/profile track: SBPL authoring, launcher integration, and code signing.
- Supervisor/UX track: prompts, policy persistence, audit, AW CLI.
- Test/acceptance track: macOS specific harnesses (simulator where possible), reproducible E2E runs.

### Milestones (with automated success criteria)

Phase 1: Foundations (2–3 weeks)

M1. Project bootstrap (macOS sandbox) ✅ COMPLETED (1–2d)

- Deliverables:
  - Scaffolding for ES client (system extension target), FSKit host app+extension linkage, and sandbox launcher.
  - CI lanes for macOS build/lints; lint‑specs for documentation.
- Verification:
  - `xcodebuild` builds ES extension, FSKit extension, and host app (without signing) on CI.
  - `cargo check --workspace` succeeds for AgentFS crates.
- Implementation details:
  - Established Xcode targets and Rust build glue per [AgentFS status](AgentFS/AgentFS.status.md).
  - Added initial SBPL profile template and launcher stub.

M2. AgentFS mount + chroot handoff (FSKit) ✅ COMPLETED (see [AgentFS status](AgentFS/AgentFS.status.md))

- Deliverables:
  - FSKit volume mounts AgentFS; process‑scoped branch binding; XPC control plane.
  - Launcher: spawn workload in AgentFS branch, enter chroot to AgentFS root, pass environment and cwd.
- Verification:
  - Smoke: create/open/read/write/unlink under AgentFS succeeds; host paths outside AgentFS not visible after chroot.
  - Branch binding shows divergent views for two processes.

Phase 2: macOS sandbox policy (3–4 weeks)

M3. Seatbelt profile hardening (SBPL) (3–5d)

- Deliverables:
  - Custom SBPL profile enforcing: deny default; allow file‑read/write only within AgentFS mounts; deny Apple Events, debug/inspection of outside processes, and sensitive services; allow needed system calls for normal dev tools inside AgentFS.
  - Launcher applies Seatbelt to target after chroot, before exec.
- Verification:
  - E2E: writes outside AgentFS return EPERM/EACCESS under `sandbox-exec`/libsandbox.
  - E2E: process‑info to others denied; Apple Events/XPC to unknown services denied.
  - Static analyzer check for profile syntax; golden snapshot of SBPL shipped.

M4. Endpoint Security: filesystem gating (AUTH_OPEN/EXEC) (5–7d)

- Deliverables:
  - ES system extension subscribes to AUTH_OPEN, AUTH_EXEC, NOTIFY_CLOSE, NOTIFY_RENAME.
  - Authorization flow: block thread, canonicalize path within AgentFS root, consult policy cache, prompt via supervisor, return allow/deny; audit log.
  - Policy cache with TTL and scope (session/user/project/org).
- Verification:
  - E2E: first read of a non‑allowed path blocks until approval; approve unblocks without relaunch; deny returns EACCES.
  - Unit: correct path resolution across symlinks relative to AgentFS root.
  - Load/perf: prompt coalescing works for directory scopes; LRU cache hit rate measured.

M5. Endpoint Security: process and debugging controls (3–5d)

- Deliverables:
  - ES gate for `signal` (AUTH_SIGNAL) to enforce “inside→inside only”; deny signals to outside processes.
  - Debugging policy: allow LLDB/ptrace only within sandbox cohort; deny attach to outside.
- Verification:
  - E2E: `killall`/`kill` against outside PIDs denied; same‑cohort signals allowed.
  - E2E: lldb attach to in‑sandbox child works in debug mode; attach to host process denied.

M6. Endpoint Security: network gating (AUTH_SOCKET_CONNECT) (4–6d)

- Deliverables:
  - ES authorization for outbound connects: default deny; allow loopback; supervisor‑mediated approvals with domain/IP caching; optional DNS guard.
  - Optional NE DNS proxy/control integration stub for future fine‑grained domain policy.
- Verification:
  - E2E: outbound to internet blocked by default; localhost permitted; approved domains work after prompt.
  - Unit: decision cache honors TTL and scope; audit includes {pid, exe, dest}.

Phase 3: Integration & UX (2–3 weeks)

M7. Supervisor integration + policy persistence (3–5d)

- Deliverables:
  - Prompt UI (menubar or lightweight app) with decision, scope, and remember options.
  - Policy stores merged (org → project → user → session overrides) with deterministic precedence.
  - Append‑only audit logs with rotation.
- Verification:
  - Golden tests for policy serialization; audit snapshots.
  - E2E: policy persists across sessions; non‑interactive mode defaults to deny.

M8. AW CLI integration & acceptance suite (3–5d)

- Deliverables:
  - `aw sandbox` orchestration: create AgentFS branch → mount FSKit → chroot → apply Seatbelt → exec workload → ES active.
  - `aw session audit` shows ES/FS decisions; config keys wired.
- Verification:
  - Acceptance: filesystem gating, network gating, process isolation, debug toggles all pass.
  - CLI E2E: run, approve, deny, persist; teardown leaves no residue.

Phase 4: Hardening & Ops (2–3 weeks)

M9. Security review, performance, and fault injection (4–6d)

- Deliverables:
  - Stress tests for ES decision rates; denial storms; supervisor crashes (failsafe default‑deny).
  - Profile minimization; least privilege review; code signing/hardening runtime settings.
- Verification:
  - No publicly known escape vectors in configuration; updated to latest macOS with patches.
  - Throughput targets met for common dev workloads with pre‑seeded paths.

### Test strategy & tooling

- Unit: ES client authorization handlers, path canonicalization (relative to AgentFS root), policy cache.
- Integration: FSKit mount + chroot + Seatbelt flow; ES gating end‑to‑end prompts; network/signal gating.
- Acceptance: scripted workflows covering dynamic approvals, static RO policy, debugging constraints, and teardown cleanliness.
- Performance: prompt coalescing, decision cache hit rates, worst‑case denial latency.

### Deliverables

- FSKit AgentFS mount flow with chroot handoff and per‑process binding.
- ES system extension implementing file/process/network authorization gates with supervisor prompts.
- Seatbelt SBPL profile and launcher integration.
- Supervisor app with policy stores and audit.
- AW CLI orchestration and acceptance suite.

### Risks & mitigations

- ES/NE entitlements and approval flow: plan for developer provisioning profiles; fallback to reduced functionality without signing.
- Performance of ES gating: use directory‑granularity approvals, pre‑seed standard toolchain paths, and cache decisions with TTL.
- Seatbelt fragility: keep minimal allow‑list; comprehensive tests; ship diagnostics for profile failures.
- FSKit maturity: rely on AgentFS per‑process binding and integration milestones; incremental rollout.

### Parallelization notes

- M3 (Seatbelt), M4/M5/M6 (ES), and M7 (Supervisor) can proceed in parallel once M2 (AgentFS mount+chroot) is stable.
- CLI (M8) follows after ES and Supervisor provide stable APIs.

### References

- [AgentFS status](AgentFS/AgentFS.status.md)
- [Sandboxing Strategies](Sandboxing/Agents-Workflow-Sandboxing-Strategies.md)
- [Local Sandboxing on Linux (status)](Sandboxing/Local-Sandboxing-on-Linux.status.md)
