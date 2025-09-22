import json
import os
import sys
import uuid
import subprocess
from typing import Dict, Any, List
from .session_io import RolloutRecorder, SessionLogger, ClaudeSessionRecorder, _now_iso_ms
from .tools import call_tool, ToolError

def _print_trace(kind: str, msg: str) -> None:
    sys.stdout.write(f"[{kind}] {msg}\n")
    sys.stdout.flush()

def _as_json(obj: Any) -> str:
    return json.dumps(obj, ensure_ascii=False)

def _execute_hooks(hooks_config: Dict[str, Any], event_name: str, hook_input: Dict[str, Any], workspace: str) -> None:
    """Execute hooks for a given event."""
    if not hooks_config or event_name not in hooks_config:
        return

    session_id = hooks_config.get("session_id", "mock-session-123")
    cwd = workspace

    for matcher_config in hooks_config[event_name]:
        matcher = matcher_config.get("matcher", "*")
        hooks = matcher_config.get("hooks", [])

        # For simplicity, we'll execute hooks for all matchers in this mock implementation
        # In a real implementation, you'd check the matcher against the tool name
        for hook in hooks:
            if hook.get("type") == "command":
                command = hook.get("command", "")
                timeout = hook.get("timeout", 60)

                # Prepare hook input JSON
                hook_input_data = {
                    "session_id": session_id,
                    "transcript_path": "/tmp/mock-transcript.jsonl",  # Mock path
                    "cwd": cwd,
                    "hook_event_name": event_name,
                    **hook_input
                }

                try:
                    # Execute the hook command with JSON input via stdin
                    process = subprocess.Popen(
                        command,
                        shell=True,
                        cwd=cwd,
                        stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                        text=True,
                        env={**os.environ, "CLAUDE_PROJECT_DIR": workspace}
                    )

                    # Send JSON input
                    stdout, stderr = process.communicate(
                        input=json.dumps(hook_input_data),
                        timeout=timeout
                    )

                    _print_trace("hook", f"Executed {event_name} hook: {command}")
                    if stdout.strip():
                        _print_trace("hook", f"Hook stdout: {stdout.strip()}")
                    if stderr.strip():
                        _print_trace("hook", f"Hook stderr: {stderr.strip()}")

                except subprocess.TimeoutExpired:
                    _print_trace("hook", f"Hook timeout: {command}")
                    process.kill()
                except Exception as e:
                    _print_trace("hook", f"Hook execution failed: {command} - {e}")

def run_scenario(scenario_path: str, workspace: str, codex_home: str = os.path.expanduser("~/.codex"), format: str = "codex") -> str:
    os.makedirs(workspace, exist_ok=True)
    with open(scenario_path, "r", encoding="utf-8") as f:
        scenario = json.load(f)

    # Extract hooks configuration
    hooks_config = scenario.get("hooks", {})

    if format == "claude":
        return _run_scenario_claude(scenario, workspace, codex_home, hooks_config)
    else:
        return _run_scenario_codex(scenario, workspace, codex_home, hooks_config)


def _run_scenario_codex(scenario: Dict[str, Any], workspace: str, codex_home: str, hooks_config: Dict[str, Any]) -> str:
    """Run scenario using Codex format."""
    recorder = RolloutRecorder(codex_home=codex_home, cwd=workspace, instructions=scenario.get("meta",{}).get("instructions"))
    logger = SessionLogger(codex_home=codex_home)

    tc = scenario.get("meta",{}).get("turn_context", {
        "cwd": workspace,
        "approval_policy": "on_failure",
        "sandbox_policy": "workspace_write",
        "model": "mock-model",
        "effort": "medium",
        "summary": "concise"
    })
    recorder.record_turn_context(tc)
    logger.to_tui("insert_history", lines=1)

    for step in scenario.get("turns", []):
        if "user" in step:
            text = step["user"]
            _print_trace("user", text)
            recorder.record_message("user", text)
            logger.from_tui("op", payload={"type": "send_message", "content": text})
        elif "think" in step:
            text = step["think"]
            _print_trace("thinking", text)
            recorder.record_reasoning(summary_text=text)
            recorder.record_event("agent_message", {"message": text, "id": f"msg_{uuid.uuid4().hex[:6]}"})
        elif "tool" in step:
            call = step["tool"]
            name = call["name"]
            args = call.get("args", {})
            call_id = f"call_{uuid.uuid4().hex[:8]}"
            recorder.record_function_call(name=name, arguments=_as_json(args), call_id=call_id)
            _print_trace("tool", f"{name}({args}) -> executing")
            try:
                result = call_tool(name, workspace, **args)
                _print_trace("tool", f"{name} -> ok {result}")
                recorder.record_event("agent_message", {"message": f"tool {name} ok", "id": call_id})

                # Execute PostToolUse hooks
                hook_input = {
                    "tool_name": name,
                    "tool_input": args,
                    "tool_response": {"success": True}
                }
                _execute_hooks(hooks_config, "PostToolUse", hook_input, workspace)

            except ToolError as e:
                _print_trace("tool", f"{name} -> error {e}")
                recorder.record_event("agent_message", {"message": f"tool {name} error: {e}", "id": call_id})

                # Execute PostToolUse hooks for failed tools too
                hook_input = {
                    "tool_name": name,
                    "tool_input": args,
                    "tool_response": {"success": False, "error": str(e)}
                }
                _execute_hooks(hooks_config, "PostToolUse", hook_input, workspace)
        elif "assistant" in step:
            text = step["assistant"]
            _print_trace("assistant", text)
            recorder.record_message("assistant", text)
        elif "shell" in step:
            cmd = step["shell"]["cmd"]
            call_id = recorder.record_local_shell_call(command=cmd, cwd=workspace, status="in_progress")
            recorder.record_local_shell_call(command=cmd, cwd=workspace, status="completed", call_id=call_id)
            _print_trace("shell", f"{cmd} (simulated)")
        elif "event" in step:
            e = step["event"]
            recorder.record_event(e.get("type","agent_message"), e.get("payload", {}))
        elif "compacted" in step:
            recorder.record_compacted(step["compacted"])
        elif "turn_context" in step:
            recorder.record_turn_context(step["turn_context"])
        else:
            _print_trace("warn", f"Unknown step: {step}")
    recorder.flush()
    logger.close()
    return recorder.rollout_path


