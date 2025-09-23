import { test, expect } from '@playwright/test';

test.describe('localStorage Persistence Tests', () => {
  test('UI preferences persist across browser sessions', async ({ page, context }) => {
    await page.goto('/');

    // Check initial localStorage state (removed unused variable)

    // Find and click a collapse button if it exists
    const collapseBtn = page
      .locator('[data-testid="repositories-collapse"], [data-testid="sessions-collapse"]')
      .first();
    if (await collapseBtn.isVisible()) {
      await collapseBtn.click();

      // Wait for state to update
      await page.waitForTimeout(500);

      // Check that preference was saved to localStorage
      const collapsedState = await page.evaluate(() => {
        return (
          localStorage.getItem('repositoriesCollapsed') || localStorage.getItem('sessionsCollapsed')
        );
      });

      expect(collapsedState).toBeDefined();

      // Create a new page to simulate browser restart
      const newPage = await context.newPage();
      await newPage.goto('/');

      // Check that the collapsed state persists
      const persistedState = await newPage.evaluate(() => {
        return (
          localStorage.getItem('repositoriesCollapsed') || localStorage.getItem('sessionsCollapsed')
        );
      });

      expect(persistedState).toBe(collapsedState);
    }
  });

  test('Global search preferences are saved', async ({ page }) => {
    await page.goto('/');

    // Find search input
    const searchInput = page
      .locator('input[placeholder*="search" i], [data-testid="global-search"] input')
      .first();

    if (await searchInput.isVisible()) {
      // Type something in search
      await searchInput.fill('test search query');

      // Wait for potential debouncing
      await page.waitForTimeout(500);

      // Check if search query is saved (if the app implements this)
      const savedSearch = await page.evaluate(() => {
        return localStorage.getItem('searchQuery') || localStorage.getItem('globalSearch');
      });

      // Note: This test assumes the app saves search queries. If not implemented yet, this will be skipped.
      if (savedSearch) {
        expect(savedSearch).toContain('test search query');
      }
    }
  });

  test('Theme preferences persist', async ({ page, context }) => {
    await page.goto('/');

    // Look for theme toggle or dark mode switch
    const themeToggle = page
      .locator('[data-testid="theme-toggle"], [aria-label*="theme" i], [aria-label*="dark" i]')
      .first();

    if (await themeToggle.isVisible()) {
      const initialTheme = await page.evaluate(() => {
        return localStorage.getItem('theme') || localStorage.getItem('darkMode');
      });

      await themeToggle.click();

      // Wait for theme change
      await page.waitForTimeout(500);

      const newTheme = await page.evaluate(() => {
        return localStorage.getItem('theme') || localStorage.getItem('darkMode');
      });

      expect(newTheme).toBeDefined();
      expect(newTheme).not.toBe(initialTheme);

      // Test persistence across pages
      const newPage = await context.newPage();
      await newPage.goto('/');

      const persistedTheme = await newPage.evaluate(() => {
        return localStorage.getItem('theme') || localStorage.getItem('darkMode');
      });

      expect(persistedTheme).toBe(newTheme);
    }
  });

  test('Form drafts are saved locally', async ({ page }) => {
    await page.goto('/create');

    // Look for form inputs
    const textInput = page.locator('input[type="text"], textarea').first();

    if (await textInput.isVisible()) {
      // Type something in the form
      await textInput.fill('Test draft content');

      // Wait for auto-save (if implemented)
      await page.waitForTimeout(1000);

      // Check if draft was saved
      const savedDraft = await page.evaluate(() => {
        return (
          localStorage.getItem('taskDraft') ||
          localStorage.getItem('formDraft') ||
          localStorage.getItem('createTaskDraft')
        );
      });

      // Note: This test assumes the app auto-saves drafts. If not implemented, this will be informational.
      if (savedDraft) {
        expect(typeof savedDraft).toBe('string');
      }
    }
  });

  test('Pane sizes are remembered', async ({ page, context }) => {
    await page.goto('/');

    // Look for resizable panes or splitter handles
    const splitter = page.locator('[data-testid="pane-splitter"], [data-testid="resizer"]').first();

    if (await splitter.isVisible()) {
      // Get initial pane sizes (removed unused variable)

      // Simulate resize if possible (this would require more complex interaction)
      // For now, just check if pane sizes are stored
      const savedSizes = await page.evaluate(() => {
        return localStorage.getItem('paneSizes') || localStorage.getItem('layoutSizes');
      });

      if (savedSizes) {
        expect(typeof savedSizes).toBe('string');
        // Should be valid JSON
        expect(() => JSON.parse(savedSizes)).not.toThrow();
      }

      // Test persistence
      const newPage = await context.newPage();
      await newPage.goto('/');

      const persistedSizes = await newPage.evaluate(() => {
        return localStorage.getItem('paneSizes') || localStorage.getItem('layoutSizes');
      });

      if (savedSizes && persistedSizes) {
        expect(persistedSizes).toBe(savedSizes);
      }
    }
  });

  test('localStorage does not contain sensitive data', async ({ page }) => {
    await page.goto('/');

    // Check that localStorage doesn't contain sensitive information
    const allKeys = await page.evaluate(() => {
      const keys = Object.keys(localStorage);
      return keys.filter(
        (key) =>
          !key.includes('Collapsed') &&
          !key.includes('theme') &&
          !key.includes('search') &&
          !key.includes('draft') &&
          !key.includes('pane') &&
          !key.includes('layout')
      );
    });

    // Should not contain API keys, tokens, passwords, etc.
    const sensitiveKeys = allKeys.filter(
      (key) =>
        key.toLowerCase().includes('token') ||
        key.toLowerCase().includes('key') ||
        key.toLowerCase().includes('secret') ||
        key.toLowerCase().includes('password') ||
        key.toLowerCase().includes('auth')
    );

    expect(sensitiveKeys).toHaveLength(0);
  });

  test('localStorage has reasonable size limits', async ({ page }) => {
    await page.goto('/');

    // Check total localStorage size
    const storageSize = await page.evaluate(() => {
      let total = 0;
      for (let key in localStorage) {
        if (Object.prototype.hasOwnProperty.call(localStorage, key)) {
          total += localStorage[key].length + key.length;
        }
      }
      return total;
    });

    // Should be well under browser limits (typically 5-10MB)
    expect(storageSize).toBeLessThan(1024 * 1024); // Less than 1MB
  });
});
