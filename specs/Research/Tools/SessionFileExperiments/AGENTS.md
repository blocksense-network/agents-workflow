# Session File Experiments

This folder contains small, tool-specific scripts to discover and document on-disk session formats, storage paths, and safe trimming techniques for supported agent tools. These experiments help us distinguish session continuation (chat history only) from checkpointing (point-in-time restore of chat + filesystem) and provide empirical notes across versions.

Context: These experiments support the Agent Time Travel feature’s need for precise step boundaries and file state capture. See `../../../Public/Agent%20Time%20Travel.md` for goals and architecture.

## Reverse-engineering policy

- Prefer official --help and docs; when storage formats are undocumented, run controlled experiments.
- Record very short, benign sessions to create minimal transcript/state files.
- Inspect created files and parent directories; identify placeholders such as <project-id>, <session-id>, and how they are derived.
- Capture minimal examples in per-tool notes. Keep per-version observations because formats may change across releases.
- Attempt surgical trims and validate that the tool can resume from a mid-point. Always back up originals before editing.
- Never commit secrets from transcripts. Redact sensitive content in examples.

## Running experiments

- Prerequisites: Python 3.9+, pexpect (pip install pexpect), jq (optional).
- Run each script from the repository root. Scripts detect missing CLIs and print next steps when automation is limited.
- For tools that support hooks (e.g., Claude Code), scripts can add a temporary hook to capture transcript_path and session identifiers.
- For tools that require interactive approval of tool calls, scripts will guide manual confirmation if non-interactive control is unavailable.

## Interactive testing strategy

We use pexpect-only automation in the experiment scripts. When exploring a new tool manually, you can still drive tmux yourself from the regular shell to observe behavior and then translate those observations into explicit pexpect expectations. The scripts intentionally avoid embedding tmux control to keep responsibilities clear.

## Deliverables: per-tool documentation updates

- Primary goal: populate and update the per-tool markdown files in `../../../Public/3rd-Party Agents/` with empirical findings:
  - Exact storage paths and file formats (with minimal examples)
  - Checkpointing vs session continuation behavior and commands
  - Hook/MCP configuration details relevant to recording and automation
  - Credentials/config paths that affect session storage
- Each experiment should produce an edit to the corresponding file (e.g., `Claude Code.md`, `Gemini CLI.md`, `Goose.md`, `OpenCode.md`, `OpenAI Codex CLI.md`) including tool version, date, and a brief “Findings” subsection.

## Scripts

- claude.py — pexpect-only. Creates a short Claude Code session, backs up and trims the JSONL transcript mid‑point (when long enough), and prints resume guidance. Falls back to ~/.claude/projects discovery if hooks don’t fire.
- gemini.py — pexpect-only. Starts a short Gemini CLI run with --checkpointing and prints restore guidance.
- goose.py — pexpect-only. Starts a minimal Goose session, points to session storage candidates, and prints resume guidance.
- opencode.py — pexpect-only. Creates a short OpenCode interaction, suggests export locations, and prints resume guidance.
- codex.py — Notes that Codex CLI lacks persistent sessions; demonstrates diff generation and restoration where applicable.

## Status (2025-09-10)

- Removed tmux usage from all experiment scripts; they now use pexpect-only automation with minimal sleeps and simple approvals.
- Claude Code: added robust filesystem fallback for locating transcripts and safe-trim behavior; hook remains optional.
- Added a helper (`list_recent.py`) to list recent files across common tool storage roots to aid manual exploration.
