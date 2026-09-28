import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { defineConfig } from '@playwright/test';

const root = new URL('../..', import.meta.url).pathname;

function localFrontendUrl(): string {
  const status = JSON.parse(execSync('icp network status -e local --json', { cwd: root, encoding: 'utf8' }));
  const ids = JSON.parse(readFileSync(`${root}.icp/cache/mappings/local.ids.json`, 'utf8'));
  const gateway = new URL(status.gateway_url);
  return `${gateway.protocol}//${ids.frontend}.localhost:${gateway.port}`;
}

export default defineConfig({
  testDir: 'tests/e2e',
  timeout: 90_000,
  use: {
    baseURL: process.env.FRONTEND_URL ?? localFrontendUrl(),
    channel: process.env.PW_CHANNEL ?? 'chrome',
    screenshot: 'only-on-failure',
  },
});
