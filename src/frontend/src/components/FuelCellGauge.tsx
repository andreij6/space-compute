import React from 'react';
import { BatteryCharging, AlertTriangle, ShieldAlert, Zap } from 'lucide-react';
import { Link } from 'react-router-dom';

interface FuelCellGaugeProps {
  daysRemaining: number;
  cyclesFormatted?: string;
  showRefuelButton?: boolean;
}

export const FuelCellGauge: React.FC<FuelCellGaugeProps> = ({ 
  daysRemaining, 
  cyclesFormatted = '14.8 TCycles',
  showRefuelButton = true 
}) => {
  let status: 'healthy' | 'low' | 'critical' | 'paused' = 'healthy';
  let color = 'var(--cyan-nebula)';
  let bgGlow = 'var(--cyan-glow)';
  let label = 'Healthy (Autonomy Confirmed)';
  let icon = <BatteryCharging size={18} style={{ color: 'var(--cyan-nebula)' }} />;

  if (daysRemaining <= 0) {
    status = 'paused';
    color = 'var(--text-dim)';
    bgGlow = 'rgba(255, 255, 255, 0.05)';
    label = 'Canister Paused (Cycles Depleted)';
    icon = <ShieldAlert size={18} style={{ color: 'var(--red-nova)' }} />;
  } else if (daysRemaining < 3) {
    status = 'critical';
    color = 'var(--red-nova)';
    bgGlow = 'rgba(255, 107, 107, 0.2)';
    label = 'Critical (Freeze Imminent)';
    icon = <ShieldAlert size={18} style={{ color: 'var(--red-nova)' }} />;
  } else if (daysRemaining <= 14) {
    status = 'low';
    color = 'var(--amber-star)';
    bgGlow = 'var(--amber-glow)';
    label = 'Low Fuel (Refuel Recommended)';
    icon = <AlertTriangle size={18} style={{ color: 'var(--amber-star)' }} />;
  }

  const fillPercent = Math.min(100, Math.max(0, (daysRemaining / 30) * 100));

  return (
    <div className="card" style={{ padding: '1.25rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.75rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          {icon}
          <span style={{ fontWeight: 600, fontSize: '0.95rem' }}>Canister Fuel Cell</span>
        </div>
        <span 
          style={{ 
            fontSize: '0.75rem', 
            fontWeight: 700, 
            padding: '0.2rem 0.5rem', 
            borderRadius: 'var(--radius-full)',
            backgroundColor: bgGlow,
            color: color,
            border: `1px solid ${color}40`,
            textTransform: 'uppercase'
          }}
        >
          {status}
        </span>
      </div>

      <div style={{ margin: '0.75rem 0' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.35rem' }}>
          <span style={{ color: 'var(--text-muted)' }}>Estimated Runway</span>
          <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--text-main)' }}>
            {daysRemaining} Days ({cyclesFormatted})
          </span>
        </div>

        <div className="fuel-progress-bar">
          <div 
            className="fuel-progress-fill" 
            style={{ 
              width: `${fillPercent}%`,
              background: status === 'critical' ? 'var(--red-nova)' : undefined
            }} 
          />
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '0.75rem', fontSize: '0.8rem' }}>
        <span style={{ color: 'var(--text-dim)' }}>{label}</span>
        {showRefuelButton && (
          <Link to="/fuel" className="btn-secondary" style={{ padding: '0.35rem 0.75rem', fontSize: '0.8rem' }}>
            <Zap size={13} style={{ color: 'var(--amber-star)' }} />
            <span>Refuel</span>
          </Link>
        )}
      </div>
    </div>
  );
};
