### Overview

This document tracks the implementation status of the [AgentFS](AgentFS.md) subsystem and serves as the single source of truth for the execution plan, milestones, automated success criteria, and cross‑team integration points.

Goal: deliver a cross‑platform, high‑performance, user‑space filesystem with snapshots, writable branches, and per‑process branch binding, usable by AW across Linux, macOS, and Windows.

Approach: Build a reusable Rust core (`agentfs-core`) with a strict API and strong test harnesses. Provide thin platform adapters (FUSE/libfuse, WinFsp, FSKit) that delegate semantics to the core and expose a small, versioned control plane for CLI and tools. Land functionality in incremental milestones with CI gates and platform‑specific acceptance suites.

### Crate and component layout (parallel tracks)

- crates/agentfs-core: Core VFS, snapshots, branches, storage (CoW), locking, xattrs/ADS, events.
- crates/agentfs-proto: JSON schemas, request/response types, validation helpers, error mapping.
- crates/agentfs-fuse-host: libfuse high‑level host for Linux/macOS dev; `.agentfs/control` ioctl.
- crates/agentfs-winfsp-host: WinFsp host mapping `FSP_FILE_SYSTEM_INTERFACE` to core; DeviceIoControl control path.
- xcode/AgentFSKitExtension: FSKit Unary File System extension bridging to core via C ABI; XPC control path.
- tools/agentfs-smoke: Cross‑platform smoke test binary to mount, exercise basic ops, and validate control plane.
- tests/: Core unit/component/integration suites + adapter acceptance suites.

All crates target stable Rust. Platform‑specific hosts are conditionally compiled or built under platform CI.

### Milestones and tasks (with automated success criteria)

**M1. Project Bootstrap** COMPLETED (1–2d)

- **Deliverables**:
  - Initialize Cargo workspace and scaffolding for `agentfs-core`, `agentfs-proto`, adapter crates, and tests.
  - Set up CI: build + test on Linux/macOS/Windows; clippy, rustfmt, coverage (grcov/llvm-cov).
  - Success criteria: CI runs `cargo build` and a minimal `cargo test` on all platforms, with lints and formatting enforced.

- **Implementation Details**:
  - Created Cargo workspace structure with 5 AgentFS crates: `agentfs-core`, `agentfs-proto`, `agentfs-fuse-host`, `agentfs-winfsp-host`, `agentfs-ffi`
  - Implemented core type definitions from AgentFS-Core.md: `FsConfig`, `FsError`, `CaseSensitivity`, `MemoryPolicy`, `FsLimits`, `CachePolicy`, `Attributes`, `FileTimes`, etc.
  - Added basic control plane message types in `agentfs-proto` crate based on AgentFS Control Messages.md
  - Created C ABI surface in `agentfs-ffi` with proper error mappings and function signatures
  - Set up platform-specific host crates with conditional dependencies (FUSE for Linux/macOS, WinFsp for Windows)
  - Added minimal unit tests in `agentfs-core` demonstrating config creation and error handling
  - All crates compile successfully with `cargo check` and pass `cargo test`, `cargo clippy`, and `cargo fmt`

- **Key Source Files**:
  - `crates/agentfs-core/src/lib.rs` - Main library interface and re-exports
  - `crates/agentfs-core/src/config.rs` - Configuration types and policies
  - `crates/agentfs-core/src/error.rs` - Error type definitions
  - `crates/agentfs-core/src/types.rs` - Core data structures (IDs, attributes, etc.)
  - `crates/agentfs-proto/src/messages.rs` - Control plane message types
  - `crates/agentfs-ffi/src/c_api.rs` - C ABI definitions and stubs
  - `crates/agentfs-fuse-host/src/main.rs` - FUSE host binary scaffolding
  - `crates/agentfs-winfsp-host/src/main.rs` - WinFsp host binary scaffolding

- **Verification Results**:
  - [x] CI builds succeed on Linux, macOS, Windows
  - [x] `cargo test` runs at least one core unit test per platform
  - [x] clippy + rustfmt gates enabled and passing

