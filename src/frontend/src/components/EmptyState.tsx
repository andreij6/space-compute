import { ButtonLink } from './ui/Button';
import { svgAsset } from './ui/assets';
import styles from './EmptyState.module.css';

export type EmptyStateType = 'not_found' | 'canister_paused' | 'agent_offline' | 'image_unavailable' | 'empty_dashboard';

interface StateDef {
  art: string;
  title: string;
  description: string;
  tone: 'default' | 'accent' | 'danger' | 'quiet';
  heading: 'h2' | 'h3' | 'p';
  actions: { to: string; label: string; primary?: boolean }[];
}

export const EMPTY_STATES: Record<EmptyStateType, StateDef> = {
  not_found: {
    art: 'state_404_lost_in_space',
    title: '404: Lost in Deep Space',
    description: 'The celestial coordinates you requested do not correspond to any known catalog entry or canister route.',
    tone: 'default',
    heading: 'h2',
    actions: [
      { to: '/', label: 'Return to Base Orbit', primary: true },
      { to: '/discoveries', label: 'Explore Museum' },
    ],
  },
  canister_paused: {
    art: 'state_canister_paused',
    title: 'AAA Canister Suspended: Fuel Depleted',
    description:
      'Your agent canister has consumed its cycle balance and entered freezing protection. Stable memory is safe. Refuel with cycles or fiat to resume autonomous observation.',
    tone: 'danger',
    heading: 'h3',
    actions: [{ to: '/fuel', label: 'Refuel Canister Now', primary: true }],
  },
  agent_offline: {
    art: 'state_agent_offline',
    title: 'Local Agent Offline',
    description:
      'No heartbeat detected from your local Claude Code or Python agent in the last 30 minutes. Your canister is waiting for instructions.',
    tone: 'accent',
    heading: 'h3',
    actions: [{ to: '/connect', label: 'View Connect Instructions' }],
  },
  image_unavailable: {
    art: 'state_image_unavailable',
    title: 'Telescope Cutout Unavailable',
    description: 'Archive FITS tile pending photometric recalibration.',
    tone: 'quiet',
    heading: 'p',
    actions: [],
  },
  empty_dashboard: {
    art: 'state_empty_dashboard',
    title: 'No activity yet',
    description: 'Your AAA has not classified anything yet. Connect your local agent to start observing.',
    tone: 'default',
    heading: 'h3',
    actions: [{ to: '/connect', label: 'Connect Agent', primary: true }],
  },
};

export function EmptyState({ type, title, description }: { type: EmptyStateType; title?: string; description?: string }) {
  const def = EMPTY_STATES[type];
  const Heading = def.heading;
  return (
    <div className={`${styles.state} ${styles[def.tone]}`}>
      <img className={styles.art} src={svgAsset('states', def.art)} alt="" />
      <Heading className={styles.title}>{title || def.title}</Heading>
      <p className={styles.description}>{description || def.description}</p>
      {def.actions.length > 0 && (
        <div className={styles.actions}>
          {def.actions.map((a) => (
            <ButtonLink key={a.to} to={a.to} variant={a.primary ? 'primary' : 'secondary'}>
              {a.label}
            </ButtonLink>
          ))}
        </div>
      )}
    </div>
  );
}
