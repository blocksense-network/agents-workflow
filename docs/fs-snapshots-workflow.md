# Development Plan: Isolated Agent Runtimes for Agents-Workflow

## Main Objectives

* **Isolated Per-Agent Workspaces:** Extend the `agent-task` CLI command to spawn new agent runtimes (e.g. Codex CLI, Claude Code, Copilot, Gemini CLI, Goose, etc.) in *isolated* environments. Each agent runs against an independent, snapshot-based copy of the user's current Git working tree, ensuring deterministic execution and no cross-contamination between agents.
* **Copy-on-Write Efficiency:** Use copy-on-write (CoW) filesystem snapshot or container layering techniques so that creating an agent’s workspace is fast and storage-efficient. Snapshots/clones on CoW filesystems like ZFS and Btrfs are near-instant and consume no extra space initially. This allows us to cheaply provision multiple workspace copies from the same baseline.
* **Cross-Platform Support:** Design the solution to work on **Linux, macOS, and Windows** development environments. On Linux, leverage native filesystem capabilities (ZFS, Btrfs, or OverlayFS) for local isolation. On macOS/Windows (which lack such native CoW FS for our purposes), seamlessly orchestrate a persistent Linux VM or container where snapshots and Docker can run. The CLI should hide these details, giving a uniform experience.
* **Local and Remote Execution:** Support launching agent runtimes **locally** (on the same host) or on a **remote/virtual host** accessed via SSH. For example, a user on macOS might use a lightweight Linux VM (like Colima or a preconfigured cloud instance) as the execution host. The library will handle file synchronization and remote commands so that agents always run against the latest working copy state.

## Key Requirements and Environment Strategy

To meet the objectives, the implementation must adapt to different OS and filesystem environments. Below is the strategy for each scenario:

* **Linux (ZFS filesystem):** If the user's repository resides on a ZFS volume, use ZFS native snapshots and clones. Creating a snapshot and clone is extremely fast and initially consumes no additional disk space. The plan is to run `zfs snapshot <dataset>@<tag>` followed by `zfs clone <dataset>@<tag> <new_dataset>` to produce a writable clone of the current working tree. Once the clone is created, it will be mounted at the same path as the original repository within an isolated environment using Docker containers. This path preservation is critical to maintain the validity of any existing build artifacts, compiled binaries, or configuration files that contain absolute path references and would be invalidated if the working directory path changed. The agent process will run within this isolated namespace where it sees the cloned filesystem mounted at the familiar path, ensuring increnental builds are as efficient as possible. After execution, the clone can be destroyed to free space. This approach leverages both ZFS's CoW semantics to avoid copying data and Linux namespace isolation to maintain path consistency while ensuring complete isolation between agent runs. (We may require that the repository is on its own ZFS dataset, or document that snapshots will encompass the whole dataset containing the repo.)

* **Linux (Btrfs filesystem):** If the repo is on Btrfs, use Btrfs subvolume snapshots. Btrfs supports quick CoW snapshots of subvolumes. We will ensure the repository directory is a Btrfs subvolume (creating one if necessary). Then for each agent run, execute `btrfs subvolume snapshot <repo> <snapshot_dir>` to create a writable snapshot copy. This snapshot is a CoW clone sharing data with the original, so initial copy cost is minimal. The agent will run with its working dir set to the snapshot directory. Cleanup involves deleting the subvolume snapshot. Btrfs snapshots, like ZFS, are near-instant and efficient due to CoW.

* **Linux (Non-CoW filesystem, e.g. ext4/XFS):** If ZFS/Btrfs are not available, fall back to **OverlayFS or equivalent** to avoid full copies. Using OverlayFS, we can mount an overlay union with the original repo as the read-only lower layer and an empty tmpfs or directory as the upper layer (for writes), resulting in a merged mount that looks like a copy of the repo. The agent will operate in this merged mount, so any file modifications occur in the upper layer, leaving the real working tree untouched. Since any change to the original working copy will be visble to the agents under this setup, we temporary make the working copy read-only while remaining agents are running. **Note:** OverlayFS mounting requires elevated privileges (CAP\_SYS\_ADMIN); the CLI will attempt to use `sudo` or instruct the user to run with proper rights if needed. If OverlayFS is not feasible, the last resort is a full recursive copy of the working tree (preferably using hard links or reflinks to save time/storage if supported). For example, on newer filesystems that support reflink (copy-on-write file copies), we can use `cp --reflink=auto` to quickly clone the tree at file block level. These fallbacks ensure compatibility even on vanilla ext4 systems, albeit with some performance cost.

