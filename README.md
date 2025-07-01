## Overview

This repository provides an opinionated workflow designed to
enhance the performace of teams working with coding agents,
such as OpenAI Codex, Claude Code, Google Jules, GitHub Copilot,
Goose, OpenHands and others.

The workflow standardizes how tasks are defined, and tracked,
leveraging your VCS (e.g. git) as the primary driving interface.

## Purpose

The primary goal of this workflow is to:

1.  **Use git as the primary interface for driving Codex:**

    Provides a convenient way to assign tasks to Codex right
    from your command-line.

2.  **Maintain a transparent history:**

    All task descriptions are committed to Git, creating an
    auditable trail and a knowledge base demonstrating how
    tasks are approached and solved.

    This allows team members to learn from each other's practices
    and makes `git blame` an effective tool for understanding the
    intention behind all code. The workflow injects instructions
    that teach the agents how to leverage this.

3.  **Deal with the current limitations around internet connectivity:**

    By pre-fetching internet resources mentioned in the task
    descriptions, agents such as Codex are more successful at
    dealing with problems that require information that is not
    part of the codebase.

4.  **Simplify the agents workspace setup:**

    The `.agents/codex-setup` script is stored in your repository,
    simplifying the maintainance of the workspace.

## Using the Workflow

1.  **Starting a Task (Developer):**

    When a developer needs to assign a task to the agent, they run
    the `agent-task` command. If a branch name is provided it starts a
    new branch, otherwise it appends a follow-up task on the current
    branch.

    ```bash
    agent-task [branch-name]
    ```

    This script will:
    -   When a branch name is supplied, create the branch first and abort
        early with the VCS error message if the name is invalid.
    -   Prompt the developer to enter the task description in an editor.
    -   Commit the task description to a file within a `.agents/tasks/`
        directory on the new branch or append it as a follow-up task if
        no branch was given.
    -   Push the branch to the default remote.

    The command accepts a few options for non-interactive use:

    - `--push-to-remote=BOOL` – automatically push to the default remote without prompting.
    - `--prompt=STRING` – use `STRING` as the task description instead of launching an editor.
    - `--prompt-file=FILE` – read the task description from `FILE`.
    - `--devshell=NAME` (`-s`) – record the given Nix dev shell in the initial commit message.

    The command also provides a `setup` subcommand that prints the versions of `codex` and `goose` available in the current `PATH`.

2.  **Retrieving a Task (Coding Agent):**

    Once the developer has set up the task, they instruct the agent to
    switch to the right branch and retrieve its instructions.
    A typical prompt for an agent would be:

    ```
    Run the `get-task` command and follow the provided instructions.
    ```

    The `get-task` script will print the task description for the agent,
    along with instructions for accessing the downloaded internet resources
    and working with the git history.
    It also supports a `--get-setup-env` option which prints only the
    environment variable assignments gathered from `@agents-setup` lines.

### Workflow Commands

Task descriptions may include lines beginning with `/name`. When `get-task`
is executed these lines are replaced with the output of a matching program in
`.agents/workflows/name` (or the contents of `.agents/workflows/name.txt`).
Lines starting with `@agents-setup` in either the task file or the workflow
output are stripped from the final message and interpreted as environment
variable assignments. These variables can be listed with `get-task --get-setup-env`
and are automatically exported by the `*-setup` scripts before executing the
project-specific setup steps.

## Supported Agent Systems

This workflow supports setup for multiple AI coding agent systems:

- **[Codex](https://openai.com/blog/openai-codex)** - OpenAI's code generation model
- **[Jules](https://jules.google.com/)** - AI pair programming assistant
- **[Goose](https://github.com/square/goose)** - AI-powered development tool
- **[Open Hands](https://github.com/All-Hands-AI/OpenHands)** - Open-source AI coding assistant
- **[GitHub Copilot](https://github.com/features/copilot)** - GitHub's AI pair programmer

## Setup Script Architecture

Each agent system has a dedicated setup script (e.g., `codex-setup`, `jules-setup`) that follows a three-phase setup process:

1. `.agents/common-pre-setup` runs first (if it exists in your project)
2. `.agents/{agent}-setup` runs for agent-specific configuration (if it exists)
3. `.agents/common-post-setup` runs last for finalization tasks (if it exists)

This architecture allows you to share common setup logic across all agent systems while customizing setup for specific agents when needed.

## Usage by Agent System

### Codex

In your Codex environment's Advanced settings, enter the following setup script:

```
git clone https://github.com/metacraft-labs/agents-workflow
agents-workflow/codex-setup
```

### Jules

In the Jules web-interface, select a codebase in the left-hand-side panel, click
"Configuration" and enter the following Initial Script:

```
git clone https://github.com/metacraft-labs/agents-workflow
agents-workflow/codex-setup
```

### Goose

**TBD** - Usage instructions for Goose will be added once integration is tested.

### Open Hands

**TBD** - Usage instructions for Open Hands will be added once integration is tested.

### GitHub Copilot

**TBD** - Usage instructions for GitHub Copilot will be added once integration is tested.

## Environment Variables

- `NIX=1` - Set this to enable Nix installation during common-pre-setup

### Installing as a Ruby gem

The scripts can be installed as a gem for easier reuse:

```bash
gem install --local agents-workflow.gem
```

This will provide the `agent-task`, `get-task`, and `download-internet-resources` executables in your `PATH`.

To enable bash completion for `agent-task`, source the script `scripts/agent-task-completion.bash` in your shell profile.

### Installing with Nix

This repository also provides a Nix flake. The default package installs the `agent-task` binary with `codex` and `goose` available in its `PATH`. An additional `agent-utils` package bundles the `get-task` and `start-work` binaries.

```bash
nix run github:metacraft-labs/agents-workflow
```

Or install the utilities package:

```bash
nix profile install github:metacraft-labs/agents-workflow#agent-utils
```

### What's included?

The core components include:
-   `codex-setup`: A script to initialize the workspace, download necessary internet resources, and run project-specific setup.
-   `agent-task`: A script for developers to begin a new task, automatically creating a dedicated branch and storing the task description.
-   `agent-task setup`: Prints the versions of `codex` and `goose` available in `PATH`.
-   `get-task`: A script for the coding agent to retrieve its current task instructions.
-   `start-work`: A helper that configures a freshly checked-out repository for development.
-   `download-internet-resources`: A helper script that scans task descriptions for URLs and downloads them (or clones Git repositories) for offline access.

## Future Direction

We envision that the manual step of prompting the agent to run `get-task` could be automated in the future through:

-   An API integration with Codex.
-   (interim) A browser extension that drives the Codex WebUI.
