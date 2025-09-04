# Lima VM Setup — Linux Images for macOS Multi-OS Testing

## Summary

Define Lima VM image variants for agents-workflow multi-OS testing on macOS.
All variants use Nix for agents-workflow components to ensure consistency
across image types.

## VM Image Variants

### Alpine + Nix

- **Base**: Alpine Linux (minimal footprint)
- **Purpose**: Nix-first development environment
- **Package management**: Nix for all development tools and agents-workflow
  components
- **Target users**: Developers preferring declarative, reproducible environments

### Ubuntu LTS

- **Base**: Ubuntu 22.04/24.04 LTS
- **Purpose**: Maximum compatibility and familiar tooling
- **Package management**: APT for system packages, Nix for agents-workflow
  components, wide range of pre-installed package managers and language version
  managers for quick set up specific dependencies.
- **Target users**: General development teams wanting conventional Linux
  environment

## Common Requirements

All images include:

- **Agents-workflow tooling**: Installed via Nix for version consistency
- **Filesystem snapshots**: ZFS or Btrfs support for Agent Time-Travel
- **Multi-OS integration**: SSH access, Tailscale/Netbird/overlay networking
- **Development essentials**: Git, build tools, terminal multiplexers

## Build Components

### Shared Infrastructure

- Common provisioning scripts (reused from Docker container setup)
- Nix flake for agents-workflow tools
