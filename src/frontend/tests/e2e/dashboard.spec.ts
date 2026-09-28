import { execFileSync } from 'node:child_process';
import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.7/', import.meta.url).pathname;
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
  await ii.getByRole('textbox').fill(`e2e-dashboard-${Date.now()}`);
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });
  await expect(page).toHaveURL(/\/spawn$/, { timeout: 30_000 });

  await page.getByLabel('Agent name').fill(name);
  const accountInput = page.getByLabel('Send ICP to this account');
  await expect(accountInput).toHaveValue(/^[0-9a-f]{64}$/, { timeout: 15_000 });
  const accountId = await accountInput.inputValue();

  execFileSync('icp', ['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user'], {
    cwd: repoRoot,
    stdio: 'pipe',
  });

  await page.getByRole('button', { name: "I've sent it" }).click();
  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
}

test('dashboard: live fuel, activity, credits and mandate render for a freshly spawned AAA (05 §2 row 7)', async ({
  page,
  context,
}) => {
  test.setTimeout(120_000);
  const name = `E2E-Dashboard-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, name);

  await expect(page).toHaveURL(/\/dashboard$/, { timeout: 30_000 });
  await expect(page.getByRole('heading', { level: 1, name })).toBeVisible();
  await expect(page.getByText(/Days of fuel remaining/)).toBeVisible();
  await expect(page.getByText(/T Cycles/)).toBeVisible();
  await expect(page.getByText('No auto top-up configured.')).toBeVisible();
  await expect(page.getByText('No credits yet.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Connect your agent' })).toBeVisible();

  await page.screenshot({ path: `${demo}dashboard-live.png`, fullPage: true });
});
