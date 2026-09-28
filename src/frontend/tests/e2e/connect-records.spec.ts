import { execFileSync } from 'node:child_process';
import { expect, test } from '@playwright/test';
import { signInAndSpawn } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.8/', import.meta.url).pathname;
const repoRoot = new URL('../../../..', import.meta.url).pathname;

test('connect: operator add/revoke and records renders (05 §2 rows 8-9)', async ({ page, context }) => {
  test.setTimeout(150_000);
  const name = `E2E-Connect-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, 'e2e-connect', name);
  await page.getByRole('button', { name: 'Connect your agent' }).click();
  await expect(page).toHaveURL(/\/connect$/, { timeout: 15_000 });

  await expect(page.getByRole('heading', { level: 1, name: 'Connect your agent' })).toBeVisible();
  await expect(page.getByText('Not connected yet')).toBeVisible();
  await expect(page.getByText('No operators yet.')).toBeVisible();

  const identityName = `sc-operator-e2e-${Date.now() % 1_000_000}`;
  execFileSync('icp', ['identity', 'new', identityName, '--storage', 'plaintext'], { cwd: repoRoot, stdio: 'pipe' });
  const operatorPrincipal = execFileSync('icp', ['identity', 'principal', '--identity', identityName], {
    cwd: repoRoot,
  })
    .toString()
    .trim();

  await page.getByLabel('Operator principal').fill(operatorPrincipal);
  await page.getByLabel('Label').fill('e2e-operator');
  await page.getByRole('button', { name: 'Add operator' }).click();

  await expect(page.getByText(operatorPrincipal)).toBeVisible({ timeout: 15_000 });
  await expect(page.getByText('No operators yet.')).toHaveCount(0);

  await page.screenshot({ path: `${demo}connect-operator-added.png`, fullPage: true });

  await page.getByRole('button', { name: 'Revoke' }).click();
  await expect(page.getByText(operatorPrincipal)).toHaveCount(0, { timeout: 15_000 });
  await expect(page.getByText('No operators yet.')).toBeVisible();

  await page.goto('/records');
  await expect(page.getByRole('heading', { level: 1, name: 'Activity & records' })).toBeVisible();

  await page.screenshot({ path: `${demo}records.png`, fullPage: true });
});
