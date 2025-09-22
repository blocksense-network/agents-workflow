import os
import io
import json
import uuid
from dataclasses import dataclass, field
from typing import Any, Dict, Optional, List
from datetime import datetime, timezone

ISOZ = "%Y-%m-%dT%H:%M:%S.%fZ"

def _now_iso_ms() -> str:
    dt = datetime.now(timezone.utc)
    return dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{int(dt.microsecond/1000):03d}Z"

def _now_stamp_for_filename() -> str:
    dt = datetime.now(timezone.utc)
    return dt.strftime("%Y-%m-%dT%H-%M-%S")

def _date_parts():
    dt = datetime.now(timezone.utc)
    return dt.strftime("%Y"), dt.strftime("%m"), dt.strftime("%d")

@dataclass
class RolloutRecorder:
    codex_home: str = os.path.expanduser("~/.codex")
    originator: str = "mock-agent"
    cli_version: str = "0.1.0"
    instructions: Optional[str] = None
    cwd: Optional[str] = None
    git: Optional[Dict[str, Any]] = None

    rollout_dir: str = field(init=False)
    rollout_path: str = field(init=False)
    session_id: str = field(init=False)
    _fh: io.TextIOWrapper = field(init=False, repr=False)

    def __post_init__(self):
        year, month, day = _date_parts()
        sess_root = os.path.join(self.codex_home, "sessions", year, month, day)
        os.makedirs(sess_root, exist_ok=True)
        self.session_id = str(uuid.uuid4())
        fname = f"rollout-{_now_stamp_for_filename()}-{self.session_id}.jsonl"
        self.rollout_dir = sess_root
        self.rollout_path = os.path.join(sess_root, fname)
        self._fh = open(self.rollout_path, "w", encoding="utf-8")
        try:
            os.chmod(self.rollout_path, 0o600)
        except PermissionError:
            pass
        self._write_session_meta()

    def _write_jsonl(self, obj: Dict[str, Any]) -> None:
        self._fh.write(json.dumps(obj, ensure_ascii=False) + "\n")
        self._fh.flush()

    def _write_session_meta(self) -> None:
        line = {
            "timestamp": _now_iso_ms(),
            "type": "session_meta",
            "payload": {
                "meta": {
                    "id": self.session_id,
                    "timestamp": _now_iso_ms(),
                    "cwd": self.cwd or os.getcwd(),
                    "originator": self.originator,
                    "cli_version": self.cli_version,
                    "instructions": self.instructions,
                },
                "git": self.git or {}
            }
        }
        self._write_jsonl(line)

    def record_turn_context(self, payload: Dict[str, Any]) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "turn_context",
            "payload": payload
        })

    def record_message(self, role: str, text: str) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "message",
            "payload": {
                "role": role,
                "content": [ { "type": "text", "text": text } ]
            }
        })

    def record_reasoning(self, summary_text: str, content: Optional[List[Dict[str, Any]]] = None) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "reasoning",
            "payload": {
                "summary": [ { "type": "reasoning_text", "text": summary_text } ],
                "content": content or []
            }
        })

    def record_function_call(self, name: str, arguments: str, call_id: Optional[str] = None) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "function_call",
            "payload": {
                "name": name,
                "arguments": arguments,
                "call_id": call_id or f"call_{uuid.uuid4().hex[:6]}"
            }
        })

    def record_local_shell_call(self, command: str, cwd: Optional[str] = None, env: Optional[Dict[str, str]] = None, status: str = "in_progress", call_id: Optional[str] = None) -> str:
        cid = call_id or f"call_{uuid.uuid4().hex[:6]}"
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "local_shell_call",
            "payload": {
                "call_id": cid,
                "status": status,
                "action": {
                    "command": command,
                    "cwd": cwd or (self.cwd or os.getcwd()),
                    "env": env or {}
                }
            }
        })
        return cid

    def record_event(self, event_type: str, payload: Dict[str, Any]) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": event_type,
            "payload": payload
        })

    def record_compacted(self, message: str) -> None:
        self._write_jsonl({
            "timestamp": _now_iso_ms(),
            "type": "compacted",
            "payload": { "message": message }
        })

    def flush(self) -> None:
        self._fh.flush()

    def close(self) -> None:
        try:
            self._fh.flush()
            self._fh.close()
        except Exception:
            pass


class SessionLogger:
    """
    Optional UI session logger controlled by env var CODEX_TUI_RECORD_SESSION=1.
    Writes to ~/.codex/logs/session-YYYYMMDDTHHMMSSZ.jsonl
    """
    def __init__(self, codex_home: str = os.path.expanduser("~/.codex")):
        self.codex_home = codex_home
        self.enabled = os.getenv("CODEX_TUI_RECORD_SESSION", "0") == "1"
        self._fh = None
        if self.enabled:
            log_dir = os.path.join(self.codex_home, "logs")
            os.makedirs(log_dir, exist_ok=True)
            ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
            self.path = os.path.join(log_dir, f"session-{ts}.jsonl")
            self._fh = open(self.path, "w", encoding="utf-8")
            try:
                os.chmod(self.path, 0o600)
            except PermissionError:
                pass
            self._write_meta("session_start")

    def _now(self) -> str:
        dt = datetime.now(timezone.utc)
        return dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{int(dt.microsecond/1000):03d}Z"

    def _write(self, obj: dict) -> None:
        if not self.enabled:
            return
        self._fh.write(json.dumps(obj, ensure_ascii=False) + "\n")
        self._fh.flush()

    def _write_meta(self, kind: str, **extra) -> None:
        self._write({
            "ts": self._now(),
            "dir": "meta",
            "kind": kind,
            **extra
        })

    def to_tui(self, kind: str, **payload) -> None:
        self._write({
            "ts": self._now(),
            "dir": "to_tui",
            "kind": kind,
            **payload
        })

    def from_tui(self, kind: str, **payload) -> None:
        self._write({
            "ts": self._now(),
            "dir": "from_tui",
            "kind": kind,
            **payload
        })

    def close(self) -> None:
        if self.enabled and self._fh:
            self._write_meta("session_end")
            self._fh.flush()
            self._fh.close()
