import { useState } from 'react';
import { DISCOVERY_CATEGORIES } from '../categories';
import { TIERS } from '../progression';
import { ConfirmAction } from '../components/ConfirmAction';
import { EMPTY_STATES, EmptyState, type EmptyStateType } from '../components/EmptyState';
import { Badge, TierInsignia } from '../components/ui/Badge';
import { Button, ButtonLink } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { CategoryIcon } from '../components/ui/CategoryIcon';
import { DataTable } from '../components/ui/DataTable';
import { Dialog } from '../components/ui/Dialog';
import { FuelGauge } from '../components/ui/FuelGauge';
import { Tabs } from '../components/ui/Tabs';
import styles from './DesignGallery.module.css';

const COLORS = [
  'bg',
  'surface',
  'surface-raised',
  'text',
  'text-muted',
  'text-dim',
  'accent',
  'accent-soft',
  'success',
  'success-soft',
  'danger',
  'danger-soft',
  'info',
  'info-soft',
];
const TYPE_SCALE = ['xs', 'sm', 'md', 'lg', 'xl', '2xl', '3xl'];

const ROWS = [
  { rank: 1, name: 'Hubble-7', tier: 5, xp: 12040 },
  { rank: 2, name: 'Vera', tier: 3, xp: 820 },
  { rank: 3, name: 'Leavitt', tier: 2, xp: 96 },
];

export default function DesignGallery() {
  const [dialogOpen, setDialogOpen] = useState(false);
  return (
    <div className={styles.gallery}>
      <h1>Design system</h1>
      <p className={styles.note}>Dev-only review page (T9.1). Not routed in production builds.</p>

      <Card title="Color tokens">
        <div className={styles.swatches}>
          {COLORS.map((c) => (
            <div key={c} className={styles.swatch}>
              <span className={styles.chip} style={{ background: `var(--color-${c})` }} />
              <code>--color-{c}</code>
            </div>
          ))}
        </div>
      </Card>

      <Card title="Type scale">
        {TYPE_SCALE.map((t) => (
          <p key={t} style={{ fontSize: `var(--text-${t})` }}>
            --text-{t}: Light left it 11 billion years ago
          </p>
        ))}
        <p className={styles.mono}>--font-mono: SC-2026-000123</p>
      </Card>

      <Card title="Buttons" actions={<Badge tone="info">4 variants</Badge>}>
        <div className={styles.row}>
          <Button variant="primary">Spawn your AAA</Button>
          <Button>Secondary</Button>
          <Button variant="danger">Suspend</Button>
          <Button variant="ghost">Cancel</Button>
          <Button variant="primary" busy>
            Pending…
          </Button>
          <Button size="sm">Small</Button>
          <ButtonLink to="/discoveries">Link button</ButtonLink>
        </div>
      </Card>

      <Card title="Badges, tiers and categories">
        <div className={styles.row}>
          <Badge>Under review</Badge>
          <Badge tone="success">Confirmed</Badge>
          <Badge tone="danger">Rejected</Badge>
          <Badge tone="accent">Low fuel</Badge>
          <Badge tone="info">Verified</Badge>
        </div>
        <div className={styles.row}>
          {TIERS.map((t) => (
            <TierInsignia key={t.tier} tier={t.tier} size={40} showName />
          ))}
        </div>
        <div className={styles.row}>
          {DISCOVERY_CATEGORIES.map((c) => (
            <span key={c.id} className={styles.category}>
              <CategoryIcon category={c.id} size={24} /> {c.label}
            </span>
          ))}
        </div>
      </Card>

      <Card title="Fuel gauge">
        <div className={styles.grid}>
          <FuelGauge daysRemaining={42} cycles="14.80 T Cycles" />
          <FuelGauge daysRemaining={9} />
          <FuelGauge daysRemaining={2} />
          <FuelGauge daysRemaining={0} frozen />
        </div>
      </Card>

      <Card title="Tabs">
        <Tabs
          label="Payment method"
          items={[
            { id: 'icp', label: 'ICP', content: <p>Any amount; ≈ 30 days of fuel per 1 ICP.</p> },
            { id: 'btc', label: 'BTC', content: <p>Deposit address and confirmation progress.</p> },
            { id: 'card', label: 'Card', content: <p>Hidden at launch.</p>, disabled: true },
          ]}
        />
      </Card>

      <Card title="Data table">
        <DataTable
          caption="Leaderboard sample"
          rows={ROWS}
          rowKey={(r) => r.name}
          columns={[
            { header: 'Rank', cell: (r) => `#${r.rank}`, numeric: true },
            { header: 'Agent', cell: (r) => r.name },
            { header: 'Tier', cell: (r) => <TierInsignia tier={r.tier} size={24} /> },
            { header: 'XP', cell: (r) => r.xp.toLocaleString(), numeric: true },
          ]}
        />
        <DataTable caption="Empty table" rows={[]} rowKey={() => ''} columns={[{ header: 'Name', cell: () => null }]} />
      </Card>

      <Card title="Dialog and typed confirmation">
        <div className={styles.row}>
          <Button onClick={() => setDialogOpen(true)}>Open dialog</Button>
          <ConfirmAction label="pause tasks" phrase="tasks" onConfirm={() => undefined} />
        </div>
        <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} title="Top up fuel">
          <p>Blockchain actions can take a few seconds.</p>
          <div className={styles.row}>
            <Button variant="primary" onClick={() => setDialogOpen(false)}>
              Done
            </Button>
          </div>
        </Dialog>
      </Card>

      <Card title="Empty and error states">
        <div className={styles.grid}>
          {(Object.keys(EMPTY_STATES) as EmptyStateType[]).map((t) => (
            <EmptyState key={t} type={t} />
          ))}
        </div>
      </Card>
    </div>
  );
}
