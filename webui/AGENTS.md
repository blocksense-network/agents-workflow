# WebUI Development Guide

## Running Tests

### Local Development Testing

**WebUI App:**
```bash
cd webui/app
npm run lint          # ESLint code quality checks
npm run format        # Prettier code formatting
npm run type-check    # TypeScript type checking
npm run build         # Production build verification
npm run dev           # Start development server (http://localhost:3000)
```

**Mock Server:**
```bash
cd webui/mock-server
npm run lint          # ESLint code quality checks
npm run format        # Prettier code formatting
npm run type-check    # TypeScript type checking
npm run build         # Production build verification
npm run dev           # Start mock API server (http://localhost:3001)
```

**E2E Tests:**
```bash
cd webui/e2e-tests
npm run lint                    # ESLint code quality checks
npm run format                  # Prettier code formatting
npm run install-browsers        # Install Playwright browsers
npm test                        # Run all E2E tests
npm run test:headed             # Run tests in headed mode (visible browser)
npm run test:debug              # Debug tests step-by-step
npm run test:ui                 # Interactive test runner UI
npm run report                  # View test reports after runs
```

### Full WebUI Test Suite

Run all WebUI components together for integration testing:

```bash
# Terminal 1: Start mock server
cd webui/mock-server && npm run dev

# Terminal 2: Start WebUI app
cd webui/app && npm run dev

# Terminal 3: Run E2E tests
cd webui/e2e-tests && npm test
```

### Repository-wide Testing

Use the project's just targets for comprehensive testing:

```bash
just test              # Run all Rust tests
just lint-specs        # Lint markdown files
```

## Development Workflow

1. **Start development servers:**
   ```bash
   # Terminal 1: Mock API server
   cd webui/mock-server && npm run dev

   # Terminal 2: WebUI app
   cd webui/app && npm run dev
   ```

2. **Run tests continuously:**
   ```bash
   # Terminal 3: E2E tests
   cd webui/e2e-tests && npm test
   ```

3. **Code quality checks:**
   ```bash
   # Lint all WebUI projects
   cd webui/app && npm run lint
   cd ../mock-server && npm run lint
   cd ../e2e-tests && npm run lint
   ```

## Architecture

The WebUI consists of three main components:

- **`webui/app/`**: SolidJS + Tailwind CSS frontend application
- **`webui/mock-server/`**: Express.js mock REST API server
- **`webui/e2e-tests/`**: Playwright end-to-end test suite

### Data Flow

```
Browser → WebUI App → Mock Server → REST API Responses
                    ↓
            Playwright Tests
```

### Technology Stack

- **Frontend**: SolidJS, TypeScript, Tailwind CSS, Vite
- **Backend**: Node.js, Express, TypeScript
- **Testing**: Playwright, ESLint, Prettier
- **Build**: Vite (frontend), TypeScript compiler (backend)

## Contributing

1. Follow the established patterns in the codebase
2. Write tests for new features
3. Ensure all linting passes
4. Test across different browsers when making UI changes
5. Update this guide when adding new development workflows
