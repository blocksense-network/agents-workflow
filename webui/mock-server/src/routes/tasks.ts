import express from 'express';

const router = express.Router();

// Mock session storage (shared with sessions route)
let mockSessions: any[] = [];

// POST /api/v1/tasks - Create Task/Session
router.post('/', (req, res) => {
  // Generate a unique session ID
  const sessionId = `01HVZ6K9T1N8S6M3V3Q3F0X${Math.random().toString(36).substr(2, 9).toUpperCase()}`;

  // Create session from task data
  const session = {
    id: sessionId,
    tenantId: req.body.tenantId || 'default',
    projectId: req.body.projectId || 'default',
    status: 'running', // Start as running for testing
    createdAt: new Date().toISOString(),
    prompt: req.body.prompt,
    repo: req.body.repo,
    runtime: req.body.runtime,
    workspace: req.body.workspace,
    agent: req.body.agent,
    delivery: req.body.delivery,
    labels: req.body.labels || {},
    webhooks: req.body.webhooks || [],
    links: {
      self: `/api/v1/sessions/${sessionId}`,
      events: `/api/v1/sessions/${sessionId}/events`,
      logs: `/api/v1/sessions/${sessionId}/logs`
    }
  };

  // Add to mock storage
  mockSessions.push(session);

  // Return response as per REST spec
  res.status(201).json({
    id: session.id,
    status: session.status,
    links: session.links
  });
});

// Export the mock sessions for use in sessions route
export { mockSessions };
export const tasksRouter = router;

