import path from 'node:path';
import { defineConfig, devices } from '@playwright/test';

const projectRoot = path.resolve(import.meta.dirname);
const tauriTargetDir = process.env.TAURI_TARGET_DIR
  ? path.resolve(process.env.TAURI_TARGET_DIR)
  : path.join(projectRoot, 'src-tauri', 'target', 'debug');
const tauriExecutableName = process.platform === 'win32' ? 'rusa-app.exe' : 'rusa-app';
const tauriExecutablePath = process.env.PLAYWRIGHT_TAURI_EXECUTABLE_PATH
  ? path.resolve(process.env.PLAYWRIGHT_TAURI_EXECUTABLE_PATH)
  : path.join(tauriTargetDir, tauriExecutableName);

/**
 * Playwright E2E configuration for RUSA IMS Tauri desktop integration.
 * To run the tests:
 *   1. Build desktop binary:  cargo build --manifest-path src-tauri/Cargo.toml
 *   2. Run the suite:         npm run test:e2e
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',
  globalSetup: './tests/tauri.global-setup.ts',

  /* Retry once on CI to reduce flakiness from network timing */
  retries: process.env.CI ? 1 : 0,

  /* Single worker to avoid auth state conflicts between modules */
  workers: 1,

  reporter: 'html',

  use: {
    /*
     * Tauri desktop executable under test.
     * Can be overridden with PLAYWRIGHT_TAURI_EXECUTABLE_PATH.
     */
    launchOptions: {
      executablePath: tauriExecutablePath,
    },

    /* Capture a trace on the first retry so failures are diagnosable */
    trace: 'on-first-retry',

    /* Take a screenshot automatically on every test failure */
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'tauri-desktop',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
