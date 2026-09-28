import { expect, test } from '@playwright/test';
import { signInAndSpawn } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.7/', import.meta.url).pathname;

test('dashboard: live fuel, activity, credits and mandate render for a freshly spawned AAA (05 §2 row 7)', async ({
  page,
  context,
}) => {
  test.setTimeout(120_000);
  const name = `E2E-Dashboard-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, 'e2e-dashboard', name);
  await page.getByRole('button', { name: 'Go to dashboard' }).click();

  await expect(page).toHaveURL(/\/dashboard$/, { timeout: 30_000 });
  await expect(page.getByRole('heading', { level: 1, name })).toBeVisible();
  await expect(page.getByText(/Days of fuel remaining/)).toBeVisible();
  await expect(page.getByText(/T Cycles/)).toBeVisible();
  await expect(page.getByText('No auto top-up configured.')).toBeVisible();
  await expect(page.getByText('No credits yet.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Connect your agent' })).toBeVisible();

  await page.screenshot({ path: `${demo}dashboard-live.png`, fullPage: true });
});
