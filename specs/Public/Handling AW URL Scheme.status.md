# Handling `agents-workflow://` URL Scheme — Status and Plan

Spec: See “Handling AW URL Scheme.md” for the required behavior. This file tracks implementation tasks, milestones, and a precise test plan.

## Goal

Implement a secure, cross‑platform URL scheme handler that opens tasks and (optionally) creates tasks with an explicit confirmation dialog. Support TUI reuse when a task is already followed in an existing terminal window.

## Milestones and Tasks

0. Select test automation framework(s)

- Evaluate candidates for reliably handling browser external‑protocol prompts and the handler’s native confirmation dialog across Windows/macOS/Linux:
  - Playwright + OS accessibility automation (macOS AppleScript/JXA; Windows UIAutomation; Linux AT‑SPI/dogtail).
  - OpenQA (openQA) with VM orchestration and needle‑based UI assertions for full desktop flows.
  - Optional: WebDriver BiDi + OS automation glue.
- Criteria: flake rate in CI, cross‑OS coverage, maintenance overhead, artifact quality (video/screenshot/traces), ability to drive native dialogs, and total runtime.
- Deliverable: a short report with PoC repos, CI configs, and a recommended primary+fallback stack.

1. Core handler skeleton (parse → normalize → ensure WebUI → open)

- Rust bin `aw-url-handler` (Windows/macOS/Linux) with structured logs.
- URL parsing/validation; reject unknown hosts/components and secrets in query.
- Config resolution (webui base, rest base) + health probes.

2. WebUI bootstrap + REST probing

- Start WebUI locally when needed, wait for `/_aw/healthz`.
- Probe optional REST at `/api/v1/readyz`.
- Open `${webuiBase}/tasks/<id>` reliably across OSes.

3. TUI integration and reuse

- Read TUI control index `${STATE_DIR}/tui-sessions.json` and/or query `${XDG_RUNTIME_DIR}/agents-workflow/tui.sock`.
- For existing sessions, reuse window via tmux/WezTerm/Kitty commands; platform focus helpers wired.
- Fallback: start `aw tui --follow <id>`.

4. Create flow with confirmation (security)

- Native confirmation dialog with required fields (title, source, summary, where/how it will run, access scope, buttons, trust checkbox).
- Trust store with per‑source ephemeral rules; global policy from config.
- Safe rendering (escape all user content); cancellation cancels creation.

5. Packaging and registration

- Windows registry (HKCU) and MSIX URI activation.
- macOS `CFBundleURLTypes` and Electron deep‑link handling.
- Linux `.desktop` with `x-scheme-handler/agents-workflow`.

6. Telemetry, docs, and hardening

- Rotating logs; error surfacing; minimal diagnostics.
- Docs update and examples in spec and CLI.md.

Success criteria

- Cold start to task page < 3s p95 on macOS/Windows/Linux default dev boxes.
- TUI reuse succeeds when index/socket indicates an existing session (see tests).
- Create flow always presents the handler confirmation UI (unless test override explicitly enabled) and behaves per spec.

## Test Plan (precise)

Test matrix

- OS: Windows (Win10/11), macOS (13+), Linux (GNOME/KDE; X11 and Wayland when feasible).
- Browsers: Chrome/Chromium, Edge (Win), Firefox, Safari (macOS).

Test harness components

- Browser automation: Playwright.
- Desktop automation for external‑protocol prompts (browser‑controlled dialogs) and handler confirmation:
  - Windows: PowerShell + UIAutomation (or Python `uiautomation`) to locate and click buttons by name; fallback AutoHotkey for stubborn cases.
  - macOS: AppleScript/JXA using Accessibility API to detect Chrome/Firefox/Safari confirmation sheets; standard `osascript` runner in CI.
  - Linux: AT‑SPI via `dogtail` (preferred) to find “External Protocol Request” dialogs; X11 fallback via `xdotool` (Wayland: `ydotool`/portal hints).
- Handler observability: temporary test log sink `${TMPDIR}/aw-url-handler-test.log` with structured events to assert flows without scraping UI text.

