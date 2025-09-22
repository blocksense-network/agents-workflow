{
  description = "agents-workflow";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    git-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    git-hooks,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
  in {
    checks = forAllSystems (system: let
      pkgs = import nixpkgs { inherit system; };
      preCommit = git-hooks.lib.${system}.run {
        src = ./.;
        hooks = {
          # Markdown formatting (run first)
          prettier-md = {
            enable = true;
            name = "prettier --write (Markdown)";
            entry = "prettier --loglevel warn --write";
            language = "system";
            pass_filenames = true;
            files = "\\.md$";
          };
          # Fast auto-fixers and sanity checks
          # Local replacements for common sanity checks (portable, no Python deps)
          check-merge-conflict = {
            enable = true;
            name = "check merge conflict markers";
            entry = ''
              bash -lc 'set -e; rc=0; for f in "$@"; do [ -f "$f" ] || continue; if rg -n "^(<<<<<<<|=======|>>>>>>>)" --color never --hidden --glob "!*.rej" --no-ignore-vcs -- "$f" >/dev/null; then echo "Merge conflict markers in $f"; rc=1; fi; done; exit $rc' --
            '';
            language = "system";
            pass_filenames = true;
            types = [ "text" ];
          };
          check-added-large-files = {
            enable = true;
            name = "check added large files (>1MB)";
            entry = ''
              bash -lc 'set -e; limit="$LIMIT"; [ -z "$limit" ] && limit=1048576; rc=0; for f in "$@"; do [ -f "$f" ] || continue; sz=$(stat -c %s "$f" 2>/dev/null || stat -f %z "$f"); if [ "$sz" -gt "$limit" ]; then echo "File too large: $f ($sz bytes)"; rc=1; fi; done; exit $rc' --
            '';
            language = "system";
            pass_filenames = true;
          };

          # Markdown: fix then lint
          markdownlint-fix = {
            enable = true;
            name = "markdownlint-cli2 (fix)";
            entry = "markdownlint-cli2 --fix";
            language = "system";
            pass_filenames = true;
            files = "\\.md$";
          };

          lint-specs = {
            enable = true;
            name = "Lint Markdown specs";
            entry = "just lint-specs";
            language = "system";
            pass_filenames = false;
          };

          # Spelling
          cspell = {
            enable = true;
            name = "cspell (cached)";
            entry = "cspell --no-progress --cache --config .cspell.json --exclude .obsidian/**";
            language = "system";
            pass_filenames = true;
            files = "\\.(md|rb|rake|ya?ml|toml|json)$";
          };

          # Ruby formatting/linting (safe auto-correct)
          rubocop-autocorrect = {
            enable = true;
            name = "rubocop --safe-auto-correct";
            entry = "rubocop -A --force-exclusion";
            language = "system";
            pass_filenames = true;
            files = "\\.(rb|rake)$";
          };

          # Shell formatting
          shfmt = {
            enable = true;
            name = "shfmt";
            entry = "shfmt -w";
            language = "system";
            pass_filenames = true;
            files = "\\.(sh|bash)$";
          };

          # TOML formatting
          taplo-fmt = {
            enable = true;
            name = "taplo fmt";
            entry = "taplo fmt";
            language = "system";
            pass_filenames = true;
            files = "\\.toml$";
          };

          # Fast link check on changed files (CI will run full scan)
          lychee-fast = {
            enable = true;
            name = "lychee (changed files)";
            entry = "lychee --no-progress --require-https --cache";
            language = "system";
            pass_filenames = true;
            files = "\\.md$";
          };
        };
        # Ensure all hook entries (language = "system") have their executables available
        # when running in CI or via `nix flake check` (outside the dev shell).
        tools = {
          # Commands invoked by hooks or scripts they call
          prettier = pkgs.nodePackages.prettier;
          rubocop = pkgs.rubocop;
          shfmt = pkgs.shfmt;
          taplo = pkgs.taplo;
          lychee = pkgs.lychee;
          markdownlint-cli2 = pkgs.nodePackages.markdownlint-cli2;
          cspell = pkgs.nodePackages.cspell;
          just = pkgs.just; # for the lint-specs hook
          rg = pkgs.ripgrep; # used by check-merge-conflict
          mmdc = pkgs.nodePackages."@mermaid-js/mermaid-cli"; # used by md-mermaid-check via just lint-specs
        };
      };
    in {
      pre-commit-check = preCommit;
    });
    packages = forAllSystems (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true; # Allow unfree packages like claude-code
        };
        agent-task-script = pkgs.writeShellScriptBin "agent-task" ''
          PATH=${pkgs.lib.makeBinPath [
            pkgs.ruby
            pkgs.goose-cli
            pkgs.claude-code
            pkgs.gemini-cli
            pkgs.codex
            pkgs.opencode
          ]}:$PATH
          exec ruby ${./bin/agent-task} "$@"
        '';
        get-task = pkgs.writeShellScriptBin "get-task" ''
          exec ${pkgs.ruby}/bin/ruby ${./bin/get-task} "$@"
        '';
        start-work = pkgs.writeShellScriptBin "start-work" ''
          exec ${pkgs.ruby}/bin/ruby ${./bin/start-work} "$@"
        '';
        agent-utils = pkgs.symlinkJoin {
          name = "agent-utils";
          paths = [get-task start-work];
        };
      in {
        agent-task = agent-task-script;
        agent-utils = agent-utils;
      }
    );

    apps = forAllSystems (system: {
      agent-task = {
        type = "app";
        program = "${self.packages.${system}.agent-task}/bin/agent-task";
      };
      default = self.apps.${system}.agent-task;
    });

    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config.allowUnfree = true; # Allow unfree packages like claude-code
      };
      isLinux = pkgs.stdenv.isLinux;
      isDarwin = pkgs.stdenv.isDarwin;
    in {
      default = pkgs.mkShell {
        buildInputs = [
          # Rust toolchain
          (pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rustfmt" "clippy"];
            targets = [
              # Linux
              "x86_64-unknown-linux-gnu"
              "aarch64-unknown-linux-gnu"
              # macOS
              "x86_64-apple-darwin"
              "aarch64-apple-darwin"
              # Windows (GNU)
              "x86_64-pc-windows-gnu"
              "aarch64-pc-windows-gnullvm"
            ];
          })

          pkgs.just
          pkgs.python3
          pkgs.python3Packages.pexpect
          pkgs.ruby
          pkgs.bundler
          pkgs.rubocop
          pkgs.git
          pkgs.fossil
          pkgs.mercurial
          pkgs.nodejs # for npx-based docson helper
          # Mermaid validation (diagram syntax)
          (pkgs.nodePackages."@mermaid-js/mermaid-cli")
          pkgs.noto-fonts

          # Markdown linting & link/prose checking
          (pkgs.nodePackages.markdownlint-cli2)
          pkgs.lychee
          pkgs.vale
          (pkgs.nodePackages.cspell)
          (pkgs.nodePackages.prettier)
          pkgs.shfmt
          pkgs.taplo

          # AI Coding Assistants (latest versions from nixpkgs-unstable)
          pkgs.goose-cli # Goose AI coding assistant
          pkgs.claude-code # Claude Code - agentic coding tool
          pkgs.gemini-cli # Gemini CLI
          pkgs.codex # OpenAI Codex CLI (Rust implementation)
          pkgs.opencode # OpenCode AI coding assistant
        ]
        ++ self.checks.${system}.pre-commit-check.enabledPackages
        # Optional schema/validation tooling (only if available in this nixpkgs)
        ++ (builtins.filter (x: x != null) [
          (if pkgs ? taplo then pkgs.taplo else null)
          (if (builtins.hasAttr "nodePackages" pkgs) && (builtins.hasAttr "ajv-cli" pkgs.nodePackages)
           then pkgs.nodePackages."ajv-cli" else null)
        ])
        ++ pkgs.lib.optionals isLinux [
          # Use Chromium on Linux for mermaid-cli's Puppeteer
          pkgs.chromium
          # Linux-only filesystem utilities for snapshot functionality
          pkgs.zfs # ZFS utilities for copy-on-write snapshots
          pkgs.btrfs-progs # Btrfs utilities for subvolume snapshots
          # Docker for containerized testing
          pkgs.docker
          # System monitoring tools for performance tests
          pkgs.procps # ps, top, etc. for memory monitoring
        ] ++ pkgs.lib.optionals isDarwin [
          # macOS-only VM manager
          pkgs.lima # Linux virtual machines on macOS
          # Provide a reproducible Chrome for Puppeteer on macOS (unfree)
          pkgs.google-chrome
        ];

        shellHook = ''
          # Install git pre-commit hook invoking our Nix-defined hooks
          ${self.checks.${system}.pre-commit-check.shellHook}
          echo "Agent workflow development environment loaded"
          # Provide a convenience function for Docson; no fallbacks in Nix shell
          docson () {
            if command -v docson >/dev/null 2>&1; then
              command docson "$@"
              return
            fi
            if [ -n "''${IN_NIX_SHELL:-}" ]; then
              echo "Docson is not available in this Nix dev shell. Add it to flake.nix (or choose an alternative) — no fallbacks allowed." >&2
              return 127
            fi
            if command -v npx >/dev/null 2>&1; then
              npx -y docson "$@"
            else
              echo "Docson not found and npx unavailable. Install Docson or enter nix develop with it provisioned." >&2
              return 127
            fi
          }
          echo "Tip: run: docson -d ./specs/schemas  # then open http://localhost:3000"
          # Use the Nix-provided browser path (fully reproducible)
          ''
          + (if isLinux then ''
            export PUPPETEER_EXECUTABLE_PATH="${pkgs.chromium}/bin/chromium"
          '' else "")
          + (if isDarwin then ''
            export PUPPETEER_EXECUTABLE_PATH="${pkgs.google-chrome}/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
          '' else "")
          + ''
          export PUPPETEER_PRODUCT=chrome
        '';
      };
    });
  };
}
