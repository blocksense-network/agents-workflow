### Overview

This document tracks the implementation status of the [TUI-PRD.md](TUI-PRD.md) functionality.

Goal: deliver a production-ready terminal-based dashboard for creating, monitoring, and managing agent coding sessions with seamless multiplexer integration, keyboard-driven workflows, and REST service connectivity.

**Current Status**: T2 completed, Testing Framework fully implemented with production-ready golden files, T3 ready
**Test Results**: 9 tests passing (layout rendering, MVVM integration, scenario execution, golden files with multi-line TUI visualization)
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

✅ **Infrastructure established (T1-T2 completed)** - All development tracks can now proceed in parallel:

- **UI Components Track** ✅ **COMPLETED**: Full MVVM architecture with Ratatui widgets, state management, and keyboard navigation
- **REST Client Track** ✅ **COMPLETED**: Rust REST API client crate fully implemented and tested
- **Multiplexer Integration Track** 📋 **READY**: tmux/zellij/screen abstraction layer (T3 Task Creation and Launch)
- **CLI Integration Track** ✅ **COMPLETED**: `aw tui` command with proper flag handling integrated
- **Testing Infrastructure Track** ✅ **COMPLETED**: Comprehensive TUI testing framework with scenario automation and interactive debugging

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

**T1. Infrastructure Setup and REST Client** ✅ **COMPLETED** (September 26, 2025)

- **Deliverables**:

  - ✅ Rust REST API contracts crate (`aw-rest-api-contract`) with schema types and validation
  - ✅ Rust REST client crate (`aw-rest-client`) with full API coverage
  - ✅ Ratatui + Crossterm project scaffolding with basic event loop
  - ✅ Mock server integration for development and testing
  - ✅ Basic CLI command structure with `--remote-server` flag support
  - ✅ Project structure following Repository-Layout.md guidelines
  - ✅ Development tooling configuration (Cargo, Clippy, testing framework)

- **Test Coverage** (Comprehensive API Contract + Unit):

  - [x] REST client API contract tests against mock server: All endpoints match REST-Service.md specs
  - [x] Authentication handling tests against mock server: API key, JWT, and OIDC flows
  - [x] Error response parsing tests against mock server: Problem+JSON error format handling
  - [x] Pagination handling tests against mock server: page/perPage query params and response metadata
  - [x] CLI flag parsing tests: `--remote-server` flag validation and config integration
  - [x] Basic UI rendering tests: Ratatui widget initialization and layout

- **Verification** (Automated Unit + Integration):

  - [x] Unit tests for REST API contracts crate covering schema validation
  - [x] Unit tests for REST client crate covering all API endpoints against mock server
  - [x] Integration tests against mock server for end-to-end API flows and error scenarios
  - [x] CLI tests verifying `aw tui --remote-server` command parsing
  - [x] Build tests: All crates compile successfully with Cargo
  - [x] Tooling tests: Clippy and rustfmt configurations work across all crates

- **Implementation Details**:

  - **Architecture**: Clean separation between API contracts, client, and UI layers following Repository-Layout.md guidelines
  - **REST API Contracts**: Complete type definitions for all REST-Service.md endpoints with serde serialization and validator-based input validation
  - **REST Client**: Full async HTTP client with reqwest, supporting authentication (API key, JWT), error handling, and SSE streaming (placeholder)
  - **TUI Framework**: Ratatui + Crossterm with event-driven architecture, basic dashboard layout, and keyboard navigation
  - **CLI Integration**: `aw tui --remote-server` command with authentication options integrated into existing aw-cli structure
  - **Mock Server Ready**: Client configured to work with mock server at `http://localhost:3001` as specified

- **Key Source Files**:

  - `crates/aw-rest-api-contract/src/types.rs` - Complete API schema definitions
  - `crates/aw-rest-api-contract/src/validation.rs` - Input validation logic
  - `crates/aw-rest-client/src/client.rs` - HTTP client implementation
  - `crates/aw-rest-client/src/auth.rs` - Authentication handling
  - `crates/aw-tui/src/app.rs` - Main TUI application logic
  - `crates/aw-tui/src/ui.rs` - UI components and rendering
  - `crates/aw-cli/src/tui.rs` - CLI command integration

- **Integration Points**:

  - REST client can be used by WebUI components for consistent API access
  - TUI ready for mock server integration (`just webui-mock-server`)
  - CLI follows existing patterns for seamless user experience