* **macOS and Windows (via Persistent VM + Mutagen):** macOS and Windows hosts lack native Linux CoW filesystems and container support is indirect (e.g. Docker Desktop). The strategy here is to utilize a long-lived lightweight **Linux VM** (such as a Colima or Lima VM on macOS, or a Hyper-V/WSL2 VM on Windows) to serve as the agent execution environment. We will synchronize the user’s working copy into this VM using **Mutagen** or a similar bidirectional sync tool. Mutagen will keep a *real copy* of all files inside the VM, avoiding slow network filesystem access and yielding near-native IO performance. Within the VM, we then apply the same snapshot techniques as above (ZFS/Btrfs/Overlay as available). For instance, the VM disk could be formatted with Btrfs to leverage snapshotting. The CLI library will manage the VM and sync: e.g., on first use it can ensure the VM is running (possibly by invoking Colima or a custom lightweight Docker/LinuxKit VM), set up a Mutagen sync session between the host repo and the VM’s filesystem, and then trigger agent execution inside the VM (via SSH or `docker exec`). On Windows, if WSL2 is available, it could be used similarly (with P9 file shares or rsync, though Mutagen provides more control). The key requirement is that from the perspective of our tool, the remote VM acts like a Linux host with the latest code, accessible via SSH or Docker API. All snapshot and container operations happen in that Linux environment. The user experience on Mac/Windows will be nearly identical to Linux, aside from an initial setup of the sync mechanism. (We will document any one-time setup, like installing Mutagen or Colima, for developers on these platforms.). This setup will require the project to define a devcontainer image, offering a ZFS/btrfs workspace and all AI agent software pre-installed.

* **Remote Hosts via SSH:** In addition to local and VM scenarios, the design will allow **remote execution on any SSH-accessible Linux host**. A user might configure an IP/hostname (and credentials or keys) for a server or cloud VM that should run agent tasks. The library will then connect to that host and perform the same steps: create a snapshot of the code and run the agent. To get the code there, two approaches will be supported: (1) **On-demand sync** – e.g. use `rsync` or `scp` to copy the current working tree to the remote host into a designated path before each run (including uncommitted changes); or (2) **Persistent sync** – use Mutagen or a similar daemon to continuously sync the repository to the remote host (much like the local VM case). The second approach is more efficient for repeated runs, so we will integrate Mutagen support for any remote, not just local VMs (Mutagen can sync to an SSH target as well). In either case, the remote host should have the necessary tools (ZFS/Btrfs or overlay/Docker) installed. We will strive to make the remote orchestration as automated as possible (for example, the first run could check for required tools on the remote and print instructions if missing).

In summary, the solution will adapt to use the **best available isolation method per platform**: CoW snapshots on CoW filesystems, overlay or copy on others, and a VM+sync approach on non-Linux hosts. This maximizes performance and compatibility across environments.

## Implementation Plan

The development will proceed in phases, starting with lower-level primitives and building up systematically with comprehensive testing at each layer.

**Phase 1: Core Filesystem Abstraction Layer**
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

**Phase 2: Mock Agent Integration Testing**
*Implementation:* Create a realistic mock agent that simulates actual AI agent behavior without requiring real AI services:

* **Mock Agent Behavior:** The mock agent will perform realistic file operations (reading source files, creating output files, modifying existing files), include configurable work duration with sleep calls, and generate logs of its activities. It will simulate the patterns of real agents like Codex or Goose.
* **Docker Test Environment:** Build a minimal Alpine Linux Docker image containing the mock agent and Ruby runtime. This image will serve as a controlled environment for testing isolation and concurrency.
* **Parallel Execution Tests:** Write integration tests that launch multiple mock agents simultaneously in separate isolated workspaces. Verify that agents cannot see each other's changes and that all file modifications remain properly isolated.
* **Performance Testing:** Measure snapshot creation time, workspace cleanup time, and resource usage under concurrent load to ensure the system scales appropriately.

