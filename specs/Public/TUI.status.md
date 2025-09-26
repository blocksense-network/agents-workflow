### Overview

This document tracks the implementation status of the [TUI-PRD.md](TUI-PRD.md) functionality.

Goal: deliver a production-ready terminal-based dashboard for creating, monitoring, and managing agent coding sessions with seamless multiplexer integration, keyboard-driven workflows, and REST service connectivity.

**Current Status**: Planning complete, ready to start T1 implementation
**Test Results**: 0 tests (implementation pending)
**Last Updated**: September 26, 2025

Total estimated timeline: 6-8 weeks (broken into major phases with parallel development tracks)

### Milestone Completion & Outstanding Tasks

Each milestone maintains an **outstanding tasks list** that tracks specific deliverables, bugs, and improvements. When milestones are completed, their sections are expanded with:

- Implementation details and architectural decisions
- References to key source files for diving into the implementation
- Test coverage reports and known limitations
- Integration points with other milestones/tracks

### TUI Feature Set

The TUI implementation provides these core capabilities:

- **Simplified Dashboard**: Project, Branch, and Agent selectors with task description editor
- **Multiplexer Integration**: Auto-attach to tmux/zellij/screen sessions with split-pane layouts
- **Keyboard-Driven UI**: Full keyboard navigation with contextual hotkeys and shortcuts
- **REST Service Backend**: Remote server connectivity for task creation and monitoring
- **Session Management**: Launch tasks directly into multiplexer windows with proper pane layout
- **Responsive Layout**: Fixed-height selectors with resizable description editor
- **Contextual Help**: Dynamic footer showing relevant shortcuts based on current UI state
- **Error Handling**: Inline validation messages and graceful error recovery

### Parallel Development Tracks

Multiple development tracks can proceed in parallel once the core infrastructure (T1) is established:

- **UI Components Track**: Build Ratatui widgets for selectors, editors, and status displays
- **REST Client Track**: Implement and test the Rust REST API client crate
- **Multiplexer Integration Track**: Develop tmux/zellij/screen abstraction layer
- **CLI Integration Track**: Wire `aw tui` command with proper flag handling
- **Testing Infrastructure Track**: Develop integration tests for TUI interactions and REST connectivity

### Approach

- **Ratatui + Crossterm**: Modern Rust terminal UI framework with cross-platform terminal handling
- **REST Client Crate**: Dedicated `aw-rest-client` crate for API communication with mock server development
- **Multiplexer Abstraction**: Unified interface for tmux/zellij/screen with auto-detection and fallback
- **State-Driven UI**: Reactive UI updates based on REST API responses and user interactions
- **Mock-First Development**: Start with comprehensive REST client testing against mock server ([Mock Server README](../webui/mock-server/README.md))
- **Integration Testing**: Playwright-style terminal automation for comprehensive E2E coverage
- **Progressive Enhancement**: Core functionality works without advanced terminal features

**Mock Server Setup**: See [Mock Server README](../webui/mock-server/README.md) for instructions on starting the mock REST API server (`just webui-mock-server`) required for TUI development and testing.

### Development Phases (with Parallel Tracks)

**Phase 1: Foundation** (3-4 weeks total)

**T1. Infrastructure Setup and REST Client** (2 weeks)

- **Deliverables**:

  - Rust REST API contracts crate (`aw-rest-api-contract`) with schema types and validation
  - Rust REST client crate (`aw-rest-client`) with full API coverage
  - Ratatui + Crossterm project scaffolding with basic event loop
  - Mock server integration for development and testing
  - Basic CLI command structure with `--remote-server` flag support
  - Project structure following Repository-Layout.md guidelines
  - Development tooling configuration (Cargo, Clippy, testing framework)

- **Test Coverage** (Comprehensive API Contract + Unit):

  - [ ] REST client API contract tests against mock server: All endpoints match REST-Service.md specs
  - [ ] Authentication handling tests against mock server: API key, JWT, and OIDC flows
  - [ ] Error response parsing tests against mock server: Problem+JSON error format handling
  - [ ] Pagination handling tests against mock server: page/perPage query params and response metadata
  - [ ] SSE streaming tests against mock server: EventSource connection and event parsing
  - [ ] CLI flag parsing tests: `--remote-server` flag validation and config integration
  - [ ] Basic UI rendering tests: Ratatui widget initialization and layout

