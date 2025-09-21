#!/usr/bin/env bash
set -euo pipefail

SOCKET_PATH="/tmp/agent-workflow/aw-fs-snapshots-daemon"

if [ -e "$SOCKET_PATH" ]; then
  if ruby -e "require 'socket'; UNIXSocket.open('$SOCKET_PATH').close" 2>/dev/null; then
    echo "AW filesystem snapshots daemon is running (socket: $SOCKET_PATH)"
  else
    echo "Warning: Socket exists but daemon is not responding"
  fi
else
  echo "AW filesystem snapshots daemon is not running"
  echo "Start it with: just launch-aw-fs-snapshots-daemon"
fi
