import express from 'express';
import { mockSessions } from './tasks.js';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { setInterval, clearInterval } from 'timers';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const router = express.Router();

interface ScenarioTurn {
  user?: string;
  think?: string;
  tool?: {
    name: string;
    args: Record<string, any>;
  };
  assistant?: string;
  shell?: {
    cmd: string;
  };
}

interface ScenarioMeta {
  instructions?: string;
  turn_context?: Record<string, any>;
}

interface Scenario {
  description?: string;
  meta?: ScenarioMeta;
  turns: ScenarioTurn[];
}

interface ScenarioEvent {
  type: 'status' | 'log' | 'progress';
  sessionId: string;
  status?: string;
  level?: string;
  message?: string;
  progress?: number;
  stage?: string;
  ts: string;
}

class ScenarioReplayer {
  private scenarios: Map<string, Scenario> = new Map();
  private sessionScenarios: Map<string, string> = new Map();

  constructor() {
    this.loadScenarios();
  }

  private loadScenarios() {
    const scenarioDirs = [
      path.join(__dirname, '../../../tests/tools/mock-agent/examples'),
      path.join(__dirname, '../../../tests/tools/mock-agent/scenarios'),
    ];

    for (const scenarioDir of scenarioDirs) {
      if (fs.existsSync(scenarioDir)) {
        const files = fs.readdirSync(scenarioDir).filter((f) => f.endsWith('.json'));
        for (const file of files) {
          try {
            const filePath = path.join(scenarioDir, file);
            const content = fs.readFileSync(filePath, 'utf-8');
            const scenario: Scenario = JSON.parse(content);
            const scenarioName = path.basename(file, '.json');
            this.scenarios.set(scenarioName, scenario);
            console.log(`Loaded scenario: ${scenarioName}`);
          } catch (error) {
            console.warn(`Failed to load scenario ${file}:`, error);
          }
        }
      }
    }
  }

  assignScenarioToSession(sessionId: string): string | null {
    const scenarioNames = Array.from(this.scenarios.keys());
    if (scenarioNames.length > 0) {
      const randomScenario = scenarioNames[Math.floor(Math.random() * scenarioNames.length)];
      this.sessionScenarios.set(sessionId, randomScenario);
      console.log(`Assigned scenario '${randomScenario}' to session ${sessionId}`);
      return randomScenario;
    }
    return null;
  }

  assignSpecificScenario(sessionId: string, scenarioName: string): string | null {
    if (this.scenarios.has(scenarioName)) {
      this.sessionScenarios.set(sessionId, scenarioName);
      console.log(`Assigned specific scenario '${scenarioName}' to session ${sessionId}`);
      return scenarioName;
    }
    console.log(`Scenario '${scenarioName}' not found, falling back to random assignment`);
    return this.assignScenarioToSession(sessionId);
  }

  private manualScenarioIndex = 0;
  private readonly manualScenarios = [
    'bug_fix_scenario',
    'code_refactoring_scenario',
    'documentation_scenario',
    'feature_implementation_scenario',
    'testing_workflow_scenario',
  ];

  assignNextManualScenario(sessionId: string): string | null {
    const scenarioName = this.manualScenarios[this.manualScenarioIndex];
    this.manualScenarioIndex = (this.manualScenarioIndex + 1) % this.manualScenarios.length;

    if (this.scenarios.has(scenarioName)) {
      this.sessionScenarios.set(sessionId, scenarioName);
      console.log(
        `Assigned manual scenario '${scenarioName}' to session ${sessionId} (next: ${this.manualScenarios[this.manualScenarioIndex]})`
      );
      return scenarioName;
    }
    console.log(`Manual scenario '${scenarioName}' not found, falling back to random assignment`);
    return this.assignScenarioToSession(sessionId);
  }

  getScenarioForSession(sessionId: string): Scenario | null {
    const scenarioName = this.sessionScenarios.get(sessionId);
    return scenarioName ? this.scenarios.get(scenarioName) || null : null;
  }

