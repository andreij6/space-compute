import { execFileSync } from 'node:child_process';
import type { BrowserContext, Page } from '@playwright/test';
import { expect } from '@playwright/test';

const repoRoot = new URL('../../../../..', import.meta.url).pathname;

export function icp(args: string[]): string {
  return execFileSync('icp', args, { cwd: repoRoot, stdio: 'pipe' }).toString();
}

export async function signInNewUser(page: Page, context: BrowserContext, label: string): Promise<void> {
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
  await ii.getByRole('textbox').fill(`${label}-${Date.now()}`);
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });
  await expect(page).toHaveURL(/\/spawn$/, { timeout: 30_000 });
}

export async function spawnViaDeposit(page: Page, name: string): Promise<void> {
  await page.getByLabel('Agent name').fill(name);
  const continueButton = page.getByRole('button', { name: 'Continue' });
  await expect(continueButton).toBeEnabled({ timeout: 15_000 });
  await continueButton.click();

  const accountInput = page.getByLabel('Send ICP to this account');
  await expect(accountInput).toHaveValue(/^[0-9a-f]{64}$/, { timeout: 15_000 });
  const accountId = await accountInput.inputValue();

  icp(['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user']);

  await page.getByRole('button', { name: "I've sent it" }).click();
  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
}

export async function signInAndSpawn(
  page: Page,
  context: BrowserContext,
  label: string,
  name: string,
): Promise<void> {
  await signInNewUser(page, context, label);
  await spawnViaDeposit(page, name);
}

const adminCall = (canister: string, method: string, principal: string) =>
  icp(['canister', 'call', canister, method, `(principal "${principal}")`, '-e', 'local', '--identity', 'sc-deployer']);

export function grantAdmin(principal: string): () => void {
  for (const canister of ['platform', 'payments']) {
    const out = adminCall(canister, 'admin_add_admin', principal);
    if (out.includes('Err')) throw new Error(`${canister}.admin_add_admin failed: ${out}`);
  }
  return () => {
    for (const canister of ['platform', 'payments']) adminCall(canister, 'admin_remove_admin', principal);
  };
}
