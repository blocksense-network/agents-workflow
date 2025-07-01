{
  description = "agents-workflow";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    codex.url = "github:openai/codex";
    codex.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    codex,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
  in {
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
            codex.packages.${system}.codex-rs
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
        config.allowUnfree = true; # Allow unfree packages like claude-code
      };
    in {
      default = pkgs.mkShell {
        buildInputs = [
          pkgs.just
          pkgs.ruby
          pkgs.bundler
          pkgs.rubocop
          pkgs.git
          pkgs.fossil
          pkgs.mercurial

          # AI Coding Assistants (latest versions from nixpkgs-unstable)
          pkgs.goose-cli # Goose AI coding assistant
          pkgs.claude-code # Claude Code - agentic coding tool
          pkgs.gemini-cli # Gemini CLI
          codex.packages.${system}.codex-rs # OpenAI Codex CLI (native Rust implementation)
        ];

        shellHook = ''
          echo "Agent workflow development environment loaded"
        '';
      };
    });
  };
}
