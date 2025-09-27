## TUI — Product Requirements and UI Specification

### Summary

The TUI provides a terminal-first dashboard for launching and monitoring agent tasks, integrated with terminal multiplexers (tmux, zellij, screen). It auto-attaches to the active multiplexer session and assumes all active tasks are already visible as multiplexer windows.

Backends:

- REST: Connect to a remote REST service and mirror the WebUI experience for task creation, with windows created locally (or remotely via SSH) for launched tasks.
- Local: Operate in the current directory/repo using the SQLite state database for discovery and status.

### Auto-Attach and Window Model

- On start, `aw tui` auto-attaches to the current multiplexer session (creating one if needed). Existing task windows are left intact.
- Launching a new task immediately creates a new window in the multiplexer:
  - Split panes: right = agent activity, left = terminal or configured editor in the workspace.
  - Devcontainer runs: panes are inside the container context.

### Simplified Task-Centric Layout

The main TUI interface focuses on recent tasks and quick creation:

- **Header**: Compact 3-line header with "Agent Harbor" title (left-aligned with Charm-style powerline design).
- **Previous Tasks**: List of completed, active, merged, and draft tasks displayed as bordered cards (4 lines each) above the new task area.
- **New Task Entry**: Dedicated card at bottom with expandable text area and button-based selectors.

#### Task States

Tasks display in five different states:

- **Merged**: 1-line bordered card showing task title, merge status, and timestamp with visual separator.
- **Completed**: 4-line bordered card showing task title, completion status, result details, and timestamp.
- **Active**: 4-line bordered card showing task title, current action, progress details, and live status.
- **Draft**: 4-line bordered card showing task title, description preview, draft status, and timestamp.
- **New Task**: Interactive card at bottom with expandable text area and repository/branch/model button selectors.

#### Previous Task Cards

Each previous task displays as a bordered card with Charm-inspired styling:

- Rounded borders with proper padding
- 4-line height maximizing information density
- Status-appropriate icons and color coding
- Visual separators between cards
- Themed colors following Catppuccin Mocha palette

#### New Task Card Details

The new task card looks like a auto-expandable text area.

At the bottom of the text area, there is a single line with buttons, which can be navigated to by pressing TAB or by clicking them with the mouse. When a button is activated/clicked, it display a modal dialog that looks like Telescope in vim - a fuzzy text entry, followed by matches that can be selected with the keyboard. The label of the button always matches the currently selected item.

The following buttons are present:

- **Repository Selector**: Telescope-like dialog selector to choose which repository to work in
- **Branch Selector**: Telescope-like dialog to select the branch to work on
- **Model Multi-Selector**: Telescope-like multi-select interface for choosing AI models with instance counts
  - Left/Right arrows or +/- keys to increase/decrease instance count for selected model
  - Multiple models can be selected with different instance counts
- **Go Button**: Button with "⏎ Go" label to launch the task with selected configuration when activated/clicked.

#### Footer (Lazygit-style)

- **Single-line footer** without borders (like Lazygit) showing context-sensitive shortcuts
- **Dashboard mode**: "↑↓ Navigate • Enter Select Task • Ctrl+C x2 Quit"
- **Modal active**: "↑↓ Navigate • Enter Select • Esc Back • Ctrl+C Abort"
- **Task creation**: "Tab Cycle Buttons • Enter Activate Button • Esc Back • Ctrl+C x2 Quit"

### Task Management

- Task list shows recent tasks ordered by recency (older near the top, newest near the bottom).
- Each task displays with appropriate visual indicators for its state.
- Active tasks show live streaming of agent activity.
  This uses exactly 3 lines, showing the 2 or 3 most recent actions of the agent.
  When the action is a though, it takes a single line - a description of the thought.
  When the action is a file edit, it takes a single line - the name of the edited file (plus a number of added and deleted lines, similar to git git)
  When the action is tool use, it takes two lines - the name of the launched tool and the currently last line in the output of the tool. When the tool completes, it is collapsed to a single line which is just the name of the tool use with a visual indicator for success/failure of the command (e.g. `make test` failed with a non-zero exit).
- Draft tasks are saved locally and can be resumed later.
- New task input supports multiline editing with Shift+Enter for line breaks.
- The default values for project/branch/agent are the last ones being used.

### Commands and Hotkeys

#### Global Navigation
- **↑↓**: Navigate between sections (previous tasks and new task entry)
- **Ctrl+C** (twice): Quit the TUI

#### Task Creation Interface
- **Tab/Shift+Tab**: Cycle between buttons (Description, Repository, Branch, Models, Go)
- **Enter**: Activate focused button or select item in modal
- **Esc**: Close modal or go back; Go back to text area when the buttons are selected.
- **Type directly**: Enter text in description area when focused

#### Modal Navigation (Telescope-style)
- **↑↓**: Navigate through options in fuzzy search
- **Enter**: Select current item
- **Esc**: Close modal
- **Left/Right** or **+/-**: Adjust model instance counts in model selection

#### Text Input
- **Any key**: Type in description area when focused
- **Backspace**: Delete characters
- **Enter**: Activate buttons (not for newlines in description)
- **Auto-complete menu**: When certain characters like / or @ are entered in the text area, the UI shows auto-completion menu with dynamically populated choices (@ is used for citing files, / is used to select workflows, etc).

### Error Handling and Status

- Inline validation messages under selectors (e.g., branch not found, agent unsupported).
- Status bar shows backend (`local`/`rest`), selected multiplexer, and last operation result.

### Remote Sessions

- If the REST service indicates the task will run on another machine, the TUI uses provided SSH details to create/attach a remote multiplexer window.

### Persistence

- Last selections (project, agent, branch) are remembered per repo/user scope via the configuration layer.
- Selected theme preference is persisted across sessions.

### Visual Design & Theming

#### Charm-Inspired Aesthetics

The TUI follows Charm (Bubble Tea/Lip Gloss) design principles with multiple theme options:

- **Default Theme**: Catppuccin Mocha - Dark theme with cohesive colors
  - Background: `#11111B`
  - Surface/Card backgrounds: `#242437`
  - Text: `#CDD6F4`
  - Primary: `#89B4FA` (blue for actions)
  - Accent: `#A6E3A1` (green for success)
  - Muted: `#7F849C` (secondary text)
- **Multiple Theme Support**: Users can choose from various themes including:
  - Catppuccin variants (Latte, Frappe, Macchiato, Mocha)
  - Other popular dark themes (Nord, Dracula, Solarized Dark, etc.)
  - High contrast accessibility theme
- **Rounded borders**: `BorderType::Rounded` on all cards and components
- **Proper padding**: Generous spacing with `Padding::new()` for breathing room
- **Powerline-style titles**: ` Title ` glyphs for card headers
- **Truecolor support**: 24-bit RGB colors for rich visual experience

#### Component Styling

- **Cards**: Rounded borders, themed backgrounds, proper padding
- **Buttons**: Background color changes on focus, bold text
- **Modals**: Shadow effects, centered positioning, fuzzy search interface
- **Status indicators**: Color-coded icons (✓ completed, ● active, 📝 draft)

### Accessibility

- **Theme Selection**: Multiple themes including high-contrast accessibility theme
- **High-contrast theme option**: Enhanced contrast ratios for better visibility
- **Full keyboard operation**: All features accessible without mouse
- **Predictable focus order**: Logical tab navigation through all interactive elements
- **Charm theming**: Provides excellent contrast ratios and visual hierarchy
