#
# Nix Dev Shell Policy (reproducibility)
# -------------------------------------
# When running inside the Nix dev shell (environment variable `IN_NIX_SHELL` is set),
# Just tasks and helper scripts MUST NOT use fallbacks such as `npx`, brew installs,
# network downloads, or any ad-hoc tool bootstrap. If a required command is missing
# in that context, the correct fix is to add it to `flake.nix` (devShell.buildInputs)
# and re-enter the shell, not to fall back. Outside of the Nix shell, tasks may use
# best-effort fallbacks for convenience, but scripts should gate them like:
#   if [ -n "$IN_NIX_SHELL" ]; then echo "missing <tool>; fix flake.nix" >&2; exit 127; fi
# This keeps `nix develop` fully reproducible and prevents hidden network variability.

# Run the test suite

set shell := ["./scripts/nix-env.sh", "-c"]

# IMPORTANT: Never use long scripts in Justfile recipes!
# Long scripts set a custom shell, overriding our nix-env.sh setting.
# Move complex scripts to the scripts/ folder instead.

# Check Rust code for compilation errors
check:
    cargo check --workspace

# Run Rust tests
test-rust:
    cargo test --workspace

# Run Rust tests with verbose output
test-rust-verbose:
    cargo test --workspace --verbose

# Lint Rust code
lint-rust:
    cargo clippy --workspace

# Format Rust code
fmt-rust:
    cargo fmt --all --check

# Build release binary for sbx-helper
build-sbx-helper-release:
    cargo build --release --bin sbx-helper

legacy-test:
    export RUBYLIB=legacy/ruby/lib && ruby -Ilegacy/ruby/test legacy/ruby/test/run_tests_shell.rb

# Run codex-setup integration tests (Docker-based)
legacy-test-codex-setup-integration:
    ./setup-tests/test-runner.sh

# Run only snapshot-related tests (ZFS, Btrfs, and Copy providers)
legacy-test-snapshot:
    RUBYLIB=legacy/ruby/lib ruby scripts/run_snapshot_tests.rb

# Lint the Ruby codebase
legacy-lint:
    rubocop legacy/ruby

# Auto-fix lint issues where possible
legacy-lint-fix:
    rubocop --autocorrect-all legacy/ruby

# Build and publish the gem
legacy-publish-gem:
    gem build legacy/ruby/agent-task.gemspec && gem push agent-task-*.gem

# Validate all JSON Schemas with ajv (meta-schema compile)
conf-schema-validate:
    scripts/conf-schema-validate.sh

# Check TOML files with Taplo (uses schema mapping if configured)
conf-schema-taplo-check:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v taplo >/dev/null 2>&1; then
        echo "taplo is not installed. Example to run once: nix shell ~/nixpkgs#taplo -c taplo check" >&2
        exit 127
    fi
    taplo check

# Serve schema docs locally with Docson (opens http://localhost:3000)
conf-schema-docs:
    docson -d specs/schemas

# Validate Mermaid diagrams in Markdown with mermaid-cli (mmdc)
md-mermaid-check:
    bash scripts/md-mermaid-validate.sh specs/**/*.md

# Lint Markdown structure/style in specs with markdownlint-cli2
md-lint:
    markdownlint-cli2 "specs/**/*.md"

# Check external links in Markdown with lychee
md-links:
    lychee --config .lychee.toml --accept "200..=299,403" --no-progress --require-https --max-concurrency 8 "specs/**/*.md"

# Spell-check Markdown with cspell (uses default dictionaries unless configured)
md-spell:
    cspell "specs/**/*.md"

# Create reusable file-backed filesystems for testing ZFS and Btrfs providers
# This sets up persistent test environments in ~/.cache/agents-workflow
create-test-filesystems:
    scripts/create-test-filesystems.sh

# Check the status of test filesystems
check-test-filesystems:
    scripts/check-test-filesystems.sh

# Clean up test filesystems created by create-test-filesystems
cleanup-test-filesystems:
    scripts/cleanup-test-filesystems.sh

# Launch the AW filesystem snapshots daemon for testing (requires sudo)
legacy-start-aw-fs-snapshots-daemon:
    legacy/scripts/launch-aw-fs-snapshots-daemon.sh

# Stop the AW filesystem snapshots daemon
legacy-stop-aw-fs-snapshots-daemon:
    legacy/scripts/stop-aw-fs-snapshots-daemon.sh

# Check status of AW filesystem snapshots daemon
legacy-check-aw-fs-snapshots-daemon:
    ruby legacy/scripts/check-aw-fs-snapshots-daemon.rb

# Launch the new Rust AW filesystem snapshots daemon for testing (requires sudo)
start-aw-fs-snapshots-daemon:
    scripts/start-aw-fs-snapshots-daemon.sh

# Stop the new Rust AW filesystem snapshots daemon
stop-aw-fs-snapshots-daemon:
    scripts/stop-aw-fs-snapshots-daemon.sh

# Check status of the new Rust AW filesystem snapshots daemon
check-aw-fs-snapshots-daemon:
    scripts/check-aw-fs-snapshots-daemon.sh

# Run comprehensive daemon integration tests (requires test filesystems)
test-daemon-integration:
    cargo test --package aw-fs-snapshots-daemon -- --nocapture integration

