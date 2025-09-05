This templates defines the information we try to collect for each agentic coding software supported by Agents Workflow. Usually this information can be obtained by checking out command-line help screens or man pages for the agent software.

# <Agent Tool> — Integration Notes

## Overview

Provide a brief introduction to the tool by listing its web-site, documentation, etc.

### Task start-up command

How do we start the agent with a specific task prompt?

### Support for custom hooks

Does the agent support custom hooks or commands to be executed during its work (e.g. before or after each file modification or tool use)? Detail how this is configured.

Please note that by custom hooks, We are not referring just to MCP tools, but specifically about the ability to configure certain commands to be executed after every agent step, so we can implement our [Agent Time Travel feature](../Agent%20Time%20Travel.md).

### Built-in support for checkpoints

Does the agent support "checkpoints" or other mechanisms for saving and restoring its work along the way? Do the checkpoints cover the chat content, the file system state or both? How do restore a session from a specific moment in time (specific chat message or agent prompt position)? Is the checkpointing mechanism going to ensure that the file system state is restored too?

### How is the use of MCP servers configured?

What are the precise command-line options, ENV variables (if available) and configuration files that control this?

### Credentials

Where are the agent login credentials stored? What are the precise paths of its settings and credentials files? If the help screens don't provide this information, use web search to find a definitive answer and provide links to the discovered resources.

### Known Issues

Are there any platform quirks, rate limits, stability notes, etc?
