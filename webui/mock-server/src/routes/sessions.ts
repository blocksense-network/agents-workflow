import express from 'express';
import { mockSessions } from './tasks.js';

const router = express.Router();

// Add a default session for testing
if (mockSessions.length === 0) {
  mockSessions.push({
    id: '01HVZ6K9T1N8S6M3V3Q3F0X5B7',
    tenantId: 'acme',
    projectId: 'storefront',
    status: 'running',
    createdAt: '2025-01-01T12:00:00Z',
    prompt: 'Default test session',
    repo: { mode: 'git', url: 'https://github.com/test/repo.git', branch: 'main' },
    agent: { type: 'claude-code', version: 'latest' },
    runtime: { type: 'devcontainer' },
    links: {
      self: '/api/v1/sessions/01HVZ6K9T1N8S6M3V3Q3F0X5B7',
      events: '/api/v1/sessions/01HVZ6K9T1N8S6M3V3Q3F0X5B7/events',
      logs: '/api/v1/sessions/01HVZ6K9T1N8S6M3V3Q3F0X5B7/logs'
    }
  });
}

// Sessions creation is now handled by POST /api/v1/tasks

// GET /api/v1/sessions - List Sessions
router.get('/', (req, res) => {
  const { status, projectId, page = 1, perPage = 20 } = req.query;

  let filteredSessions = mockSessions;

  if (status) {
    filteredSessions = filteredSessions.filter(s => s.status === status);
  }

  if (projectId) {
    filteredSessions = filteredSessions.filter(s => s.projectId === projectId);
  }

  const start = (Number(page) - 1) * Number(perPage);
  const end = start + Number(perPage);
  const paginatedSessions = filteredSessions.slice(start, end);

  res.json({
    items: paginatedSessions,
    pagination: {
      page: Number(page),
      perPage: Number(perPage),
      total: filteredSessions.length,
      totalPages: Math.ceil(filteredSessions.length / Number(perPage))
    }
  });
});

// GET /api/v1/sessions/:id - Get Session
router.get('/:id', (req, res) => {
  const session = mockSessions.find(s => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`
    });
  }

  res.json(session);
});

// POST /api/v1/sessions/:id/stop - Stop Session
router.post('/:id/stop', (req, res) => {
  const session = mockSessions.find(s => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`
    });
  }

  session.status = 'stopping';
  res.json({ status: 'accepted' });
});

// DELETE /api/v1/sessions/:id - Cancel Session
router.delete('/:id', (req, res) => {
  const index = mockSessions.findIndex(s => s.id === req.params.id);

  if (index === -1) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`
    });
  }

  mockSessions.splice(index, 1);
  res.status(204).send();
});

// GET /api/v1/sessions/:id/events - SSE Events
router.get('/:id/events', (req, res) => {
  // For testing purposes, return immediately with proper headers
  // In a real implementation, this would keep the connection open
  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Headers', 'Cache-Control');

  // Send initial status event and close immediately for testing
  const data = `data: ${JSON.stringify({
    type: 'status',
    status: 'running',
    ts: new Date().toISOString()
  })}\n\n`;

  res.end(data);
});

// GET /api/v1/sessions/:id/logs - Get Logs
router.get('/:id/logs', (req, res) => {
  const session = mockSessions.find(s => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`
    });
  }

  // Mock logs
  const logs = [
    { level: 'info', message: 'Session started', ts: new Date(Date.now() - 300000).toISOString() },
    { level: 'info', message: 'Agent initialized', ts: new Date(Date.now() - 250000).toISOString() },
    { level: 'info', message: 'Task execution in progress', ts: new Date(Date.now() - 200000).toISOString() }
  ];

  res.json({ logs });
});

export { router as sessionsRouter };