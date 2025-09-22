#!/usr/bin/env python3
"""
Start a mock API server for integration testing.

This script starts the mock-agent server with appropriate configuration
for testing with claude and codex CLI tools.
"""

import argparse
import os
import sys
import signal
import tempfile
from pathlib import Path

# Add the src directory to Python path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))
from cli import main as cli_main


def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully."""
    print("\nShutting down server...")
    sys.exit(0)


def main():
    parser = argparse.ArgumentParser(description="Start mock API server for testing")
    parser.add_argument("--host", default="127.0.0.1", help="Server host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=18080, help="Server port (default: 18080)")
    parser.add_argument("--playbook", help="Playbook JSON file (default: examples/comprehensive_playbook.json)")
    parser.add_argument("--format", choices=["codex", "claude"], default="codex",
                       help="Session file format (default: codex)")
    parser.add_argument("--session-dir", help="Directory for session files (default: temp dir)")
    
    args = parser.parse_args()
    
    # Set up signal handler for graceful shutdown
    signal.signal(signal.SIGINT, signal_handler)
    
    # Determine playbook path
    if args.playbook:
        playbook_path = args.playbook
    else:
        script_dir = os.path.dirname(__file__)
        playbook_path = os.path.join(script_dir, "examples", "comprehensive_playbook.json")
    
    if not os.path.exists(playbook_path):
        print(f"Error: Playbook file not found: {playbook_path}")
        return 1
    
    # Determine session directory
    if args.session_dir:
        session_dir = args.session_dir
        os.makedirs(session_dir, exist_ok=True)
    else:
        session_dir = tempfile.mkdtemp(prefix="mock_agent_sessions_")
        print(f"Using temporary session directory: {session_dir}")
    
    print(f"Starting mock API server...")
    print(f"  Host: {args.host}")
    print(f"  Port: {args.port}")
    print(f"  Playbook: {playbook_path}")
    print(f"  Format: {args.format}")
    print(f"  Session dir: {session_dir}")
    print()
    
    # Show usage instructions
    print("Usage with CLI tools:")
    print("=" * 40)
    
    print("\nFor Codex:")
    print(f"  export CODEX_API_BASE=http://{args.host}:{args.port}/v1")
    print(f"  export CODEX_API_KEY=mock-key")
    print(f"  codex exec --dangerously-bypass-approvals-and-sandbox 'Create hello.py'")
    
    print("\nFor Claude Code:")
    print("  # Note: Claude Code may not support custom API endpoints")
    print("  # This server primarily supports Codex-style API calls")
    
    print("\nAPI Endpoints:")
    print(f"  OpenAI-compatible: http://{args.host}:{args.port}/v1/chat/completions")
    print(f"  Anthropic-compatible: http://{args.host}:{args.port}/v1/messages")
    
    print("\nPress Ctrl+C to stop the server")
    print("=" * 40)
    
    # Prepare arguments for the CLI
    cli_args = [
        "server",
        "--host", args.host,
        "--port", str(args.port),
        "--playbook", playbook_path,
        "--codex-home", session_dir,
        "--format", args.format
    ]
    
    # Override sys.argv to pass arguments to the CLI
    original_argv = sys.argv
    try:
        sys.argv = ["mockagent"] + cli_args
        return cli_main()
    finally:
        sys.argv = original_argv


if __name__ == "__main__":
    sys.exit(main())