**M2. Core VFS skeleton and in‑memory storage** COMPLETED (3–5d)

- Implement minimal path resolver, directories, create/open/read/write/close, unlink/rmdir, getattr/set_times.
- Provide `InMemoryBackend` storage and `FsConfig`, `OpenOptions`, `Attributes` types as specified in [AgentFS-Core.md](AgentFS%20Core.md).
- Success criteria (unit tests):
  - Create/read/write/close round‑trip works; metadata times update; readdir lists contents.
  - Unlink exhibits delete‑on‑close semantics at core level.

**Implementation Details:**

- Implemented core data structures: `FsCore`, `Node`, `Handle`, `Branch` with internal node management
- Added `InMemoryBackend` with content-addressable storage and basic COW operations (clone_cow, seal)
- Implemented path resolution with proper parent/child relationship tracking
- Added handle management with delete-on-close semantics for unlink operations
- Basic directory operations (mkdir, rmdir, readdir) with proper empty-directory checks
- File operations (create, open, read, write, close) with permission checking
- Metadata operations (getattr, set_times) with timestamp updates

**Key Source Files:**

- `crates/agentfs-core/src/vfs.rs` - Main VFS implementation and FsCore
- `crates/agentfs-core/src/storage.rs` - InMemoryBackend storage implementation
- `crates/agentfs-core/src/types.rs` - Core type definitions (OpenOptions, ContentId, etc.)

**Verification Results:**

- [x] U1 Create/Read/Write passes - Round-trip create/write/close/open/read verified
- [x] U2 Delete-on-close passes - Unlink marks handles deleted, cleanup on last close
- [x] Readdir lists expected entries after create/rename/unlink - Directory operations validated

**M3. Copy‑on‑Write content store and snapshots** COMPLETED (4–6d)

- Implement chunked content store with refcounts and `clone_cow` mechanics; seal snapshots immutable.
- Implement `snapshot_create`, `snapshot_list`, `snapshot_delete`; persistent directory tree nodes per snapshot.
- Success criteria (unit tests + property tests):
  - Snapshot immutability preserved under concurrent writes on branches.
  - Path‑copy on write maintains sharing and bounded memory growth.

**Implementation Details:**

- Implemented content-addressable storage with reference counting and CoW mechanics in `InMemoryBackend`
- Added `clone_cow` and `seal` methods for content management
- Implemented snapshot and branch data structures with ULID-based identifiers
- Added snapshot creation, listing, and deletion with dependency checking
- Implemented branch creation from snapshots and current state
- Added process-scoped branch binding (basic implementation)
- Implemented content-level CoW in write operations for branches created from snapshots
- Added comprehensive unit tests for snapshot immutability and branch operations

**Key Source Files:**

- `crates/agentfs-core/src/storage.rs` - CoW storage backend implementation
- `crates/agentfs-core/src/vfs.rs` - Snapshot and branch management
- `crates/agentfs-core/src/types.rs` - SnapshotId, BranchId, BranchInfo types

**Verification Results:**

- [x] U3 Snapshot immutability passes - Snapshots preserve original content
- [x] Basic CoW invariants pass - Content is cloned on write for snapshot branches
- [x] Branch and snapshot operations work correctly

**M4. Branching and process‑scoped binding** COMPLETED (4–5d)

- Implement `branch_create_from_snapshot`, `branch_create_from_current`, branch listing, and process→branch map.
- Expose `bind_process_to_branch` and `unbind_process` with PID‑aware context.
- Success criteria (unit + scenario tests):
  - Two bound processes see divergent contents for identical absolute paths.
  - Handles opened before binding switch remain stable per invariants.

**Implementation Details:**

- Implemented per-process branch binding with `process_branches: HashMap<u32, BranchId>` mapping PIDs to branches
- Modified all filesystem operations (`resolve_path`, `write`, `snapshot_create`, `branch_create_from_current`) to use `current_branch_for_process()` instead of global branch state
- Implemented recursive CoW cloning for branch creation to ensure complete isolation between branches and snapshots
- Added `bind_process_to_branch_with_pid` and `unbind_process_with_pid` methods for explicit PID-based binding
- Ensured handles remain stable by referencing specific `node_id`s independent of branch context

