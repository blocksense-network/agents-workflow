## Agent Time-Travel — Product and Technical Specification

### Summary

Agent Time-Travel lets a user review an agent’s coding session and jump back to precise moments in time to intervene by inserting a new chat message. Seeking to a timestamp restores the corresponding filesystem state using filesystem snapshots (FsSnapshots). The feature integrates across CLI, TUI, WebUI, and REST, and builds on the snapshot provider model referenced by other docs (see `docs/fs-snapshots/overview.md`).

### Implementation Phasing

The initial implementation will focus on supporting regular FsSnapshot on copy-on-write (CoW) Linux filesystems (such as ZFS, Btrfs, and NILFS2), using a session recorder based on Claude Code hooks. An end-to-end prototype will be developed for the entire Agent Time-Travel system, including session recording, timeline navigation, and snapshot/seek/branch operations, to validate the core workflow and user experience. Once this prototype is functional, we will incrementally add support for additional recording and snapshotting mechanisms, including user-space overlay filesystems for macOS and Windows, and advanced recording integrations.

### Goals

- Enable scrubbing through an agent session with exact visual terminal playback and consistent filesystem state.
- Allow the user to pause at any moment, inspect the workspace at that time, and create a new SessionBranch with an injected instruction.
- Provide first-class support for ZFS/Btrfs/NILFS2 where available; offer robust fallbacks on non‑CoW Linux, macOS and Windows through file system in user space CoW overlays.
- Expose a consistent API and UX across WebUI, TUI, and CLI.

### Non-Goals

- Full semantic capture of each application’s internal state (e.g., Vim buffers). We replay terminal output and restore filesystem state.
- Reflowing terminal content to arbitrary sizes. Playback uses a fixed terminal grid with recorded resize events.
- A kernel-level journaling subsystem; we rely on filesystem snapshots and pragmatic fallbacks.

### Concepts and Terminology

- **SessionRecording**: A terminal I/O session timeline (e.g., asciinema v2). Visually faithful to what the user saw; does not encode TUI semantics.
- **SessionMoment**: A labeled point in the session recording timeline (auto or manual). Used for navigation.
- **FsSnapshot**: A SessionMoment that has an associated filesystem snapshot reference (snapshot created near‑synchronously with the moment).
- **SessionFrame**: A visual state at a specific timestamp; the player can seek and render the SessionFrame.
- **SessionTimeline**: The ordered set of events (logs, SessionMoments, FsSnapshots, resizes) across a session.
- **SessionBranch**: A new session created from a SessionMoment and its associated FsSnapshot’s filesystem state with an injected chat message.

### Architecture Overview

- **Recorder**: Captures terminal output as an asciinema session recording (preferred) or ttyrec; emits SessionMoments at logical boundaries (e.g., per-command). The initial prototype will use a recorder based on Claude Code hooks.
- **FsSnapshot Manager**: Creates and tracks filesystem snapshots; maintains mapping {moment → snapshotId}.
- **Snapshot Provider Abstraction**: Chooses provider per host (ZFS → Btrfs → NILFS2 → Overlay → copy; FSKit/WinFsp overlays on macOS/Windows). See Provider Matrix below.
- **SessionTimeline Service (REST)**: Lists FsSnapshots/SessionMoments, seeks, and creates SessionBranches; streams session recording events via SSE.
- **Players (WebUI/TUI)**: Embed the session recording; render streaming SessionRecordings in real-time and allows seeking to arbitrary SessionFrames; orchestrate SessionBranch actions.
- **Workspace Manager**: Mounts read-only snapshots for inspection and prepares writable clones/upper layers for SessionBranches.

### SessionRecording and SessionTimeline Model

- **Format**: asciinema v2 JSON with events [time, type, data]; optional input events for richer analysis. Idle compression is configurable.
- **SessionMoments**: Auto moments at shell boundaries (preexec/precmd/DEBUG trap) and runtime milestones (provisioned, tests passed). Manual moments via UI/CLI.
- **Random Access**: Web player supports `startAt` and moments; for power users/offline analysis we may store a parallel ttyrec to enable IPBT usage.
- **Alternate Screen Semantics**: Full-screen TUIs (vim, less, nano) switch to the alternate screen; scrollback of earlier output is not available while paused on the alternate screen. Navigation uses session timeline seek rather than scrollback.

### FsSnapshots and Providers (multi‑OS)

- **Creation Policy**:
  - Default: Create an FsSnapshot at each shell command boundary and at important runtime milestones.
  - Max frequency controls and deduplication to avoid thrashing during rapid events.
  - FsSnapshots include: id, ts, label, provider, snapshotRef, notes.

