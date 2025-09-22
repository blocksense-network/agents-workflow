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

# Check Rust code for compilation errors
check:
    cargo check --workspace

# Run Rust tests
test-rust:
    cargo test --workspace

# Lint Rust code
lint-rust:
    cargo clippy --workspace -- -D warnings

# Format Rust code
fmt-rust:
    cargo fmt --all --check

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
    scripts/launch-aw-fs-snapshots-daemon.sh

# Stop the AW filesystem snapshots daemon
legacy-stop-aw-fs-snapshots-daemon:
    scripts/stop-aw-fs-snapshots-daemon.sh

# Check status of AW filesystem snapshots daemon
legacy-check-aw-fs-snapshots-daemon:
    scripts/check-aw-fs-snapshots-daemon.rb

# Run all spec linting/validation in one go
lint-specs:
    scripts/lint-specs.sh
