# Mock Agent Integration Testing

This document describes the integration testing framework for the mock-agent with real CLI tools (claude and codex).

## Overview

The integration testing framework verifies that:

1. **Mock agent can serve as an API server** for OpenAI-compatible and Anthropic-compatible requests
2. **CLI tools can connect** to the mock agent and execute commands
3. **File operations work correctly** in temporary workspaces
4. **Multi-step workflows** complete successfully
5. **Session recording** captures all interactions properly

## Test Architecture

```
┌─────────────────┐    HTTP API     ┌─────────────────┐
│   CLI Tools     │◄────────────────┤   Mock Agent    │
│ (claude/codex)  │                 │   API Server    │
└─────────────────┘                 └─────────────────┘
         │                                   │
         │                                   │
         ▼                                   ▼
┌─────────────────┐                ┌─────────────────┐
│   Workspace     │                │   Session       │
│   Files         │                │   Recording     │
└─────────────────┘                └─────────────────┘
```

## Components

### 1. Mock API Server (`src/server.py`)

- Implements OpenAI-compatible `/v1/chat/completions` endpoint
- Implements Anthropic-compatible `/v1/messages` endpoint
- Uses playbook rules to generate deterministic responses
- Records all interactions in session files

### 2. Integration Test Suite (`tests/test_agent_integration.py`)

- Comprehensive test cases for file operations
- Multi-step workflow verification
- Workspace isolation testing
- Server health checks

### 3. Test Playbooks (`examples/comprehensive_playbook.json`)

- Rules for common coding scenarios
- File creation, modification, and testing workflows
- Git operations and project setup
- Error handling and edge cases

### 4. Test Utilities

- `start_test_server.py` - Start mock server for manual testing
- `run_integration_tests.py` - Test runner with dependency checks

## Quick Start

### 1. Start the Mock Server

```bash
# Start server with comprehensive playbook
python start_test_server.py

# Or with custom configuration
python start_test_server.py --port 8080 --playbook examples/comprehensive_playbook.json
```

### 2. Configure CLI Tools

For **Codex**:

```bash
export CODEX_API_BASE=http://127.0.0.1:18080/v1
export CODEX_API_KEY=mock-key
```

For **Claude Code**:

```bash
# Note: Claude Code may not support custom API endpoints
# Integration testing may be limited
```

### 3. Run Manual Tests

```bash
# Create a test workspace
mkdir test_workspace
cd test_workspace

# Test file creation
codex exec --dangerously-bypass-approvals-and-sandbox "Create hello.py that prints Hello, World!"

# Test multi-step workflow
codex exec --dangerously-bypass-approvals-and-sandbox "Create calculator.py with add and subtract functions"
codex exec --dangerously-bypass-approvals-and-sandbox "Create test calculator with unit tests"
```

### 4. Run Automated Tests

```bash
# Run full integration test suite
python tests/test_agent_integration.py

# Or run with the test runner
python run_integration_tests.py --tool codex --scenario all
```

## Test Scenarios

### Basic File Operations

- **hello_world**: Create a simple Python file
- **file_modification**: Modify existing files
- **file_reading**: Read and verify file contents

### Multi-Step Workflows

- **calculator_development**: Create calculator + tests
- **web_server_creation**: Build a simple web server
- **documentation_generation**: Create README files
- **git_workflow**: Initialize repo and commit changes

### Advanced Scenarios

- **refactoring**: Improve existing code structure
- **error_handling**: Test error conditions and recovery
- **workspace_isolation**: Verify test isolation

## API Compatibility

### OpenAI Format (`/v1/chat/completions`)

```json
{
  "model": "gpt-4",
  "messages": [{ "role": "user", "content": "Create hello.py" }]
}
```

Response:

```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "I'll create hello.py for you.",
        "tool_calls": [
          {
            "id": "call_xxx",
            "type": "function",
            "function": {
              "name": "write_file",
              "arguments": "{\"path\": \"hello.py\", \"text\": \"print('Hello, World!')\\n\"}"
            }
          }
        ]
      }
    }
  ]
}
```

### Anthropic Format (`/v1/messages`)

```json
{
  "model": "claude-3-sonnet",
  "messages": [{ "role": "user", "content": "Create hello.py" }]
}
```

Response:

```json
{
  "id": "msg_xxx",
  "type": "message",
  "role": "assistant",
  "content": [
    { "type": "text", "text": "I'll create hello.py for you." },
    {
      "type": "tool_use",
      "id": "toolu_xxx",
      "name": "write_file",
      "input": { "path": "hello.py", "text": "print('Hello, World!')\\n" }
    }
  ]
}
```

## Session Recording

All interactions are recorded in session files with the specified format (Codex or Claude):

### Codex Format

```json
{"type": "user_message", "text": "Create hello.py", "timestamp": "..."}
{"type": "thinking", "text": "[openai] planning response for: Create hello.py"}
{"type": "assistant_message", "text": "I'll create hello.py for you."}
{"type": "function_call", "name": "write_file", "arguments": "{...}"}
```

### Claude Format

```json
{"message_uuid": "...", "type": "user_message", "content": "Create hello.py"}
{"message_uuid": "...", "type": "assistant_message", "content": "I'll create hello.py for you."}
{"message_uuid": "...", "type": "tool_use", "tool_name": "write_file", "parameters": {...}}
```

## Troubleshooting

### Server Issues

- **Port already in use**: Change port with `--port` flag
- **Playbook not found**: Verify path to playbook JSON file
- **Permission errors**: Check write permissions for session directory

### CLI Tool Issues

- **Tool not found**: Verify claude/codex are in PATH
- **Connection refused**: Ensure mock server is running
- **Authentication errors**: Set proper API base URL and key

### Test Failures

- **File not created**: Check playbook rules match prompts
- **Workspace isolation**: Verify temp directories are separate
- **Timeout errors**: Increase timeout values in test configuration

## Extending Tests

### Adding New Scenarios

1. Update `comprehensive_playbook.json` with new rules:

```json
{
  "if_contains": ["create", "new_feature"],
  "response": {
    "assistant": "I'll create the new feature.",
    "tool_calls": [...]
  }
}
```

2. Add test methods to `test_agent_integration.py`:

```python
def test_new_feature_creation(self):
    result = self.run_codex_command("Create new feature")
    self.assertEqual(result.returncode, 0)
    # Verify expected files/behavior
```

### Supporting New CLI Tools

1. Study the tool's API format and configuration
2. Add support in `MockAPIHandler` if needed
3. Create test methods following existing patterns
4. Update documentation and examples

## Limitations

- **Claude Code**: May not support custom API endpoints
- **Authentication**: Mock server uses simple API key validation
- **Tool Execution**: Mock agent doesn't actually execute tools, only records calls
- **Error Simulation**: Limited error condition testing

## Future Enhancements

- **Real tool execution**: Actually perform file operations
- **Advanced scenarios**: Database interactions, network requests
- **Performance testing**: Load testing with multiple concurrent clients
- **Error injection**: Simulate API failures and recovery
- **Session replay**: Replay recorded sessions for debugging
