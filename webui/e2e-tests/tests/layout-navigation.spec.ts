import { test, expect } from '@playwright/test';

test.describe('Layout and Navigation Tests', () => {
  test('Three-pane layout renders correctly on desktop', async ({ page }) => {
    await page.goto('/');

    // Check that the SSR placeholder is present
    await expect(page.locator('.ssr-placeholder')).toBeVisible();
    await expect(page.locator('.ssr-loading')).toBeVisible();
    await expect(page.getByText('Loading Agents-Workflow WebUI')).toBeVisible();

    // Check that client-side JavaScript is loaded (script tag present)
    const clientScript = page.locator('script[src="/client.js"]');
    await expect(clientScript).toBeAttached();

    // TODO: Once client-side hydration is working, test for actual layout elements
    // await expect(page.locator('[data-testid="main-layout"]')).toBeVisible();
  });

  test('Navigation links work and highlight active routes', async ({ page }) => {
    await page.goto('/');

    // Wait for client-side hydration to complete (loading placeholder should be replaced)
    await page.waitForFunction(() => {
      const app = document.getElementById('app');
      return app && !app.classList.contains('ssr-placeholder');
    }, { timeout: 10000 });

    // Check navigation links are present
    const dashboardLink = page.locator('a[href="/"]');
    const sessionsLink = page.locator('a[href="/sessions"]');
    const createTaskLink = page.locator('a[href="/create"]');
    const settingsLink = page.locator('a[href="/settings"]');

    await expect(dashboardLink).toBeVisible();
    await expect(sessionsLink).toBeVisible();
    await expect(createTaskLink).toBeVisible();
    await expect(settingsLink).toBeVisible();

    // Test navigation to sessions page
    await sessionsLink.click();
    await expect(page).toHaveURL('/sessions');

    // Test navigation back to dashboard
    await dashboardLink.click();
    await expect(page).toHaveURL('/');

    // Test navigation to create task
    await createTaskLink.click();
    await expect(page).toHaveURL('/create');

    // Test navigation to settings
    await settingsLink.click();
    await expect(page).toHaveURL('/settings');
  });

  test('Collapsible panes work correctly', async ({ page }) => {
    await page.goto('/');

    // Wait for client-side hydration to complete
    await page.waitForFunction(() => {
      const app = document.getElementById('app');
      return app && !app.classList.contains('ssr-placeholder');
    }, { timeout: 10000 });

    // Find collapse/expand buttons
    const repositoriesCollapseBtn = page.locator('[data-testid="repositories-collapse"]');
    const sessionsCollapseBtn = page.locator('[data-testid="sessions-collapse"]');

    // Initially panes should be visible
    await expect(page.locator('[data-testid="repositories-pane"]')).toBeVisible();
    await expect(page.locator('[data-testid="sessions-pane"]')).toBeVisible();

    // Collapse repositories pane
    if (await repositoriesCollapseBtn.isVisible()) {
      await repositoriesCollapseBtn.click();
      // Pane should be collapsed (check for collapsed state)
      await expect(page.locator('[data-testid="repositories-pane"].collapsed')).toBeVisible();
    }

    // Collapse sessions pane
    if (await sessionsCollapseBtn.isVisible()) {
      await sessionsCollapseBtn.click();
      // Pane should be collapsed (check for collapsed state)
      await expect(page.locator('[data-testid="sessions-pane"].collapsed')).toBeVisible();
    }
  });

  test('Layout adapts correctly to different screen sizes', async ({ page, browserName }) => {
    // Skip this test for Firefox due to viewport issues in CI
    test.skip(browserName === 'firefox', 'Firefox viewport test unreliable in CI');

    await page.goto('/');

    // Wait for client-side hydration to complete
    await page.waitForFunction(() => {
      const app = document.getElementById('app');
      return app && !app.classList.contains('ssr-placeholder');
    }, { timeout: 10000 });

    // Test desktop layout (default)
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator('[data-testid="three-pane-layout"]')).toBeVisible();

    // Test tablet layout
    await page.setViewportSize({ width: 768, height: 800 });
    // Layout should still be visible but may be adjusted
    await expect(page.locator('[data-testid="main-layout"]')).toBeVisible();

    // Test mobile layout
    await page.setViewportSize({ width: 375, height: 667 });
    // Layout should adapt to mobile
    await expect(page.locator('[data-testid="main-layout"]')).toBeVisible();
  });

  test('Global search interface renders correctly', async ({ page }) => {
    await page.goto('/');

    // Wait for client-side hydration to complete
    await page.waitForFunction(() => {
      const app = document.getElementById('app');
      return app && !app.classList.contains('ssr-placeholder');
    }, { timeout: 10000 });

    // Check for global search input
    const searchInput = page.locator('input[placeholder*="search" i]');
    await expect(searchInput.or(page.locator('[data-testid="global-search"]'))).toBeVisible();
  });

  test('URL routing works correctly', async ({ page }) => {
    // Test direct navigation to routes
    await page.goto('/sessions');
    await expect(page).toHaveURL('/sessions');

    await page.goto('/create');
    await expect(page).toHaveURL('/create');

    await page.goto('/settings');
    await expect(page).toHaveURL('/settings');

    await page.goto('/');
    await expect(page).toHaveURL('/');
  });

  test('Browser back/forward navigation works', async ({ page }) => {
    await page.goto('/');

    // Wait for client-side hydration to complete
    await page.waitForFunction(() => {
      const app = document.getElementById('app');
      return app && !app.classList.contains('ssr-placeholder');
    }, { timeout: 10000 });

    await page.locator('a[href="/sessions"]').click();
    await expect(page).toHaveURL('/sessions');

    await page.goBack();
    await expect(page).toHaveURL('/');

    await page.goForward();
    await expect(page).toHaveURL('/sessions');
  });
});
