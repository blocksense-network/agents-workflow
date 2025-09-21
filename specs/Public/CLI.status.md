### Overview

Goal: deliver a production-ready `aw` CLI/TUI that matches `specs/Public/CLI.md`, ships cross-platform binaries, and verifies every major capability with automated tests.

### Approach

- Build a Rust workspace with reusable crates (`aw-core`, `aw-cli`, `aw-tui`) and trait-based adapters so local and remote flows share logic.
- Treat the daemon, sandbox supervisor, and REST client as services with contract tests and fuzz/snapshot coverage where applicable.
- Enforce documentation parity by snapshot-testing generated `--help` and man pages against the spec.

### Milestones (automated verification)

**M1. Workspace bootstrap**

- Deliverables: Rust workspace, Clap command skeletons, config loader, initial CI.
- Verification: `cargo test` (unit skeletons), `cargo fmt --check`, `cargo clippy` (CI gate), snapshot tests capturing `aw --help` vs golden file.

**M2. Core task/session commands**

- Deliverables: `aw task`, `aw session <list|attach|logs|run>`, local SQLite state management.
- Verification: integration tests with temporary Git repos using `assert_cmd`; golden snapshots for `--json` outputs; property tests ensuring branch/task file workflows round-trip.

**M3. Repo automation**

- Deliverables: `aw repo <init|instructions|link>`, prompt template engine, template configuration keys.
- Verification: integration tests that scaffold mock repos, validate generated files, and diff expected symlinks; snapshot of prompt JSON; lint checks run via `just lint-specs` on generated repos.

**M4. Daemon & access point parity**

- Deliverables: shared `aw daemon run`, wrappers `aw webui`, `aw agent access-point`, `aw agent enroll`.
- Verification: platform-targeted tests using `cargo test --features macos` (launchctl mocked), systemd user session test harness with `systemd-run --user`, Windows service tests via GitHub Actions Windows runner; health-check integration verifying QUIC handshake against a test server.

**M5. Sandbox supervisor integration**

- Deliverables: sandbox helper crate bindings, audit event stream, `aw session audit` CLI.
- Verification: Linux-only integration tests using rootless namespaces; golden audit log snapshots; resource limit tests (fork-bomb, memory cap) executed under CI privileged job; regression for audit streaming through REST stub.

**M6. Packaging & cross-platform release**

- Deliverables: Release pipelines (macOS universal, Linux x86_64/aarch64, Windows), man pages, shell completions.
- Verification: GitHub Actions release matrix builds; codesign/notarization smoke test on macOS; `winget validate` & `sudo apt-get install` (local repo) for Linux; release artifacts verified via `shasum` in CI; auto-update of completions validated with snapshot tests.

### Test & QA strategy

- Maintain `just ci-cli` workflow running unit, integration, lints, and snapshot tests per milestone.
- Use `cargo insta` for doc/help snapshots so spec/CLI drift is caught automatically.
- Nightly end-to-end job: spin up REST service + sandbox stack, execute sample multi-OS task via GitHub Actions macOS runner, assert artifacts and logs published.

### Risks & mitigations

- **Platform differences**: abstract launch mechanics behind traits; record platform-specific fixtures to avoid brittle logic.
- **Command surface drift**: enforce snapshot comparison between spec-generated docs and `clap` output each CI run.
- **Long-running tests**: split privileged sandbox tests into dedicated workflow; parallelize by feature flag to keep CI under 10 minutes.
