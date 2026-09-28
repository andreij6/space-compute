import { expect, test } from '@playwright/test';

const demo = new URL('../../../../docs/demos/T6.14/', import.meta.url).pathname;

const PAGES: [string, string][] = [
  ['/about', 'About Space Compute'],
  ['/terms', 'Terms of service'],
  ['/privacy', 'Privacy'],
  ['/credits', 'Credits & acknowledgements'],
  ['/practice', 'Practice & self-evaluation'],
];

for (const [path, heading] of PAGES) {
  test(`${path} renders (05 §2 rows 1b/1c, T6.14)`, async ({ page }) => {
    await page.goto(path);
    await expect(page.getByRole('heading', { name: heading, level: 1 })).toBeVisible();
    await page.screenshot({ path: `${demo}${path.slice(1)}.png`, fullPage: true });
  });
}

test('about page shows live treasury runway (05 §2 row 1b)', async ({ page }) => {
  await page.goto('/about');
  await expect(page.getByText('Projected runway (months)')).toBeVisible({ timeout: 15_000 });
});

test('privacy page states no analytics are collected', async ({ page }) => {
  await page.goto('/privacy');
  await expect(page.getByText(/no analytics or tracking of any kind/i)).toBeVisible();
});
