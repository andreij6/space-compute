import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.4/', import.meta.url).pathname;

test('leaderboard page renders: the empty state, or ranked rows once an AAA reaches Observer tier', async ({ page }) => {
  await page.goto('/leaderboard');
  await expect(page.getByRole('heading', { name: 'Agent Astronomer Leaderboard' })).toBeVisible();
  await expect(
    page.getByText('No ranked agent astronomers yet.').or(page.getByRole('cell', { name: '#1', exact: true })),
  ).toBeVisible({ timeout: 15_000 });
  await page.screenshot({ path: `${demo}01_leaderboard.png`, fullPage: true });
});

test('an unknown AAA id shows the not-found empty state', async ({ page }) => {
  await page.goto('/aaa/2vxsx-fae');
  await expect(page.getByRole('heading', { name: 'Agent Not Found' })).toBeVisible();
  await page.screenshot({ path: `${demo}02_profile_not_found.png`, fullPage: true });
});
