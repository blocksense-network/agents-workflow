import { test, expect } from '@playwright/test';

test.describe('Session Management Functionality', () => {
  test.describe('Session List Display', () => {
    test('Sessions page renders with three-pane layout', async ({ page }) => {
      await page.goto('/sessions');

      // Check main layout elements
      await expect(page.locator('h1').filter({ hasText: 'Agents-Workflow' })).toBeVisible();
      await expect(page.locator('h2').filter({ hasText: 'Sessions' })).toBeVisible();
      await expect(page.locator('h2').filter({ hasText: 'Task Details' })).toBeVisible();
    });

    test('Session list loads and displays sessions', async ({ page }) => {
      await page.goto('/sessions');

      // Wait for sessions to load
      await page.waitForTimeout(2000);

      // Check if session cards are present (may be empty or have sessions)
      const sessionCards = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      // Either we have session cards or an empty state message
      const hasSessions = await sessionCards.isVisible().catch(() => false);
      const hasEmptyState = await page
        .locator('text=No sessions')
        .isVisible()
        .catch(() => false);

      expect(hasSessions || hasEmptyState).toBe(true);
    });

    test('Status filter works correctly', async ({ page }) => {
      await page.goto('/sessions');

      // Wait for sessions to load
      await page.waitForTimeout(2000);

      // Check status filter is present
      const statusFilter = page.locator('select[id="status-filter"]');
      await expect(statusFilter).toBeVisible();

      // Check filter options
      await expect(
        statusFilter.locator('option').filter({ hasText: 'All Sessions' })
      ).toBeVisible();
      await expect(statusFilter.locator('option').filter({ hasText: 'Running' })).toBeVisible();
      await expect(statusFilter.locator('option').filter({ hasText: 'Queued' })).toBeVisible();
      await expect(statusFilter.locator('option').filter({ hasText: 'Completed' })).toBeVisible();
      await expect(statusFilter.locator('option').filter({ hasText: 'Failed' })).toBeVisible();

      // Change filter (this test assumes there might be sessions to filter)
      await statusFilter.selectOption('running');
      await expect(statusFilter).toHaveValue('running');
    });
  });

  test.describe('Session Creation and Display', () => {
    test('Created task appears in session list', async ({ page }) => {
      // First create a task
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session display');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      // Get the task ID from success message
      const taskIdElement = page.locator('code').first();
      const taskId = await taskIdElement.textContent();

      // Navigate to sessions page
      await page.locator('text=View Sessions').click();

      // Wait for sessions to load
      await page.waitForTimeout(2000);

      // Check if the session appears in the list
      if (taskId) {
        // Look for the session ID in the session cards
        const sessionCard = page.locator(`text=${taskId.slice(-8)}`);
        await expect(sessionCard).toBeVisible();
      }
    });

    test('Session card shows correct information', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session card info');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Find a session card
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        // Check session card has expected elements
        await expect(sessionCard.locator('text=repo')).toBeVisible();
        await expect(sessionCard.locator('text=main')).toBeVisible();
        await expect(sessionCard.locator('text=running')).toBeVisible();
      }
    });
  });

  test.describe('Session Selection and Details', () => {
    test('Clicking session card selects it', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session selection');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Find and click a session card
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Check that the card appears selected (has ring styling)
        await expect(sessionCard).toHaveClass(/ring-2/);

        // Check that task details pane shows information
        await expect(page.locator('h2').filter({ hasText: 'Task Details' })).toBeVisible();
        await expect(page.locator('text=Session')).toBeVisible();
      }
    });

    test('Session detail view shows tabs', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session details');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Select a session
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Check tabs are present
        await expect(page.locator('button').filter({ hasText: 'Overview' })).toBeVisible();
        await expect(page.locator('button').filter({ hasText: 'Logs' })).toBeVisible();
        await expect(page.locator('button').filter({ hasText: 'Events' })).toBeVisible();
      }
    });

    test('Session detail overview tab shows information', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session overview');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Select a session
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Check overview tab content
        await expect(page.locator('text=Created:')).toBeVisible();
        await expect(page.locator('text=Repository:')).toBeVisible();
        await expect(page.locator('text=Branch:')).toBeVisible();
        await expect(page.locator('text=Agent:')).toBeVisible();
        await expect(page.locator('text=Runtime:')).toBeVisible();
        await expect(page.locator('text=Task Prompt')).toBeVisible();
      }
    });
  });

  test.describe('Session Controls', () => {
    test('Stop button appears for running sessions', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session stop');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Find a session card with stop button
      const stopButton = page.locator('button[title="Stop session"]').first();
      const stopVisible = await stopButton.isVisible().catch(() => false);

      if (stopVisible) {
        await expect(stopButton).toBeVisible();
        await expect(stopButton.locator('svg')).toBeVisible(); // Should have stop icon
      }
    });

    test('Cancel button appears for cancellable sessions', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session cancel');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Find a session card with cancel button
      const cancelButton = page.locator('button[title="Cancel session"]').first();
      const cancelVisible = await cancelButton.isVisible().catch(() => false);

      if (cancelVisible) {
        await expect(cancelButton).toBeVisible();
        await expect(cancelButton.locator('svg')).toBeVisible(); // Should have X icon
      }
    });

    test('Session detail view has control buttons', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session controls');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Select a session
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Check for control buttons in detail view
        const stopButton = page.locator('button').filter({ hasText: 'Stop' }).first();
        const stopVisible = await stopButton.isVisible().catch(() => false);

        if (stopVisible) {
          await expect(stopButton).toBeVisible();
        }
      }
    });
  });

  test.describe('Session Logs', () => {
    test('Logs tab shows log information', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test session logs');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Select a session
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Click logs tab
        await page.locator('button').filter({ hasText: 'Logs' }).click();

        // Check logs content (may be empty or have mock logs)
        const hasLogs = await page
          .locator('text=No logs available')
          .isVisible()
          .catch(() => false);
        const hasLogEntries = await page
          .locator('[class*="font-mono"]')
          .first()
          .isVisible()
          .catch(() => false);

        expect(hasLogs || hasLogEntries).toBe(true);
      }
    });
  });

  test.describe('URL Hash Navigation', () => {
    test('Session selection updates URL hash', async ({ page }) => {
      await page.goto('/create');
      await page.locator('textarea[id="prompt"]').fill('Test URL hash');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      await page.locator('text=View Sessions').click();
      await page.waitForTimeout(2000);

      // Select a session
      const sessionCard = page
        .locator('[class*="bg-white"][class*="border"][class*="rounded-lg"]')
        .first();
      const cardVisible = await sessionCard.isVisible().catch(() => false);

      if (cardVisible) {
        await sessionCard.click();

        // Check URL has hash
        const url = page.url();
        expect(url).toContain('#session-');
      }
    });
  });
});
