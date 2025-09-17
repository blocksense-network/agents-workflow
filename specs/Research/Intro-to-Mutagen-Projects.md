A **Mutagen project** is a small, declarative setup (a `mutagen.yml` file) that lets you define and manage **multiple sync and/or network-forwarding sessions** together—with lifecycle hooks and custom commands—using `mutagen project …`. It’s a lightweight alternative when Mutagen Compose doesn’t fit your use case. ([Mutagen][1])

How it works (in practice):

- You run it from a directory containing `mutagen.yml` (or pass `-f/--project-file`). Mutagen locks that file so only one instance runs, then creates/controls the sessions declared inside. ([Mutagen][1])
- Common commands: `start`, `list`, `flush` (force a sync cycle and wait), `pause`, `resume`, `reset`, `terminate`, plus `run <name>` for custom commands you define. All operate on the project’s sessions at once. ([Mutagen][1])
- Sessions in a project are still “normal” sessions—you can manage any of them individually with `mutagen sync …`/`mutagen forward …` if you want. ([Mutagen][1])

What you can put in `mutagen.yml`:

- **Named sessions** under `sync:` and/or `forward:` with **defaults** and per-endpoint overrides (`configurationAlpha/Beta` for sync; `configurationSource/Destination` for forwarding). ([Mutagen][1])
- **Hooks** (`beforeCreate`, `afterCreate`, `beforeTerminate`, `afterTerminate`) to run shell commands around project lifecycle steps (e.g., spin containers up/down). ([Mutagen][1])
- **Custom commands** in a `commands:` block, invokable via `mutagen project run <name>`. ([Mutagen][1])
- Optional **`flushOnCreate`** to force an initial full sync right after session creation. ([Mutagen][1])

Tiny example:

```yaml
# mutagen.yml
sync:
  defaults:
    flushOnCreate: true
  app:
    alpha: ./app
    beta: ssh://user@host/~/app
    configurationAlpha:
      ignore:
        vcs: true
forward:
  db:
    source: tcp:127.0.0.1:5432
    destination: ssh://user@host/tcp:127.0.0.1:5432
commands:
  web-shell: docker compose exec web bash
```

```bash
mutagen project start
mutagen project list
mutagen project flush
mutagen project terminate
```

([Mutagen][1])

If you want, tell me your stack (SSH vs Docker, one-way vs two-way) and I’ll tailor a starter `mutagen.yml`.

[1]: https://mutagen.io/documentation/orchestration/projects/ "Projects | Mutagen
"
