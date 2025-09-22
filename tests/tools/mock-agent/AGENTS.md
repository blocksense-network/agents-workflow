# Mock Agent Testing Guide

This document provides comprehensive instructions for running and validating the mock coding agent's non-interactive tests.

## Overview

The mock agent includes a full test suite that verifies:

- CLI functionality and help text
- File operations and workspace management
- Session file creation in both Codex and Claude formats
- Tool execution and result handling
- Terminal output validation with pexpect

## Quick Start

```bash
# Run the full test suite
python tests/test_agent_simple.py

# Expected output: All tests pass
[TEST] 🎉 All tests passed!
```

## Test Suite Structure

### Core Test Classes

**`TestRunner`**: Main test orchestrator that manages test execution and reporting.

**Test Methods:**

- `test_cli_help()`: Validates CLI help text and format flag options
- `test_hello_scenario_file_creation()`: Tests basic file creation workflow
- `test_hello_scenario_terminal_output()`: Validates terminal output using pexpect
- `test_demo_scenario()`: Tests built-in demo functionality
- `test_rollout_file_creation()`: Verifies Codex format session files
- `test_file_operations()`: Tests various file operations (write, read, append)
- `test_claude_format_session_files()`: Validates Claude format session files

## Detailed Test Descriptions

### 1. CLI Help Test (`test_cli_help`)

**Purpose**: Verifies command-line interface functionality

**Validation:**

- Main help displays correctly
- All subcommands are listed (run, demo, server)
- Format flag is available with correct choices
- Help text includes expected content

```bash
# Manual verification
python -m src.cli --help
python -m src.cli run --help
```

### 2. File Creation Test (`test_hello_scenario_file_creation`)

**Purpose**: Tests basic scenario execution and file creation

**Process:**

1. Creates temporary workspace and codex home
2. Runs hello scenario with Codex format
3. Verifies `hello.py` file creation
4. Validates file content matches expected output

**Expected Result:**

- File created: `hello.py`
- Content: `print('Hello, World!')`

### 3. Terminal Output Test (`test_hello_scenario_terminal_output`)

**Purpose**: Validates real-time terminal output using pexpect

**Process:**

1. Spawns agent process with pexpect
2. Captures and validates output patterns:
   - `[user] Please create hello.py...`
   - `[thinking] I'll create hello.py...`
   - `[tool] write_file`
   - `[tool] write_file -> ok`
   - `[assistant] Created hello.py...`

**Technical Notes:**

- Uses regex patterns for flexible matching
- Handles process exit status (may be None on success)
- Validates file creation as additional confirmation

### 4. Demo Scenario Test (`test_demo_scenario`)

**Purpose**: Tests built-in demo functionality

**Process:**

1. Runs demo command with temporary workspace
2. Verifies demo scenario JSON creation
3. Validates JSON structure (meta, turns sections)
4. Confirms expected files are created

### 5. Rollout File Creation Test (`test_rollout_file_creation`)

**Purpose**: Validates Codex format session file creation

**Process:**

1. Runs scenario with Codex format
2. Verifies rollout file creation in date-based directory structure
3. Validates JSONL format (each line is valid JSON)
4. Confirms file permissions and location

**Expected Location:**

```
~/.codex/sessions/YYYY/MM/DD/rollout-YYYY-MM-DDThh-mm-ss-<uuid>.jsonl
```

### 6. File Operations Test (`test_file_operations`)

**Purpose**: Tests various file manipulation operations

**Process:**

1. Creates custom scenario with multiple file operations
2. Tests write_file, read_file, append_file operations
3. Verifies file content through each operation
4. Validates operation sequencing

**Operations Tested:**

- Initial file creation
- Content reading
- Content appending
- Final content verification

### 7. Claude Format Test (`test_claude_format_session_files`)

**Purpose**: Validates Claude format session file creation and structure

**Process:**

1. Runs scenario with `--format claude`
2. Verifies file creation in Claude directory structure
3. Validates session entry format and required fields
4. Confirms tool usage entries exist
5. Validates conversation threading

**Expected Location:**

```
~/.claude/projects/<encoded-project-path>/<session-uuid>.jsonl
```

**Validated Fields:**

- `parentUuid`, `isSidechain`, `userType`
- `cwd`, `sessionId`, `version`, `gitBranch`
- `type`, `message`, `uuid`, `timestamp`

## Running Individual Tests

### Test Specific Functionality

```bash
# Test only CLI help
python -c "
from tests.test_agent_simple import TestRunner
runner = TestRunner()
runner.run_test('CLI Help', runner.test_cli_help)
"

# Test only Claude format
python -c "
from tests.test_agent_simple import TestRunner
runner = TestRunner()
runner.run_test('Claude Format', runner.test_claude_format_session_files)
"
```

### Manual Testing Commands

