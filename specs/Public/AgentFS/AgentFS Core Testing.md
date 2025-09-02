## AgentFS Core — Testing Strategy

### Goals

- Verify functional correctness of the Rust core across all features: files/dirs ops, metadata, xattrs/ADS, locks, snapshots, branches, and process‑scoped bindings.
- Validate platform semantics parity: POSIX (Linux/macOS) and NTFS (Windows) behaviors via glue‑agnostic unit tests and glue‑backed integration tests.
- Ensure performance characteristics (latency/throughput) meet targets under representative workloads.
- Prove robustness under concurrency, failure injection, and resource pressure (memory spill, ENOSPC).

### Test Layers

1. Unit tests (Rust, core‑only)

- Scope: Pure in‑process testing of `FsCore`, VFS operations, snapshot engine, storage backends, locking, xattrs, streams.
- Tools: `cargo test` with `#[cfg(test)]`, property tests (quickcheck/proptest) for path normalization and CoW invariants.
- Fault injection: simulate I/O failures from `StorageBackend`, forced allocation failures, artificial delays.

2. Component tests (Rust, minimal glue shims)

- Scope: Exercise the C ABI (FFI) surface to validate ABI stability and error mapping.
- Tools: `ctest` or a small C harness called from Rust (`cc` crate), round‑trip UTF‑8/UTF‑16 name mapping.

3. Integration tests (per‑platform glue)

- Linux/macOS (FUSE/libfuse or macFUSE for early tests):
  - Mount a temp volume backed by AgentFS core.
  - Reuse libfuse example patterns (hello/passthrough variants) to ensure compliance on getattr/readdir/open/read/write/rename/unlink.
  - Run pjdfstests / libfuse tests where feasible.
- Windows (WinFsp):
  - Mount a drive letter using a thin WinFsp host backed by AgentFS core.
  - Run WinFsp’s test batteries: winfstest, IfsTest, fsbench read/write tests, change notification tests.
  - Validate delete‑on‑close, share modes, ADS.

4. Scenario tests (AW workflows)

- Simulate AW ‘task session’ lifecycle: snapshot baseline → branch per task → writes → verify isolation → branch discard/keep.
- Codify per‑process view isolation: two processes bound to different branches operating on same absolute paths get divergent results.
- Browser automation artifacts: ensure xattrs (quarantine, FinderInfo) round‑trip on macOS.

### Test Matrix

- Case sensitivity: Sensitive vs Insensitive‑preserving (macOS/Windows default) trees; conflicting names ("Readme" vs "README").
- Streams & xattrs: presence/absence; large values; list ordering; Unicode names.
- Locks: POSIX record locks (overlapping ranges), BSD flock; Windows share modes and mandatory locks.
- Paths: deep hierarchies; long names; illegal name rejection on Windows (<>:"|?\* and reserved names); Unicode normalization.
- Symlinks & hardlinks: creation, traversal, unlink of one link preserving data when other exists.
- Concurrency: parallel readdir with concurrent creates/renames; lock contention; mixed readers/writers.
- Snapshots/branches:
  - snapshot immutability
  - writable clones diverge
  - whiteouts (deletes) are branch‑local
  - handle stability across snapshot creation
  - many branches (limits), GC of reference‑counted content when branches are deleted
- Storage backends: in‑memory vs temp‑file spill; ENOSPC on spill directory; restart behavior (ephemeral semantics).

### Representative Unit Tests (Rust)

```rust
#[test]
fn create_read_write_roundtrip() {
    let core = test_core();
    core.mkdir("/dir".as_ref(), 0o755).unwrap();
    let h = core.create("/dir/a.txt".as_ref(), &rw_create()).unwrap();
    core.write(h, 0, b"hello").unwrap();
    core.close(h).unwrap();
    let h2 = core.open("/dir/a.txt".as_ref(), &ro()).unwrap();
    let mut buf = [0u8; 5];
    let n = core.read(h2, 0, &mut buf).unwrap();
    assert_eq!(n, 5);
    assert_eq!(&buf, b"hello");
}

#[test]
fn snapshot_immutable_branch_writable() {
    let core = test_core();
    core.create("/f".as_ref(), &rw_create()).unwrap();
    let snap = core.snapshot_create(Some("base")).unwrap();
    let b = core.branch_create_from_snapshot(snap, Some("b")).unwrap();
    core.bind_process_to_branch(b).unwrap();
    let h = core.open("/f".as_ref(), &rw()).unwrap();
    core.write(h, 0, b"branch").unwrap();
    core.close(h).unwrap();
    // Reading via snapshot should still see empty/original
    // (accessed via a helper that resolves by snapshot id)
    assert_eq!(read_through_snapshot(&core, snap, "/f"), b"");
}

#[test]
fn unlink_delete_on_close_semantics() {
    let core = test_core();
    let h = core.create("/x".as_ref(), &rw_create()).unwrap();
    core.unlink("/x".as_ref()).unwrap();
    // Still readable until close
    let mut b = [0; 0];
    core.read(h, 0, &mut b).unwrap();
    core.close(h).unwrap();
    assert!(core.open("/x".as_ref(), &ro()).is_err());
}
```

### Adapter/Glue Integration Tests

- FUSE Host (Linux/macOS dev):
  - Build a minimal binary linking libfuse high‑level API and calling into `FsCore`.
  - Run libfuse `example/` ops and test suite: getattr/readdir/open ioctls; readdir+ path; cache toggles.

- WinFsp Host (Windows CI):
  - Build a host (similar to WinFsp MEMFS) delegating to `FsCore`.
  - Execute: `winfsp-tests`, `winfstest`, `IfsTest`, `fsbench`.
  - Validate ADS (`file:stream`), change notifications, delete‑on‑close.

### Performance Tests

- Microbenchmarks (criterion): small file create/delete, sequential RW (1MB–1GB), random RW, stat heavy ops.
- Macro tests (fsbench/fio): throughput and p99 latency under concurrency; with/without writeback cache; readdir+ on large directories.
- Memory pressure: tune `max_bytes_in_memory`; verify spill path IO; ensure bounded memory growth with steady workload.

### Reliability & Fault Injection

- Force `StorageBackend` to return transient and permanent errors; verify error mapping and recovery.
- Simulate thread preemption under lock contention; deadlock detection by timeouts on CI.
- Abrupt termination: kill host during sustained writes; on restart, verify no core invariants broken (ephemeral state).

### CI & Environments

- Linux (Ubuntu): run unit, component, and FUSE integration tests (with `--privileged` when needed for mount in CI).
- macOS: unit tests + macFUSE/libfuse integration (until FSKit host is available); later add FSKit extension tests with Xcode build.
- Windows: GitHub Actions or self‑hosted runner with WinFsp installed; run WinFsp test suites and fsbench.

### Coverage & Quality Gates

- Line and branch coverage thresholds for core modules (storage, vfs, snapshots, locking) ≥ 85%.
- Mutation testing for critical path (metadata ops, CoW layer).
- Lints: `clippy` (pedantic where reasonable), `rustfmt` stable; no unsafe in public API; unsafe internals reviewed.

### Artifacts and Debugging Aids

- Deterministic test seeds and reproducible fixtures.
- Structured tracing (feature‑gated) for snapshot/branch ops, lock decisions, and spill events.
- Dump utilities to serialize tree roots and refcounts for failure triage.

### Exit Criteria

- All unit/component/integration suites green across platforms.
- WinFsp core test battery passes (with allowed exceptions documented).
- libfuse example/behavioral tests show parity for expected operations; readdir+ validated.
- Performance targets met: comparable to memfs baselines for RAM‑backed workloads; bounded degradation when spilling.
