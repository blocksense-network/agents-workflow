# OpenAI Codex CLI — Integration Notes

> Usually this information can be obtained by checking out command-line help screens or man pages for the agent software.

## Overview

OpenAI Codex CLI is a Rust-based command-line interface for OpenAI's Codex models, providing an AI-powered coding assistant with advanced sandboxing and execution capabilities.

- **Website**: <https://openai.com/blog/openai-codex/>
- **Documentation**: <https://github.com/OpenAI/openai-codex-cli>
- **GitHub**: <https://github.com/OpenAI/openai-codex-cli>
- **Version**: 0.27.0 (as of this writing)

### Task start-up command

Codex CLI can be started with a specific task prompt in several ways:

1. **Interactive session with prompt**:

   ```bash
   codex "Implement a user authentication system"
   ```

2. **Non-interactive execution**:

   ```bash
   codex exec "Generate unit tests for this function"
   ```

3. **Read from stdin**:

   ```bash
   echo "Refactor this legacy code" | codex exec -
   ```

4. **With specific model**:

   ```bash
   codex --model o3 "Implement a REST API endpoint"
   ```

5. **With sandbox mode**:
   ```bash
   codex --sandbox workspace-write "Modify these files safely"
   ```

### Built-in support for checkpoints

Limited checkpoint support through Git integration. The checkpoints cover **file system state only**.

**Checkpoint Management:**

- **Git apply functionality**: `codex apply` can apply the latest diff produced by the agent as a `git apply` to the working tree
- **Git repository requirement**: By default requires running in a Git repository (can be bypassed with `--skip-git-repo-check`)
- **Diff-based checkpoints**: Changes are tracked as diffs that can be applied later

**Checkpoint Coverage:**

- **Chat content**: NOT preserved - no conversation history is maintained
- **File system state**: Changes are tracked as Git diffs that can be applied
- **Tool execution history**: NOT preserved - no session continuity

**Restoring from Specific Moments:**

- **No session restoration**: Codex CLI does not support resuming conversations or sessions
- **Git diff application**: Use `codex apply` to apply the last generated diff to the working tree
- **No granular restoration**: Cannot restore to a specific chat message or prompt position
- **File system restoration**: Can apply Git diffs to restore file changes, but requires manual Git management

**Session Storage:**

- **No persistent sessions**: No built-in session management or storage
- **Git-based checkpoints**: File changes are tracked as diffs but not automatically saved
- **Manual recovery**: Requires using `codex apply` to restore changes

**Limitations:**

- No conversation persistence between CLI invocations
- No automatic session resumption capabilities
- Checkpointing requires Git repository and manual diff application
- No integrated session history or conversation restoration
- File system restoration depends on Git diff availability

### How is the use of MCP servers configured?

OpenAI Codex CLI has experimental MCP server support:

**Command-line options:**

- No specific MCP configuration command-line options documented (experimental feature)

**MCP management commands:**

- `codex mcp`: Experimental command to run Codex as an MCP server
- Configuration overrides via `-c, --config` options can be applied to MCP mode

**Configuration files:**

- `~/.codex/config.toml`: Main configuration file that may include MCP-related settings
- No specific MCP configuration file format documented

**Environment variables:**

- No specific MCP-related environment variables documented in help screens

Note: MCP support is marked as experimental in Codex CLI.

### Support for custom hooks

For Agent Time Travel feature (commands executed after every agent step), OpenAI Codex CLI does not appear to have built-in support for custom step-level hooks. However, it does support broader customization through:

1. **Configuration profiles**: `-p, --profile` to use predefined configuration profiles from `~/.codex/config.toml`
2. **Runtime configuration overrides**: `-c, --config` to override configuration values at runtime
   - Examples: `-c model="o3"`, `-c 'sandbox_permissions=["disk-full-read-access"]'`
3. **Experimental MCP support**: `codex mcp` for running as an MCP server

Note: While configuration profiles and MCP support exist, there is no documented support for automatic execution of custom commands after every agent step as required for the Agent Time Travel feature.

### Credentials

Codex CLI uses OpenAI authentication with the following precise storage locations:

**Authentication methods:**

- **API Key authentication**: `codex login --api-key <API_KEY>`
- **Interactive login**: `codex login` (launches browser for OAuth)
- **Login status**: `codex login status` shows current authentication state
- **Logout**: `codex logout` removes stored credentials

**Configuration files:**

- **Main config file**: `~/.codex/config.toml` - Contains API keys, authentication tokens, default model settings, sandbox permissions, and configuration profiles
- Configuration stored in TOML format with support for profiles and overrides

**Environment variables:**

- Standard OpenAI environment variables (OPENAI_API_KEY, etc.)
- Custom configuration overrides via environment variables

**Precise storage paths:**

- **Configuration directory**: `~/.codex/`
- **Config file**: `~/.codex/config.toml`
- **Authentication tokens**: Stored within the config file or OS keychain (depending on authentication method)

### Known Issues

- **Git repository dependency**: By default requires execution within a Git repository
- **Sandbox limitations**: Sandbox modes may have performance or functionality limitations
- **Experimental MCP**: MCP server functionality is marked as experimental
- **OpenAI API dependency**: Requires active OpenAI API access and is subject to rate limits
- **Limited session management**: Less sophisticated session resumption compared to other agents
- **Configuration complexity**: Advanced configuration requires understanding of TOML structure
