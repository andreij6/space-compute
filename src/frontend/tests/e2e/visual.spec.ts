import { expect, test, type Locator, type Page } from '@playwright/test';
import { grantAdmin, signInNewUser, spawnViaDeposit } from './helpers/spawn';

const demo = new URL('../../../../docs/demos/T9.5/', import.meta.url).pathname;

const VIEWPORTS = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1280, height: 850 },
] as const;

const NARROWEST = { width: 360, height: 800 };

const PUBLIC_ROUTES = [
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
  '/no-such-route',
];

const OWNER_ROUTES = ['/dashboard', '/connect', '/records', '/fuel'];

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

const DEMO_ROUTES = new Set(['/', '/dashboard', '/fuel', '/admin']);

const dynamicData = (page: Page): Locator[] => [
  page.locator('dd'),
  page.locator('code'),
  page.locator('time'),
  page.locator('table tbody'),
  page.locator('[role="status"]'),
  page.locator('[role="meter"]'),
  page.locator('input[readonly]'),
  page.locator('main img:not([alt=""])'),
  page.locator('main a[href^="/d/"]'),
  page.locator('main a[href^="/aaa/"]'),
  page.getByText(/Mo\. Runway|Loading runway|Runway unavailable/),
  page.getByRole('button', { name: /Sign out/ }),
];

const signedInData = (page: Page): Locator[] => [page.locator('main h1'), page.locator('main li'), page.locator('main p')];

const slug = (route: string) => (route === '/' ? 'home' : route.slice(1).replace(/[^a-z0-9]+/gi, '-'));

async function settle(page: Page, route: string) {
  await page.goto(route);
  await page.waitForLoadState('networkidle');
  await page.evaluate(() => document.fonts.ready);
}

async function expectNoHorizontalScroll(page: Page, route: string) {
  await page.setViewportSize(NARROWEST);
  await settle(page, route);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow, `horizontal page scroll at 360 px on ${route}`).toBeLessThanOrEqual(0);
}

async function expectTapTargets(page: Page, route: string) {
  const small = await page
    .locator('main button:visible, header button:visible, nav a:visible, [role="tab"]:visible')
    .evaluateAll((els) =>
      els
        .map((el) => ({ el, box: el.getBoundingClientRect() }))
        .filter(({ box }) => box.width > 0 && (box.height < 44 || box.width < 44))
        .map(({ el, box }) => `${el.textContent?.trim() || el.getAttribute('aria-label')} ${Math.round(box.width)}x${Math.round(box.height)}`),
    );
  expect(small, `tap targets under 44 px at mobile width on ${route}`).toEqual([]);
}

async function snapRoutes(page: Page, routes: string[], extraMasks: (page: Page) => Locator[] = () => []) {
  for (const route of routes) {
    await expectNoHorizontalScroll(page, route);
    for (const vp of VIEWPORTS) {
      await page.setViewportSize(vp);
      await settle(page, route);
      if (vp.name === 'mobile') await expectTapTargets(page, route);
      await expect(page).toHaveScreenshot(`${slug(route)}-${vp.name}.png`, {
        fullPage: true,
        animations: 'disabled',
        caret: 'hide',
        maxDiffPixelRatio: 0.02,
        mask: [...dynamicData(page), ...extraMasks(page)],
      });
      if (DEMO_ROUTES.has(route)) await page.screenshot({ path: `${demo}${slug(route)}-${vp.name}.png`, fullPage: true });
    }
  }
}

test('visual baseline: signed-out routes at 390 and 1280, no horizontal scroll at 360', async ({ page }) => {
  test.setTimeout(240_000);
  await settle(page, '/discoveries');
  const firstDiscovery = await page.locator('main a[href^="/d/"]').first().getAttribute('href').catch(() => null);
  const routes = firstDiscovery ? [...PUBLIC_ROUTES, firstDiscovery] : PUBLIC_ROUTES;
  await snapRoutes(page, routes, (p) => (p.url().includes('/d/') ? [p.locator('main p')] : []));
});

test('visual baseline: spawn wizard and owner routes', async ({ page, context }) => {
  test.setTimeout(300_000);
  await signInNewUser(page, context, 'e2e-visual-owner');
  await snapRoutes(page, ['/spawn'], signedInData);
  await page.setViewportSize(VIEWPORTS[1]);
  await page.goto('/spawn');
  await spawnViaDeposit(page, `Visual-${Date.now() % 100_000}`);
  await snapRoutes(page, OWNER_ROUTES, signedInData);
});

test('visual baseline: admin console at 390 and 1280', async ({ page, context }) => {
  test.setTimeout(300_000);
  await signInNewUser(page, context, 'e2e-visual-admin');
  const principal = await page.getByRole('button', { name: /Sign out/ }).getAttribute('title');
  if (!principal) throw new Error('Could not read the signed-in principal from the navbar.');
  const revokeAdmin = grantAdmin(principal);
  try {
    await snapRoutes(page, ADMIN_ROUTES, signedInData);
  } finally {
    revokeAdmin();
  }
});
