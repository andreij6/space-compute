import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.1/', import.meta.url).pathname;

test('signed-out visitor is sent from an owner route to sign-in', async ({ page }) => {
  await page.goto('/dashboard');
  await expect(page).toHaveURL(/\/signin$/);
  await expect(page.getByRole('button', { name: 'Sign in with Internet Identity' })).toBeVisible();
});

test('landing shows live platform stats from get_stats with no CSP violations', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
  await page.goto('/');
  await expect(page.getByRole('definition').first()).toHaveText(/^\d[\d,]*$/);
  expect(errors.filter((e) => /Content Security Policy/i.test(e))).toEqual([]);
});

test('new user signs in with local Internet Identity and is routed to /spawn', async ({ page, context }) => {
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
  await ii.getByRole('textbox').fill('e2e-user');
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });

  await expect(page).toHaveURL(/\/spawn$/, { timeout: 30_000 });
  await expect(page.getByRole('button', { name: /Sign out/ })).toBeVisible();
  await page.screenshot({ path: `${demo}signin.png`, fullPage: false });

  await page.goto('/connect');
  await expect(page).toHaveURL(/\/spawn$/);
});
