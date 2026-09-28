import { fuelLevel, type FuelLevel } from '../../lib/dashboard';
import { svgAsset } from './assets';
import styles from './FuelGauge.module.css';

export type FuelGaugeState = FuelLevel | 'paused';

const VIEW: Record<FuelGaugeState, { art: string; label: string }> = {
  healthy: { art: 'fuel_gauge_healthy', label: 'Healthy' },
  low: { art: 'fuel_gauge_low', label: 'Low fuel' },
  critical: { art: 'fuel_gauge_critical', label: 'Critical' },
  paused: { art: 'fuel_gauge_frozen', label: 'Paused' },
};

export const fuelGaugeState = (daysRemaining: number, frozen: boolean): FuelGaugeState =>
  frozen || daysRemaining <= 0 ? 'paused' : fuelLevel(daysRemaining);

export function FuelGauge({ daysRemaining, cycles, frozen = false }: { daysRemaining: number; cycles?: string; frozen?: boolean }) {
  const state = fuelGaugeState(daysRemaining, frozen);
  const { art, label } = VIEW[state];
  return (
    <div role="group" aria-label="Canister fuel" className={`${styles.gauge} ${styles[state]}`}>
      <img className={styles.art} src={svgAsset('fuel', art)} alt="" />
      <div className={styles.readout}>
        <span className={styles.label}>{label}</span>
        <span className={styles.days}>
          {state === 'paused' ? 'Out of fuel: data is safe, top up to resume' : `≈ ${daysRemaining} days of fuel`}
          {cycles && state !== 'paused' ? ` (${cycles})` : ''}
        </span>
      </div>
      <meter className="sr-only" aria-label="Days of fuel" min={0} max={30} low={3} high={14} optimum={30} value={Math.max(0, daysRemaining)} />
    </div>
  );
}
