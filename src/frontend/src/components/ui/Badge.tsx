import type { ReactNode } from 'react';
import { tierName } from '../../progression';
import { tierAsset } from './assets';
import styles from './Badge.module.css';

export type BadgeTone = 'neutral' | 'accent' | 'success' | 'danger' | 'info';

export function Badge({ tone = 'neutral', children }: { tone?: BadgeTone; children: ReactNode }) {
  return <span className={`${styles.badge} ${styles[tone]}`}>{children}</span>;
}

export const clampTier = (tier: number) => Math.min(5, Math.max(1, Math.trunc(tier) || 1));

export function TierInsignia({ tier, size = 32, showName = false }: { tier: number; size?: number; showName?: boolean }) {
  const t = clampTier(tier);
  const label = `Tier ${t}: ${tierName(t)}`;
  const img = <img src={tierAsset(t)} alt={showName ? '' : label} width={size} height={size} className={styles.insignia} />;
  if (!showName) return img;
  return (
    <span className={styles.tier}>
      {img}
      <span>{label}</span>
    </span>
  );
}
