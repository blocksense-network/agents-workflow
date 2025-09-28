#!/usr/bin/env bash
set -euo pipefail

# Cleanup function for graceful exit
cleanup() {
  echo "Cleaning up servers..."
  kill $MOCK_PID $SSR_PID 2>/dev/null || true
}
trap cleanup EXIT

# Start mock server (port 3001)
cd ../mock-server
npm run dev &
MOCK_PID=$!
echo "Started mock server (PID: $MOCK_PID)"

# Start SSR sidecar (port 3000)
cd ../app-ssr-server
npm run start &
SSR_PID=$!
echo "Started SSR server (PID: $SSR_PID)"

# Wait for ports to be ready (using nc from Nix)
echo "Waiting for servers to be ready..."
for i in {1..30}; do
  if nc -z localhost 3001 && nc -z localhost 3000; then
    echo "Servers ready!"
    break
  fi
  sleep 1
done

if ! nc -z localhost 3001 || ! nc -z localhost 3000; then
  echo "Timeout: Servers not ready"
  exit 1
fi

# Run tests (back in e2e-tests dir)
cd ../e2e-tests
npx playwright test "$@"

# Cleanup happens via trap
