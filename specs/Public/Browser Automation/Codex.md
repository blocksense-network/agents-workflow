## Codex Browser Automation (Playwright)

### Purpose

Automate the Codex WebUI to initiate a coding session for a repository/branch
using a shared agent browser profile. This is the first automation built on the
Agent Browser Profiles convention.

### Behavior (happy path)

1. Determine ChatGPT username: accept optional `--chatgpt-username` (see
   `docs/cli-spec.md`).
2. Discover profiles: list agent browser profiles whose
   `loginExpectations.origins` include `https://chatgpt.com`.3. Filter by username: if `--chatgpt-username` is provided, restrict toprofiles whose `loginExpectations.username` matches.4. Select or create profile:- If one or more profiles match, choose the best candidate (prompt ifmultiple).
   - If none match, create a new profile named `chatgpt-<username>` when a
     username is provided, otherwise `chatgpt`.
3. Override behavior: if `--browser-profile` is provided, skip
   discovery/creation and use that profile name directly (create fresh if missing).
4. Launch Playwright with a persistent context in headless mode.
5. If the expected login is not present, relaunch in visible mode to let the
   user authenticate, then continue.
6. Navigate to Codex, select workspace and branch, enter the task description,
   and press "Code":
   - Workspace comes from `--codex-workspace` or `config: codex-workspace` (see
     `docs/configuration.md`).
   - Branch comes from the `aw task --branch` value.
7. Record success.

If the automation code fails to execute due to potential changes in the Codex
WebUI. Report detailed diagnostic information for the user (e.g. which UI
element you were trying to locate; Which selectors were used and what happened

- the expected element was not found, more than one element was found, etc).

### Visibility and Login Flow

- Runs headless by default; when login is not present, restarts headful to
  allow the user to log in, then proceeds automatically.

### Configuration

Controlled via AW configuration (see `docs/cli-spec.md` and
`docs/configuration.md`):

- Enable/disable automation for `aw task`.
- Select or override the agent browser profile name.
- Set default Codex workspace: `codex-workspace`.

### Notes

- Playwright selectors should prefer role/aria/test id attributes to resist UI
  text changes.
- Use stable navigation points inside Codex (workspace and branch selectors)
  and fail fast with helpful error messages when not found; optionally open
  DevTools in headful mode for investigation.
