#!/usr/bin/env bash
set -euo pipefail

SOCKET_PATH="/tmp/agent-workflow/aw-fs-snapshots-daemon"

if [ ! -e "$SOCKET_PATH" ]; then
  echo "AW filesystem snapshots daemon is not running (socket not found: $SOCKET_PATH)"
  exit 1
fi

echo "Stopping AW filesystem snapshots daemon..."

# Send interrupt signal to the daemon process (let it clean up gracefully)
sudo pkill -INT -f "aw-fs-snapshots-daemon" || true

# Wait for graceful shutdown
for i in {1..10}; do
  if [ ! -e "$SOCKET_PATH" ]; then
    echo "AW filesystem snapshots daemon stopped successfully"
    exit 0
  fi
  sleep 0.5
done

# If still not cleaned up, force kill
echo "Warning: Daemon didn't shut down gracefully, force killing..."
sudo pkill -KILL -f "aw-fs-snapshots-daemon" || true
sleep 1

if [ -e "$SOCKET_PATH" ]; then
  echo "Warning: Socket still exists, manually cleaning up..."
  sudo rm -f "$SOCKET_PATH"
fi

echo "AW filesystem snapshots daemon forcefully stopped"
