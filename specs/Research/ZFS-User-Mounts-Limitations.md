User mounts in Linux refer to the ability for non-root (ordinary) users to mount filesystems without requiring superuser privileges, typically achieved through specific configurations or mechanisms that relax the default root-only restrictions on the `mount` system call. This is useful for scenarios like attaching removable media (e.g., USB drives, CDs) or loop-mounted disk images, where security risks are mitigated by limiting what the user can do. Here's how they work:

### Mechanisms for User Mounts

1. **Via /etc/fstab Configuration (Most Common for Block Devices)**:
   - The system administrator (root) pre-configures entries in `/etc/fstab` (the filesystem table) with the `user` option. This explicitly allows any non-root user to mount that specific filesystem.
   - Example `/etc/fstab` entry: `/dev/sdb1 /mnt/usb ext4 user,noauto 0 0`
     - Here, a user can run `mount /mnt/usb` (or `mount /dev/sdb1`) without sudo.
   - Restrictions:
     - Implies security options like `noexec` (no executables), `nosuid` (ignore setuid bits), and `nodev` (no device nodes) unless overridden.
     - Only the user who mounted it can unmount it (via `umount`), unless the `users` option is used instead (allowing any user to unmount).
     - The `owner` option limits mounting to the device file's owner, while `group` limits it to the device's group members.
   - This works for many filesystems (e.g., FAT, NTFS, ext4) but requires root to set up initially and is limited to predefined devices/mountpoints.

2. **Via FUSE (Filesystem in Userspace)**:
   - FUSE allows users to mount custom filesystems implemented in user space (e.g., SSHFS, NTFS-3G) using the `fusermount` utility.
   - No root needed if the FUSE module is loaded and the user has access to `/dev/fuse`.
   - Example: `sshfs user@host:/remote /mnt/remote`
   - The kernel forwards filesystem operations to a user-space daemon, keeping the mount isolated and safe. Since util-linux 2.35, the `mount` command can drop suid privileges and run as non-root for FUSE-like cases.

3. **Via User Namespaces (For Unprivileged Mounts)**:
   - Modern kernels (3.8+) support user namespaces, where a non-root user can create an isolated namespace and act as "root" inside it (with capabilities like CAP_SYS_ADMIN limited to that namespace).
   - Enabled via sysctl (`kernel.unprivileged_userns_clone=1`, set by root initially).
   - Tools like `unshare` create the namespace: `unshare -U -m -r bash`, then inside: `mount /dev/loop0 /mnt`.
   - Mounts are visible only within the namespace (process and children), ideal for containers or isolation. Supports loop devices (since kernel 4.18 for unprivileged) and id-mapped mounts (kernel 5.12+).

4. **Loop Mounts for Files**:
   - Users can mount disk images (e.g., ISO files) via loop devices if accessible: `mount -o loop image.iso /mnt`.
   - Kernel auto-allocates loop devices (since 2.6.25), but may need /etc/fstab setup or namespaces for full unprivileged use.

In all cases, the kernel enforces security: Non-root mounts can't affect the global namespace broadly, and options like `nosuid` prevent privilege escalation. Desktop environments (e.g., GNOME via gvfs) often automate this for USB drives.

### What Prevents User Mounts with ZFS

ZFS datasets cannot be mounted by non-root users in the global namespace on Linux due to kernel and implementation restrictions, even if you try /etc/fstab or delegation. Here's why:

1. **Kernel Restrictions on Mount Syscall**:
   - The `mount(2)` syscall requires CAP_SYS_ADMIN in the global namespace, which is only available to root. ZFS uses kernel-level mounts (not FUSE), so it inherits this limitation—unlike user-space filesystems.

2. **Delegation Doesn't Override on Linux**:
   - ZFS's `zfs allow ... mount` delegates the 'mount' permission, but this is not honored on Linux for non-root users. It works on illumos/FreeBSD but fails here because `zfs mount` still needs to modify the global mount table.

3. **No /etc/fstab 'user' Support**:
   - ZFS datasets aren't traditional block devices; they're managed via the ZFS module and /dev/zfs ioctl. You can't easily add them to /etc/fstab with 'user'—attempts fail due to the above syscall restrictions.

4. **Other Factors**:
   - ZFS requires rw access to /dev/zfs, which can be granted to users, but mounting still needs root.
   - No FUSE equivalent for ZFS exists, as it's a full kernel filesystem.

Workarounds include user namespaces (as above: set `zoned=on`, attach via `zfs zone`, then mount inside namespace) or tools like LXD/Incus for delegated management. Sudo wrappers or setuid helpers are possible but insecure. This is a longstanding Linux-specific issue with no fix as of 2025.
