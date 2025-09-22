import { test, expect } from '@playwright/test';

test('basic test - app loads', async ({ page }) => {
  // Navigate to the app
  await page.goto('/');

  // Check that the page loaded
  await expect(page).toHaveTitle(/Welcome/);

  // Check for basic content
  await expect(page.locator('h1')).toContainText('Server rendered heading');
});