```bash
# Test Codex format manually
WS=$(mktemp -d) && python -m src.cli demo --workspace "$WS" --format codex && ls -la "$WS"

# Test Claude format manually
WS=$(mktemp -d) && python -m src.cli demo --workspace "$WS" --format claude && ls -la "$WS"

# Verify session file creation
python -m src.cli run --scenario examples/hello_scenario.json --workspace /tmp/test-ws --format claude
find ~/.claude/projects -name "*.jsonl" | head -1 | xargs cat | jq .
```

## Environment Requirements

### Python Dependencies

- **pytest** (optional, for pytest-style testing)
- **pexpect** (required for terminal output testing)

### System Requirements

- Unix-like environment (macOS, Linux)
- Python 3.9+
- Git (for branch detection)
- Temporary directory access

### Development Environment Setup

The tests run in the current development environment. Ensure you're in the mock-agent directory:

```bash
cd tests/tools/mock-agent
python -m src.cli --help  # Verify basic functionality
```

## Troubleshooting

### Common Issues

**Test failures due to file permissions:**

```bash
# Ensure temp directories are writable
ls -la /tmp/
```

**Pexpect timeout issues:**

```bash
# Increase timeout in test if needed
# Edit tests/test_agent_simple.py, line ~109:
proc = pexpect.spawn(..., timeout=60)  # Increase from 30
```

**Missing git branch:**

```bash
# Initialize git repo if needed
git init
git checkout -b main
```

### Debugging Test Failures

**Verbose test output:**

```python
# Edit test_agent_simple.py to add debug output
def assert_true(condition, message="Assertion failed"):
    print(f"DEBUG: Checking {message}: {condition}")  # Add this line
    if not condition:
        # ... rest of function
```

**Examine session files:**

```bash
# Check Codex format files
find ~/.codex/sessions -name "*.jsonl" | head -1 | xargs cat | jq .

# Check Claude format files
find ~/.claude/projects -name "*.jsonl" | head -1 | xargs cat | jq .
```

**Manual scenario testing:**

```bash
# Run scenario step by step
python -c "
import json
from src.agent import run_scenario
result = run_scenario('examples/hello_scenario.json', '/tmp/debug-ws', format='claude')
print(f'Session file: {result}')
"
```

## Test Data and Fixtures

### Temporary Directory Management

Tests create and clean up temporary directories automatically:

- Workspace directories for file operations
- Codex home directories for session files
- Automatic cleanup on test completion (success or failure)

### Test Scenarios

**Built-in hello scenario** (`examples/hello_scenario.json`):

- User requests hello.py creation
- Agent thinks about the task
- Tool call to write_file
- Agent confirms completion
- Tool call to read_file for verification

**Custom test scenarios** (created during testing):

- Multi-step file operations
- Error condition testing
- Complex tool interaction patterns

## Performance Expectations

### Test Execution Times

- **Full test suite**: ~30-60 seconds
- **Individual tests**: ~3-10 seconds each
- **Terminal output test**: Longest due to pexpect interaction

### Resource Usage

- **Disk space**: Minimal (temp files cleaned up)
- **Memory**: Low footprint
- **CPU**: Brief spikes during JSON processing

## Integration Testing

### IDE Integration Testing

The mock agent can be used to test IDE integrations:

```bash
# Start mock server for IDE testing
python -m src.cli server --host 127.0.0.1 --port 8080 --playbook examples/playbook.json --format claude

# Test with curl
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Help me code"}]}'
```

### Session File Validation

```bash
# Validate session file format compliance
python -c "
import json
with open('path/to/session.jsonl') as f:
    for i, line in enumerate(f):
        try:
            json.loads(line.strip())
            print(f'Line {i+1}: Valid JSON')
        except json.JSONDecodeError as e:
            print(f'Line {i+1}: Invalid JSON - {e}')
"
```

## Continuous Integration

### Automated Testing

For CI/CD pipelines, run tests with:

```bash
#!/bin/bash
set -e

cd tests/tools/mock-agent

# Run test suite
python tests/test_agent_simple.py

# Verify exit code
if [ $? -eq 0 ]; then
    echo "✅ All mock agent tests passed"
else
    echo "❌ Mock agent tests failed"
    exit 1
fi
```

### Test Coverage

The current test suite provides:

- **CLI coverage**: All commands and flags
- **Format coverage**: Both Codex and Claude formats
- **Tool coverage**: All supported file operations
- **Error handling**: Basic error conditions
- **Integration**: End-to-end scenario execution

## Contributing

### Adding New Tests

1. Add test method to `TestRunner` class
2. Follow naming convention: `test_<description>()`
3. Include in `run_all_tests()` method
4. Use `assert_true()` for validation
5. Include cleanup in `finally` blocks

### Test Guidelines

- **Isolation**: Each test should be independent
- **Cleanup**: Always clean up temporary resources
- **Assertions**: Use descriptive assertion messages
- **Timeouts**: Set reasonable timeouts for pexpect tests
- **Documentation**: Include docstrings explaining test purpose
