import { test } from '@playwright/test';
import { grantAdmin, signInAndSpawn, signInNewUser } from './helpers/spawn';
import { expectAccessible, trackConsoleErrors, visitAndCheck } from './helpers/a11y';

const SIGNED_OUT_ROUTES = [
  '/',
  '/discoveries',
  '/d/SC-0000-000000',
  '/aaa/2vxsx-fae',
  '/leaderboard',
  '/signin',
  '/about',
  '/practice',
  '/terms',
  '/privacy',
  '/credits',
];

const ADMIN_ROUTES = [
  '/admin',
  '/admin/aaas',
  '/admin/discoveries',
  '/admin/data',
  '/admin/payments',
  '/admin/releases',
  '/admin/settings',
  '/admin/invites',
  '/admin/treasury',
  '/admin/moderation',
  '/admin/audit',
];

for (const route of SIGNED_OUT_ROUTES) {
  test(`a11y smoke: signed-out ${route} has no console errors and no serious/critical axe violations`, async ({
    page,
  }) => {
    await visitAndCheck(page, route);
  });
}

test('a11y smoke: signed-in owner routes (spawn, dashboard, connect, records, fuel)', async ({ page, context }) => {
  test.setTimeout(150_000);
  const errors = trackConsoleErrors(page);

  await page.goto('/spawn');
  await page.waitForLoadState('networkidle');
  await expectAccessible(page, errors);

  const name = `E2E-A11y-${Date.now() % 100_000}`;
  await signInAndSpawn(page, context, 'e2e-a11y-owner', name);

  for (const route of ['/dashboard', '/connect', '/records', '/fuel']) {
    await page.goto(route);
    await page.waitForLoadState('networkidle');
    await expectAccessible(page, errors);
  }
});

test('a11y smoke: admin console routes', async ({ page, context }) => {
  test.setTimeout(150_000);
  await signInNewUser(page, context, 'e2e-a11y-admin');
  const principal = await page.getByRole('button', { name: /Sign out/ }).getAttribute('title');
  if (!principal) throw new Error('Could not read the signed-in principal from the navbar.');

  const revokeAdmin = grantAdmin(principal);
  try {

    const errors = trackConsoleErrors(page);
    for (const route of ADMIN_ROUTES) {
      await page.goto(route);
      await page.waitForLoadState('networkidle');
      await expectAccessible(page, errors);
      errors.length = 0;
    }
  } finally {
    revokeAdmin();
  }
});