- **Provider Preference (host‑specific)**:
  - Linux:
    - ZFS: instantaneous snapshots and cheap writable clones (SessionBranch from snapshot via clone).
    - Btrfs: subvolume snapshots (constant-time), cheap writable snapshots for SessionBranching.
    - NILFS2: continuous checkpoints; promote relevant checkpoints to snapshots (mkcp -s); mount past checkpoints read-only (`-o ro,cp=<cno>`).
    - Overlay fallback: lower = base tree, upper/work on fast storage (tmpfs or RAM-backed NILFS2/zram/brd) for ephemeral SessionBranches.
    - Copy fallback: `cp --reflink=auto` when possible; otherwise deep copy (last resort).
  - macOS:
    - User-space overlay: Use FSKit to provide a copy-on-write overlay filesystem for both inspection and SessionBranching, as APFS snapshots are not fast enough for our needs.
  - Windows:
    - User-space overlay: Use WinFsp to provide a copy-on-write overlay filesystem for both inspection and SessionBranching, as VSS snapshots are not fast enough for our needs.

- **SessionBranch Semantics**:
  - Writable clones are native on ZFS/Btrfs. On macOS and Windows, SessionBranching is implemented via user-space overlay filesystems (FSKit/WinFsp) rather than native snapshotting.
  - SessionBranches are isolated workspaces; original session remains immutable.

### User‑Space Filesystem Overlay (macOS and Windows)

- macOS (FSKit): Ship an FSKit filesystem extension implementing a copy‑on‑write overlay over the host filesystem. For each task, mount a per‑task overlay root and `chroot` the agent process into it to preserve original project path layout while writing to the CoW upper. This preserves build and config paths and enables efficient incremental builds.
- Windows (WinFsp): Ship a WinFsp filesystem implementing the same CoW overlay. Mount per‑task at a stable path and map that path to a per‑process drive letter (e.g., `S:`) using per‑process device maps so the agent sees the original project path under `S:`. This provides a chroot‑like illusion on Windows.
- Windows Containers (alternative): Support process‑isolated containers where the container FS view (wcifs overlay) provides the consistent working directory path, analogous to Linux containers.

### Syncing Terminal Time to Filesystem State

- **Runtime Integration**: The agent runner executes a hook that creates the snapshot in between chat messages, agent thinking streams and tool executions.
- **Advanced (future)**: eBPF capture of PTY I/O and/or FS mutations
- **Multi‑OS Sync Fence**: When multi‑OS testing is enabled, each execution cycle performs `fs_snapshot_and_sync` on the leader (create FsSnapshot, then fence Mutagen sessions to followers) before invoking `run-everywhere`. See `docs/multi-os-testing.md`.

### Restarting the agent from a SessionMoment

TODO: Document that this depends on our ability to restore the internal state of the agent software at the precise SessionMoment.
To achieve this, we leverage the support for session checkpoints or session resuming that some agents provide. See the markdown files in "3rd-Party Agents" folder for mode info. We apply complicated agent-specific logic to trim the recorded session to approximately the SessionMoment and then we inject the new prompts of the user. We don't modify the original software session files in the file system - Instead, we mount the trimmed files only in the file system view of the agent in the local sandbox mode or we copy the trimmed session files to the devcontainer or the VM.

TODO: Once this is documented prepare an implementation plan for this feature.

### REST API Extensions

- `GET /api/v1/sessions/{id}/timeline`
  - Returns SessionMoments and FsSnapshots ordered by time.
  - Response:

  ```json
  {
    "sessionId": "...",
    "durationSec": 1234.5,
    "recording": { "format": "cast", "uri": "s3://.../cast.json" },
    "moments": [
      { "id": "m1", "ts": 12.34, "label": "git clone", "kind": "auto" }
    ],
    "fsSnapshots": [
      {
        "id": "s1",
        "ts": 12.4,
        "label": "post-clone",
        "provider": "btrfs",
        "snapshot": { "id": "repo@tt-001", "mount": "/.snapshots/..." }
      }
    ]
  }
  ```

- `POST /api/v1/sessions/{id}/fs-snapshots`
  - Create a manual FsSnapshot near a timestamp; returns snapshot ref.

- `POST /api/v1/sessions/{id}/moments`
  - Create a manual SessionMoment at/near a timestamp.

