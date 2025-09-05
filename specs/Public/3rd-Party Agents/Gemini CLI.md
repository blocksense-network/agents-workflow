# Gemini CLI — Integration Notes

> Usually this information can be obtained by checking out command-line help screens or man pages for the agent software.

## Overview

Gemini CLI is Google's official command-line interface for interacting with Gemini AI models, providing an AI-powered coding assistant with MCP support and experimental features.

- **Website**: <https://ai.google.dev/gemini-api>
- **Documentation**: <https://github.com/google/gemini-cli>
- **GitHub**: <https://github.com/google/gemini-cli>
- **Version**: 0.2.2 (as of this writing)

### Task start-up command

Gemini CLI can be started with a specific task prompt in several ways:

1. **Direct prompt**:

   ```bash
   gemini --prompt "Implement a user authentication system"
   ```

2. **Interactive session with initial prompt**:

   ```bash
   gemini --prompt-interactive "Start building a REST API"
   ```

3. **Non-interactive mode** (for automation):

   ```bash
   gemini -p "Generate unit tests for this function"
   ```

4. **With specific model**:

   ```bash
   gemini --model gemini-pro "Refactor this legacy code"
   ```

5. **With checkpointing enabled**:
   ```bash
   gemini --checkpointing --prompt "Implement feature X"
   ```

### Built-in support for checkpoints

Yes, Gemini CLI supports checkpoints for file edits. The checkpoints cover **file system state only**.

**Checkpoint Management:**

- **Checkpointing flag**: Use `--checkpointing` or `-c` to enable checkpointing of file edits
- **Automatic saving**: File modifications are automatically checkpointed during the session
- **Recovery capability**: Checkpointed edits can be recovered if the session is interrupted
- **Experimental ACP mode**: `--experimental-acp` starts the agent in ACP (Advanced Checkpointing Protocol) mode

**Checkpoint Coverage:**

- **Chat content**: NOT preserved - no conversation history is maintained between sessions
- **File system state**: File modifications are tracked and can be recovered
- **Tool execution history**: NOT preserved - no session continuity for conversation

**Restoring from Specific Moments:**

- **No session restoration**: Gemini CLI does not support resuming conversations or sessions
- **File recovery**: Checkpointed file edits can be recovered, but only within the current session
- **No granular restoration**: Cannot restore to a specific chat message or prompt position
- **File system restoration**: Can recover file edits made during the current session if interrupted

**Session Storage:**

- **No persistent sessions**: Sessions are not saved between CLI invocations
- **Temporary checkpoints**: File edits are checkpointed only during active sessions
- **Recovery scope**: Limited to recovering from interruptions within the same session

**Limitations:**

- No conversation persistence between sessions
- No ability to resume interrupted conversations
- Checkpointing is limited to file system changes only
- No session history or conversation restoration capabilities

### How is the use of MCP servers configured?

Gemini CLI provides MCP server configuration through command-line options and management commands:

**Command-line options:**

- `--allowed-mcp-server-names <names>`: Comma or space-separated list of allowed MCP server names

**MCP management commands:**

- `gemini mcp add <name> <commandOrUrl> [args...]`: Add a server (stdio or URL-based)
- `gemini mcp remove <name>`: Remove a server
- `gemini mcp list`: List all configured MCP servers

**Configuration files:**

- MCP server configurations stored in Gemini CLI's configuration directory
- No specific MCP configuration file format documented

**Environment variables:**

- No specific MCP-related environment variables documented in help screens

### Support for custom hooks

For Agent Time Travel feature (commands executed after every agent step), Gemini CLI does not appear to have built-in support for custom step-level hooks. However, it does support MCP-based extensibility:

1. **MCP Server Management**:
   - **Add servers**: `gemini mcp add <name> <commandOrUrl>` to add stdio or URL-based servers
   - **Remove servers**: `gemini mcp remove <name>` to remove configured servers
   - **List servers**: `gemini mcp list` to view all configured MCP servers
   - **Allowed servers**: `--allowed-mcp-server-names` to restrict which MCP servers can be used

2. **Extensions**:
   - **Extension loading**: `-e, --extensions` to specify which extensions to use
   - **List extensions**: `-l, --list-extensions` to see all available extensions
   - **Automatic extension discovery**: If no extensions specified, all available extensions are used

3. **Additional directories**: `--include-directories` to add extra directories to the workspace context

Note: While MCP servers and extensions exist, there is no documented support for automatic execution of custom commands after every agent step as required for the Agent Time Travel feature.

### Credentials

Gemini CLI uses Google's authentication system for credentials with the following precise storage locations:

**Authentication methods:**

- **Application Default Credentials (ADC)**: Automatically uses credentials from `gcloud auth application-default login`
- **Service Account Keys**: JSON key files for service accounts
- **OAuth 2.0**: Interactive OAuth flow for user authentication

**Environment variables:**

- `GOOGLE_API_KEY`: Environment variable for Gemini API key
- `GOOGLE_CLOUD_PROJECT`: Environment variable for GCP project ID
- `GOOGLE_APPLICATION_CREDENTIALS`: Path to service account key file

**Configuration files:**

- **ADC credentials**: `~/.config/gcloud/application_default_credentials.json`
- **Service account keys**: Custom paths specified via `GOOGLE_APPLICATION_CREDENTIALS`
- **gcloud config**: `~/.config/gcloud/configurations/config_default` (if using gcloud CLI)
- **Gemini CLI config**: Stored in platform-specific user configuration directories

**Precise storage paths** (platform-dependent):

- **Linux/macOS**: `~/.config/gemini-cli/` or `~/.config/gcloud/`
- **Windows**: `%APPDATA%\gemini-cli\` or `%APPDATA%\gcloud\`
- **OAuth tokens**: Stored in OS keychain/keyring or gcloud credential files

### Known Issues

- **Experimental status**: Many features are marked as experimental (ACP mode, some extensions)
- **Google Cloud dependency**: Requires Google Cloud authentication or API keys
- **Network dependency**: Requires internet access for Gemini API communication
- **Rate limiting**: Subject to Google's API rate limits and quotas
- **Checkpoint stability**: Checkpointing feature may have stability issues in complex scenarios
- **MCP compatibility**: MCP server compatibility may vary