  replayScenario(sessionId: string, eventIndex: number): ScenarioEvent | null {
    const scenario = this.getScenarioForSession(sessionId);
    if (!scenario || !scenario.turns) {
      return null;
    }

    const turnIndex = Math.floor(eventIndex / 2); // Map events to turns (2 events per turn)
    if (turnIndex >= scenario.turns.length) {
      return null;
    }

    const turn = scenario.turns[turnIndex];
    const eventOffset = eventIndex % 2;

    // Convert scenario turn to events
    if (eventOffset === 0) {
      // First event: log the user/assistant message
      if (turn.user) {
        return {
          type: 'log',
          sessionId,
          level: 'info',
          message: `User: ${turn.user}`,
          ts: new Date().toISOString(),
        };
      } else if (turn.assistant) {
        return {
          type: 'log',
          sessionId,
          level: 'info',
          message: `Assistant: ${turn.assistant}`,
          ts: new Date().toISOString(),
        };
      } else if (turn.think) {
        return {
          type: 'log',
          sessionId,
          level: 'info',
          message: `Thinking: ${turn.think}`,
          ts: new Date().toISOString(),
        };
      }
    } else {
      // Second event: tool execution or status update
      if (turn.tool) {
        return {
          type: 'log',
          sessionId,
          level: 'info',
          message: `Tool: ${turn.tool.name}(${JSON.stringify(turn.tool.args)})`,
          ts: new Date().toISOString(),
        };
      } else {
        // Status/progress update
        return {
          type: 'status',
          sessionId,
          status: turnIndex === scenario.turns.length - 1 ? 'completed' : 'running',
          ts: new Date().toISOString(),
        };
      }
    }

    return null;
  }

  getAvailableScenarios(): string[] {
    return Array.from(this.scenarios.keys());
  }
}

// Global scenario replayer instance
const scenarioReplayer = new ScenarioReplayer();

// Add a default session for testing
if (mockSessions.length === 0) {
  // Create default session
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
      logs: '/api/v1/sessions/01HVZ6K9T1N8S6M3V3Q3F0X5B7/logs',
    },
  });

  // Assign a scenario to the default session too
  const assignedScenario = scenarioReplayer.assignScenarioToSession('01HVZ6K9T1N8S6M3V3Q3F0X5B7');
  if (assignedScenario) {
    const defaultSession = mockSessions.find((s) => s.id === '01HVZ6K9T1N8S6M3V3Q3F0X5B7');
    if (defaultSession) {
      defaultSession.metadata = { scenario: assignedScenario };
    }
  }
}

// Sessions creation is now handled by POST /api/v1/tasks

// GET /api/v1/sessions - List Sessions
router.get('/', (req, res) => {
  const { status, projectId, page = 1, perPage = 20 } = req.query;

  let filteredSessions = mockSessions;

  if (status) {
    filteredSessions = filteredSessions.filter((s) => s.status === status);
  }

  if (projectId) {
    filteredSessions = filteredSessions.filter((s) => s.projectId === projectId);
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
      totalPages: Math.ceil(filteredSessions.length / Number(perPage)),
    },
  });
});

// GET /api/v1/sessions/:id - Get Session
router.get('/:id', (req, res) => {
  const session = mockSessions.find((s) => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`,
    });
  }

  res.json(session);
});

// POST /api/v1/sessions/:id/stop - Stop Session
router.post('/:id/stop', (req, res) => {
  const session = mockSessions.find((s) => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`,
    });
  }

  session.status = 'stopping';
  res.json({ status: 'accepted' });
});

