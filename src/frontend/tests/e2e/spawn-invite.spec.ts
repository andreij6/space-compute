import { expect, test } from '@playwright/test';
import { icp, signInNewUser } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.6/', import.meta.url).pathname;

function mintInviteCode(): string {
  const expiresAtNs = (BigInt(Date.now()) + 86_400_000n) * 1_000_000n;
  const out = icp([
    'canister',
    'call',
    'payments',
    'admin_mint_invites',
    `(record { sponsor_cycles = 1_200_000_000_000 : nat; count = 1 : nat32; expires_at = ${expiresAtNs} : nat64 })`,
    '-e',
    'local',
    '--identity',
    'sc-deployer',
  ]);
  const match = out.match(/"([A-Z0-9-]+)"/);
  if (!match) throw new Error(`could not parse invite code from: ${out}`);
  return match[1];
}

function fundTreasury(): void {
  const out = icp(['canister', 'call', 'payments', 'get_treasury_account', '()', '-e', 'local', '--identity', 'sc-user']);
  const match = out.match(/"([0-9a-f]{64})"/);
  if (!match) throw new Error(`could not parse treasury account from: ${out}`);
  icp(['token', 'transfer', '2', match[1], '-e', 'local', '--identity', 'sc-user']);
}

test('spawn: sponsored invite path completes end to end locally (04 §0b, §4; 05 §2 row 6)', async ({ page, context }) => {
  test.setTimeout(120_000);
  fundTreasury();
  const code = mintInviteCode();

  await signInNewUser(page, context, 'e2e-invite');
  await page.getByLabel('Agent name').fill(`E2E-Invite-${Date.now() % 100_000}`);
  await expect(page.getByRole('button', { name: 'Continue' })).toBeEnabled({ timeout: 15_000 });
  await page.screenshot({ path: `${demo}spawn-step1-name-avatar.png`, fullPage: false });
  await page.getByRole('button', { name: 'Continue' }).click();

  await page.getByRole('tab', { name: 'Invite code' }).click();
  await page.getByLabel('Invite code').fill(code);
  await page.getByRole('button', { name: 'Redeem invite' }).click();

  await expect(page.getByRole('status')).toHaveText('Done.', { timeout: 60_000 });
  await page.screenshot({ path: `${demo}spawn-invite-done.png`, fullPage: false });
});
