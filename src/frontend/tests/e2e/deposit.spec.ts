import { test } from '@playwright/test';
import { signInAndSpawn } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.5/', import.meta.url).pathname;

test('spawn: deposit path completes end to end locally (04 §5.2, 05 §5.3)', async ({ page, context }) => {
  test.setTimeout(120_000);
  await signInAndSpawn(page, context, 'e2e-deposit', `E2E-Deposit-${Date.now() % 100_000}`);
  await page.screenshot({ path: `${demo}deposit-done.png`, fullPage: false });
});
