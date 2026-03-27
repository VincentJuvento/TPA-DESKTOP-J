import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E configuration for RUSA IMS.
 *
 * The application is served by the Tauri/Vite dev server on port 1420.
 * To run the tests:
 *   1. Start the dev server:  npm run dev   (or npm run tauri dev for the full desktop app)
 *   2. Run the suite:         npm run test:e2e
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',

  /* Retry once on CI to reduce flakiness from network timing */
  retries: process.env.CI ? 1 : 0,

  /* Single worker to avoid auth state conflicts between modules */
  workers: 1,

  reporter: 'html',

  use: {
    /* Base URL for the Tauri/Vite dev server */
    baseURL: 'http://localhost:1420',

    /* Capture a trace on the first retry so failures are diagnosable */
    trace: 'on-first-retry',

    /* Take a screenshot automatically on every test failure */
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
