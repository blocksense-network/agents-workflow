import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import { sessionsRouter } from './routes/sessions.js';
import { agentsRouter } from './routes/agents.js';
import { runtimesRouter } from './routes/runtimes.js';
import { executorsRouter } from './routes/executors.js';
import { tasksRouter } from './routes/tasks.js';

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(helmet());
app.use(
  cors({
    origin: process.env.NODE_ENV === 'production' ? false : true,
    credentials: true,
  })
);
app.use(morgan('combined'));
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// API routes
app.use('/api/v1/sessions', sessionsRouter);
app.use('/api/v1/agents', agentsRouter);
app.use('/api/v1/runtimes', runtimesRouter);
app.use('/api/v1/executors', executorsRouter);
app.use('/api/v1/tasks', tasksRouter);

// Mock capability discovery endpoints
app.get('/api/v1/agents', (req, res) => {
  res.json({
    items: [
      {
        type: 'claude-code',
        versions: ['latest'],
        settingsSchemaRef: '/api/v1/schemas/agents/claude-code.json',
      },
      {
        type: 'openhands',
        versions: ['latest'],
        settingsSchemaRef: '/api/v1/schemas/agents/openhands.json',
      },
    ],
  });
});

app.get('/api/v1/runtimes', (req, res) => {
  res.json({
    items: [
      {
        type: 'devcontainer',
        images: ['ghcr.io/acme/base:latest'],
        paths: ['.devcontainer/devcontainer.json'],
      },
      {
        type: 'local',
        sandboxProfiles: ['default', 'disabled'],
      },
    ],
  });
});

app.get('/api/v1/executors', (req, res) => {
  res.json({
    items: [
      {
        id: 'executor-linux-01',
        os: 'linux',
        arch: 'x86_64',
        snapshotCapabilities: ['zfs', 'btrfs', 'overlay', 'copy'],
        status: 'online',
      },
    ],
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    type: 'https://docs.example.com/errors/not-found',
    title: 'Not Found',
    status: 404,
    detail: `Route ${req.originalUrl} not found`,
  });
});

// Error handler
app.use((err: any, req: express.Request, res: express.Response, _next: express.NextFunction) => {
  console.error(err.stack);
  res.status(500).json({
    type: 'https://docs.example.com/errors/internal-server-error',
    title: 'Internal Server Error',
    status: 500,
    detail: 'An unexpected error occurred',
  });
});

app.listen(PORT, () => {
  console.log(`Mock API server running on http://localhost:${PORT}`);
  console.log(`Health check: http://localhost:${PORT}/health`);
});
