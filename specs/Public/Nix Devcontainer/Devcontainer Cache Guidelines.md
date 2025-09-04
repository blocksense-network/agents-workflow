## Host↔Guest Cache Sharing — Guidelines and Goals

### Goals

- Reduce build/install times across container rebuilds and branches.
- Avoid secret/material leakage via shared caches.
- Keep cache consistency and correctness (no silent corruption, deterministic
  rebuilds when needed).

### General Principles

- Prefer Docker volumes for persistence; use bind mounts for read‑only
  configs when necessary.
- Set clear ownership/permissions by aligning container user UID/GID with host
  where possible.
- Use content‑addressed caches (where supported) and verify cache keys
  include OS/arch/toolchain.
- Provide a kill‑switch env/flag to disable cache sharing when debugging
  purity issues.

### Package Manager Patterns

- Node (npm/pnpm/yarn):
  - Volumes: `~/.npm`, `~/.cache/pnpm`, `~/.yarn`, project `.pnpm-store`.
  - Ensure lockfiles are honored; enable `corepack` where applicable.
  - Tests: cold vs warm install; lockfile change invalidation.

- Python (pip/pipx/poetry):
  - Volumes: `~/.cache/pip`, `~/.cache/pipx`, virtualenv directories under
    project `.venv`.
  - Pin interpreter from Nix; ensure ABI compatibility.
  - Tests: wheel reuse, virtualenv isolation, hash mismatch behavior.

- Rust (cargo):
  - Volumes: `~/.cargo`, `~/.cargo/registry`, `~/.cargo/git`.
  - Consider `sccache` for compiler outputs; volume for `~/.cache/sccache`.
  - Tests: build twice, ensure network disabled on second run.

- Go:
  - Volumes: `~/go/pkg/mod`, `~/.cache/go-build`.
  - Module proxy configuration via env; verify GOPATH hygiene.
  - Tests: module reuse and build cache hit rate across rebuilds.

- Java (Maven/Gradle):
  - Volumes: `~/.m2`, `~/.gradle`.
  - Ensure Gradle Enterprise/daemon settings are sane for containers.
  - Tests: offline build viability after warm run.

- System Caches (ccache/sccache):
  - Volumes: `~/.ccache`, `~/.cache/sccache`.
  - Configure size limits and compression; record hit/miss metrics.

### Security Considerations

- Treat caches as untrusted inputs when switching branches or contributors;
  prefer read‑write volumes scoped per project or per user.
- Do not mount credential stores (e.g., `~/.aws`) as caches; keep those
  separate and read‑only.
- Avoid mounting host package manager sockets/daemons unless required.

### Observability

- Expose cache directories and sizes via `aw doctor`.
- Emit basic metrics (hits/misses) where tools provide them (cargo, sccache).