def _run_scenario_claude(scenario: Dict[str, Any], workspace: str, codex_home: str, hooks_config: Dict[str, Any]) -> str:
    """Run scenario using Claude format."""
    recorder = ClaudeSessionRecorder(codex_home=codex_home, cwd=workspace)
    
    # Record initial user message if present in meta
    instructions = scenario.get("meta", {}).get("instructions")
    if instructions:
        recorder.record_user_message(instructions, is_meta=True)

    for step in scenario.get("turns", []):
        if "user" in step:
            text = step["user"]
            _print_trace("user", text)
            recorder.record_user_message(text)
            
        elif "think" in step:
            text = step["think"]
            _print_trace("thinking", text)
            recorder.record_assistant_message(f"I need to think about this: {text}")
            
        elif "tool" in step:
            call = step["tool"]
            name = call["name"]
            args = call.get("args", {})
            
            # Record tool use
            tool_call_id = recorder.record_assistant_tool_use(name, args)
            _print_trace("tool", f"{name}({args}) -> executing")
            
            try:
                result = call_tool(name, workspace, **args)
                _print_trace("tool", f"{name} -> ok {result}")

                # Create tool result data based on the tool type
                tool_result_data = _create_tool_result_data(name, result, args)
                recorder.record_tool_result(tool_call_id, str(result), is_error=False, tool_result_data=tool_result_data)

                # Execute PostToolUse hooks
                hook_input = {
                    "tool_name": name,
                    "tool_input": args,
                    "tool_response": {"success": True}
                }
                _execute_hooks(hooks_config, "PostToolUse", hook_input, workspace)

            except ToolError as e:
                _print_trace("tool", f"{name} -> error {e}")
                recorder.record_tool_result(tool_call_id, str(e), is_error=True, tool_result_data=f"Error: {e}")

                # Execute PostToolUse hooks for failed tools too
                hook_input = {
                    "tool_name": name,
                    "tool_input": args,
                    "tool_response": {"success": False, "error": str(e)}
                }
                _execute_hooks(hooks_config, "PostToolUse", hook_input, workspace)
                
        elif "assistant" in step:
            text = step["assistant"]
            _print_trace("assistant", text)
            recorder.record_assistant_message(text)
            
        elif "shell" in step:
            cmd = step["shell"]["cmd"]
            _print_trace("shell", f"{cmd} (simulated)")
            # For shell commands, record as bash tool use
            tool_call_id = recorder.record_assistant_tool_use("Bash", {"command": cmd, "description": f"Execute: {cmd}"})
            recorder.record_tool_result(tool_call_id, f"Command executed: {cmd}", tool_result_data={"stdout": f"Simulated output for: {cmd}", "stderr": "", "interrupted": False})
            
        else:
            _print_trace("warn", f"Unknown step: {step}")
            
    recorder.flush()
    recorder.close()
    return recorder.session_path


def _create_tool_result_data(tool_name: str, result: Any, args: Dict[str, Any]) -> Any:
    """Create appropriate tool result data based on tool type."""
    if tool_name == "write_file":
        return {
            "type": "text", 
            "file": {
                "filePath": args.get("path", "unknown"),
                "content": args.get("text", ""),
                "numLines": len(str(args.get("text", "")).split("\n")),
                "startLine": 1,
                "totalLines": len(str(args.get("text", "")).split("\n"))
            }
        }
    elif tool_name == "read_file":
        return {
            "type": "text",
            "file": {
                "filePath": args.get("path", "unknown"),
                "content": str(result),
                "numLines": len(str(result).split("\n")) if result else 0,
                "startLine": 1,
                "totalLines": len(str(result).split("\n")) if result else 0
            }
        }
    elif tool_name in ["append_file", "replace_in_file"]:
        return {"path": args.get("path", "unknown"), "operation": tool_name}
    else:
        # Generic result for other tools
        return str(result) if result else "Operation completed"

def demo_scenario(workspace: str) -> Dict[str, Any]:
    return {
        "meta": {
            "instructions": "You are a helpful coding agent.",
            "turn_context": {
                "cwd": workspace,
                "approval_policy": "on_failure",
                "sandbox_policy": "workspace_write",
                "model": "mock-model",
                "effort": "medium",
                "summary": "concise"
            }
        },
        "turns": [
            {"user": "Please create hello.py that prints Hello, World!"},
            {"think": "I will create hello.py with a print statement."},
            {"tool": {"name": "write_file", "args": {"path": "hello.py", "text": "print('Hello, World!')\n"}}},
            {"assistant": "Created hello.py. Run with: python hello.py"},
            {"tool": {"name": "read_file", "args": {"path": "hello.py"}}},
            {"assistant": "Confirmed content of hello.py.\n```python\nprint('Hello, World!')\n```"}
        ]
    }
