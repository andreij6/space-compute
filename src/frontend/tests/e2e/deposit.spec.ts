import { execFileSync } from 'node:child_process';
import type { BrowserContext, Page } from '@playwright/test';
import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.5/', import.meta.url).pathname;
const repoRoot = new URL('../../../..', import.meta.url).pathname;

async function signInNewUser(page: Page, context: BrowserContext) {
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
  await ii.getByRole('textbox').fill(`e2e-deposit-${Date.now()}`);
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });
  await expect(page).toHaveURL(/\/spawn$/, { timeout: 30_000 });
}

test('spawn: deposit path completes end to end locally (04 §5.2, 05 §5.3)', async ({ page, context }) => {
  test.setTimeout(120_000);
  await signInNewUser(page, context);

  await page.getByLabel('Agent name').fill(`E2E-Deposit-${Date.now() % 100_000}`);

  const accountInput = page.getByLabel('Send ICP to this account');
  await expect(accountInput).toHaveValue(/^[0-9a-f]{64}$/, { timeout: 15_000 });
  const accountId = await accountInput.inputValue();

  execFileSync('icp', ['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user'], {
    cwd: repoRoot,
    stdio: 'pipe',
  });

  await page.getByRole('button', { name: "I've sent it" }).click();
  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
  await page.screenshot({ path: `${demo}deposit-done.png`, fullPage: false });
});