**Key Source Files:**

- `crates/agentfs-core/src/vfs.rs` - Process binding implementation and branch isolation
- `crates/agentfs-core/src/lib.rs` - Unit tests for process isolation and handle stability

**Verification Results:**

- [x] U4 Branch process isolation passes - Different processes bound to different branches see different content for same paths
- [x] Handle stability verified by opening pre-bind and post-bind - Handles maintain correct node references across binding changes

M5. Locking, share modes, xattrs, and ADS (5–8d)

- Add byte‑range locks and Windows share mode admission logic; open handle tables.
- Implement xattrs and ADS surface (`streams_list`, `OpenOptions.stream`).
- Success criteria:
  - POSIX lock tests for overlapping ranges; flock semantics where applicable.
  - Windows share mode admission tests (hosted via WinFsp adapter component tests).
  - xattr/ADS round‑trip unit tests.

Acceptance checklist (M5)

- [ ] U5 Xattrs/ADS basic flows pass
- [ ] U6 POSIX locks conflict matrix passes
- [ ] Windows share mode admission validated via adapter test harness

M6. Events, stats, and caching knobs (2–3d)

- Add event subscription (`EventSink`), stats reporting, and cache policy mapping (readdir+, attr/entry TTLs).
- Success criteria:
  - Event emission on create/remove/rename and branch/snapshot ops validated by unit tests.
  - Readdir+ returns attributes without extra getattr calls in adapter harness.

Acceptance checklist (M6)

- [ ] Event subscription receives create/remove/rename and snapshot/branch events
- [ ] Stats report non-zero counters after representative workload

M7. FUSE adapter host (Linux/macOS dev path) (4–6d)

- Implement libfuse high‑level `struct fuse_operations` mapping to core; support `.agentfs/control` ioctl.
- Provide mount binary for tests; map cache knobs to `fuse_config`.
- Success criteria (integration):
  - Mounts on Linux CI; libfuse example ops pass; snapshot/branch/bind via control file works.
  - pjdfstests subset green; readdir+ validated; basic fsbench throughput measured.

Acceptance checklist (M7)

- [ ] I1 FUSE host basic ops pass on CI
- [ ] I2 Control plane ioctl flows pass with schema validation
- [ ] pjdfstests subset green (documented list)

M8. WinFsp adapter host (Windows) (5–8d)

- Map `FSP_FILE_SYSTEM_INTERFACE` ops; implement DeviceIoControl control plane.
- Implement share modes, delete‑on‑close, ADS enumeration.
- Success criteria (integration):
  - winfsp `memfs` parity for create/open/read/write/rename/unlink; `winfstest` and `IfsTest` critical cases pass.
  - `GetStreamInfo` returns ADS; delete‑on‑close behaves per tests; control ops functional.

Acceptance checklist (M8)

- [ ] I3 WinFsp basic ops pass
- [ ] WinFsp test batteries: core subsets pass; exceptions documented
- [ ] DeviceIoControl control ops pass schema validation

M9. FSKit adapter (macOS 15+) (6–10d)

- Build FSKit Unary File System extension; bridge to core via C ABI; provide XPC control.
- Success criteria (integration):
  - Mounts on macOS CI or local; file/basic directory ops pass; control ops functional.
  - Case‑insensitive‑preserving names honored by default; xattrs round‑trip for quarantine/FinderInfo.

Acceptance checklist (M9)

- [ ] I4 FSKit adapter smoke tests pass locally/CI lane
- [ ] XPC control plane snapshot/branch/bind functions
- [ ] FinderInfo/quarantine xattrs round-trip validated

M10. Control plane and CLI integration (3–5d)

- Finalize `agentfs-proto` JSON schemas (already spec’d) and validators; generate Rust types.
- Implement `aw agent fs` subcommands invoking platform transport: DeviceIoControl (Windows), ioctl on control file (FUSE), XPC (FSKit).
- Success criteria (CLI tests):
  - `aw agent fs snapshot create/list` and `branch create/bind/exec` behave as specified across platforms.
  - Schema validation enforced; informative errors on invalid payloads.

