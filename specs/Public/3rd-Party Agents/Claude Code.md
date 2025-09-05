# Claude Code — Integration Notes

## Overview

Claude Code is Anthropic's official command-line interface for interacting with Claude, providing an AI-powered coding assistant with MCP (Model Context Protocol) support.

- **Website**: <https://docs.anthropic.com/en/docs/agents-and-tools/claude-code>
- **Documentation**: <https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview>
- **GitHub**: <https://github.com/anthropics/claude-code>
- **Version**: 1.0.98 (as of this writing)

### Task start-up command

Claude Code can be started with a specific task prompt in several ways:

1. **Direct prompt**:

   ```bash
   claude "Implement a REST API endpoint for user management"
   ```

2. **Interactive session with prompt**:

   ```bash
   claude --continue "Continue working on the authentication system"
   ```

3. **Resume specific session**:

   ```bash
   claude --resume <session-id>
   ```

4. **Non-interactive mode** (for automation):

   ```bash
   claude --print "Generate unit tests for this function"
   ```

5. **With specific model**:
   ```bash
   claude --model sonnet "Refactor this legacy code"
   ```

### Built-in support for checkpoints

Yes, Claude Code supports checkpoints through its conversation/session management. The checkpoints cover **chat content only**.

**Session Management:**

- **Session resumption**: Use `--resume [sessionId]` to continue from a previous conversation
- **Continue mode**: `--continue` resumes the most recent conversation
- **Session IDs**: Each conversation has a unique session ID for targeted resumption via `--session-id <uuid>`
- **Automatic session tracking**: Conversations are automatically saved and can be resumed

**Checkpoint Coverage:**

- **Chat content**: Full conversation history and context is preserved and restored
- **File system state**: NOT restored - only the conversation state is maintained
- **Tool execution history**: Preserved within the conversation transcript

**Restoring from Specific Moments:**

- **By session ID**: Use `--resume <sessionId>` to restore to the end of a specific session
- **By recent session**: Use `--continue` to restore the most recent session
- **No granular restoration**: Cannot restore to a specific chat message or prompt position within a session
- **File system restoration**: No mechanism to restore file system to a previous state

**Limitations:**

- Checkpoints preserve conversation flow but do not include file system snapshots
- No ability to rollback file changes made during a session
- Session restoration maintains conversation context but not workspace state

### How is the use of MCP servers configured?

Claude Code provides comprehensive MCP server configuration through multiple methods:

**Command-line options:**

- `--mcp-config <configs...>`: Load MCP servers from JSON files or strings (space-separated)
- `--strict-mcp-config`: Only use MCP servers from --mcp-config, ignoring all other MCP configurations
- `--mcp-debug`: [DEPRECATED] Enable MCP debug mode (shows MCP server errors)
- `--settings <file-or-json>`: Load session-specific settings including MCP configuration

**MCP management commands:**

- `claude mcp serve`: Start the Claude Code MCP server
- `claude mcp add <name> <commandOrUrl> [args...]`: Add a server (stdio or URL-based)
- `claude mcp remove <name>`: Remove an MCP server
- `claude mcp list`: List configured MCP servers
- `claude mcp get <name>`: Get details about an MCP server
- `claude mcp add-json <name> <json>`: Add an MCP server with a JSON string
- `claude mcp add-from-claude-desktop`: Import MCP servers from Claude Desktop (Mac and WSL only)
- `claude mcp reset-project-choices`: Reset all approved and rejected project-scoped (.mcp.json) servers

**Configuration files:**

- Project-specific: `.mcp.json` files for project-scoped MCP configurations
- Global MCP server configurations stored in Claude Code's configuration directory

**Environment variables:**

- No specific MCP-related environment variables documented in help screens

### Support for custom hooks

**YES, Claude Code supports custom hooks for the Agent Time Travel feature!** Claude Code has a comprehensive hook system that allows executing custom commands after every agent step (tool use).

#### Hook Configuration

Hooks are configured through settings files with the following hierarchy:

- `~/.claude/settings.json` - User global settings
- `.claude/settings.json` - Project-specific settings
- `.claude/settings.local.json` - Local project settings (not committed)
- Enterprise managed policy settings

#### Session-Specific Hook Configuration

**Yes, hooks can be enabled for particular sessions using the `--settings` option:**

```bash
# Use session-specific settings file
claude --settings /path/to/session-hooks.json "Start coding session"

# Or use inline JSON
claude --settings '{"hooks":{"PostToolUse":[{"matcher":"*","hooks":[{"type":"command","command":"echo \"Tool executed\" >> /tmp/agent-log.txt"}]}]}}' "Start session"
```

#### Hook Events for Time Travel

The most relevant hook event for Agent Time Travel is `PostToolUse`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/time-travel-log.sh",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

#### Available Hook Events

- **PreToolUse**: Runs before tool execution
- **PostToolUse**: Runs after tool execution (perfect for Agent Time Travel)
- **UserPromptSubmit**: Runs when user submits a prompt
- **Stop**: Runs when agent stops
- **SubagentStop**: Runs when subagent stops
- **SessionStart**: Runs when session starts
- **SessionEnd**: Runs when session ends
- **Notification**: Runs on notifications

#### Hook Input/Output

Hooks receive JSON data via stdin and can return structured responses:

- **Input**: Session info, tool details, event data
- **Output**: Exit codes (0=success, 2=blocking error) or JSON responses
- **Environment**: `$CLAUDE_PROJECT_DIR` available for project-relative paths

#### Example Time Travel Hook

```bash
#!/bin/bash
# .claude/hooks/time-travel-log.sh
echo "$(date): Tool ${TOOL_NAME} executed in session ${SESSION_ID}" >> /tmp/agent-time-travel.log
```

This demonstrates that Claude Code has **full support** for the Agent Time Travel feature through its comprehensive hook system.

### Credentials

Claude Code stores credentials and configuration in the following precise locations:

**Authentication methods:**

- **Token setup**: `claude setup-token` for long-lived authentication (requires Claude subscription)
- **OAuth/browser authentication**: Automatic browser-based authentication for Claude API access

**Configuration files:**

- **Global config**: Stored in OS-specific application data directories (e.g., `~/.config/claude-code/` on Linux, `%APPDATA%\claude-code\` on Windows)
- **Local/project config**: Project-specific configuration files in the working directory
- **Settings files**: Custom JSON settings files loaded via `--settings` flag

**Environment variables:**

- Various `CLAUDE_*` environment variables for API keys and settings
- Standard Anthropic API environment variables

**Precise storage paths** (platform-dependent):

- **Linux/macOS**: `~/.config/claude-code/` or `~/.local/share/claude-code/`
- **Windows**: `%APPDATA%\claude-code\` or `%LOCALAPPDATA%\claude-code\`
- **Token storage**: Authentication tokens stored securely in OS keychain/keyring systems

The configuration system supports hierarchical settings (global → local → command-line overrides).

### Known Issues

- **Subscription requirement**: Some features require a Claude Pro subscription
- **Network dependency**: Requires internet access for Claude API communication
- **Rate limiting**: Subject to Anthropic's API rate limits
- **MCP compatibility**: Not all MCP servers may be fully compatible
- **Platform limitations**: Some features (like Claude Desktop import) are platform-specific
