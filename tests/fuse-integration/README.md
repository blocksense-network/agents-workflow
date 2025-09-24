# AgentFS FUSE Integration Tests

This directory contains comprehensive integration tests for the AgentFS FUSE adapter, implementing milestone M10.5 "FUSE Integration Testing Suite".

## Overview

The FUSE integration testing suite provides:

- **Mount Cycle Tests**: Full filesystem mount/unmount cycle validation
- **Filesystem Operations**: Complete file and directory operation testing
- **Control Plane Tests**: Snapshot, branch, and process binding validation
- **Compliance Tests**: pjdfstest integration for POSIX compliance
- **Stress Tests**: Performance benchmarking and concurrent operation testing
- **Device Management**: Utilities for creating and managing test devices

## Prerequisites

### Linux/macOS with FUSE Support

**Linux:**
- Install FUSE: `sudo apt install fuse3 libfuse3-dev` (Ubuntu/Debian)
- Kernel module: `modprobe fuse`
- User permissions: Add user to `fuse` group

**macOS:**
- Install macFUSE from https://osxfuse.github.io/
- Or use built-in FUSE support in newer versions

### Building with FUSE Support

```bash
# Build with FUSE features enabled
cargo build --features fuse

# Or use the Just target
just build-fuse-integration-tests
```

## Running Tests

### All Tests

Run the complete test suite:

```bash
just test-fuse-integration
```

Or run individual test categories:

```bash
# Mount cycle tests
just test-fuse-mount-cycle

# Filesystem operations
just test-fuse-fs-ops

# Control plane operations
just test-fuse-control-plane

# POSIX compliance tests
just test-fuse-pjdfs

# Stress and performance tests
just test-fuse-stress
```

### Manual Test Execution

You can also run the test binaries directly:

```bash
# Build first
just build-fuse-integration-tests

# Run all tests
cargo run -p fuse-integration-tests --features fuse -- all

# Run specific test suite
cargo run -p fuse-integration-tests --features fuse -- mount-cycle
```

### Test Options

```bash
# Skip pjdfstest and stress tests for faster runs
cargo run -p fuse-integration-tests --features fuse -- all --skip-pjdfs --skip-stress

# Enable verbose logging
cargo run -p fuse-integration-tests --features fuse -- all --verbose
```

## Test Structure

### Mount Cycle Tests (`mount-cycle`)
- Creates FUSE mount point
- Mounts AgentFS filesystem
- Performs basic file operations while mounted
- Verifies `.agentfs` directory exists
- Checks filesystem statistics
- Unmounts and cleans up

### Filesystem Operations Tests (`fs-ops`)
- File creation, reading, writing, deletion
- Directory creation, listing, removal
- Permission and attribute testing
- Extended attribute (xattr) support
- Large file operations
- Error condition handling

### Control Plane Tests (`control-plane`)
- Validates `.agentfs/control` file exists
- Tests snapshot creation interface
- Tests branch creation and management
- Tests process-to-branch binding
- SSZ serialization validation

### pjdfstest Compliance (`pjdfstest`)
- Runs pjdfstest suite against mounted AgentFS
- Validates POSIX filesystem compliance
- Reports test results and failures

### Stress Tests (`stress`)
- Concurrent file operations
- Memory pressure testing
- Large directory operations
- Performance benchmarking

## Device Management

### Creating Test Devices

```bash
# List existing devices
just fuse-device-setup list

# Create a 100MB test device
just fuse-device-setup create --name my_test_device --size-mb 100

# Create device for macOS (creates .dmg instead of .img)
just fuse-device-setup create --name mac_test --size-mb 50
```

### Cleanup

```bash
# Remove test device
just fuse-device-setup cleanup --name my_test_device
```

## Stress Testing Utilities

### Performance Benchmarking

```bash
# Run 60-second benchmark on mounted filesystem
just fuse-stress-test benchmark --duration 60 --mount-point /tmp/agentfs_mount

# Run concurrent operations test
just fuse-stress-test concurrent --concurrency 10 --operations 100 --mount-point /tmp/agentfs_mount
```

### Memory Stress Testing

```bash
# Create 1000 files of 1KB each
just fuse-stress-test memory --file-count 1000 --file-size 1024 --mount-point /tmp/agentfs_mount
```

### Directory Stress Testing

```bash
# Create 50 directories with 200 files each
just fuse-stress-test directory --dir-count 50 --files-per-dir 200 --mount-point /tmp/agentfs_mount
```

## CI Integration

The tests are designed to work in CI environments with proper FUSE permissions. For privileged operations, ensure:

- CI runners have FUSE kernel module loaded
- Test user has FUSE permissions
- Sufficient disk space for test devices
- Proper cleanup on test failures

## Troubleshooting

### Common Issues

**FUSE not supported:**
```
FUSE support not compiled in. Use --features fuse to enable FUSE testing.
```
Solution: Build with `--features fuse` flag.

**Permission denied:**
```
fusermount: mount failed: Operation not permitted
```
Solution: Ensure user is in `fuse` group and FUSE is properly configured.

**Mount point busy:**
```
fusermount: failed to unmount: Device or resource busy
```
Solution: Force unmount with `sudo umount -f /mount/point` or wait for processes to finish.

**macOS code signing:**
```
FUSE adapter cannot be loaded due to code signing requirements
```
Solution: Ensure proper code signing for FUSE extensions on macOS.

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run -p fuse-integration-tests --features fuse -- all
```

## Acceptance Criteria

✅ **Full mount cycle integration tests pass**
- Device creation, mounting, operations, unmounting, cleanup

✅ **All filesystem operations work through FUSE interface**
- create, read, write, delete, mkdir, rmdir, readdir

✅ **Control plane operations functional via mounted filesystem**
- Snapshots, branches, binding via `.agentfs/control`

✅ **pjdfstest compliance tests pass**
- POSIX filesystem compliance validation

✅ **Cross-platform mounting validated**
- Linux/macOS compatibility

## Architecture

The test suite is structured as:

- `src/main.rs`: Test runner with CLI interface
- `src/fuse_tests.rs`: Core test implementations
- `src/test_utils.rs`: Common utilities and helpers
- `src/device_setup.rs`: Device management utilities
- `src/stress_test.rs`: Stress testing and benchmarking

Tests use conditional compilation (`#[cfg(feature = "fuse")]`) to gracefully skip FUSE-specific tests when FUSE support is not available.