*Integration Testing:* These tests will use real filesystem operations but controlled environments. Test on multiple filesystems (ext4, btrfs) in CI. Verify isolation guarantees and measure performance characteristics.

**Phase 3: Credential Management System**
*Implementation:* Based on research of actual AI agent tools, implement a comprehensive credential management system:

* **Credential Pattern Detection:** Support for different agent types with their specific credential requirements:
  - **Codex:** `OPENAI_API_KEY` environment variable, `~/.codex/auth.json` and config files
  - **GitHub Copilot:** `GITHUB_TOKEN` environment variable, `~/.config/gh/hosts.yml` for GitHub CLI auth
  - **Goose:** `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` environment variables
  - **Gemini:** `GEMINI_API_KEY` environment variable
  - **Claude:** `ANTHROPIC_API_KEY` environment variable
* **Secure Mounting:** For containerized execution, mount credential files read-only and propagate environment variables securely. For VM execution, sync credential files safely while preserving file permissions (0600 for auth files).
* **Colima/VM Support:** Research confirms that Colima and similar VM solutions support both environment variable propagation and bind mounts for credential files. The system will use Docker's `--env-file` and `--mount` options for secure credential injection.

*Testing:* Test credential mounting with dummy credentials in controlled environments. Verify that credentials are accessible to agents. Test both file-based and environment variable credentials.

**Phase 4: SSH/Remote Execution Framework**
*Implementation:* Build remote execution capabilities using Docker containers with SSH servers for realistic testing:

* **SSH Test Infrastructure:** Extend the test-purpose docker image with a configured SSH server. Use this for testing remote execution without requiring actual remote machines.
* **File Synchronization:** Implement both one-shot sync (using `rsync`) and persistent sync (using Mutagen) approaches. Handle edge cases like network interruptions and large file transfers.
* **Remote Filesystem Detection:** Extend the filesystem detection logic to work over SSH connections. Cache detection results to avoid repeated SSH calls.
* **Error Handling:** Comprehensive error handling for network failures, authentication issues, insufficient remote permissions, and missing remote tools.

*Integration Testing:* Use Docker containers as SSH targets to test the complete remote workflow. Launch a container, establish SSH connection, sync files, create snapshots remotely, execute agents, and retrieve results. Test concurrent remote executions and verify isolation.

**Phase 5: Full Integration and CI/CD Pipeline**
*Implementation:* Integrate all components and establish comprehensive CI testing:

* **CI Matrix Enhancement:** Add test jobs for different OS/filesystem combinations:
  - Ubuntu with btrfs support
  - Ubuntu with overlay-only (simulating basic ext4 systems)
  - macOS with Docker/Colima simulation
  - Windows with WSL2/Docker simulation
* **End-to-End Testing:** Test complete workflows from `agent-task` CLI invocation through workspace creation, agent execution, and cleanup.
* **Performance Monitoring:** Add benchmarks for snapshot creation/destruction, file sync performance, and concurrent agent execution. Set performance regression thresholds.
* **Documentation and Examples:** Complete user documentation with setup instructions for each platform, credential configuration guides, and troubleshooting sections.

*Integration Testing:* The CI pipeline will run the full test suite across the matrix of supported platforms and configurations. This includes both unit tests of individual components and integration tests of complete workflows. All tests will run against real filesystem operations and network conditions to catch issues that mocks might miss.

## Conclusion

By following this development plan, we will create a **robust, cross-platform workspace isolation engine** for `agents-workflow`. The `agent-task` command will gain the ability to spawn deterministic, isolated agent processes that each see a consistent view of the project’s code at runtime. This will greatly improve the reliability of running multiple AI agents in parallel or in sequence, ensuring one agent’s changes or side-effects do not pollute another’s environment. The plan emphasizes use of proven filesystem snapshot techniques and modern devops tools (like Mutagen for sync) to balance performance with broad compatibility. Each step of implementation is backed by integration tests in a variety of environments, so contributors can iterate with confidence. Once completed, this feature will allow developers to leverage advanced AI agents (Codex, Claude, Copilot, etc.) concurrently on their codebase with minimal friction, which is a key stepping stone toward more complex and deterministic AI-driven workflows. The project maintainers and peer reviewers (human or AI) can use this document as a blueprint to implement and validate the feature in the repository.