- **Verification** (Automated Unit + Integration):

  - [ ] Unit tests for REST API contracts crate covering schema validation
  - [ ] Unit tests for REST client crate covering all API endpoints against mock server
  - [ ] Integration tests against mock server for end-to-end API flows and error scenarios
  - [ ] CLI tests verifying `aw tui --remote-server` command parsing
  - [ ] Build tests: All crates compile successfully with Cargo
  - [ ] Tooling tests: Clippy and rustfmt configurations work across all crates

**T2. Core Dashboard Layout** (2 weeks, parallel with T1)

- **Deliverables**:

  - Three-section dashboard layout (selectors + description editor)
  - Fixed-height list widgets for Project, Branch, and Agent selectors
  - Resizable multiline task description editor
  - Keyboard navigation between sections (Tab/Shift+Tab)
  - Basic REST data loading and display
  - Dynamic footer with contextual shortcuts

- **Test Coverage** (Comprehensive Integration + UI):

  - [ ] Layout rendering tests: Dashboard renders correctly on different terminal sizes
  - [ ] Keyboard navigation tests: Tab cycling and arrow key navigation work
  - [ ] Selector interaction tests: List filtering and selection functionality
  - [ ] Editor resizing tests: Ctrl+Up/Down resize operations
  - [ ] REST data integration tests: API responses populate selectors correctly
  - [ ] Footer shortcut tests: Context-sensitive shortcuts display appropriately

- **Verification** (Automated E2E + Manual):

  - Playwright-style terminal automation tests for UI interactions
  - Manual verification: Layout adapts to terminal width/height changes
  - Manual verification: Keyboard shortcuts work as specified in TUI-PRD.md

**T3. Task Creation and Launch** (2 weeks)

- **Deliverables**:

  - Task creation workflow from dashboard input
  - Multiplexer auto-detection and session management
  - New window creation with split-pane layout (agent right, editor/shell left)
  - Session launch and monitoring integration
  - Error handling for failed task creation
  - Success feedback and session attachment

- **Test Coverage** (Integration + E2E):

  - [ ] Task creation tests: Full workflow from input to REST API call
  - [ ] Multiplexer detection tests: Auto-detection of available multiplexers
  - [ ] Window creation tests: New multiplexer windows with correct pane layout
  - [ ] Session attachment tests: Proper attachment to running sessions
  - [ ] Error handling tests: Failed task creation shows appropriate errors

- **Verification** (Automated E2E):

  - End-to-end tests simulating complete task creation workflow
  - Multiplexer integration tests across tmux/zellij/screen

**Phase 2: Advanced Features** (3-4 weeks total)

**T4. Real-time Session Monitoring** (2 weeks)

- **Deliverables**:

  - Live session status updates via SSE
  - Session list integration with running tasks
  - Real-time log streaming in multiplexer panes
  - Status indicators and progress feedback
  - Connection error handling and reconnection

- **Test Coverage** (Real-time + Integration):

  - [ ] SSE event handling tests: Real-time updates from REST service
  - [ ] Session monitoring tests: Live status display and updates
  - [ ] Log streaming tests: Real-time log display in TUI
  - [ ] Connection resilience tests: Network disconnections handled gracefully

- **Verification** (Automated E2E):

  - Real-time update tests with mock server event streaming
  - Network failure simulation and recovery testing

**T5. Advanced Multiplexer Features** (2 weeks, parallel with T4)

- **Deliverables**:

  - Full multiplexer abstraction layer
  - Devcontainer-aware pane launching
  - Remote session attachment via SSH
  - Cross-multiplexer compatibility (tmux/zellij/screen)
  - Pane layout customization and persistence

- **Test Coverage** (Cross-platform + Integration):

  - [ ] Multiplexer abstraction tests: Unified interface works across all supported multiplexers
  - [ ] Devcontainer integration tests: Pane launching inside container context
  - [ ] Remote attachment tests: SSH-based remote session connections
  - [ ] Layout persistence tests: Custom layouts saved and restored

- **Verification** (Automated E2E):

  - Multi-multiplexer compatibility tests
  - Devcontainer workflow integration testing

**Phase 3: Polish and Production** (2-3 weeks total)

**T6. Comprehensive Testing and UX Polish** (2 weeks)

- **Deliverables**:

  - Full E2E test coverage for all user journeys
  - Accessibility improvements (high-contrast themes, keyboard navigation)
  - Performance optimization for large session lists
  - Error message improvements and user guidance
  - Configuration persistence and restoration

- **Test Coverage** (E2E + Performance):

  - [ ] Complete user journey tests: Task creation → launch → monitoring → completion
  - [ ] Performance regression tests: Large session lists and high-frequency updates
  - [ ] Accessibility tests: Keyboard navigation and screen reader compatibility
  - [ ] Cross-terminal tests: Different terminal emulators and sizes

