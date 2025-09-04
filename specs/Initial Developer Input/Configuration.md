Help me create a specification for the configuration system of the
Blocksense-network agents-workflow project (you can find it on GitHub) The
configuration will respect multiple layers:

1. Default values provided by the projects maintainers.
2. OS-level configuration values that may be specified by a Linux distribution
   maintainer or a system administrator managing a team of users.
3. User preferences stored in the profile
4. Settings specified for a particular project in its GitHub repository.
5. Per-project settings that may be overridden by the user.
6. Settings specified in ENV variables
7. Settings specified on the command line.

The order above matches the order of priority with one exception. The system
administrator can enforce certain setting preventing the user from overriding
them. The system should be cross-platform. It will offer the same interface
across all operating systems, but optionally may support OS-specific mechanisms
that system administrators typically use for managing employee configurations
in enterprise environments.

We'll use TOML as the configuration format. I’m open to suggestions regarding
the admin settings enforcement, but file system permissions seem like a
reasonable way to handle this (or other similar read-only configuration
stores). Our design should aim to provide consistent user experience across
platforms, but it may provide an opt-in support for platform-specific
approaches where this would enhance the ability of system administrators to
manage the configuration of the employee’s computers in an enterprise
environment. The agents workflow project is expected to have mobile
clients/front-ends (iOS, Android, etc), so exploring their configurability
options to cover our requirements will be good.

We’ll prefix all ENV variables with AGENTS*WORKFLOW* for maximum clarity in
scripts where they are used. Each setting will have its own flag (e.g.
—log-level) 3). We will support the `~/.config/…` directory on Windows too,
as some users might prefer the consistency. When settings are available in both
`APPDATA` and `~/.config`, the later will take precedence. Agents workflow
already uses an `.agents` folder in the user’s repo, so we can store the
per-project configuration files there. There will be a command `aw config` for
reading and applying changes to the various configuration files. When you ask
for a value of a particular setting, the tool will give you a detailed report
in which files the setting appeared. This would serve as an explanation why it
was enforced when this is the case. When the log level is increased to debug,
the CLI tools will report the config files they are trying to read. The
agent-task command presents an editor where the user enters the task
description. Within the loaded editor, there are commented out lines that
explain what the user must do. Within this message there will be also an
indication of the action that will be executed after the editor is closed. This
action will depend on the loaded configuration, so the message will indicate
which configuration source specified it. This requires that the configuration
loading system maintains information about the origin of each setting.
