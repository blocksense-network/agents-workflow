# Goose — Integration Notes

> Usually this information can be obtained by checking out command-line help screens or man pages for the agent software.

## Overview

Goose is an AI-powered coding assistant CLI tool developed by Block. It provides an interactive coding experience with support for various AI providers and extensions.

- **Website**: <https://block.github.io/goose/>
- **Documentation**: <https://block.github.io/goose/docs/>
- **GitHub**: <https://github.com/block/goose>
- **Version**: 1.6.0 (as of this writing)

### Task start-up command

Goose can be started with a specific task prompt in several ways:

1. **Direct text input**:

   ```bash
   goose run -t "Implement a user authentication system"
   ```

2. **From a file**:

   ```bash
   goose run -i instructions.txt
   ```

3. **Interactive session**:

   ```bash
   goose session
   ```

4. **With custom system instructions**:
   ```bash
   goose run --system "You are a senior Ruby developer" -t "Refactor this legacy code"
   ```

### Built-in support for checkpoints

Yes, Goose supports checkpoints through its session management system. The checkpoints cover **both chat content and file system state**.

**Session Management:**

- **Session creation**: Sessions are automatically created when running commands
- **Resume capability**: Use `goose session --resume` to continue from where you left off
- **Named sessions**: Use `--name <NAME>` to create and resume identifiable sessions
- **Path-based sessions**: Use `--path <PATH>` to specify session location
- **Session export**: `goose session export` can export sessions to Markdown format
- **Session listing**: `goose session list` shows all available sessions
- **Session removal**: `goose session remove` can delete specific sessions

**Checkpoint Coverage:**

- **Chat content**: Full conversation history and context is preserved and restored
- **File system state**: Context about the working directory and project is maintained
- **Tool execution history**: Preserved within the session data

**Restoring from Specific Moments:**

- **By session name**: Use `goose session --resume --name <session_name>` to restore a named session
- **By session path**: Use `goose session --resume --path <path>` to restore a path-specific session
- **By last session**: Use `goose session --resume` to restore the most recent session
- **No granular restoration**: Cannot restore to a specific chat message or prompt position within a session
- **File system restoration**: Maintains working directory context but does not restore file snapshots

**Session Storage:**

- **Location**: `~/.local/share/goose/sessions/`
- **Format**: Internal Goose format (can be exported to Markdown)
- **Persistence**: Sessions persist across Goose restarts

**Limitations:**

- No file system snapshots - only maintains working directory context
- No ability to rollback file changes made during a session
- Session restoration maintains conversation and directory context but not file states

### How is the use of MCP servers configured?

Goose supports MCP servers through both built-in and external configurations:

**Command-line options:**

- `--with-builtin <NAME>`: Add one or more builtin extensions that are bundled with goose
- `--with-extension <COMMAND>`: Add stdio extensions from full commands with environment variables
- `--with-remote-extension <URL>`: Add remote extensions from a URL
- `--with-streamable-http-extension <URL>`: Add streamable HTTP extensions from a URL

**Built-in MCP servers:**

- `goose mcp <NAME>`: Run one of the MCP servers bundled with goose

**Configuration files:**

- `~/.config/goose/config.yaml`: Main configuration file for MCP server settings

**Environment variables:**

- No specific MCP-related environment variables documented

### Support for custom hooks

For Agent Time Travel feature (commands executed after every agent step), Goose does not appear to have built-in support for custom step-level hooks. However, it does support broader customization through:

1. **Recipes**: YAML-based configuration files that define custom agent behaviors
   - Use `--recipe` to specify a recipe file
   - Recipes can include custom parameters with `--params`

2. **Extensions**: Various extension mechanisms that could potentially be adapted for hook-like functionality
   - **Built-in extensions**: `--with-builtin` to add bundled extensions
   - **Stdio extensions**: `--with-extension` for custom command-based extensions
   - **Remote extensions**: `--with-remote-extension` for URL-based extensions
   - **Streamable HTTP extensions**: `--with-streamable-http-extension` for HTTP-based extensions

Note: While extensions exist, there is no documented support for automatic execution of custom commands after every agent step as required for the Agent Time Travel feature.

### Credentials

Goose stores credentials and configuration in the following precise locations:

**Configuration files:**

- **Main config file**: `~/.config/goose/config.yaml` - Contains provider settings, API keys, and authentication details
- **Sessions directory**: `~/.local/share/goose/sessions` - Stores session data and conversation history
- **Logs directory**: `~/.local/state/goose/logs` - Contains log files

**Environment variables:**

- Various `GOOSE_*` environment variables for provider configuration (exact variables depend on configured providers)

**Provider-specific storage:**

- API keys are stored in the main config file (`~/.config/goose/config.yaml`) for different AI providers (OpenAI, Anthropic, etc.)
- Provider-specific authentication tokens and credentials are managed within the config file structure

### Known Issues

- **Provider compatibility**: Some features may vary depending on the chosen AI provider
- **Extension stability**: Custom extensions may have varying levels of stability
- **Resource usage**: Sessions and extensions can consume significant system resources
- **Network dependency**: Requires internet access for AI provider communication
