#!/usr/bin/env bash
set -euo pipefail

SOCKET_PATH="/tmp/agent-workflow/aw-fs-snapshots-daemon"

if [ -e "$SOCKET_PATH" ]; then
  # Check if socket is actually accepting connections
  if ruby -e "require 'socket'; UNIXSocket.open('$SOCKET_PATH').close" 2>/dev/null; then
    echo "AW filesystem snapshots daemon is running (socket exists: $SOCKET_PATH)"
  else
    echo "AW filesystem snapshots daemon socket exists but is not responding"
    exit 1
  fi
else
  echo "AW filesystem snapshots daemon is not running"
  exit 1
fi
