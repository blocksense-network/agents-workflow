import { test, expect } from '@playwright/test';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

test.describe('Build and Tooling Tests', () => {
  test('App SSR server builds successfully with TypeScript strict mode', async () => {
    // This test runs the build process for the app-ssr-server
    try {
      execSync('cd ../app-ssr-server && npm run build', { stdio: 'pipe' });
      expect(true).toBe(true); // Build succeeded
    } catch (error) {
      console.error('Build failed:', error);
      expect(false).toBe(true); // Build failed
    }
  });

  test('App builds successfully with TypeScript strict mode', async () => {
    // This test runs the build process for the app
    try {
      execSync('cd ../app && npm run build', { stdio: 'pipe' });
      expect(true).toBe(true); // Build succeeded
    } catch (error) {
      console.error('Build failed:', error);
      expect(false).toBe(true); // Build failed
    }
  });

  test('Mock server builds successfully with TypeScript strict mode', async () => {
    // This test runs the build process for the mock-server
    try {
      execSync('cd ../mock-server && npm run build', { stdio: 'pipe' });
      expect(true).toBe(true); // Build succeeded
    } catch (error) {
      console.error('Build failed:', error);
      expect(false).toBe(true); // Build failed
    }
  });

  test('E2E tests build successfully with TypeScript strict mode', async () => {
    // This test runs the TypeScript compilation for e2e tests
    try {
      execSync('npx tsc --noEmit', { stdio: 'pipe' });
      expect(true).toBe(true); // TypeScript compilation succeeded
    } catch (error) {
      console.error('TypeScript compilation failed:', error);
      expect(false).toBe(true); // TypeScript compilation failed
    }
  });

  test('ESLint configuration works across all projects', async () => {
    // Test ESLint on e2e tests
    try {
      execSync('npm run lint', { stdio: 'pipe' });
      expect(true).toBe(true); // ESLint succeeded
    } catch (error) {
      console.error('ESLint failed:', error);
      expect(false).toBe(true); // ESLint failed
    }
  });

  test('Prettier configuration works across all projects', async () => {
    // Test Prettier formatting check
    try {
      execSync('npm run format:check', { stdio: 'pipe' });
      expect(true).toBe(true); // Prettier check succeeded
    } catch (error) {
      console.error('Prettier check failed:', error);
      expect(false).toBe(true); // Prettier check failed
    }
  });

  test('TypeScript configuration files are valid JSON', async () => {
    const tsconfigPath = 'tsconfig.json';
    expect(fs.existsSync(tsconfigPath)).toBe(true);

    const tsconfig = fs.readFileSync(tsconfigPath, 'utf8');
    expect(() => JSON.parse(tsconfig)).not.toThrow();
  });

  test('Playwright configuration is valid', async () => {
    const playwrightConfigPath = 'playwright.config.ts';
    expect(fs.existsSync(playwrightConfigPath)).toBe(true);

    // Test that Playwright can parse the config without errors
    try {
      execSync('npx playwright test --list | head -1', { stdio: 'pipe' });
      expect(true).toBe(true); // Playwright config is valid
    } catch (error) {
      console.error('Playwright config validation failed:', error);
      expect(false).toBe(true); // Playwright config is invalid
    }
  });
});