Fixtures

- Stub WebUI server exposing `/_aw/healthz` and routes `/tasks/:id` (no auth), launched on random free port.
- Optional stub REST at `/api/v1/readyz`, `/api/v1/sessions/:id`, `/api/v1/tasks`.
- TUI control stub: script to create `tui-sessions.json` with entries mapping task id to mux/window identifiers; optional UNIX socket echo server that replies to `FIND <taskId>`.

Scenarios

1. Open task — happy path

- Start stub WebUI on port P.
- Invoke handler with `agents-workflow://task/ABC`.
- Assert: browser navigates to `http://127.0.0.1:P/tasks/ABC` (Playwright URL check) and handler log shows `action=open`.

2. Open task — start WebUI when absent

- Ensure no WebUI running.
- Invoke handler.
- Assert: handler starts WebUI (detect child process or health probe), then opens the route; p95 time < 3s on CI hardware.

3. TUI reuse present

- Create `tui-sessions.json` with `ABC → { mux: tmux, windowId: X }` and ensure `tmux` is available.
- Invoke `agents-workflow://task/ABC?tui=1`.
- Assert: handler executes `tmux select-window -t X`; no new `aw tui` process is spawned; optional platform focus succeeds.

4. TUI fallback when absent

- Ensure no `tui-sessions.json` entry.
- Invoke `...tui=1`.
- Assert: handler launches `aw tui --follow ABC` without blocking; WebUI still opens.

5. Create flow — confirmation required

- From each browser, open a page with `<a href="agents-workflow://create?spec=%7B...%7D">`.
- Trigger click via Playwright; handle the browser’s external‑protocol prompt via OS automation.
- Assert: handler’s own native confirmation dialog appears with:
  - prompt summary snippet
  - execution venue (local/remote)
  - agent type and snapshot mode
  - “Create Task” and “Cancel” buttons, confirmation checkbox
- Click “Cancel”: assert no task created; WebUI shows create page with prefilled payload via local hand‑off id.
- Click “Create Task”: assert REST/CLI invoked; new `taskId` logged; browser navigates to task page.

6. Trust policy

- In confirmation dialog, tick “Trust this site for 1 hour”.
- Repeat click within the hour: assert dialog suppressed, creation proceeds; after expiry, dialog returns.

7. Negative URLs

- Malformed scheme, unsupported path, oversized payloads.
- Assert rejection with safe error UI and no external side effects.

8. Browser coverage — external protocol prompts

- Validate that the test harness detects and clicks the correct control for:
  - Chrome/Edge: “Open <app>” button in the external protocol dialog.
  - Firefox: “Open link” confirmation dialog.
  - Safari: sheet asking to open the application.

9. Accessibility labels presence

- Ensure confirmation dialog controls have stable accessibility names/roles to make tests resilient.

10. Headless test mode (non‑UI)

- With `AW_URL_HANDLER_TEST_AUTOCONFIRM=true`, bypass the confirmation UI for unit/integration tests.
- E2E tests MUST run without this flag to exercise the real UI.

CI wiring

- GitHub Actions matrix: `windows-latest`, `macos-latest`, `ubuntu-latest`.
- Install Playwright browsers; enable accessibility permissions on macOS runner for `osascript`.
- Preload AutoHotkey on Windows for fallback flows.
- Publish artifacts: handler logs, Playwright traces, screenshots of dialogs.

Exit criteria

- All scenarios above pass on the CI matrix.
- Manual spot‑checks confirm reasonable UX and focus behavior.

### GUI Scenarios (when AW GUI is installed)

- Delegation: The protocol handler delegates to the GUI main process via IPC when the GUI is running.
- Window reuse: A new browser window is not spawned if the GUI window exists; the GUI focuses its window and navigates to the route.
- TUI preference: With `tui=1`, the GUI checks the TUI control index/socket and reuses an existing terminal window if present; otherwise it spawns a follower according to user preferences.
- Create flow: For `create?...`, the GUI shows the same native confirmation dialog (owned by handler/GUI) before creation proceeds.
