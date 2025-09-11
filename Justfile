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

test:
    ruby -Itest test/run_tests_shell.rb

# Lint the Ruby codebase
lint:
    rubocop

# Auto-fix lint issues where possible
lint-fix:
    rubocop --autocorrect-all

# Build and publish the gem
publish-gem:
    gem build agent-task.gemspec && gem push agent-task-*.gem

# Validate all JSON Schemas with ajv (meta-schema compile)
conf-schema-validate:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v ajv >/dev/null 2>&1; then
        AJV=ajv
    else
        if [ -n "${IN_NIX_SHELL:-}" ]; then
            echo "Error: 'ajv' is missing inside Nix dev shell. Add pkgs.nodePackages.\"ajv-cli\" to flake.nix devShell inputs." >&2
            exit 127
        fi
        echo "ajv not found; falling back to 'npx ajv-cli' outside Nix shell (requires network)" >&2
        AJV='npx -y ajv-cli'
    fi
    for f in specs/schemas/*.json; do
        echo Validating $$f
        $$AJV compile -s "$$f"
    done
    echo All schemas valid.

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
    lychee --no-progress --require-https --max-concurrency 8 "specs/**/*.md"

# Spell-check Markdown with cspell (uses default dictionaries unless configured)
md-spell:
    cspell "specs/**/*.md"

# Run all spec linting/validation in one go
lint-specs:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "${IN_NIX_SHELL:-}" ]; then
        echo "Running lint-specs inside Nix dev shell (no fallbacks)." >&2
    fi
    just md-lint
    just md-links
    just md-spell
    # Prose/style linting via Vale (warn-only): our custom style lowers
    # spelling to warnings and uses project vocab so this won't fail commits.
    if command -v vale >/dev/null 2>&1; then
        vale specs/Public || true
    else
        if [ -n "${IN_NIX_SHELL:-}" ]; then
            echo "vale is missing inside Nix dev shell; add pkgs.vale to flake.nix." >&2
            exit 127
        fi
        echo "vale not found; skipping outside Nix shell." >&2
    fi
    # Mermaid syntax validation (enabled by default)
    just md-mermaid-check
