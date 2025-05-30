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
    the `agent-task` command with the desired branch name:

    ```bash
    agent-task <branch-name>
    ```

    This script will:
    -   Create the branch first and abort early with the VCS error message
        if the name is invalid.
    -   Prompt the developer to enter the task description in an editor.
    -   Commit the task description to a file within a `.agents/tasks/`
        directory on the new branch.
    -   Push the branch to the default remote.

    The command accepts a few options for non-interactive use:

    - `--push-to-remote=BOOL` – automatically push to the default remote without prompting.
    - `--prompt=STRING` – use `STRING` as the task description instead of launching an editor.
    - `--prompt-file=FILE` – read the task description from `FILE`.

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

## Setting up with OpenAI Codex

The `codex-setup` script in this repo is intended to be used as a
custom setup step within an OpenAI Codex workspace.

Just add the following commands in all of our workspaces:

```bash
git clone https://github.com/metacraft-labs/agents-workflow
agents-workflow/codex-setup
```

This process will:
-   Install Nix if specified (via the `NIX=1` environment variable).
-   Download internet resources mentioned in task descriptions.
-   Execute a project-specific setup script located at `.agents/codex-setup` in your repository, if one exists. This allows for custom setup actions tailored to the project Codex will be working on.

### Installing as a Ruby gem

TBD

### What's included?

The core components include:
-   `codex-setup`: A script to initialize the workspace, download necessary internet resources, and run project-specific setup.
-   `agent-task`: A script for developers to begin a new task, automatically creating a dedicated branch and storing the task description.
-   `get-task`: A script for the coding agent to retrieve its current task instructions.
-   `download-internet-resources`: A helper script that scans task descriptions for URLs and downloads them (or clones Git repositories) for offline access.

## Future Direction

We envision that the manual step of prompting the agent to run `get-task` could be automated in the future through:

-   An API integration with Codex.
-   (interim) A browser extension that drives the Codex WebUI.

### RubyGem Installation

All Ruby scripts are bundled into a single gem for easier reuse:

```bash
gem install --local agent-task-*.gem
```

This will provide the `agent-task`, `get-task`, and `download-internet-resources` executables in your `PATH`.
The gem also exposes its functionality via `AgentTask::CLI` so it can be invoked programmatically.
