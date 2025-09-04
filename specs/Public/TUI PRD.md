## TUI — Product Requirements and UI Specification

### Summary

The TUI provides a terminal-first dashboard for launching and monitoring agent
tasks, integrated with terminal multiplexers (tmux, zellij, screen). It
auto-attaches to the active multiplexer session and assumes all active tasks
are already visible as multiplexer windows.

Backends:

- REST: Connect to a remote REST service and mirror the WebUI experience for
  task creation, with windows created locally (or remotely via SSH) for launched
  tasks.
- Local: Operate in the current directory/repo using the SQLite state database
  for discovery and status.

### Auto-Attach and Window Model

- On start, `aw tui` auto-attaches to the current multiplexer session (creating
  one if needed). Existing task windows are left intact.
- Launching a new task immediately creates a new window in the multiplexer:
  - Split panes: right = agent activity, left = terminal or configured editor
    in the workspace.
  - Devcontainer runs: panes are inside the container context.

### Simplified Dashboard Layout

The main TUI dashboard focuses on quick launch:

- Top area: selectors for Project, Branch, Agent (fixed-height lists with
  filter input and arrow-key navigation).
- Bottom area: task description editor (multiline input) with resizable height.
- A single Start action (hotkey + button) to launch the task, which creates a
  new multiplexer window immediately.

### Selectors and Filtering

- Fixed-height list widgets for Project, Branch, and Agent.
- Each list includes:
  - A text input to filter entries (prefix/substring), updated as you type.
  - Arrow keys/PageUp/PageDown/Home/End navigation within the fixed-height
    viewport.
  - Enter selects the highlighted entry.
- Branch source:
  - Local mode: standard git commands against the local repo (e.g., `git
for-each-ref`), cached with debounce.
  - REST mode: server capability endpoint backed by git protocol
    (`ls-remote`/refs) against admin-configured repository URLs.
- Agent list: from local config or REST `/api/v1/agents`.
- Project list:
  - REST mode: admin-configured workspace/projects and repositories.
  - Local mode: repositories previously used in agents-workflow (WebUI or CLI)
    with add/remove gestures available elsewhere in CLI.

### Commands and Hotkeys (illustrative)

- Tab/Shift+Tab: cycle between Project, Branch, Agent, Description.
- Ctrl+F: focus filter input of the active list.
- Ctrl+J/Ctrl+K or Arrow keys: navigate list items.
- Ctrl+Up/Down: resize the description editor.
- Enter in Description with modifier (e.g., Ctrl+Enter): Start task.
- F1: Help overlay with keymap.

### Error Handling and Status

- Inline validation messages under selectors (e.g., branch not found, agent
  unsupported).
- Status bar shows backend (`local`/`rest`), selected multiplexer, and last
  operation result.

### Remote Sessions

- If the REST service indicates the task will run on another machine, the TUI
  uses provided SSH details to create/attach a remote multiplexer window.

### Persistence

- Last selections (project, agent, branch) are remembered per repo/user scope
  via the configuration layer.

### Accessibility

- High-contrast theme option; full keyboard operation; predictable focus order.
