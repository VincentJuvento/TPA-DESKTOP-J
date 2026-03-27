import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import type { FullConfig } from '@playwright/test';

function resolveTauriExecutable(projectRoot: string): string {
  const targetDir = process.env.TAURI_TARGET_DIR
    ? path.resolve(process.env.TAURI_TARGET_DIR)
    : path.join(projectRoot, 'src-tauri', 'target', 'debug');
  const executableName = process.platform === 'win32' ? 'rusa-app.exe' : 'rusa-app';
  return process.env.PLAYWRIGHT_TAURI_EXECUTABLE_PATH
    ? path.resolve(process.env.PLAYWRIGHT_TAURI_EXECUTABLE_PATH)
    : path.join(targetDir, executableName);
}

export default async function globalSetup(_config: FullConfig) {
  const projectRoot = path.resolve(import.meta.dirname, '..');
  const manifestPath = path.join(projectRoot, 'src-tauri', 'Cargo.toml');
  const executablePath = resolveTauriExecutable(projectRoot);

  // Allow CI/devs to skip build if they pre-built and want faster iteration.
  if (process.env.PLAYWRIGHT_SKIP_TAURI_BUILD !== '1') {
    execSync(`cargo build --manifest-path "${manifestPath}"`, {
      cwd: projectRoot,
      stdio: 'inherit',
      env: process.env,
    });
  }

  if (!fs.existsSync(executablePath)) {
    throw new Error(
      `Tauri executable not found at: ${executablePath}. ` +
        `Set PLAYWRIGHT_TAURI_EXECUTABLE_PATH or run cargo build first.`,
    );
  }
}
