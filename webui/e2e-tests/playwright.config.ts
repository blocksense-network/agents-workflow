import { defineConfig, devices } from '@playwright/test';

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html'],
    ['junit', { outputFile: 'results.xml' }],
    ['github']
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.BASE_URL || 'http://localhost:3000',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
  },

  /* Configure projects for different test types */
  projects: [
    {
      name: 'api-tests',
      testMatch: '**/api-contract.spec.ts',
      use: {
        baseURL: process.env.BASE_URL || 'http://localhost:3000',
      },
    },
    {
      name: 'build-tooling-tests',
      testMatch: '**/build-tooling.spec.ts',
      use: {
        baseURL: process.env.BASE_URL || 'http://localhost:3000',
      },
    },
    {
      name: 'infrastructure-tests',
      testMatch: '**/example.spec.ts',
      use: {
        baseURL: process.env.BASE_URL || 'http://localhost:3000',
      },
    },
    {
      name: 'browser-tests',
      testMatch: ['**/layout-navigation.spec.ts', '**/accessibility.spec.ts', '**/localstorage-persistence.spec.ts'],
      use: {
        ...devices['Desktop Chrome'],
        baseURL: process.env.BASE_URL || 'http://localhost:3000',
        // Use Nix store browser when PLAYWRIGHT_BROWSERS_PATH is set
        ...(process.env.PLAYWRIGHT_BROWSERS_PATH && {
          executablePath: `${process.env.PLAYWRIGHT_BROWSERS_PATH}/chromium-1169/chrome-linux/chrome`
        })
      },
    },
  ],

  /* Note: Servers are started via start-servers.sh script for reliable orchestration */
});