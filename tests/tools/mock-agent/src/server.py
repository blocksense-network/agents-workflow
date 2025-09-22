import json
import os
import uuid
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse
from typing import Dict, Any
from .session_io import RolloutRecorder, SessionLogger

class Playbook:
    """
    Deterministic mapping from user prompts to responses/tool-calls.
    Format:
    { "rules": [ { "if_contains": [...], "response": {
        "assistant": "...", "tool_calls": [ { "name": "...", "args": {...}} ] } } ] }
    """
    def __init__(self, path: str):
        with open(path, "r", encoding="utf-8") as f:
            self.data = json.load(f)
        self.rules = self.data.get("rules", [])

    def match(self, text: str) -> Dict[str, Any]:
        t = text.lower()
        for r in self.rules:
            conds = [c.lower() for c in r.get("if_contains", [])]
            if all(c in t for c in conds):
                return r.get("response", {})
        return {"assistant": "Acknowledged. (no matching rule)", "tool_calls": []}

def _json_body(handler: BaseHTTPRequestHandler):
    length = int(handler.headers.get("Content-Length", "0"))
    raw = handler.rfile.read(length) if length > 0 else b"{}"
    return json.loads(raw.decode("utf-8"))

class MockAPIHandler(BaseHTTPRequestHandler):
    server_version = "MockAgentServer/0.1"

    def _send_json(self, code: int, obj: Dict[str, Any]):
        body = json.dumps(obj).encode("utf-8")
        self.send_response(code)
        self.send_header("content-type", "application/json; charset=utf-8")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        parsed = urlparse(self.path)
        if parsed.path == "/v1/chat/completions":
            self._handle_openai_chat_completions()
        elif parsed.path == "/v1/messages":
            self._handle_anthropic_messages()
        else:
            self._send_json(404, {"error":"not found"})

    def _infer_text_from_messages(self, messages) -> str:
        for m in reversed(messages):
            if m.get("role") == "user":
                content = m.get("content")
                if isinstance(content, list):
                    for b in content:
                        if b.get("type") == "text":
                            return b.get("text", "")
                    return ""
                return str(content)
        return ""

    def _respond_with(self, user_text: str, provider: str):
        pb: Playbook = self.server.playbook  # type: ignore
        resp = pb.match(user_text)
        assistant_text = resp.get("assistant", "")
        tool_calls = resp.get("tool_calls", [])

        recorder: RolloutRecorder = self.server.recorder  # type: ignore
        recorder.record_message("user", user_text)
        if assistant_text:
            recorder.record_reasoning(summary_text=f"[{provider}] planning response for: {user_text}")
            recorder.record_message("assistant", assistant_text)
        for tc in tool_calls:
            recorder.record_function_call(name=tc["name"], arguments=json.dumps(tc.get("args", {})))
        return assistant_text, tool_calls

    def _handle_openai_chat_completions(self):
        body = _json_body(self)
        messages = body.get("messages", [])
        user_text = self._infer_text_from_messages(messages)
        assistant_text, tool_calls = self._respond_with(user_text, provider="openai")

        tc = []
        for _idx, t in enumerate(tool_calls):
            tc.append({
                "id": f"call_{uuid.uuid4().hex[:8]}",
                "type": "function",
                "function": {
                    "name": t["name"],
                    "arguments": json.dumps(t.get("args", {}))
                }
            })
        obj = {
            "id": f"chatcmpl-{uuid.uuid4().hex}",
            "object": "chat.completion",
            "created": int(uuid.uuid1().time/1e7),
            "model": body.get("model", "mock-model"),
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": assistant_text,
                    "tool_calls": tc if tc else None
                },
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
        }
        if obj["choices"][0]["message"]["tool_calls"] is None:
            del obj["choices"][0]["message"]["tool_calls"]
        self._send_json(200, obj)

    def _handle_anthropic_messages(self):
        body = _json_body(self)
        messages = body.get("messages", [])
        user_text = self._infer_text_from_messages(messages)
        assistant_text, tool_calls = self._respond_with(user_text, provider="anthropic")

        content = []
        if assistant_text:
            content.append({"type": "text", "text": assistant_text})
        for t in tool_calls:
            content.append({
                "type": "tool_use",
                "id": f"toolu_{uuid.uuid4().hex[:8]}",
                "name": t["name"],
                "input": t.get("args", {})
            })
        obj = {
            "id": f"msg_{uuid.uuid4().hex}",
            "type": "message",
            "role": "assistant",
            "model": body.get("model", "mock-model"),
            "content": content,
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 0, "output_tokens": 0}
        }
        self._send_json(200, obj)

class MockAPIServer(HTTPServer):
    def __init__(self, server_address, RequestHandlerClass, codex_home, playbook_path):
        super().__init__(server_address, RequestHandlerClass)
        self.playbook = Playbook(playbook_path)
        self.recorder = RolloutRecorder(codex_home=codex_home, originator="mock-api-server")

def serve(host: str, port: int, playbook: str, codex_home: str):
    httpd = MockAPIServer((host, port), MockAPIHandler, codex_home=codex_home, playbook_path=playbook)
    print(f"Mock API server listening on http://{host}:{port}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("Shutting down server...")
    finally:
        httpd.server_close()
        httpd.recorder.close()
