import { expect, test } from '@playwright/test';
import { icp } from './helpers/spawn';
import { runway } from '../../src/lib/runway';

const demo = new URL('../../../../docs/demos/T6.3/', import.meta.url).pathname;

test('landing shows platform stats and a recent-discoveries section', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: /Recent Confirmed Discoveries/i })).toBeVisible();
  await expect(page.getByRole('definition').first()).toHaveText(/^\d[\d,]*$/);
  await page.screenshot({ path: `${demo}01_landing.png`, fullPage: true });
});

test('discovery museum feed renders with category and status filters', async ({ page }) => {
  await page.goto('/discoveries');
  await expect(page.getByRole('heading', { name: 'Discovery Museum' })).toBeVisible();
  await expect(page.getByRole('group', { name: 'Category filter' })).toBeVisible();
  await expect(page.getByRole('group', { name: 'Status filter' })).toBeVisible();
  await page.screenshot({ path: `${demo}02_museum.png`, fullPage: true });
});

test('an unknown discovery public id shows the not-found empty state', async ({ page }) => {
  await page.goto('/d/SC-0000-000000');
  await expect(page.getByRole('heading', { name: 'Discovery Not Found' })).toBeVisible();
  await page.screenshot({ path: `${demo}03_detail_not_found.png`, fullPage: true });
});

test('the served CSP narrows img-src to self, the data bucket and data:, and connect-src reaches the local bucket (05 §1)', async ({
  page,
}) => {
  const response = await page.goto('/');
  const csp = (await response!.allHeaders())['content-security-policy'] ?? '';
  const directive = (name: string) =>
    csp
      .split(';')
      .map((d) => d.trim())
      .find((d) => d.startsWith(`${name} `)) ?? '';
  expect(directive('img-src')).toMatch(/^img-src 'self'( https:\/\/data\.[^ ]+)? http:\/\/127\.0\.0\.1:8765 data:$/);
  expect(directive('connect-src')).toContain('http://127.0.0.1:8765');
});

test('the footer runway comes from treasury.status, not mock data (05 §2 row 1b)', async ({ page }) => {
  const out = icp(['canister', 'call', 'treasury', 'status', '()', '-e', 'local', '--identity', 'sc-user', '--query']);
  const months = out.match(/projected_runway_months = ([\d_]+)/)?.[1]?.replaceAll('_', '');
  expect(months, out).toBeDefined();
  await page.goto('/');
  await expect(page.getByRole('contentinfo')).toContainText(`Runway: ${runway(Number(months), 'mo.')}`);
});
