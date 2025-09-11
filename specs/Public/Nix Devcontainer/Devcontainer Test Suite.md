## Nix Devcontainer — Cache Sharing Test Suite

### Purpose

Validate cache persistence and correctness for supported package managers and toolchains across container rebuilds and typical workflows.

### Test Matrix

- Platforms: Linux, macOS (Docker Desktop), Windows (WSL2)
- Images: Nix Base, Agents Base, representative project images
- Package Managers: npm/pnpm/yarn, pip/pipx/poetry, cargo, go, maven/gradle, ccache/sccache

### Scenarios

1. Cold → Warm Install
   - Cold: clear volumes; install dependencies; record time/size.
   - Warm: rebuild container; reinstall; expect significant speedup and no network fetches where offline cache applies.

2. Lockfile Change Invalidation
   - Modify lockfile (add dependency); ensure caches do not cause stale results; verify correct new graph.

3. Toolchain Change
   - Change Nix devshell tool versions; validate caches with different ABI get invalidated as needed.

4. Concurrent Builds
   - Run two builds concurrently in separate containers sharing volumes; ensure no corruption.

5. Security Hygiene
   - Verify no secrets present in cache volumes; permissions are scoped to devcontainer user.

6. Offline Build
   - Disable network; confirm warm caches enable successful builds where expected (cargo/go/java).

### Measurements

- Wall‑clock durations (cold vs warm)
- Network requests count/bytes (where observable)
- Cache sizes before/after
- Hit/miss metrics (cargo, sccache)

### Automation

- Provide `aw health --caches` to print configured mounts and sizes.
- CI jobs per package manager with synthetic sample projects.

TODO: Prepare an implementation plan for the Devcontainer functionality described in this directory
