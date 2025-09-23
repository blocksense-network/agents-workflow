#!/bin/bash
# Build script for AgentFSKitExtension
# This script builds the Rust crates and prepares them for Swift linking

set -e

echo "Building AgentFS FSKit Extension..."

# Get the project root (assuming this script is in adapters/macos/xcode/AgentFSKitExtension/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
echo "Script dir: $SCRIPT_DIR"
echo "Calculated project root: $PROJECT_ROOT"

# Verify we have the correct project root by checking for Cargo.toml
if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
    echo "Error: Could not find Cargo.toml in $PROJECT_ROOT"
    echo "Script dir: $SCRIPT_DIR"
    echo "Calculated project root: $PROJECT_ROOT"
    exit 1
fi

echo "Project root: $PROJECT_ROOT"

# Build the Rust crates first
echo "Building Rust crates..."
cd "$PROJECT_ROOT"
cargo build --release -p agentfs-fskit-sys -p agentfs-fskit-bridge -p agentfs-core -p agentfs-proto -p agentfs-ffi

# Copy the built libraries to the Swift project
echo "Copying Rust libraries to Swift project..."
mkdir -p "$SCRIPT_DIR/libs"
cp "$PROJECT_ROOT/target/release/libagentfs_fskit_sys.a" "$SCRIPT_DIR/libs/"
cp "$PROJECT_ROOT/target/release/libagentfs_fskit_bridge.a" "$SCRIPT_DIR/libs/"
cp "$PROJECT_ROOT/target/release/libagentfs_ffi.a" "$SCRIPT_DIR/libs/"

# Copy system dependencies (if any)
# Note: On macOS, we might need to link against system libraries
# This is handled by the Package.swift linker settings

echo "Rust build complete. Libraries are ready for Swift linking."
echo "You can now build the Swift extension with: swift build"

# Note: The actual Swift build should be done in Xcode
# This script just prepares the dependencies