// DELETE /api/v1/sessions/:id - Cancel Session
router.delete('/:id', (req, res) => {
  const index = mockSessions.findIndex((s) => s.id === req.params.id);

  if (index === -1) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`,
    });
  }

  mockSessions.splice(index, 1);
  res.status(204).send();
});

// GET /api/v1/sessions/:id/events - SSE Events
router.get('/:id/events', (req, res) => {
  const session = mockSessions.find((s) => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`,
    });
  }

  // Check if this is a test request (no Accept header or specific test header)
  const isTestRequest =
    !req.headers.accept?.includes('text/event-stream') || req.headers['x-test-request'] === 'true';

  // Set SSE headers
  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Headers', 'Cache-Control');

  // Send initial heartbeat
  const sendEvent = (eventType: string, data: any) => {
    res.write(`event: ${eventType}\n`);
    res.write(`data: ${JSON.stringify(data)}\n\n`);
  };

  // Send initial status
  sendEvent('status', {
    sessionId: session.id,
    status: session.status,
    ts: new Date().toISOString(),
  });

  if (isTestRequest) {
    // For test requests, send one event and close immediately
    res.end();
    return;
  }

  // For real SSE clients, replay scenario events for realistic behavior
  let eventCount = 0;
  const maxEvents = 20;

  // Use faster interval (1 second) for test scenarios to speed up tests
  const scenario = scenarioReplayer.getScenarioForSession(session.id);
  const isTestScenario = scenario && scenario.description?.includes('test');
  const intervalMs = isTestScenario ? 1000 : 2000;

  const interval = setInterval(() => {
    if (eventCount >= maxEvents) {
      clearInterval(interval);
      res.end();
      return;
    }

    eventCount++;

    // Try to replay scenario event
    const scenarioEvent = scenarioReplayer.replayScenario(session.id, eventCount - 1);

    if (scenarioEvent) {
      // Send the scenario event
      sendEvent(scenarioEvent.type, scenarioEvent);

      // Update session status if it's a status event
      if (scenarioEvent.type === 'status' && scenarioEvent.status) {
        session.status = scenarioEvent.status;
      }
    } else {
      // Fallback to generic progress event if no scenario available
      sendEvent('progress', {
        sessionId: session.id,
        progress: Math.min(100, eventCount * 5),
        stage: 'processing',
        ts: new Date().toISOString(),
      });
    }
  }, intervalMs);

  // Handle client disconnect
  req.on('close', () => {
    clearInterval(interval);
    res.end();
  });
});

// GET /api/v1/sessions/:id/logs - Get Logs
router.get('/:id/logs', (req, res) => {
  const session = mockSessions.find((s) => s.id === req.params.id);

  if (!session) {
    return res.status(404).json({
      type: 'https://docs.example.com/errors/not-found',
      title: 'Session Not Found',
      status: 404,
      detail: `Session ${req.params.id} not found`,
    });
  }

  // Generate logs based on the session's assigned scenario
  const logs: Array<{ level: string; message: string; ts: string }> = [];
  const scenario = scenarioReplayer.getScenarioForSession(session.id);

  if (scenario && scenario.turns) {
    // Generate logs from scenario turns
    let timestamp = Date.now() - 300000; // Start 5 minutes ago
    logs.push({
      level: 'info',
      message: 'Session started',
      ts: new Date(timestamp).toISOString(),
    });

    timestamp += 50000; // 50 seconds later
    logs.push({
      level: 'info',
      message: 'Agent initialized',
      ts: new Date(timestamp).toISOString(),
    });

    // Add logs for each scenario turn
    for (let i = 0; i < scenario.turns.length; i++) {
      const turn = scenario.turns[i];
      timestamp += 30000; // 30 seconds between turns

      if (turn.user) {
        logs.push({
          level: 'info',
          message: `User: ${turn.user}`,
          ts: new Date(timestamp).toISOString(),
        });
      }

      if (turn.think) {
        timestamp += 10000; // 10 seconds for thinking
        logs.push({
          level: 'info',
          message: `Thinking: ${turn.think}`,
          ts: new Date(timestamp).toISOString(),
        });
      }

      if (turn.tool) {
        timestamp += 5000; // 5 seconds for tool execution
        logs.push({
          level: 'info',
          message: `Tool: ${turn.tool.name}(${JSON.stringify(turn.tool.args)})`,
          ts: new Date(timestamp).toISOString(),
        });
      }

      if (turn.assistant) {
        timestamp += 15000; // 15 seconds for response
        logs.push({
          level: 'info',
          message: `Assistant: ${turn.assistant}`,
          ts: new Date(timestamp).toISOString(),
        });
      }
    }
  } else {
    // Fallback logs if no scenario available
    logs.push(
      {
        level: 'info',
        message: 'Session started',
        ts: new Date(Date.now() - 300000).toISOString(),
      },
      {
        level: 'info',
        message: 'Agent initialized',
        ts: new Date(Date.now() - 250000).toISOString(),
      },
      {
        level: 'info',
        message: 'Task execution in progress',
        ts: new Date(Date.now() - 200000).toISOString(),
      }
    );
  }

  // Apply tail parameter if provided
  const tail = req.query.tail ? parseInt(req.query.tail as string) : null;
  const responseLogs = tail ? logs.slice(-tail) : logs;

  res.json({ logs: responseLogs });
});

export { router as sessionsRouter, scenarioReplayer };
