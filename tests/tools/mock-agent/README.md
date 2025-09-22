# Mock Coding Agent

A lightweight, deterministic mock “coding agent” that:
- Edits files in a workspace (create/overwrite/append/replace).
- Streams **thinking traces** and **tool-use** messages to stdout.
- Writes **Codex-compatible rollout and session log JSONL files**.
- Can run as a **mock OpenAI/Anthropic API server** to drive IDE agents (Claude Code / Codex CLI) during tests/simulations.

> Session files follow the [Codex Session File Format](../../specs/Research/Codex-Session-File-Format.md) (rollout `.jsonl` and UI session logs `.jsonl`).

## Quickstart

```bash
# 1) Install (editable install)
pip install -e .

# 2) Run the built-in demo
mockagent demo --workspace /tmp/mock-ws

# 3) Run a scripted scenario
mockagent run --scenario examples/hello_scenario.json --workspace /tmp/mock-ws
````

## Mock API Server

```bash
# Start server on port 8080 with a playbook (deterministic responses)
mockagent server --host 127.0.0.1 --port 8080 --playbook examples/playbook.json

# Then hit it (OpenAI-like)
curl -s http://127.0.0.1:8080/v1/chat/completions -H 'content-type: application/json' -d @- <<'JSON'
{
  "model": "gpt-4o-mini",
  "messages": [{"role":"user","content":"Create hello.py that prints Hello"}]
}
JSON
```

The server returns predetermined tool calls and assistant messages based on the playbook. Both the server and the CLI **record Codex rollout+log files** under `~/.codex/`.

## Codex Rollout & Session-Log

* **Rollout**: `~/.codex/sessions/YYYY/MM/DD/rollout-YYYY-MM-DDThh-mm-ss-<uuid>.jsonl`
* **Session Log**: `~/.codex/logs/session-YYYYMMDDTHHMMSSZ.jsonl`

See `src/session_io.py` for the writer, faithful to the spec.

## License

MIT
