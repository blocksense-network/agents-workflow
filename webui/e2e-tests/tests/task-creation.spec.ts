import { test, expect } from '@playwright/test';

test.describe('Task Creation Functionality', () => {
  test.describe('Task Creation Form', () => {
    test('Form renders with all required fields', async ({ page }) => {
      await page.goto('/create');

      // Check form title and structure
      await expect(page.locator('h1').filter({ hasText: 'Create New Task' })).toBeVisible();

      // Check form fields are present
      await expect(page.locator('label').filter({ hasText: 'Task Prompt *' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Repository URL *' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Branch *' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Agent *' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Runtime *' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Delivery Mode' })).toBeVisible();
      await expect(page.locator('label').filter({ hasText: 'Target Branch' })).toBeVisible();
    });

    test('Form loads agent and runtime options from API', async ({ page }) => {
      await page.goto('/create');

      // Wait for form to load and populate dropdowns
      await page.waitForTimeout(1000);

      // Check that agent dropdown has options
      const agentSelect = page.locator('select[id="agentType"]');
      await expect(agentSelect).toBeVisible();
      const agentOptions = await agentSelect.locator('option').all();
      expect(agentOptions.length).toBeGreaterThan(1); // Should have at least "Select an agent..." and one agent

      // Check that runtime dropdown has options
      const runtimeSelect = page.locator('select[id="runtimeType"]');
      await expect(runtimeSelect).toBeVisible();
      const runtimeOptions = await runtimeSelect.locator('option').all();
      expect(runtimeOptions.length).toBeGreaterThan(1); // Should have at least "Select a runtime..." and one runtime
    });

    test('Form validates required fields', async ({ page }) => {
      await page.goto('/create');

      // Try to submit empty form
      await page.locator('button[type="submit"]').click();

      // Check for validation errors
      await expect(page.locator('text=Prompt is required')).toBeVisible();
      await expect(page.locator('text=Repository URL is required')).toBeVisible();
      await expect(page.locator('text=Branch is required')).toBeVisible();
      await expect(page.locator('text=Please select an agent')).toBeVisible();
      await expect(page.locator('text=Please select a runtime')).toBeVisible();
    });

    test('Form validates repository URL format', async ({ page }) => {
      await page.goto('/create');

      // Fill in invalid repository URL
      await page.locator('input[id="repoUrl"]').fill('invalid-url');
      await page.locator('button[type="submit"]').click();

      // Check for URL validation error
      await expect(page.locator('text=Please enter a valid repository URL')).toBeVisible();
    });

    test('Form submits successfully with valid data', async ({ page }) => {
      await page.goto('/create');

      // Fill in valid form data
      await page.locator('textarea[id="prompt"]').fill('Test task for automated testing');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');

      // Select agent and runtime
      await page.locator('select[id="agentType"]').selectOption({ index: 1 }); // Select first agent
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 }); // Select first runtime

      // Submit form
      await page.locator('button[type="submit"]').click();

      // Check success message appears
      await expect(page.locator('text=Task Created Successfully!')).toBeVisible();
      await expect(page.locator('text=Task ID:')).toBeVisible();

      // Check navigation buttons are present
      await expect(page.locator('text=Create Another Task')).toBeVisible();
      await expect(page.locator('text=View Sessions')).toBeVisible();
    });

    test('Agent version selection appears when agent is selected', async ({ page }) => {
      await page.goto('/create');

      // Initially no version selector
      await expect(page.locator('select').filter({ hasText: 'latest' })).not.toBeVisible();

      // Select an agent
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });

      // Version selector should appear
      await expect(page.locator('select').filter({ hasText: 'latest' })).toBeVisible();
    });

    test('Delivery mode and target branch work correctly', async ({ page }) => {
      await page.goto('/create');

      // Check default values
      const deliveryModeSelect = page.locator('select[id="deliveryMode"]');
      await expect(deliveryModeSelect).toHaveValue('pr');

      const targetBranchInput = page.locator('input[id="targetBranch"]');
      await expect(targetBranchInput).toHaveValue('main');

      // Change delivery mode
      await deliveryModeSelect.selectOption('branch');
      await expect(targetBranchInput).toHaveValue('main'); // Should keep value

      // Change target branch
      await targetBranchInput.fill('develop');
      await expect(targetBranchInput).toHaveValue('develop');
    });
  });

  test.describe('Task Creation Navigation', () => {
    test('Can navigate to create task from main navigation', async ({ page }) => {
      await page.goto('/');

      // Click create task link
      await page.locator('a[href="/create"]').click();

      // Should navigate to create page
      await expect(page).toHaveURL('/create');
      await expect(page.locator('h1').filter({ hasText: 'Create New Task' })).toBeVisible();
    });

    test('Create Another Task button works', async ({ page }) => {
      await page.goto('/create');

      // Fill and submit form
      await page.locator('textarea[id="prompt"]').fill('Test task');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      // Click create another task
      await page.locator('text=Create Another Task').click();

      // Should reset form
      await expect(page.locator('textarea[id="prompt"]')).toHaveValue('');
      await expect(page.locator('text=Task Created Successfully!')).not.toBeVisible();
    });

    test('View Sessions button navigates to sessions page', async ({ page }) => {
      await page.goto('/create');

      // Fill and submit form
      await page.locator('textarea[id="prompt"]').fill('Test task');
      await page.locator('input[id="repoUrl"]').fill('https://github.com/test/repo.git');
      await page.locator('input[id="repoBranch"]').fill('main');
      await page.locator('select[id="agentType"]').selectOption({ index: 1 });
      await page.locator('select[id="runtimeType"]').selectOption({ index: 1 });
      await page.locator('button[type="submit"]').click();

      // Click view sessions
      await page.locator('text=View Sessions').click();

      // Should navigate to sessions page
      await expect(page).toHaveURL(/\/sessions/);
    });
  });
});
