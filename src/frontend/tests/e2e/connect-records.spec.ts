import { execFileSync } from 'node:child_process';
import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.8/', import.meta.url).pathname;
const repoRoot = new URL('../../../..', import.meta.url).pathname;

async function signInAndSpawn(page: Page, context: BrowserContext, name: string): Promise<void> {
  await page.goto('/signin');
  const [ii] = await Promise.all([
    context.waitForEvent('page'),
    page.getByRole('button', { name: 'Sign in with Internet Identity' }).click(),
  ]);
  ii.on('dialog', (d) => d.accept(String(Date.now() % 1_000_000)));
  await expect(ii).toHaveURL(/id\.ai\.localhost/);
  const create = ii.getByRole('button', { name: /^Create/ }).first();
  const createWithPasskey = ii.getByRole('button', { name: 'Create with passkey' });
  await expect(create).toBeVisible();
  if (!(await createWithPasskey.isVisible())) await create.click();
  await createWithPasskey.click();
  await ii.getByRole('textbox').fill(`e2e-connect-${Date.now()}`);
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });
  await expect(page).toHaveURL(/\/spawn$/, { timeout: 30_000 });

  await page.getByLabel('Agent name').fill(name);
  await expect(page.getByRole('button', { name: 'Continue' })).toBeEnabled({ timeout: 15_000 });
  await page.getByRole('button', { name: 'Continue' }).click();

  const accountInput = page.getByLabel('Send ICP to this account');
  await expect(accountInput).toHaveValue(/^[0-9a-f]{64}$/, { timeout: 15_000 });
  const accountId = await accountInput.inputValue();

  execFileSync('icp', ['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user'], {
    cwd: repoRoot,
    stdio: 'pipe',
  });

  await page.getByRole('button', { name: "I've sent it" }).click();
  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
  await page.getByRole('button', { name: 'Connect your agent' }).click();
  await expect(page).toHaveURL(/\/connect$/, { timeout: 15_000 });
}

test('connect: operator add/revoke and records renders (05 §2 rows 8-9)', async ({ page, context }) => {
  test.setTimeout(150_000);
  const name = `E2E-Connect-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, name);

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
