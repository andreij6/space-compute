import { execFileSync } from 'node:child_process';
import { expect, test } from '@playwright/test';
import { signInAndSpawn } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.9/', import.meta.url).pathname;
const repoRoot = new URL('../../../..', import.meta.url).pathname;

test('fuel: live fuel gauge, no-mandate state and a Deposit one-time top-up reach Done (05 §2 row 10)', async ({
  page,
  context,
}) => {
  test.setTimeout(150_000);
  const name = `E2E-Fuel-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, 'e2e-fuel', name);

  await page.goto('/fuel');
  await expect(page.getByRole('heading', { level: 1, name: 'Fuel & billing' })).toBeVisible();
  await expect(page.getByText(/Days of fuel remaining/)).toBeVisible();
  await expect(page.getByText('No auto top-up configured.')).toBeVisible();
  await page.screenshot({ path: `${demo}01_fuel_none.png`, fullPage: true });

  const accountInput = page.getByLabel('Send ICP to this account');
  await expect(accountInput).toHaveValue(/^[0-9a-f]{64}$/, { timeout: 15_000 });
  const accountId = await accountInput.inputValue();

  execFileSync('icp', ['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user'], {
    cwd: repoRoot,
    stdio: 'pipe',
  });

  await page.getByRole('button', { name: "I've sent it" }).click();
  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
  await page.screenshot({ path: `${demo}02_topup_done.png`, fullPage: true });
});

test('fuel: the auto top-up setup form renders; the wallet approve step is exercised with OISY in staging (05 §5 #3), not headless e2e', async ({
  page,
  context,
}) => {
  test.setTimeout(120_000);
  const name = `E2E-Mandate-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, 'e2e-mandate', name);
  await page.goto('/fuel');

  await expect(page.getByText('No auto top-up configured.')).toBeVisible();
  await expect(page.getByLabel('Per top-up amount (ICP, min 0.1)')).toBeVisible();
  await expect(page.getByLabel('Monthly limit (ICP)')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Connect wallet, approve & save' })).toBeVisible();
  await page.screenshot({ path: `${demo}03_mandate_setup_form.png`, fullPage: true });
});
