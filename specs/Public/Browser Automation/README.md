## Browser Automation

Each document in this folder describes an automation targeting a specific site
that agents‑workflow interacts with. Automations share the Agent Browser
Profiles convention in `../agent-browsers/spec.md` for persistent, named
profiles.

### Structure

- `<site>.md` — High‑level behavior of the automation (e.g., `codex.md`).
- `<site>-testing.md` — Testing strategy and edge cases for the automation.

### Common Principles

- Use Playwright persistent contexts bound to a selected profile.
- Prefer headless execution when the profile’s login expectations are met;
  otherwise, switch to headful and guide the user.
- Detect UI drift and fail fast with actionable diagnostics. When possible,
  surface the browser window to help the user investigate.
