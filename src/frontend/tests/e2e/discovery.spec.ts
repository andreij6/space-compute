import { expect, test } from '@playwright/test';

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
