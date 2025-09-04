# Handling `agents-workflow://` URL Scheme

Developer guide for implementing a cross‑platform custom URL handler that launches a small utility (the **AW URL Handler**) to route `agents-workflow://...` links into your app’s local **AW WebUI** and task database.

> Target OSes: **Windows**, **macOS**, **Linux**

---

# Installing the custom URL Handler

Below are minimal, production‑safe ways to register a URL scheme named `agents-workflow` that invokes a small utility executable (the **AW URL Handler**). The handler’s job is only to parse, validate, and forward—keep it tiny and updateable.

## Windows

### Option A — Classic (unpackaged) apps via Registry (per‑user)

Create these keys (per‑user is recommended so no admin rights are needed):

```reg
Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\Software\Classes\agents-workflow]
@="URL:Agents Workflow Protocol"
"URL Protocol"=""
"DefaultIcon"="\"C:\\Program Files\\AW\\aw-handler.exe\",1"

[HKEY_CURRENT_USER\Software\Classes\agents-workflow\shell]

[HKEY_CURRENT_USER\Software\Classes\agents-workflow\shell\open]

[HKEY_CURRENT_USER\Software\Classes\agents-workflow\shell\open\command]
@="\"C:\\Program Files\\AW\\aw-handler.exe\" \"%1\""
```

**Notes**

- Use `"%1"` so the full URL is passed to the handler.
- `HKEY_LOCAL_MACHINE\Software\Classes` (system‑wide) also works but requires elevation; prefer HKCU for installers that avoid UAC prompts.
- To open a URL in the default browser from your handler, call `ShellExecuteW(NULL, L"open", L"https://…", NULL, NULL, SW_SHOWNORMAL);`.

### Option B — MSIX/Packaged apps

Declare a **Protocol** extension in the package manifest and handle **URI activation**. This is the recommended path for MSIX‑packaged apps.

## macOS

### Registering the scheme in the app’s Info.plist

Add **CFBundleURLTypes** to the bundle that will receive the URL (see the helper‑bundle pattern below):

```xml
<key>CFBundleURLTypes</key>
<array>
  <dict>
    <key>CFBundleURLName</key>
    <string>agents-workflow</string>
    <key>CFBundleURLSchemes</key>
    <array>
      <string>agents-workflow</string>
    </array>
  </dict>
</array>
```

Implement `application(_:open:)` (AppKit) or equivalent to receive the URL and forward it to the AW URL Handler logic.

#### Helper‑bundle pattern (optional with Electron)

For an **Electron‑based GUI**, you can either:

- **Register the main Electron app** as the protocol handler, or
- Embed a tiny **native helper .app** inside your Electron bundle (e.g., `MyApp.app/Contents/Resources/AW Handler.app`) and register the helper as the handler. The helper parses/validates the URL and forwards it to the running Electron app over IPC (XPC/local socket) or launches the app if needed.

If you use the main Electron app:

- Register with `app.setAsDefaultProtocolClient('agents-workflow')` in the **main process**.
- Handle **macOS** deep links via `app.on('open-url', (event, url) => { ... })`.
- Handle **Windows/Linux** deep links via a **single‑instance** lock and the `'second-instance'` event; the deep link will arrive in `commandLine` (Windows/Linux) when a second instance is triggered.
- On first run, ensure the bundle is known to Launch Services (e.g., `LSRegisterURL`) so the scheme resolves reliably.

**Cross‑platform main‑process skeleton:**

```js
// main.js
const { app, BrowserWindow } = require("electron");

// Register as default protocol client (packaged apps)
if (process.defaultApp) {
  // Dev mode on Windows requires exe + args
  const path = require("node:path");
  if (process.argv.length >= 2) {
    app.setAsDefaultProtocolClient("agents-workflow", process.execPath, [
      path.resolve(process.argv[1]),
    ]);
  }
} else {
  app.setAsDefaultProtocolClient("agents-workflow");
}

// Keep a single instance to funnel deep links into one process
const gotLock = app.requestSingleInstanceLock();
if (!gotLock) app.quit();

let mainWindow;
function handleDeepLink(url) {
  // Validate/normalize the URL, then ensure WebUI → open route
}

app.on("second-instance", (event, argv) => {
  const urlArg = argv.find((a) => a.startsWith("agents-workflow://"));
  if (urlArg) handleDeepLink(urlArg);
  if (mainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
  }
});

// macOS specific deep link event
app.on("open-url", (event, url) => {
  event.preventDefault();
  handleDeepLink(url);
});

app.whenReady().then(() => {
  mainWindow = new BrowserWindow({});
  // load your UI...
});
```

