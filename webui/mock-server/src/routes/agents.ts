import express from 'express';

const router = express.Router();

// GET /api/v1/agents - List supported agent types
router.get('/', (req, res) => {
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

export { router as agentsRouter };