- **Verification** (Automated E2E + Performance):

  - Comprehensive test suite covering all TUI-PRD.md workflows
  - Performance benchmarks for real-time updates and large datasets

**T7. Production Readiness** (1 week, parallel with T6)

- **Deliverables**:

  - Production build configuration
  - Binary packaging and distribution
  - Documentation and user guides
  - Final integration testing with WebUI workflows

- **Test Coverage** (Release + Integration):

  - [ ] Production build tests: Optimized release builds work correctly
  - [ ] Packaging tests: Binary distribution and installation procedures
  - [ ] Documentation tests: User guides enable successful TUI usage
  - [ ] Integration tests: TUI works alongside WebUI for same sessions

- **Verification** (Automated Release):

  - Release pipeline validation and distribution testing

### Test strategy & tooling

- **Distributed Test Coverage**: Each milestone includes specific tests verifying its deliverables, preventing regressions and ensuring quality incrementally
- **Terminal Automation Testing**: Custom terminal automation framework for E2E TUI testing, similar to Playwright but for terminal UIs
- **Mock Server Development**: Start with full REST-Service.md mock implementation for isolated TUI development
- **Component Testing**: Unit tests for individual Ratatui widgets and REST client components
- **API Contract Testing**: Verify REST client behavior matches specifications and handles edge cases
- **Integration Testing**: End-to-end workflows testing TUI against real REST service
- **Performance Testing**: Terminal rendering performance and real-time update latency measurements

### Deliverables

- Production-ready TUI application built with Ratatui + Crossterm
- Comprehensive REST client crate for API communication
- Full multiplexer abstraction layer with tmux/zellij/screen support
- Distributed test coverage across all milestones with CI integration
- Terminal automation test suite covering all user journeys
- Performance testing and optimization validation
- Binary packaging and distribution for multiple platforms
- Documentation and integration with broader AW ecosystem

### Next Milestone Priority

**T2. Core Dashboard Layout** is the next priority milestone, providing the fundamental TUI interface that users will interact with for task creation and session management.

### Current Outstanding Tasks

Here are the key tasks for TUI development:

#### **REST Client Infrastructure** 📋 **PENDING**

- [ ] `aw-rest-api-contract` crate with schema types and validation
- [ ] `aw-rest-client` crate with full REST-Service.md coverage
- [ ] SSE streaming support for real-time updates
- [ ] Authentication handling (API key, JWT, OIDC)
- [ ] Error response parsing and user-friendly error messages
- [ ] Comprehensive unit and integration test coverage
- [ ] Mock server integration for isolated development

#### **TUI Foundation** 📋 **PENDING**

- [ ] Ratatui + Crossterm application scaffolding
- [ ] Basic event loop and terminal handling
- [ ] CLI command integration with `--remote-server` flag
- [ ] Project structure following Repository-Layout.md
- [ ] Basic UI widget initialization

#### **Integration & Advanced Features** 📋 **PENDING**

- [ ] Full dashboard layout with selectors and editor
- [ ] Keyboard navigation and shortcut handling
- [ ] Multiplexer integration and auto-detection
- [ ] Real-time session monitoring and updates
- [ ] Remote session attachment via SSH
- [ ] Devcontainer-aware pane launching

### Risks & mitigations

- **Terminal Compatibility**: Wide variety of terminal emulators with different capabilities; mitigated by progressive enhancement and fallback to basic functionality
- **Multiplexer Complexity**: Each multiplexer has different APIs and behaviors; mitigated by abstraction layer with comprehensive testing
- **Real-time Performance**: SSE updates and UI rendering could impact responsiveness; mitigated by efficient rendering and update batching
- **Cross-platform Terminal Handling**: Different platforms handle terminals differently; mitigated by Crossterm abstraction and extensive testing
- **REST Client Maturity**: New crate needs thorough testing; mitigated by comprehensive API contract tests and mock server development

### Implementation Notes

- **Architecture Alignment**: TUI follows same patterns as WebUI (REST client, mock-first development, comprehensive testing)
- **Code Reuse**: REST client crate can be shared with other components (CLI tools, future UIs)
- **Testing Strategy**: Terminal automation framework enables reliable E2E testing of TUI interactions
- **User Experience**: Focus on keyboard-driven workflows optimized for terminal users
- **Integration Points**: TUI works seamlessly with WebUI for monitoring same sessions from different interfaces
