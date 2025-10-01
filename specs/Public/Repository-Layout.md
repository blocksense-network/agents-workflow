## Repository Layout (post‑Rust migration)

This document defines the ideal repository structure after the migration to Rust, aligning with CLI and sandbox specs in [CLI.md](CLI.md) and [Sandbox-Profiles.md](Sandbox-Profiles.md). It emphasizes library‑first design, thin binaries, strong testability, and clear platform boundaries, while temporarily preserving the existing Ruby implementation under `legacy/` and keeping current cloud setup scripts at their present paths.

### Principles

- Libraries first; binaries are thin entry points.
- Clear prefixes per domain: `aw-*` (application), `agentfs-*` (filesystem), `sandbox-*` (isolation).
- Cross‑platform adapters are minimal shims to a shared Rust core.
- Specs remain authoritative in `specs/`; runtime JSON schemas are vendored under `schemas/` for code to consume.
- Temporary coexistence: legacy Ruby kept intact under `legacy/` during transition.

### Top‑level layout

```text
agents-workflow/
├─ Cargo.toml                  # [workspace] with all crates
├─ rust-toolchain.toml         # pinned toolchain
├─ .cargo/config.toml          # linker/rpath, per-target cfgs
├─ Justfile
├─ flake.nix / flake.lock      # Nix dev shells and CI builds
├─ .devcontainer/              # Devcontainer definitions
├─ .github/workflows/          # CI: build, test, lint, package
├─ specs/                      # Product/spec documents (source of truth)
│  └─ Public/
├─ docs/                       # Developer docs (how-to, runbooks)
├─ scripts/                    # Small repo scripts (non-build)
├─ schemas/                    # JSON schemas used at runtime (mirrors specs/Public/Schemas)
│  └─ agentfs/
├─ tests/                      # Cross-crate integration & acceptance tests
│  ├─ integration/
│  ├─ acceptance/
│  └─ fixtures/
├─ examples/                   # Small runnable examples per subsystem
├─ apps/                       # Platform-specific application bundles
│  └─ macos/
│     └─ AgentsWorkflow/              # Main macOS application (AgentsWorkflow.app)
│        ├─ AgentsWorkflow.xcodeproj/ # Xcode project for main app
│        ├─ AgentsWorkflow/           # Main app source (SwiftUI/AppKit)
│        │  ├─ AppDelegate.swift
│        │  ├─ MainMenu.xib
│        │  └─ Info.plist
│        └─ PlugIns/                  # Embedded system extensions (PlugIns/)
│           └─ AgentFSKitExtension.appex/ # FSKit filesystem extension bundle
├─ adapters/
│  └─ macos/
│     └─ xcode/
│        ├─ AgentFSKitExtension/      # FSKit filesystem extension source code
│        │  ├─ AgentFSKitExtension/   # Extension source files
│        │  │  ├─ AgentFsUnary.swift
│        │  │  ├─ AgentFsVolume.swift
│        │  │  ├─ AgentFsItem.swift
│        │  │  └─ Constants.swift
│        │  ├─ Package.swift          # Swift Package Manager configuration
│        │  └─ build.sh               # Build script for Rust FFI integration
│        └─ (legacy Swift package artifacts and build scripts)
├─ webui/                      # WebUI and related JavaScript/Node.js projects
│  ├─ app/                     # Main SolidStart WebUI
│  ├─ mock-server/             # Mock REST API server for development/testing
│  ├─ e2e-tests/               # Playwright E2E test suite
│  └─ shared/                  # Shared utilities and types between WebUI components
├─ bins/                       # Packaging assets/manifests per final binary
│  ├─ aw/                      # CLI packaging, completions, manpages
│  ├─ agentfs-fuse/            # FUSE host packaging (Linux/macOS dev)
│  ├─ agentfs-winfsp/          # WinFsp host packaging (Windows)
│  └─ sbx-helper/              # Sandbox helper packaging
├─ crates/                     # All Rust crates
│  ├─ aw-cli/                  # Bin: `aw` (Clap subcommands; TUI glue)
│  ├─ aw-tui/                  # TUI widgets/flows (Ratatui)
│  ├─ aw-core/                 # Task/session lifecycle orchestration
│  ├─ aw-mux-core/             # Low-level, AW-agnostic multiplexer trait + shared types
│  ├─ aw-mux/                  # Monolith crate: AW adapter + all backends as feature-gated modules
│  ├─ aw-config/               # Layered config + flag mapping
│  ├─ aw-state/                # Local state (SQLite models, migrations)
│  ├─ aw-repo/                 # VCS operations (Git/Hg/Bzr/Fossil)
│  ├─ aw-rest-api-contract/    # Schema types, input validation, etc (shared between mock servers and production server)
│  ├─ aw-rest-client/          # Client for remote REST mode
│  ├─ aw-rest-mock-server/     # Mock REST API server for development and testing
│  ├─ aw-rest-server/          # Optional local REST service (lib + bin)
│  ├─ aw-connectivity/         # SSH, relays, followers, rendezvous
│  ├─ aw-notify/               # Cross-platform notifications
│  ├─ aw-fleet/                # Multi-OS fleet orchestration primitives
│  ├─ aw-workflows/            # Workflow expansion engine (`/cmd`, dynamic instructions)
│  ├─ aw-schemas/              # Load/validate JSON schemas (e.g., AgentFS control)
│  ├─ agentfs-core/            # Core FS: VFS, CoW, snapshots/branches, locks, xattrs/ADS
│  ├─ agentfs-proto/           # Control plane types + validators
│  ├─ agentfs-fuse-host/       # Bin: libfuse host → `agentfs-core`
│  ├─ agentfs-winfsp-host/     # Bin: WinFsp host → `agentfs-core`
│  ├─ agentfs-ffi/             # C ABI (FFI) for FSKit/Swift bridging
│  ├─ sandbox-core/            # Namespaces/lifecycle/exec
│  ├─ sandbox-fs/              # Mount planning (RO seal, overlays)
│  ├─ sandbox-seccomp/         # Dynamic read allow-list (seccomp notify)
│  ├─ sandbox-cgroups/         # cgroup v2 limits + metrics
│  ├─ sandbox-net/             # Loopback/slirp/veth; nftables glue
│  ├─ sandbox-proto/           # Helper⇄supervisor protocol types
│  ├─ sbx-helper/              # Bin: PID 1 inside sandbox; composes sandbox-* crates
│  ├─ aw-agent-runner/         # Bin/lib: `aw agent record` (asciinema integration)
│  └─ platform-helpers/        # Per-OS helpers (paths, perms, names)
├─ legacy/                     # Temporary home for the Ruby implementation
│  └─ ruby/
│     ├─ bin/                  # existing Ruby entrypoints (kept intact)
│     ├─ lib/
│     ├─ test/
│     ├─ Gemfile / *.gemspec
│     └─ README.md
├─ bin/                        # Thin wrappers/launchers (may exec Rust bins)
└─ (root scripts preserved; see below)
```

