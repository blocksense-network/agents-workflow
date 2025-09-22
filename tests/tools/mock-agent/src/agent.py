import json
import os
import sys
import uuid
from typing import Dict, Any, List
from .session_io import RolloutRecorder, SessionLogger, _now_iso_ms
from .tools import call_tool, ToolError

def _print_trace(kind: str, msg: str) -> None:
    sys.stdout.write(f"[{kind}] {msg}\n")
    sys.stdout.flush()

def _as_json(obj: Any) -> str:
    return json.dumps(obj, ensure_ascii=False)

def run_scenario(scenario_path: str, workspace: str, codex_home: str = os.path.expanduser("~/.codex")) -> str:
    os.makedirs(workspace, exist_ok=True)
    with open(scenario_path, "r", encoding="utf-8") as f:
        scenario = json.load(f)

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
            except ToolError as e:
                _print_trace("tool", f"{name} -> error {e}")
                recorder.record_event("agent_message", {"message": f"tool {name} error: {e}", "id": call_id})
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