**T2. Core Dashboard Layout** ✅ **COMPLETED** (September 26, 2025)

- **Deliverables**:

  - ✅ Three-section dashboard layout (selectors + description editor) with MVVM architecture
  - ✅ Fixed-height list widgets for Project, Branch, and Agent selectors with filtering
  - ✅ Resizable multiline task description editor with keyboard navigation
  - ✅ Full keyboard navigation between sections (Tab/Shift+Tab, arrow keys)
  - ✅ REST data loading and display with loading states and error handling
  - ✅ Dynamic footer with contextual shortcuts based on current focus
  - ✅ Complete message-driven state machine (keyboard, time, network events)

- **Test Coverage** (Comprehensive Integration + UI + Testing Framework):

  - [x] Layout rendering tests: Dashboard renders correctly on different terminal sizes (80x24, 120x40)
  - [x] Keyboard navigation tests: Tab cycling and arrow key navigation work
  - [x] Selector interaction tests: List filtering and selection functionality
  - [x] Editor resizing tests: Ctrl+Up/Down resize operations
  - [x] REST data integration tests: API responses populate selectors correctly
  - [x] Footer shortcut tests: Context-sensitive shortcuts display appropriately
  - [x] MVVM architecture tests: ViewModel correctly derives state from Model
  - [x] State machine tests: Message handling and state transitions work correctly

- **Verification** (Automated E2E + Manual):

  - ✅ Playwright-style terminal automation tests for UI interactions via test scenarios
  - ✅ Manual verification: Layout adapts to terminal width/height changes
  - ✅ Manual verification: Keyboard shortcuts work as specified in TUI-PRD.md
  - ✅ Interactive scenario player for manual testing and debugging

- **Implementation Details**:

  - **MVVM Architecture**: Clean separation between Model (domain logic), ViewModel (presentation logic), and View (Ratatui rendering)
  - **Message System**: Typed messages (Key, Tick, Net) drive deterministic state transitions
  - **State Machine**: Complete state management with keyboard input, time events, and REST responses
  - **Testing Framework**: Comprehensive scenario-based testing with interactive and automated runners
  - **Mock Integration**: Full mock REST client for isolated development and testing

- **Key Source Files**:

  - `crates/aw-tui/src/model.rs` - State machine and business logic
  - `crates/aw-tui/src/viewmodel.rs` - Presentation state derivation
  - `crates/aw-tui/src/msg.rs` - Message types for state transitions
  - `crates/aw-tui/src/test_runtime.rs` - Deterministic test execution engine
  - `crates/aw-tui-test/src/main.rs` - Interactive and automated test runners
  - `test_scenarios/basic_navigation.json` - Sample test scenario

- **Integration Points**:

  - Testing framework can be extended for all future TUI features
  - MVVM architecture ready for real-time updates and session monitoring
  - REST client integration tested and ready for production use

**Testing Framework Implementation** ✅ **COMPLETED** (September 26, 2025)

- **Deliverables**:

  - ✅ Complete MVVM testing architecture as specified in TUI-Testing-Architecture.md
  - ✅ Scenario-driven mock REST client with configurable responses
  - ✅ Interactive scenario player (`tui-test play`) with step navigation and state inspection
  - ✅ Automated scenario runner (`tui-test run`) with deterministic execution
  - ✅ ViewModel assertion system for testing presentation state
  - ✅ Golden file infrastructure with file-based comparison, diff reporting, and multi-line TUI visualization
  - ✅ Fake time integration for deterministic scenario execution
  - ✅ Sample scenarios with golden files demonstrating navigation and interaction testing

- **Test Coverage** (Comprehensive Testing Infrastructure):

  - [x] Scenario execution tests: Basic navigation scenario runs successfully
  - [x] Interactive player tests: Step navigation, state inspection, and replay work
  - [x] Mock client tests: Configurable REST responses for different scenarios
  - [x] ViewModel assertion tests: State inspection and validation work correctly
  - [x] Time control tests: Fake time enables deterministic execution
  - [x] MVVM integration tests: All layers work together correctly
  - [x] Golden file tests: Framework saves, loads, and compares golden files correctly

