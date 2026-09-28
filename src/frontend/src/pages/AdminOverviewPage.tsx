import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { 
  ShieldAlert 
  
  
  
  
  
  
} from 'lucide-react';
import { mockTreasury } from '../mockData';

export const AdminOverviewPage: React.FC = () => {
  const [taskIntakePaused, setTaskIntakePaused] = useState(false);
  const [reviewDispatchPaused, setReviewDispatchPaused] = useState(false);
  const [fiatPaused, setFiatPaused] = useState(false);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Admin Ops</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          System Telemetry & Controls
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1.25rem' }}>
        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Active AAAs</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--amber-star)', margin: '0.25rem 0' }}>
            1,240
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)' }}>+48 new this week</div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>24h Classifications</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--cyan-nebula)', margin: '0.25rem 0' }}>
            18,450
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Throughput: 12.8 ops/sec</div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Review Queue Depth</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: '#f3f5fa', margin: '0.25rem 0' }}>
            24
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--amber-star)' }}>Avg resolution: 4.2h</div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Platform Runway</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--blue-cosmic)', margin: '0.25rem 0' }}>
            {mockTreasury.runwayMonths} Mo.
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)' }}>Treasury Healthy</div>
        </div>
      </div>

      <div className="card" style={{ borderColor: 'var(--border-active)' }}>
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <ShieldAlert size={18} style={{ color: 'var(--red-nova)' }} />
          <span>Emergency Circuit Breakers</span>
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1.25rem' }}>
          Toggle global circuit breakers to halt specific canister functions during upstream incidents or maintenance.
        </p>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' }}>
          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Task Intake Pipeline</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Halts image batch distribution</div>
            </div>
            <button
              type="button"
              className={`badge ${taskIntakePaused ? 'badge-amber' : 'badge-cyan'}`}
              style={{ cursor: 'pointer', padding: '0.4rem 0.75rem' }}
              onClick={() => setTaskIntakePaused(!taskIntakePaused)}
            >
              {taskIntakePaused ? 'PAUSED' : 'ACTIVE'}
            </button>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Peer Review Dispatch</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Freezes consensus voting</div>
            </div>
            <button
              type="button"
              className={`badge ${reviewDispatchPaused ? 'badge-amber' : 'badge-cyan'}`}
              style={{ cursor: 'pointer', padding: '0.4rem 0.75rem' }}
              onClick={() => setReviewDispatchPaused(!reviewDispatchPaused)}
            >
              {reviewDispatchPaused ? 'PAUSED' : 'ACTIVE'}
            </button>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Non-ICP Stripe / Card</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Suspends fiat webhooks</div>
            </div>
            <button
              type="button"
              className={`badge ${fiatPaused ? 'badge-amber' : 'badge-cyan'}`}
              style={{ cursor: 'pointer', padding: '0.4rem 0.75rem' }}
              onClick={() => setFiatPaused(!fiatPaused)}
            >
              {fiatPaused ? 'PAUSED' : 'ACTIVE'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
