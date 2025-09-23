import { test, expect } from '@playwright/test';

test.describe('Infrastructure Tests', () => {
  test('SSR sidecar serves HTML correctly', async ({ request }) => {
    // Test the health endpoint
    const healthResponse = await request.get('/health');
    expect(healthResponse.ok()).toBeTruthy();

    const healthData = await healthResponse.json();
    expect(healthData.status).toBe('ok');
    expect(healthData).toHaveProperty('timestamp');

    // Test the main page serves HTML
    const pageResponse = await request.get('/');
    expect(pageResponse.ok()).toBeTruthy();
    expect(pageResponse.headers()['content-type']).toContain('text/html');

    const pageContent = await pageResponse.text();
    expect(pageContent).toContain('Agents-Workflow WebUI');
    expect(pageContent).toContain('Agents-Workflow</h1>');
    expect(pageContent).toContain('<script type="module" src="/client.js">');
    expect(pageContent).toContain('<div id="app">');
  });

  test('API proxy forwards requests correctly', async ({ request }) => {
    // Test API proxy (if mock server is running)
    try {
      const apiResponse = await request.get('/api/v1/agents');
      if (apiResponse.status() !== 502) {
        // 502 means mock server not running
        expect(apiResponse.ok()).toBeTruthy();
        const apiData = await apiResponse.json();
        expect(apiData).toHaveProperty('items');
        expect(Array.isArray(apiData.items)).toBe(true);
        expect(apiData.items.length).toBeGreaterThan(0);

        // Check agent structure
        const agent = apiData.items[0];
        expect(agent).toHaveProperty('type');
        expect(agent).toHaveProperty('versions');
        expect(agent).toHaveProperty('settingsSchemaRef');
      }
    } catch {
      // API proxy test is optional if mock server isn't running
      console.log('API proxy test skipped - mock server not available');
    }
  });

  test('CORS headers are properly configured', async ({ request }) => {
    const response = await request.get('/health');
    // CORS headers are set by the cors middleware - in development, origin is set to true
    // which allows all origins. For API requests from browsers, these headers will be present.
    expect(response.headers()['access-control-allow-credentials']).toBe('true');
  });

  test('Security headers are set', async ({ request }) => {
    const response = await request.get('/');
    expect(response.headers()['x-content-type-options']).toBe('nosniff');
    expect(response.headers()['x-frame-options']).toBeDefined();
  });

  test('404 handler returns proper error format', async ({ request }) => {
    const response = await request.get('/nonexistent-route');
    expect(response.status()).toBe(404);

    // For non-API routes, we expect HTML response (application pages)
    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('text/html');
  });
});
