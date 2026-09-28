import { expect, test } from '@playwright/test';
import { icp, signInNewUser } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T6.12/', import.meta.url).pathname;

test('non-admin gets 404 on /admin (05 §2b, acceptance #0)', async ({ page, context }) => {
  test.setTimeout(90_000);
  await signInNewUser(page, context, 'e2e-nonadmin');
  await page.goto('/admin');
  await expect(page.getByRole('heading', { name: /404/ })).toBeVisible();
  await page.screenshot({ path: `${demo}nonadmin-404.png`, fullPage: true });
});

test('admin console: gate, overview, pause/unpause mutation, audit trail (05 §2b, acceptance #0)', async ({
  page,
  context,
}) => {
  test.setTimeout(150_000);
  await signInNewUser(page, context, 'e2e-admin');
  const principal = await page.getByRole('button', { name: /Sign out/ }).getAttribute('title');
  if (!principal) throw new Error('Could not read the signed-in principal from the navbar.');

  icp(['canister', 'call', 'platform', 'admin_add_admin', `(principal "${principal}")`, '-e', 'local', '--identity', 'sc-deployer']);
  icp(['canister', 'call', 'payments', 'admin_add_admin', `(principal "${principal}")`, '-e', 'local', '--identity', 'sc-deployer']);

  await page.goto('/admin');
  await expect(page.getByRole('heading', { name: 'Admin overview' })).toBeVisible({ timeout: 15_000 });
  await expect(page.getByText('tasks: active')).toBeVisible();
  await page.screenshot({ path: `${demo}admin-overview.png`, fullPage: true });

  await page.getByRole('button', { name: 'pause tasks', exact: true }).click();
  await page.getByLabel('Type "tasks" to confirm').fill('tasks');
  await page.getByRole('button', { name: 'Confirm pause tasks' }).click();
  await expect(page.getByText('tasks: paused')).toBeVisible({ timeout: 15_000 });

  await page.getByRole('button', { name: 'unpause tasks', exact: true }).click();
  await page.getByLabel('Type "tasks" to confirm').fill('tasks');
  await page.getByRole('button', { name: 'Confirm unpause tasks' }).click();
  await expect(page.getByText('tasks: active')).toBeVisible({ timeout: 15_000 });

  await page.goto('/admin/audit');
  await expect(page.getByRole('cell', { name: 'admin_pause' }).first()).toBeVisible({ timeout: 15_000 });
  await page.screenshot({ path: `${demo}admin-audit.png`, fullPage: true });
});