### macOS Host Application Architecture

The `apps/macos/AgentsWorkflow/` directory contains the main **AgentsWorkflow.app** - the primary macOS application for the entire Agents Workflow project. This app serves as a container for multiple system extensions and provides the main user interface for all AW functionality on macOS. This design follows Apple's system extension architecture where privileged components (like filesystem extensions) must be embedded within a host application for proper registration and lifecycle management.

#### Host App Responsibilities
- **Extension Hosting**: Contains and manages multiple system extensions (currently AgentFSKitExtension)
- **User Interface**: Provides minimal UI for extension management and status monitoring
- **Extension Registration**: Handles PlugInKit registration for embedded extensions
- **Lifecycle Management**: Manages extension loading, unloading, and system approval workflows

#### Extension Architecture
- **AgentFSKitExtension**: FSKit-based filesystem extension for user-space AgentFS implementation
- **Extension Sources**: Extension source code is developed in `adapters/macos/xcode/AgentFSKitExtension/`
- **Built Extensions**: Compiled extensions are embedded in the host app's `PlugIns/` directory
- **Future Extensions**: Additional system extensions (network filters, device drivers, etc.) will follow the same pattern

#### Build and Distribution
- Built as a standard macOS application bundle with embedded appex (extension) bundles
- Requires code signing and notarization for system extension approval
- Distributed as a single `.app` bundle containing all extensions
- Uses universal binaries for Intel + Apple Silicon compatibility

