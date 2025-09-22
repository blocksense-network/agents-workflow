# Agents-Workflow WebUI

This is the SolidJS-based web interface for Agents-Workflow, providing a browser-based dashboard for creating, monitoring, and managing agent coding sessions.

## Architecture

- **Framework**: SolidJS with SolidStart (SSR-ready)
- **Styling**: Tailwind CSS for responsive, utility-first styling
- **Layout**: Three-pane dashboard (repositories, sessions, task details)
- **State**: Reactive state management with SolidJS signals
- **API**: REST client for communication with backend services

## Development

### Prerequisites

- Node.js 22+
- npm or yarn

### Getting Started

1. **Install dependencies:**

   ```bash
   just webui-install
   ```

2. **Start development server:**

   ```bash
   just webui-dev
   ```

   The app will be available at `http://localhost:3000`

### Available Commands

```bash
# Development
just webui-dev           # Start development server
just webui-build         # Build for production
just webui-lint          # Run ESLint
just webui-format        # Format code with Prettier
just webui-type-check    # Run TypeScript type checking

# Testing (requires mock server running)
just webui-test          # Run E2E tests
just webui-test-headed   # Run tests with visible browser
```

### Development Workflow

For full development setup with mock API:

```bash
# Terminal 1: Start mock API server
just webui-mock-server

# Terminal 2: Start WebUI development server
just webui-dev

# Terminal 3: Run tests (optional)
just webui-test
```

## Project Structure

```
src/
├── app.tsx              # Main app component with routing
├── components/
│   ├── layout/          # Layout components (MainLayout, ThreePaneLayout)
│   ├── repositories/    # Repository-related components
│   ├── sessions/        # Session management components
│   └── tasks/           # Task detail components
├── routes/              # File-based routing
└── global.d.ts          # Global type definitions
```

## Key Features

- **Three-Pane Layout**: Repositories list, chronological task feed, and live task details
- **Real-time Updates**: SSE integration for live session monitoring
- **Progressive Enhancement**: Core functionality works without JavaScript
- **Responsive Design**: Mobile-friendly interface with Tailwind CSS
- **Accessibility**: WCAG AA compliance with keyboard navigation

## Testing

This project uses Playwright for end-to-end testing. Tests are located in `../e2e-tests/` and can be run with:

```bash
just webui-test
```

Tests verify the complete user experience including:

- UI component rendering
- User interactions
- API integration
- Responsive behavior
- Accessibility compliance

## Building for Production

```bash
just webui-build
```

This generates optimized production assets ready for deployment.