# Run filesystem snapshot provider integration tests (requires root for ZFS/Btrfs operations)
test-fs-snapshots:
    cargo test --package aw-fs-snapshots -- --nocapture integration

# Run filesystem snapshot provider unit tests only (no root required)
test-fs-snapshots-unit:
    cargo test --package aw-fs-snapshots

# Run all spec linting/validation in one go
lint-specs:
    scripts/lint-specs.sh

# Build cgroup enforcement test binaries (fork_bomb, memory_hog, cpu_burner, test_orchestrator)
build-cgroup-test-binaries:
    cargo build --bin fork_bomb --bin memory_hog --bin cpu_burner --bin test_orchestrator

# Build overlay enforcement test binaries (overlay_test_orchestrator, blacklist_tester, overlay_writer)
build-overlay-test-binaries:
    cargo build --bin overlay_test_orchestrator --bin blacklist_tester --bin overlay_writer

# Build sbx-helper binary
build-sbx-helper:
    cargo build -p sbx-helper --bin sbx-helper

# Build all test binaries needed for cgroup enforcement tests
build-cgroup-tests: build-sbx-helper build-cgroup-test-binaries

# Build all test binaries needed for overlay enforcement tests
build-overlay-tests: build-sbx-helper build-overlay-test-binaries

# Build network enforcement test binaries (network_test_orchestrator, curl_tester, port_collision_tester)
build-network-test-binaries:
    cargo build --bin network_test_orchestrator --bin curl_tester --bin port_collision_tester

# Build all test binaries needed for network enforcement tests
build-network-tests: build-sbx-helper build-network-test-binaries

# Build debugging enforcement test binaries (debugging_test_orchestrator, ptrace_tester, process_visibility_tester, mount_test)
build-debugging-test-binaries:
    cargo build -p debugging-enforcement --bin debugging_test_orchestrator --bin ptrace_tester --bin process_visibility_tester --bin mount_test

# Build all test binaries needed for debugging enforcement tests
build-debugging-tests: build-sbx-helper build-debugging-test-binaries

# Run cgroup tests with E2E enforcement verification
test-cgroups:
    just build-cgroup-tests
    cargo test -p sandbox-integration-tests --verbose

# WebUI Development Targets
# ========================

# Install dependencies for all WebUI projects
webui-install:
    cd webui/shared && npm ci
    cd webui/app && npm ci
    cd webui/mock-server && npm ci
    cd webui/e2e-tests && npm ci

# Build WebUI application
webui-build:
    cd webui/app && npm run build

# Build mock server
webui-build-mock:
    cd webui/mock-server && npm run build

# Run WebUI development server
webui-dev:
    cd webui/app && npm run dev

# Run mock REST API server
webui-mock-server:
    cd webui/mock-server && npm run dev

# Lint all WebUI projects
webui-lint:
    cd webui/app && npm run lint
    cd ../mock-server && npm run lint
    cd ../e2e-tests && npm run lint

# Type check all WebUI projects
webui-type-check:
    cd webui/app && npm run type-check
    cd ../mock-server && npm run type-check

# Format all WebUI projects
webui-format:
    cd webui/app && npm run format
    cd ../mock-server && npm run format
    cd ../e2e-tests && npm run format

# Run WebUI E2E tests
webui-test:
    cd webui/e2e-tests && npm run test:e2e

# Run WebUI E2E tests in headed mode (visible browser)
webui-test-headed:
    cd webui/e2e-tests && npm run test:headed

# Run WebUI E2E tests in debug mode
webui-test-debug:
    cd webui/e2e-tests && npm run test:debug

# Run WebUI E2E tests in UI mode
webui-test-ui:
    cd webui/e2e-tests && npm run test:ui

# Show WebUI test reports
webui-test-report:
    cd webui/e2e-tests && npm run report

# Install Playwright browsers for E2E tests
webui-install-browsers:
    cd webui/e2e-tests && npm run install-browsers

# Run all WebUI checks (lint, type-check, build, test)
webui-check:
    just webui-lint
    just webui-type-check
    just webui-build
    just webui-build-mock
    just webui-test

# Run overlay tests with E2E enforcement verification
test-overlays:
    just build-overlay-tests
    cargo test -p sandbox-integration-tests --verbose

# Run mock-agent integration tests
test-mock-agent-integration:
    cd tests/tools/mock-agent && python3 tests/test_agent_integration.py

# Replay mock-agent session recordings (shows menu)
replay-mock-agent-sessions:
    tests/tools/mock-agent/replay-recording.sh

# Replay the most recent mock-agent session recording
replay-last-mock-agent-session:
    tests/tools/mock-agent/replay-recording.sh --latest

# Clear all mock-agent session recordings
clear-mock-agent-recordings:
    rm -rf tests/tools/mock-agent/recordings/*.json

# Run network tests with E2E enforcement verification
test-networks:
    just build-network-tests
    cargo test -p sandbox-integration-tests --verbose

# Run debugging enforcement tests with E2E verification
test-debugging:
    just build-debugging-tests
    cargo test -p sandbox-integration-tests --verbose
    ./target/debug/debugging_test_orchestrator

# Run simple mount test to verify CAP_SYS_ADMIN availability in user namespaces
test-mount-capability:
    just build-debugging-test-binaries
    ./target/debug/mount_test
