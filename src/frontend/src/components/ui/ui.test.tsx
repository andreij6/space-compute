import { describe, expect, it, vi } from 'vitest';
import type { ReactNode } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Badge, TierInsignia, clampTier } from './Badge';
import { Button, ButtonLink } from './Button';
import { Card } from './Card';
import { CategoryIcon } from './CategoryIcon';
import { DataTable } from './DataTable';
import { Dialog } from './Dialog';
import { FuelGauge, fuelGaugeState } from './FuelGauge';
import { Tabs, nextEnabledIndex } from './Tabs';
import { EMPTY_STATES, EmptyState } from '../EmptyState';
import { ConfirmAction } from '../ConfirmAction';
import { PageShell } from '../PageShell';
import { isActivePath } from '../MobileBottomBar';

vi.mock('../../auth', () => ({ useAuth: () => ({ principal: null, signOut: async () => undefined }) }));
vi.mock('../../ic', () => ({ treasuryActor: () => ({ status: () => new Promise(() => undefined) }) }));

const html = (node: ReactNode) =>
  renderToStaticMarkup(
    <QueryClientProvider client={new QueryClient()}>
      <MemoryRouter initialEntries={['/leaderboard']}>{node}</MemoryRouter>
    </QueryClientProvider>,
  );

describe('T9.1 shared components', () => {
  it('Button renders a typed button, busy disables it and sets aria-busy', () => {
    expect(html(<Button>Go</Button>)).toMatch(/^<button type="button" class="[^"]+">Go<\/button>$/);
    const busy = html(<Button busy>Paying</Button>);
    expect(busy).toContain('disabled=""');
    expect(busy).toContain('aria-busy="true"');
    expect(html(<ButtonLink to="/spawn">Spawn</ButtonLink>)).toMatch(/<a [^>]*href="\/spawn"[^>]*>Spawn<\/a>/);
  });

  it('Card with a title is a labelled region with an h2', () => {
    const out = html(<Card title="Fuel">x</Card>);
    const id = out.match(/aria-labelledby="([^"]+)"/)?.[1];
    expect(out.startsWith('<section')).toBe(true);
    expect(out).toContain(`<h2 id="${id}"`);
    expect(html(<Card>plain</Card>).startsWith('<div')).toBe(true);
  });

  it('Badge renders text; TierInsignia has an accessible name and clamps the tier', () => {
    expect(html(<Badge tone="success">Confirmed</Badge>)).toContain('>Confirmed</span>');
    expect(html(<TierInsignia tier={3} />)).toContain('alt="Tier 3: Astronomer"');
    expect(html(<TierInsignia tier={5} showName />)).toMatch(/alt=""[^>]*>.*Tier 5: Principal Investigator/);
    expect(html(<TierInsignia tier={2} />)).toMatch(/src="[^"]*tier_2_observer\.svg"/);
    expect([clampTier(0), clampTier(9), clampTier(NaN), clampTier(4)]).toEqual([1, 5, 1, 4]);
  });

  it('CategoryIcon maps every platform category to an asset and is decorative unless labelled', () => {
    for (const id of ['lens', 'merger', 'ring', 'red_dot', 'clumpy', 'tidal', 'artifact']) {
      expect(html(<CategoryIcon category={id} />), id).toMatch(/<img src="[^"]+\.svg" alt=""/);
    }
    expect(html(<CategoryIcon category="lens" labelled />)).toContain('alt="Gravitational Lens"');
    expect(html(<CategoryIcon category="unknown" />)).toBe('');
  });

  it('EmptyState uses the state illustrations, keeps its headings and action links', () => {
    const notFound = html(<EmptyState type="not_found" />);
    expect(notFound).toMatch(/<img[^>]*state_404_lost_in_space\.svg"[^>]*alt=""/);
    expect(notFound).toContain('<h2');
    expect(notFound).toContain('404: Lost in Deep Space');
    expect(notFound).toMatch(/href="\/"[^>]*>Return to Base Orbit/);
    expect(html(<EmptyState type="canister_paused" title="Out of fuel" />)).toMatch(/<h3[^>]*>Out of fuel<\/h3>.*href="\/fuel"/);
    expect(html(<EmptyState type="image_unavailable" />)).not.toMatch(/<h[1-6]/);
    for (const [type, def] of Object.entries(EMPTY_STATES)) {
      expect(html(<EmptyState type={type as keyof typeof EMPTY_STATES} />), type).toContain(`${def.art}.svg`);
    }
  });

  it('FuelGauge follows 05 §3 thresholds and exposes a labelled group and meter', () => {
    expect([fuelGaugeState(30, false), fuelGaugeState(14, false), fuelGaugeState(2, false), fuelGaugeState(0, false), fuelGaugeState(40, true)]).toEqual([
      'healthy',
      'low',
      'critical',
      'paused',
      'paused',
    ]);
    const out = html(<FuelGauge daysRemaining={9} cycles="1.00 T Cycles" />);
    expect(out).toContain('role="group" aria-label="Canister fuel"');
    expect(out).toMatch(/fuel_gauge_low\.svg/);
    expect(out).toContain('<meter');
    expect(out).toContain('≈ 9 days of fuel (1.00 T Cycles)');
    expect(html(<FuelGauge daysRemaining={5} frozen />)).toMatch(/fuel_gauge_frozen\.svg.*Paused/);
  });

  it('DataTable has a caption, column headers and an empty row', () => {
    const out = html(
      <DataTable caption="Ranks" rows={[{ n: 'Vera', xp: 5 }]} rowKey={(r) => r.n} columns={[{ header: 'Agent', cell: (r) => r.n }, { header: 'XP', cell: (r) => r.xp, numeric: true }]} />,
    );
    expect(out).toContain('>Ranks</caption>');
    expect(out).toMatch(/<th scope="col"[^>]*>Agent<\/th>/);
    expect(out).toContain('>Vera</td>');
    expect(html(<DataTable caption="E" rows={[]} rowKey={() => ''} columns={[{ header: 'A', cell: () => null }]} empty="None yet." />)).toMatch(
      /<td colSpan="1"[^>]*>None yet\.<\/td>/,
    );
  });

  it('Tabs render the ARIA tab pattern and arrow navigation skips disabled tabs', () => {
    const out = html(
      <Tabs
        label="Method"
        items={[
          { id: 'icp', label: 'ICP', content: 'icp body' },
          { id: 'card', label: 'Card', content: 'card body', disabled: true },
          { id: 'btc', label: 'BTC', content: 'btc body' },
        ]}
      />,
    );
    expect(out).toContain('role="tablist" aria-label="Method"');
    expect(out.match(/role="tab"/g)).toHaveLength(3);
    expect(out).toMatch(/aria-selected="true"[^>]*>ICP</);
    expect(out).toContain('role="tabpanel"');
    expect(out).toContain('icp body');
    expect(out).not.toContain('btc body');
    const items = [{}, { disabled: true }, {}];
    expect(nextEnabledIndex(items, 0, 1)).toBe(2);
    expect(nextEnabledIndex(items, 2, 1)).toBe(0);
    expect(nextEnabledIndex(items, 0, -1)).toBe(2);
    expect(nextEnabledIndex(items, 2, -1)).toBe(0);
  });

  it('Dialog is a native dialog labelled by its heading', () => {
    const out = html(
      <Dialog open={false} onClose={() => undefined} title="Top up">
        body
      </Dialog>,
    );
    const id = out.match(/aria-labelledby="([^"]+)"/)?.[1];
    expect(out.startsWith('<dialog')).toBe(true);
    expect(out).toContain(`<h2 id="${id}"`);
  });

  it('ConfirmAction keeps its accessible button name', () => {
    expect(html(<ConfirmAction label="pause tasks" phrase="tasks" onConfirm={() => undefined} />)).toMatch(/<button type="button"[^>]*>pause tasks<\/button>/);
  });

  it('PageShell renders skip link, labelled navs, main and contentinfo with the texts e2e relies on', () => {
    const out = html(<PageShell>page</PageShell>);
    expect(out).toContain('href="#main"');
    expect(out).toContain('<main id="main"');
    expect(out).toContain('<footer');
    expect(out).toContain('aria-label="Primary"');
    expect(out).toContain('aria-label="Toggle navigation menu"');
    expect(out).toMatch(/href="\/signin"[^>]*>.*Sign in/);
    expect(out).toMatch(/aria-current="page" href="\/leaderboard"/);
    expect(out).toContain('Loading runway…');
    expect(isActivePath('/', '/dashboard')).toBe(false);
    expect(isActivePath('/admin', '/admin/aaas')).toBe(true);
  });
});
