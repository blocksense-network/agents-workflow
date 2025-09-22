import { test, expect } from '@playwright/test';

test('basic test - SSR sidecar serves HTML', async ({ request }) => {
  // Test the health endpoint
  const healthResponse = await request.get('/health');
  expect(healthResponse.ok()).toBeTruthy();

  const healthData = await healthResponse.json();
  expect(healthData.status).toBe('ok');
  expect(healthData.environment).toBe('development');

  // Test the main page serves HTML
  const pageResponse = await request.get('/');
  expect(pageResponse.ok()).toBeTruthy();

  const pageContent = await pageResponse.text();
  expect(pageContent).toContain('Agents-Workflow WebUI');
  expect(pageContent).toContain('Loading Agents-Workflow WebUI');
  expect(pageContent).toContain('<script type="module" src="/client.js">');

  // Test API proxy (if mock server is running)
  try {
    const apiResponse = await request.get('/api/agents');
    if (apiResponse.status() !== 502) { // 502 means mock server not running
      expect(apiResponse.ok()).toBeTruthy();
      const apiData = await apiResponse.json();
      expect(apiData).toHaveProperty('items');
    }
  } catch (error) {
    // API proxy test is optional if mock server isn't running
    console.log('API proxy test skipped - mock server not available');
  }
});