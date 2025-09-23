import express from 'express';
import { scenarioReplayer } from './sessions.js';

const router = express.Router();

// Mock session storage (shared with sessions route)
let mockSessions: any[] = [];

// POST /api/v1/tasks - Create Task/Session
router.post('/', (req, res) => {
  try {
    // Basic validation
    if (!req.body || typeof req.body !== 'object') {
      return res.status(400).json({
        type: 'https://docs.example.com/errors/bad-request',
        title: 'Bad Request',
        status: 400,
        detail: 'Request body must be a valid JSON object',
      });
    }

    // Check required fields
    const requiredFields = ['prompt', 'repo', 'agent', 'runtime'];
    const missingFields = requiredFields.filter((field) => !req.body[field]);

    if (missingFields.length > 0) {
      return res.status(400).json({
        type: 'https://docs.example.com/errors/validation-error',
        title: 'Validation Error',
        status: 400,
        detail: `Missing required fields: ${missingFields.join(', ')}`,
        errors: missingFields.map((field) => ({
          field,
          message: `${field} is required`,
        })),
      });
    }

    // Generate a unique session ID
    const sessionId = `01HVZ6K9T1N8S6M3V3Q3F0X${Math.random().toString(36).substr(2, 9).toUpperCase()}`;

    // Create session from task data
    const session: any = {
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
        logs: `/api/v1/sessions/${sessionId}/logs`,
      },
      metadata: {},
    };

    // Add to mock storage
    mockSessions.push(session);

    // Assign a scenario to this session for realistic event streaming
    // Use test_scenario for prompts containing "testing" to enable fast E2E tests
    // For manual testing, cycle through 5 main scenarios
    let assignedScenario;
    if (req.body.prompt && req.body.prompt.toLowerCase().includes('testing')) {
      assignedScenario = scenarioReplayer.assignSpecificScenario(session.id, 'test_scenario');
    } else {
      assignedScenario = scenarioReplayer.assignNextManualScenario(session.id);
    }

    // Add scenario metadata for testing purposes
    if (assignedScenario) {
      session.metadata = session.metadata || {};
      session.metadata.scenario = assignedScenario;
    }

    // Return response as per REST spec
    res.status(201).json({
      id: session.id,
      status: session.status,
      links: session.links,
    });
  } catch (error) {
    console.error('Error in POST /api/v1/tasks:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Export the mock sessions for use in sessions route
export { mockSessions };
export const tasksRouter = router;
