**Milestone 1: Core Filesystem Abstraction Layer**
*Implementation:* Build fundamental filesystem operation primitives as the foundation. Create a `SnapshotProvider` abstraction with concrete implementations for each supported method:

* **Detection Logic:** Implement filesystem type detection by examining `/proc/mounts`, checking for ZFS/Btrfs tools availability, and falling back gracefully through the hierarchy (ZFS → Btrfs → OverlayFS → Copy).
* **ZFS Provider:** Implement `zfs snapshot` and `zfs clone` operations with proper dataset path resolution and cleanup. Handle permissions and error cases (e.g., insufficient privileges, quota limits).
* **Btrfs Provider:** Implement `btrfs subvolume snapshot` with automatic subvolume creation if needed. Handle the case where the repository is not yet a subvolume.
* **OverlayFS Provider:** Create overlay mounts with proper `lowerdir`, `upperdir`, and `workdir` structure. Handle sudo requirements and privilege escalation gracefully.
* **Copy Provider:** Implement fast copying using reflinks where available (`cp --reflink=auto`) or falling back to hard links and finally regular copying.

*Testing Strategy:* Create real filesystems within files using loop devices for comprehensive testing. This approach provides authentic filesystem behavior without requiring pre-configured test systems:

* **ZFS Testing:** Create ZFS pools using loop devices with `zpool create test_pool /path/to/file.img`. Create datasets, test snapshot/clone operations, verify CoW behavior, and test error conditions like insufficient space or permissions.
* **Btrfs Testing:** Create Btrfs filesystems in files with `mkfs.btrfs /path/to/file.img`, mount via loop devices, create subvolumes, and test snapshot operations. Verify that non-subvolume directories are automatically converted when needed.
* **OverlayFS Testing:** Create multiple loop-mounted filesystems to test overlay mounting with different combinations of lower/upper/work directories. Test with both writable and read-only lower layers.
* **Copy Testing:** Test on various filesystem types (ext4, xfs, etc.) created in loop devices to verify reflink support detection and fallback behavior.
* **Error Condition Testing:** Test quota limits, permission errors, disk full scenarios, and concurrent access patterns using the loop device filesystems.
* **Performance Testing:** Measure snapshot creation/deletion times and space usage with real filesystems to establish baseline performance characteristics.

*CI Integration:* The test suite will create temporary filesystem images during test runs, eliminating the need for pre-configured CI environments with specific filesystems. Tests can run on any Linux system with loop device support (standard in most CI environments).