## Linux (FreeDesktop environments)

Create a `.desktop` file that declares support for the `x-scheme-handler/agents-workflow` MIME type and point it at your handler executable:

```ini
# ~/.local/share/applications/aw-url-handler.desktop
[Desktop Entry]
Name=AW URL Handler
Exec=/usr/local/bin/aw-handler %u
Terminal=false
Type=Application
MimeType=x-scheme-handler/agents-workflow;
```

Register it as the default handler and update caches:

```bash
xdg-mime default aw-url-handler.desktop x-scheme-handler/agents-workflow
update-desktop-database ~/.local/share/applications || true
# Inspect current handler
gio mime x-scheme-handler/agents-workflow
```

To open a URL in the default browser from your handler, execute `xdg-open "http://127.0.0.1:8787/tasks/1234"`.

> **Browser prompts:** Some browsers display an “External Protocol Request” confirmation when invoking native handlers. That UX is browser‑controlled and expected; don’t try to bypass it.

---

# Handling task URLs

The `agents-workflow://` scheme transports task intents into the local AW system. The **AW URL Handler** performs four jobs: **validate → normalize → ensure WebUI → open**.

## URL shapes

- `agents-workflow://task/<id>` → Open task result page for ID
- `agents-workflow://task/<id>?tui=1` → Also launch the TUI follow view
- `agents-workflow://create?spec=…` → (optional) Create & queue task from a spec payload

All URLs should be purely declarative; don’t embed secrets. Reject unknown hosts/components.

## End‑to‑end behavior

```mermaid
flowchart TD
  A[User clicks agents-workflow://...] --> B{OS deep-link}
  B -->|Windows Reg or MSIX| C[aw-handler.exe]
  B -->|macOS CFBundleURLTypes| D[AW Handler.app]
  B -->|Linux x-scheme-handler| E[/usr/local/bin/aw-handler]

  C & D & E --> F[Parse & validate URL]
  F --> G{Is AW WebUI server running?}
  G -->|No| H[Start WebUI server]
  H --> I[Check local SQLite for task]
  G -->|Yes| I
  I -->|Found| J[Build http://127.0.0.1:8787/tasks/<id>]
  I -->|Missing| K[Graceful error page]
  J --> L[Open in default browser]
```

### Launching the default browser

\$1**Electron (any OS)**

```js
const { shell } = require("electron");
await shell.openExternal("http://127.0.0.1:8787/tasks/1234");
```

Your handler often needs to open a local WebUI route like `http://127.0.0.1:8787/tasks/<id>`.

### Windows (Browser Launch)

Use the Shell to route to the user’s default browser:

```cpp
#include <windows.h>
#include <shellapi.h>

ShellExecuteW(NULL, L"open", L"http://127.0.0.1:8787/tasks/1234", NULL, NULL, SW_SHOWNORMAL);
```

Or from a batch script: `start "" http://127.0.0.1:8787/tasks/1234`.

### macOS (Browser Launch)

From Swift:

```swift
NSWorkspace.shared.open(URL(string: "http://127.0.0.1:8787/tasks/1234")!)
```

From a script/binary: `open "http://127.0.0.1:8787/tasks/1234"`.

## Linux

From C/Go/Rust, execute `xdg-open <url>` (or the desktop portal equivalent) and let the environment choose the default browser:

```bash
xdg-open "http://127.0.0.1:8787/tasks/1234" >/dev/null 2>&1 &
```

> **Tip:** Always validate/escape the URL you pass to the OS. Never include secrets in the query string; prefer cookies or local tokens bound to `localhost`.

---

## macOS + Electron packaging notes

- **Register protocol**: Use Electron’s **Deep Links** pattern. On macOS, include the scheme in the app’s `Info.plist` (your packager will generate this if configured). In code, call `app.setAsDefaultProtocolClient('agents-workflow')` in the **main process**.
- **Events**: Handle `app.on('open-url', ...)` on macOS, and `app.requestSingleInstanceLock()` + `'second-instance'` on Windows/Linux so a running instance processes the URL.
- **Packaging**:
  - **Electron Forge**: add `packagerConfig.protocols` with `{ name: 'Agents Workflow', schemes: ['agents-workflow'] }`, and for Linux makers set `mimeType: ['x-scheme-handler/agents-workflow']`.
  - **Electron Packager**: pass a `protocols` array in options for macOS builds.
  - **electron-builder**: set `protocols` under `mac` config; it writes `CFBundleURLTypes` in `Info.plist`. (On Linux, ensure your `.desktop` file declares `x-scheme-handler/agents-workflow`.)

