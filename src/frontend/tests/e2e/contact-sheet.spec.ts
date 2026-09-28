import { expect, test, type Page } from '@playwright/test';
import { ADMIN_ROUTES, OWNER_ROUTES, PUBLIC_ROUTES } from './helpers/routes';
import { grantAdmin, signInNewUser, spawnViaDeposit } from './helpers/spawn';

const out = new URL('../../../../docs/demos/T9.6/', import.meta.url).pathname;

const VIEWPORTS = {
  desktop: { width: 1280, height: 850, columns: 4, thumb: 400 },
  mobile: { width: 390, height: 844, columns: 8, thumb: 195 },
} as const;

type Shot = { section: string; route: string; viewport: keyof typeof VIEWPORTS; png: string };

test.skip(!process.env.CONTACT_SHEET, 'run with scripts/contact-sheet.sh');

async function capture(page: Page, shots: Shot[], section: string, routes: string[]) {
  for (const route of routes) {
    for (const viewport of Object.keys(VIEWPORTS) as (keyof typeof VIEWPORTS)[]) {
      const { width, height } = VIEWPORTS[viewport];
      await page.setViewportSize({ width, height });
      await page.goto(route);
      await page.waitForLoadState('networkidle');
      await page.evaluate(() => document.fonts.ready);
      shots.push({ section, route, viewport, png: (await page.screenshot({ animations: 'disabled', caret: 'hide' })).toString('base64') });
    }
  }
}

const escape = (s: string) => s.replace(/[&<>"]/g, (c) => `&#${c.charCodeAt(0)};`);

function sheet(shots: Shot[], viewport: keyof typeof VIEWPORTS): string {
  const { columns, thumb } = VIEWPORTS[viewport];
  const sections = [...new Set(shots.map((s) => s.section))];
  const body = sections
    .map((section) => {
      const cells = shots
        .filter((s) => s.section === section && s.viewport === viewport)
        .map((s) => `<figure><img src="data:image/png;base64,${s.png}"><figcaption>${escape(s.route)}</figcaption></figure>`)
        .join('');
      return `<h2>${escape(section)}</h2><div class="grid">${cells}</div>`;
    })
    .join('');
  return `<!doctype html><style>
    body{margin:0;padding:24px;background:#0b0f1a;color:#e6e9f2;font:14px system-ui,sans-serif;width:${columns * (thumb + 16)}px}
    h1{margin:0 0 4px;font-size:22px}p{margin:0 0 16px;color:#9aa3b8}h2{margin:24px 0 8px;font-size:16px;color:#8fb4ff}
    .grid{display:grid;grid-template-columns:repeat(${columns},${thumb}px);gap:16px}
    figure{margin:0}img{width:${thumb}px;display:block;border:1px solid #2a3148;border-radius:6px}
    figcaption{margin-top:4px;font:12px ui-monospace,monospace;color:#c9d1e6;word-break:break-all}
  </style><h1>Space Compute: every screen (${viewport})</h1><p>${shots.filter((s) => s.viewport === viewport).length} screens, first viewport of each route, local deployment</p>${body}`;
}

test('T9.6 contact sheet: every screen at desktop and mobile, labeled', async ({ page, context, browser }) => {
  test.setTimeout(600_000);
  const shots: Shot[] = [];

  await page.goto('/discoveries');
  await page.waitForLoadState('networkidle');
  const firstLink = page.locator('main a[href^="/d/"]').first();
  const firstDiscovery = (await firstLink.count()) ? await firstLink.getAttribute('href') : null;
  await capture(page, shots, 'Public (signed out)', firstDiscovery ? [...PUBLIC_ROUTES, firstDiscovery] : PUBLIC_ROUTES);

  await signInNewUser(page, context, 'e2e-contact-sheet');
  await capture(page, shots, 'Spawn wizard (signed in, no agent)', ['/spawn']);
  await page.setViewportSize(VIEWPORTS.desktop);
  await page.goto('/spawn');
  await spawnViaDeposit(page, `Sheet-${Date.now() % 100_000}`);
  await capture(page, shots, 'Owner (signed in, one agent)', OWNER_ROUTES);

  const principal = await page.getByRole('button', { name: /Sign out/ }).getAttribute('title');
  if (!principal) throw new Error('Could not read the signed-in principal from the navbar.');
  const revokeAdmin = grantAdmin(principal);
  try {
    await capture(page, shots, 'Admin console', ADMIN_ROUTES);
  } finally {
    revokeAdmin();
  }

  const composer = await browser.newPage({ deviceScaleFactor: 2 });
  for (const viewport of Object.keys(VIEWPORTS) as (keyof typeof VIEWPORTS)[]) {
    await composer.setContent(sheet(shots, viewport), { waitUntil: 'load' });
    await composer.screenshot({ path: `${out}contact-sheet-${viewport}.png`, fullPage: true });
  }
  expect(shots.filter((s) => s.viewport === 'mobile')).toHaveLength(shots.length / 2);
});