- **Verification** (Automated Testing + Manual):

  - ✅ Scenario files execute without errors and produce expected state changes
  - ✅ Interactive player allows full navigation and inspection of TUI state
  - ✅ Mock client provides realistic test data for development
  - ✅ Testing framework integrates with existing TUI codebase
  - ✅ Golden files validate UI rendering with human-readable multi-line format and provide regression protection
  - ✅ Snapshot comparison with detailed diff output for debugging

- **Implementation Details**:

  - **State Machine Testing**: Deterministic execution with controlled message injection
  - **Scenario Format**: JSON-based scenarios compatible with future WebUI testing (same format as mock-api-server)
  - **Interactive Debugging**: Rich interface for stepping through and inspecting scenarios
  - **Mock-First Development**: Enables testing without external dependencies
  - **Golden Files**: File-based golden file comparison with stable normalization, diff reporting, and multi-line TUI visualization showing actual terminal rendering
  - **Golden File Storage**: Organized under `crates/aw-tui/tests/__goldens__/<scenario>/<step>.golden`

- **Key Source Files**:

  - `crates/aw-test-scenarios/src/lib.rs` - Scenario model and JSON parsing
  - `crates/aw-rest-client-mock/src/lib.rs` - Mock REST client implementation
  - `crates/aw-client-api/src/lib.rs` - Client API trait for testing
  - `crates/aw-tui-test/src/main.rs` - Interactive and automated test runners
  - `crates/aw-tui/src/test_runtime.rs` - Test runtime with full golden file integration
  - `crates/aw-tui/src/golden.rs` - Golden file management with diff support
  - `specs/Public/TUI-Testing-Architecture.md` - Complete testing framework specification

- **Integration Points**:

  - Testing framework works with existing TUI MVVM architecture
  - Mock client uses same API as production REST client
  - Scenarios can be extended for future TUI features (SSE, multiplexer integration)
  - Golden files provide UI regression protection across terminal environments

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

**T3. Task Creation and Launch** is the current priority milestone, building on the completed T2 dashboard and testing infrastructure. This milestone focuses on connecting the user interface to actual task creation workflows, including multiplexer integration for launching tasks into terminal sessions with proper pane layouts. The comprehensive testing framework ensures reliability as we add production functionality.

### Current Outstanding Tasks

Here are the key tasks for TUI development:

#### **T2. Core Dashboard Layout** ✅ **COMPLETED**

- [x] Full dashboard layout with selectors and editor (MVVM architecture)
- [x] Keyboard navigation and shortcut handling (message-driven state machine)
- [x] REST data loading and display in selectors (with loading states)
- [x] Dynamic footer with contextual shortcuts (focus-based)
- [x] Layout rendering tests: Dashboard renders correctly on different terminal sizes

#### **Testing Framework Implementation** ✅ **COMPLETED**

- [x] Complete MVVM testing architecture (TUI-Testing-Architecture.md)
- [x] Interactive scenario player (`tui-test play`) with navigation and inspection
- [x] Automated scenario runner (`tui-test run`) with deterministic execution
- [x] Mock REST client for scenario-driven testing
- [x] ViewModel assertion system and state inspection
- [x] Sample test scenarios demonstrating functionality
- [x] Golden snapshot infrastructure with file-based comparison

#### **REST Client Infrastructure** 📋 **COMPLETED**

- [x] `aw-rest-api-contract` crate with schema types and validation
- [x] `aw-rest-client` crate with full REST-Service.md coverage
- [x] SSE streaming support for real-time updates (placeholder)
- [x] Authentication handling (API key, JWT, OIDC)
- [x] Error response parsing and user-friendly error messages
- [x] Comprehensive unit and integration test coverage
- [x] Mock server integration for isolated development

#### **TUI Foundation** 📋 **COMPLETED**

- [x] Ratatui + Crossterm application scaffolding
- [x] Basic event loop and terminal handling
- [x] CLI command integration with `--remote-server` flag
- [x] Project structure following Repository-Layout.md
- [x] Basic UI widget initialization

#### **REST Client Stubs** 📋 **PENDING**

- [ ] SSE streaming tests against mock server: EventSource connection and event parsing (placeholder implemented)
- [ ] SSE streaming implementation: Replace placeholder in `aw-rest-client/src/sse.rs` with proper eventsource-client integration

#### **Integration & Advanced Features** 📋 **PENDING**

- [ ] Task creation workflow from dashboard input
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