Acceptance checklist (M10)

- [ ] `aw agent fs snapshot create/list` passes against FUSE/WinFsp/FSKit
- [ ] `branch create/bind/exec` passes including PID binding resolution
- [ ] Requests validated against schemas; error mapping covered by tests

M11. Scenario, performance, and fault‑injection suites (4–7d)

- Scenario tests for AW workflows (per [AgentFS-Core-Testing.md](AgentFS%20Core%20Testing.md)): multi‑process branches, repo tasks, discard/keep flows.
- Criterion microbenchmarks; fsbench/fio macro tests; spill‑to‑disk stress; induced failures in `StorageBackend`.
- Success criteria:
  - Latency/throughput comparable to RAM memfs baselines; bounded degradation with spill enabled.
  - Fault injection does not violate core invariants; linearizable API boundaries maintained.

Acceptance checklist (M11)

- [ ] S1 Branch-per-task scenario passes end-to-end
- [ ] P1 microbenchmark baseline within target factors; thresholds documented
- [ ] R1/R2 reliability plans pass (spill ENOSPC; crash safety)

M12. Packaging, docs, and stability gates (2–3d)

- Package adapter hosts; document setup for libfuse/macFUSE, WinFsp, and FSKit extension.
- Stabilize public API and C ABI; version crates; document upgrade/versioning policy for control plane.
- Success criteria:
  - Reproducible build artifacts; documented installation for each platform; examples runnable end‑to‑end.

Acceptance checklist (M12)

- [ ] Reproducible builds documented and verified in CI artifacts
- [ ] Platform setup docs validated via smoke scripts
- [ ] Public API/ABI versioned; upgrade notes published

### Test strategy & tooling

- Core: `cargo test` unit/property tests; mutation tests on critical modules; structured tracing behind a feature.
- Component: FFI surface exercised via a small C harness; UTF‑8/UTF‑16 round‑trips.
- Integration: libfuse adapter on Linux/macOS dev; WinFsp batteries on Windows; FSKit sample‑like flows.
- Scenario: AW lifecycle simulations; golden tests for control JSON round‑trip using schemas in `specs/Public/Schemas`.
- Performance: criterion microbenchmarks; fsbench/fio macro; memory spill and ENOSPC coverage.

### Deliverables

- Crates: agentfs-core, agentfs-proto, agentfs-fuse-host, agentfs-winfsp-host.
- FSKit extension target with XPC client/server shim.
- `aw agent fs` CLI subcommands wired to transports and schemas.
- Comprehensive CI matrix and acceptance suites per platform with documented pass/fail gates.

### Risks & mitigations

- Platform API variance (FSKit maturity; WinFsp nuances): feature‑gate and document exceptions; track upstream issues.
- CI limitations for privileged mounts: use dedicated runners and containerized privileged lanes only where required; keep unit/component coverage high.
- Performance regressions under spill: tune chunking, batching, and cache policy; benchmark thresholds enforced in CI with opt‑out for noisy environments.

### Parallelization notes

- M2–M6 (core) can proceed largely in parallel, with clear interfaces; adapters (M7–M9) can start once M3 is stable.
- CLI (M10) can begin after control plane validators land; platform transport shims can be developed with mocks.
- Performance/fault suites (M11) can evolve alongside adapters; stabilize criteria before M12.

### References

- See [specs/Public/AgentFS/AgentFS-Core.md](AgentFS%20Core.md), [AgentFS-FUSE-Adapter.md](AgentFS%20FUSE%20Adapter.md), [AgentFS-WinFsp-Adapter.md](AgentFS%20WinFsp%20Adapter.md), [AgentFS-FSKit-Adapter.md](AgentFS%20FsKit%20Adapter.md), and [AgentFS-Control-Messages.md](AgentFS%20Control%20Messages.md).
- Reference code in `reference_projects/libfuse`, `reference_projects/winfsp`, and `reference_projects/FSKitSample`.