### Crate mapping (selected)

- CLI/TUI: `aw-cli`, `aw-tui`, `aw-core`, `aw-config`, `aw-state`, `aw-repo`, `aw-workflows`, `aw-rest-client`, `aw-notify`, `aw-fleet`, `aw-agent-executor`, `aw-schemas`.
- AgentFS: `agentfs-core`, `agentfs-proto`, `agentfs-fuse-host`, `agentfs-winfsp-host`, `agentfs-ffi`.
- Sandbox (Local profile): `sandbox-core`, `sandbox-fs`, `sandbox-seccomp`, `sandbox-cgroups`, `sandbox-net`, `sandbox-proto`, `sbx-helper`.

### WebUI structure

- `webui/app/` — Main SolidJS application with server-side rendering support through SolidStart
- `webui/mock-server/` — Mock REST API server implementing the full REST-Service.md specification for development and testing
- `webui/e2e-tests/` — Playwright E2E test suite with pre-scripted scenarios controlling both mock server and UI interactions
- `webui/shared/` — Shared TypeScript utilities, API client code, and type definitions used across WebUI components

### Multiplexer crates structure

We use the [subcrates design pattern](Subcrates-Pattern.md) with a **monolith + facades strategy** to reduce compile times while preserving optional tiny crates:

- `aw-mux-core` — low‑level AW‑agnostic trait and shared types (no OS bindings).
- `aw-mux` (monolith) — contains the high‑level AW adapter and all concrete backends as modules gated by cargo features (e.g., `tmux`, `wezterm`, `kitty`, `iterm2`, `tilix`, `winterm`, `vim`, `emacs`). Only requested features are compiled.
- Optional facade crates (tiny re‑exports) to keep per‑backend packages when desired:
  - `aw-mux-tmux` depends on `aw-mux` with `features=["tmux"]` and `default-features=false`, then `pub use aw_mux::tmux::*;`
  - Same for `aw-mux-wezterm`, `aw-mux-kitty`, …

Usage

- Apps can depend directly on `aw-mux` and request the union of backends they need, compiling the monolith once.
- Or depend on multiple facades; cargo feature unification compiles `aw-mux` once with the union of features.

Why this helps

- One heavy compilation unit: all codegen happens in `aw-mux` once, even if multiple backends are used together.
- Keep or publish tiny crates: facades compile in milliseconds and maintain package boundaries.
- Flexible consumption: choose single‑dep monolith or per‑backend facades without N× compile cost.

Gotchas

- Visibility changes: code moved under `aw-mux` modules; adjust `pub(crate)`/paths accordingly.
- Proc‑macro crates cannot be merged; not applicable here.
- Tests/examples may need to move into `aw-mux/tests/` or remain in facades if they rely on crate boundaries.

Extra compile‑time wins

- Unify dependency versions/features across the workspace (consider a workspace‑hack crate).
- Use sccache/`RUSTC_WRAPPER` and check `CARGO_BUILD_TIMINGS` to validate improvements.

See [CLI.md](CLI.md) for command surface and [Sandbox-Profiles.md](Sandbox-Profiles.md) for isolation profiles and behavior.

### Cloud setup scripts (paths preserved)

The following existing setup scripts remain at the repository root to preserve current tooling and docs:

- `codex-setup`
- `copilot-setup`
- `jules-setup`
- `goose-setup`

Notes:

- These scripts are considered external helpers and may call into Rust binaries as migration proceeds.
- Additional provider scripts (if added later) should also live at the repository root for consistency.

### Legacy Ruby

- All current Ruby code is retained under `legacy/ruby/` without restructuring to minimize churn during migration.
- Existing Ruby `bin/` entrypoints are duplicated here; top‑level `bin/` may be thin shims that exec Rust `aw` as features roll over.
- Tests continue to run under `legacy/ruby/test/` until replaced by Rust acceptance tests under `tests/`.

### Testing and CI

- Unit tests live within each crate; cross‑crate tests in `tests/` mirror acceptance plans in AgentFS and CLI specs.
- CI fans out per crate (build/test/lint) and runs privileged lanes only where necessary (FUSE/WinFsp/FSKit).