- **Helper bundle (optional)**: If you embed a tiny handler `.app`, register that bundle as the URL handler and forward to the Electron app via IPC; use `LSRegisterURL` to ensure Launch Services sees the helper at install/first‑run.

---

## References (verifiable)

**Windows**

- URI activation / protocol handlers (MSIX): [https://learn.microsoft.com/en-us/windows/apps/develop/launch/handle-uri-activation](https://learn.microsoft.com/en-us/windows/apps/develop/launch/handle-uri-activation)
- `ShellExecuteW` docs (open default browser): [https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew)
- Shell launching overview: [https://learn.microsoft.com/en-us/windows/win32/shell/launch](https://learn.microsoft.com/en-us/windows/win32/shell/launch)
- Toast notifications (desktop apps): [https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/toast-notifications-overview](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/toast-notifications-overview)

**macOS**

- `CFBundleURLTypes` (Info.plist): [https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleurltypes](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleurltypes)
- Launch Services overview / tasks: [https://developer.apple.com/documentation/coreservices/launch_services](https://developer.apple.com/documentation/coreservices/launch_services)
- Register app with Launch Services `LSRegisterURL`: [https://developer.apple.com/documentation/coreservices/1446350-lsregisterurl](https://developer.apple.com/documentation/coreservices/1446350-lsregisterurl)
- Programmatically set default handler for a scheme: [https://developer.apple.com/documentation/coreservices/1447760-lssetdefaulthandlerforurlscheme](https://developer.apple.com/documentation/coreservices/1447760-lssetdefaulthandlerforurlscheme)
- `NSWorkspace.open` to open URLs: [https://manpagez.com/man/1/open/](https://manpagez.com/man/1/open/)

**Linux (FreeDesktop)**

- XDG MIME Applications (scheme handling via `x-scheme-handler/<scheme>`): [https://wiki.archlinux.org/title/XDG_MIME_Applications](https://wiki.archlinux.org/title/XDG_MIME_Applications)
- Desktop Notifications spec (for completion toasts): [https://specifications.freedesktop.org/notification-spec/latest/](https://specifications.freedesktop.org/notification-spec/latest/)
- `gio mime` manual: [https://man.archlinux.org/man/gio.1](https://man.archlinux.org/man/gio.1)
- `xdg-open` manual: [https://man.archlinux.org/man/xdg-open.1](https://man.archlinux.org/man/xdg-open.1)

**Electron**

- Deep Links tutorial (protocol handlers & cross‑platform patterns): [https://www.electronjs.org/docs/latest/tutorial/launch-app-from-url-in-another-app](https://www.electronjs.org/docs/latest/tutorial/launch-app-from-url-in-another-app)
- `app` API: `'open-url'` (macOS), `requestSingleInstanceLock`, `setAsDefaultProtocolClient`: [https://www.electronjs.org/docs/latest/api/app](https://www.electronjs.org/docs/latest/api/app)
- `shell.openExternal` to open default browser: [https://www.electronjs.org/docs/latest/api/shell](https://www.electronjs.org/docs/latest/api/shell)
- Electron Forge configuration for protocols (see Deep Links page): [https://www.electronjs.org/docs/latest/tutorial/launch-app-from-url-in-another-app](https://www.electronjs.org/docs/latest/tutorial/launch-app-from-url-in-another-app)
- electron-builder `protocols` config: [https://www.electron.build/app-builder-lib.interface.protocol](https://www.electron.build/app-builder-lib.interface.protocol)

---

## Security & UX checklist

- Validate URL inputs strictly; reject unknown commands and malformed IDs.
- Use per‑machine ephemeral tokens or session cookies for WebUI auth rather than query strings.
- Prefer per‑user registration (HKCU on Windows, `~/.local/share/applications` on Linux) to avoid elevation.
- Expect browser “Open external application?” prompts and document that for users.
- Make the handler idempotent (re‑entry on already‑running tasks should just focus the page).
