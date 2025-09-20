# Report: Creating File-Backed ZFS and Btrfs File Systems with Snapshot Support

This report outlines the process for creating temporary file-backed file systems using ZFS and Btrfs on Linux (assuming OpenZFS for ZFS and standard kernel Btrfs). The focus is on commands requiring `sudo` (root privileges) for initial creation and prerequisites to enable non-root snapshot creation later. It then lists the non-root commands for creating snapshots.

The setup assumes a modern Linux kernel (e.g., 5.15+ for ZFS namespace support, 4.18+ for unprivileged Btrfs loops), necessary modules loaded (`zfs` for ZFS, `btrfs` and `loop` for Btrfs), and packages installed (`zfsutils-linux` and `btrfs-progs`). File-backed means using a regular file as storage (via direct vdev for ZFS, loop device for Btrfs). All examples use a 1GiB sparse file in `/tmp` for temporariness, but adjust paths/sizes as needed.

Snapshots without `sudo` rely on delegation mechanisms: ZFS uses `zfs allow`; Btrfs uses ownership changes, mount options, or user namespaces. User namespaces require a one-time `sudo` sysctl enablement for unprivileged use (persistent via `/etc/sysctl.conf`).

## 1. Commands Requiring Sudo for Creation and Prerequisites

### ZFS File-Backed File System

- **Prerequisites**:
  - Ensure OpenZFS is installed and the `zfs` module is loaded (e.g., `sudo modprobe zfs`).
  - Grant non-root access to `/dev/zfs` for ZFS commands: `sudo chmod 0666 /dev/zfs` (or add user to a group like `disk` via `sudo usermod -aG disk $USER`).
  - For namespace-based isolation (optional but enhances non-root ops): Enable unprivileged user namespaces: `sudo sysctl -w kernel.unprivileged_userns_clone=1` (or set `kernel.unprivileged_userns_clone=1` in `/etc/sysctl.conf` and `sudo sysctl -p`).

- **Creation Commands**:
  - Create backing file: `sudo truncate -s 1G /tmp/zfs_backing.file` (sparse; or `sudo dd if=/dev/zero of=/tmp/zfs_backing.file bs=1M count=1024` for non-sparse).
  - Create pool: `sudo zpool create temp_pool /tmp/zfs_backing.file`.
  - Create dataset: `sudo zfs create temp_pool/my_dataset`.

- **Prerequisites for Non-Sudo Snapshots**:
  - Delegate snapshot permission to user (replace `$USER` with username): `sudo zfs allow $USER snapshot temp_pool/my_dataset`.
  - For namespace isolation (if using namespaces for further ops): Set zoned property: `sudo zfs set zoned=on temp_pool/my_dataset`.
  - Attach to namespace (run as root, where `$$` is PID of namespace process): `sudo zfs zone /proc/$$/ns/user temp_pool/my_dataset`.

### Btrfs File-Backed File System

- **Prerequisites**:
  - Ensure `btrfs-progs` is installed and modules loaded (e.g., `sudo modprobe btrfs` and `sudo modprobe loop`).
  - For unprivileged loop devices and namespaces: Enable unprivileged user namespaces: `sudo sysctl -w kernel.unprivileged_userns_clone=1` (or persistent in `/etc/sysctl.conf` as above). This is key for non-root creation/mounts inside namespaces.

- **Creation Commands** (Using Namespaces for Minimal Sudo; Otherwise Full Sudo Needed):
  - Create backing file (can be non-sudo if user has write access): `truncate -s 1G ~/btrfs_backing.img` (but if path requires sudo, use `sudo`).
  - For full sudo creation (without namespaces):
    - Set up loop: `sudo losetup -fP ~/btrfs_backing.img`.
    - Format: `sudo mkfs.btrfs /dev/loopX` (replace `X` with loop number from `losetup`).
    - Mount: `sudo mount /dev/loopX /mnt/point` (create `/mnt/point` first).
    - Create subvolume: `sudo btrfs subvolume create /mnt/point/my_subvol`.

- **Prerequisites for Non-Sudo Snapshots**:
  - Change ownership for delegation: `sudo chown $USER:$USER /mnt/point/my_subvol` (allows non-root snapshots if user owns the subvolume).
  - Mount with user deletion allowed (for snapshot management): `sudo mount -o user_subvol_rm_allowed /dev/loopX /mnt/point`.
  - (No equivalent to ZFS `zoned=on`; isolation via namespaces.)

Note: For Btrfs, if using namespaces fully, creation can occur inside the namespace without ongoing sudo (after sysctl), but the sysctl is the sudo pre-req.

## 2. Commands for Creating Snapshots (Without Sudo)

### ZFS Snapshots

- After delegation: `zfs snapshot temp_pool/my_dataset@mysnap`.
- In namespace (enter via `unshare -Urm --propagation private bash`, then): `zfs snapshot temp_pool/my_dataset@mysnap` (works as "root" inside).

### Btrfs Snapshots

- After ownership change: `btrfs subvolume snapshot /mnt/point/my_subvol /mnt/point/mysnap`.
- In namespace (enter via `unshare -Urm --propagation private bash`, set up loop/mount inside if needed, then): `btrfs subvolume snapshot /mnt/point/my_subvol /mnt/point/mysnap` (with CAP_SYS_ADMIN inside).

## Cleanup Notes

- ZFS: `zfs destroy temp_pool/my_dataset@mysnap; zfs umount temp_pool/my_dataset; zpool destroy temp_pool; rm /tmp/zfs_backing.file`.
- Btrfs: `btrfs subvolume delete /mnt/point/mysnap; umount /mnt/point; losetup -d /dev/loopX; rm ~/btrfs_backing.img`.

This setup ensures temporary, file-backed systems with non-root snapshot capabilities, minimizing privilege escalation.
