# EXTRAS Framework

A flexible system for installing additional development tools and components during environment setup.

## Quick Start

Set the `EXTRAS` environment variable with the components you want to install:

```bash
# Install Nix package manager
EXTRAS="nix" bash common-pre-setup

# Install multiple components
EXTRAS="nix,direnv,cachix" bash common-pre-setup

# Use different separators
EXTRAS="nix;direnv+cachix" bash common-pre-setup
```

## Available Components

| Component | Description                            | Dependencies |
| --------- | -------------------------------------- | ------------ |
| `nix`     | Nix package manager (foundational)     | None         |
| `direnv`  | Directory-based environment management | nix          |
| `cachix`  | Binary cache service for Nix           | nix          |

## Features

- **Flexible separators**: Use comma (`,`), semicolon (`;`), plus (`+`), pipe (`|`), or space
- **Dependency resolution**: Components are installed in the correct order automatically
- **Duplicate prevention**: Each component is installed only once
- **Legacy compatibility**: `NIX=1` is still supported for backward compatibility
- **Mock testing**: All install scripts use mock output for safe testing

## Advanced Usage

### Using Rake tasks for dependency management

```bash
# Install specific components with proper dependency resolution
rake -f Rakefile.extras nix direnv

# Clean installation markers (for testing)
ruby -Ilib bin/install-extras --clean
```

### Using the install-extras script directly

```bash
EXTRAS="nix,direnv" ruby -Ilib bin/install-extras
```

## Adding New Components

1. Create an install script in `install/install-<component>`
2. Add the component to `Rakefile.extras` with appropriate dependencies
3. Update the validation list in `lib/extras_installer.rb`
4. Add mock environment variables as needed

## Files

- `bin/install-extras` - Main Ruby implementation script
- `lib/extras_installer.rb` - Core ExtrasInstaller class
- `lib/extras_task/cli.rb` - CLI interface
- `Rakefile.extras` - Rake tasks with dependency management
- `install/install-*` - Individual component installation scripts
- `common-pre-setup` - Updated to use the Ruby framework

## Testing

Run the test suite to validate the framework:

```bash
just test
```

The Ruby test suite includes comprehensive tests for the ExtrasInstaller class and CLI functionality. All install scripts use mock output, so no actual software is installed during testing.
