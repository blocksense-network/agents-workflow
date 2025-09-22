# Mock Coding Agent

A lightweight, deterministic mock "coding agent" that can impersonate both Codex and Claude Code for testing and development:

## Features

- **🎭 Dual Format Support**: Impersonates both Codex CLI and Claude Code agents
- **📝 File Operations**: Create, overwrite, append, and replace files in workspaces
- **🖥️ Terminal Output**: Streams thinking traces and tool-use messages to stdout
- **📁 Session Recording**: Writes session files in either Codex or Claude format
- **🌐 API Server**: Mock OpenAI/Anthropic API server for testing IDE integrations
- **🧪 Test-Ready**: Comprehensive test suite with non-interactive verification

## Format Support

The agent can impersonate different tools via the `--format` flag:

- **Codex Format** (`--format codex`, default): Compatible with [Codex Session File Format](../../specs/Research/Codex-Session-File-Format.md)
- **Claude Format** (`--format claude`): Compatible with [Claude Session File Format](../../specs/Research/Claude-Session-File-Format.md)

Each format creates session files in the appropriate directory structure with tool-specific metadata, conversation threading, and environment context.

## Quickstart

### Installation

The mock agent runs in the current development environment. No separate installation needed.

```bash
# Verify the agent works
python -m src.cli --help
```

### Basic Usage

```bash
# Run built-in demo (creates hello.py)
python -m src.cli demo --workspace /tmp/mock-ws

# Run with Claude format
python -m src.cli demo --workspace /tmp/mock-ws --format claude

# Run a custom scenario
python -m src.cli run --scenario examples/hello_scenario.json --workspace /tmp/mock-ws

# Run with different formats
python -m src.cli run --scenario examples/hello_scenario.json --workspace /tmp/mock-ws --format codex
python -m src.cli run --scenario examples/hello_scenario.json --workspace /tmp/mock-ws --format claude
```

### Output Locations

**Codex Format:**

- Session files: `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`
- UI logs: `~/.codex/logs/session-*.jsonl`

**Claude Format:**

- Session files: `~/.claude/projects/<encoded-workspace-path>/<uuid>.jsonl`

## Mock API Server

Run a mock OpenAI/Anthropic-compatible API server for testing IDE integrations:

```bash
# Start server with Codex format (default)
python -m src.cli server --host 127.0.0.1 --port 8080 --playbook examples/playbook.json

# Start server with Claude format
python -m src.cli server --host 127.0.0.1 --port 8080 --playbook examples/playbook.json --format claude

# Test the server
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model": "gpt-4o-mini", "messages": [{"role":"user","content":"Create hello.py"}]}'
```

The server returns predetermined responses based on the playbook and records session files in the specified format.

## Session File Formats

The mock agent supports two session file formats via the `--format` flag, allowing it to impersonate different coding tools:

### Codex Format (`--format codex`, default)

Mimics the Codex CLI session format with rollout files and optional UI logs:

- **Location**: `~/.codex/sessions/YYYY/MM/DD/rollout-YYYY-MM-DDThh-mm-ss-<uuid>.jsonl`
- **UI Logs**: `~/.codex/logs/session-YYYYMMDDTHHMMSSZ.jsonl` (when `CODEX_TUI_RECORD_SESSION=1`)
- **Structure**: Linear sequence of events with session metadata, messages, tool calls, and results
- **Use Case**: Testing applications that integrate with Codex CLI

### Claude Format (`--format claude`)

Mimics Claude Code's session format with rich conversation threading and context:

- **Location**: `~/.claude/projects/<encoded-project-path>/<session-uuid>.jsonl`
- **Structure**: Threaded conversation with parent-child UUID relationships, detailed tool context
- **Features**: Git integration, environment tracking, usage statistics, tool result metadata
- **Use Case**: Testing applications that integrate with Claude Code

See [Codex Session File Format](../../specs/Research/Codex-Session-File-Format.md) and [Claude Session File Format](../../specs/Research/Claude-Session-File-Format.md) for detailed specifications.

## Scenario Format

Create custom scenarios using JSON files. Example structure:

```json
{
  "meta": {
    "instructions": "You are a helpful coding agent.",
    "turn_context": {
      "cwd": "/workspace",
      "model": "mock-model"
    }
  },
  "turns": [
    { "user": "Create hello.py" },
    { "think": "I'll create a Python file" },
    {
      "tool": {
        "name": "write_file",
        "args": { "path": "hello.py", "text": "print('Hello!')\n" }
      }
    },
    { "assistant": "Created hello.py successfully" }
  ]
}
```

**Supported turn types:**

- `user`: User message
- `think`: Assistant reasoning/thinking
- `tool`: Tool call with name and arguments
- `assistant`: Assistant response
- `shell`: Shell command execution

## Tool Support

The mock agent supports these file operation tools:

- **`write_file`**: Create or overwrite files
- **`read_file`**: Read file contents
- **`append_file`**: Append to existing files
- **`replace_in_file`**: Replace text within files

Tool results are properly formatted for each session format, including success/error status and metadata.

## Testing

The mock agent includes a comprehensive test suite. See [AGENTS.md](AGENTS.md) for detailed testing instructions.

Quick test:

```bash
# Run all tests
python tests/test_agent_simple.py

# Test specific format
python -m src.cli run --scenario examples/hello_scenario.json --workspace /tmp/test --format claude
```

## Implementation Details

- **`src/session_io.py`**: Session file writers for both formats
- **`src/agent.py`**: Core agent logic and scenario execution
- **`src/cli.py`**: Command-line interface
- **`src/tools.py`**: File operation implementations
- **`src/server.py`**: Mock API server

## License

MIT