- `POST /api/v1/sessions/{id}/seek`
  - Parameters: `ts`, or `fsSnapshotId`.
  - Returns a short‑lived read‑only mount (host path and/or container path) for inspection; optionally pauses the session player at `ts`.

- `POST /api/v1/sessions/{id}/session-branch`
  - Parameters: `fromTs` or `fsSnapshotId`, `name`, optional `injectedMessage`.
  - Creates a new session (SessionBranch) with a writable workspace cloned/overlaid from the FsSnapshot.
  - Response includes new `sessionId` and workspace mount info.

- `GET /api/v1/sessions/{id}/fs-snapshots`
  - Lists underlying provider snapshots/checkpoints with metadata (for diagnostics and retention tooling).

- SSE additions on `/sessions/{id}/events`
  - New event types: `timeline.sessionMoment`, `timeline.fsSnapshot.created`, `timeline.sessionBranch.created`.

### CLI Commands

See the `aw agent fs` [commands](./CLI.md).

### WebUI UX

- **Player Panel**: Embed `<asciinema-player>` with SessionMoments and a scrubber. Time cursor shows nearest FsSnapshot and label.
- **Pause & Intervene**: On pause, surface “Inspect snapshot” and “SessionBranch from here”.
- **Inspect Snapshot**: Mounts read‑only view; open a lightweight file browser and offer “Open IDE at this point”.
- **SessionBranch From Here**: Dialog to enter an injected message and name; creates a new session (SessionBranch); link both sessions for side‑by‑side comparison.
- **History View**: SessionTimeline list with filters (auto/manual SessionMoments, FsSnapshots only).

### TUI UX

- **SessionTimeline Bar**: Keyboard scrubbing with SessionMoments (jump prev/next), current time, and FsSnapshot badges.
- **Keys**:
  - Space: pause/resume
  - [ / ]: prev/next SessionMoment; { / }: prev/next FsSnapshot
  - i: Intervene (SessionBranch dialog)
  - s: Seek and open read‑only snapshot in left pane; right pane keeps the player/logs

### Data Model Additions (Session)

- `recording`: `{ format: "cast"|"ttyrec", uri, width, height, hasInput }`
- `sessionTimeline`: `{ durationSec, moments: [...], fsSnapshots: [...] }`
- `fsSnapshots[*]`: `{ id, ts, label, provider, snapshot: { id, mount?, details? } }`
- `sessionBranchOf` (optional): parent session id and fsSnapshot id when branched.

### Security and Privacy

- **Keystrokes**: If input capture is enabled, redact known password prompts (heuristics based on ECHO off and common prompts). Make input capture opt‑in.
- **Access Control**: SessionTimeline/seek/SessionBranch require the same permissions as session access; snapshot mounts use least‑privilege read‑only where applicable.
- **Data Retention**: Separate retention for session recordings vs snapshots; defaults minimize data exposure. Encrypt at rest when stored remotely.

### Performance, Retention, and Limits

- **Snapshot Rate Limits**: Min interval between FsSnapshots; coalesce within a small window (e.g., 250–500 ms) to avoid bursty commands creating many snapshots.
- **Retention**: Policies by count/age/size. Prune unreferenced checkpoints (e.g., NILFS2) and expired provider snapshots.
- **Storage**: Session recording files compressed; offload to object storage. Mounts are short‑lived and garbage‑collected.

### Failure Modes and Recovery

- **Snapshot Creation Fails**: Create a SessionMoment with `fsSnapshot=false` and reason; continue session recording; allow manual retry.
- **Seek Failure**: Report provider error and suggest nearest valid FsSnapshot.
- **Provider Degraded**: Fall back per provider preference, with explicit event logged to the session timeline.

### Provider Semantics Matrix (summary)

- **ZFS**: Snapshots and clones — ideal for FsSnapshots and SessionBranches.
- **Btrfs**: Subvolume snapshots — ideal for FsSnapshots and SessionBranches.
- **NILFS2**: Continuous checkpoints; promote to snapshots; mount via `cp=<cno>`; SessionBranch via overlay.
- **APFS**: Not targeted; APFS snapshots are not fast enough for our needs. Use FSKit overlay instead.
- **VSS**: Not targeted; VSS snapshots are not fast enough for our needs. Use WinFsp overlay instead.
- **Overlay/Copy**: Universal fallbacks when CoW is unavailable.

### Open Issues and Future Work

- eBPF PTY and FS hooks for automatic, runner‑independent capture.
- FSKit backend maturation on macOS for robust overlay SessionBranching without kexts.
- Windows containers integration to provide stronger per‑session isolation when SessionBranching.
