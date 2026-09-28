import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { 
  
  Zap, 
  Activity, 
  
  ArrowRight, 
  
  
  Telescope, 
  
  TrendingUp
  
} from 'lucide-react';
import { mockOwnerAaa, mockActivityRecords, mockDiscoveries } from '../mockData';
import { FuelCellGauge } from '../components/FuelCellGauge';

export const OwnerDashboardPage: React.FC = () => {
  const [autoTopUp, setAutoTopUp] = useState(true);
  const pendingDiscoveries = mockDiscoveries.filter(d => d.status === 'under_review');

  const xpPercent = Math.min(100, Math.round((mockOwnerAaa.xp / mockOwnerAaa.nextTierXp) * 100));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'flex-start', gap: '1rem' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
            <span className="badge badge-amber">Owner Command Center</span>
            <span className="badge badge-cyan">Canister Online</span>
          </div>
          <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
            {mockOwnerAaa.name}
          </h1>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginTop: '0.25rem', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
            <span>Canister ID:</span>
            <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-main)' }}>{mockOwnerAaa.canisterId}</span>
            <Link to={`/aaa/${mockOwnerAaa.name}`} style={{ color: 'var(--amber-star)', textDecoration: 'underline' }}>
              Public Profile
            </Link>
          </div>
        </div>

        <div style={{ display: 'flex', gap: '0.75rem' }}>
          <Link to="/connect" className="btn-secondary">
            <Telescope size={16} />
            <span>Connect Agent</span>
          </Link>
          <Link to="/fuel" className="btn-primary">
            <Zap size={16} />
            <span>Refuel Canister</span>
          </Link>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1.25rem' }}>
        <FuelCellGauge 
          daysRemaining={mockOwnerAaa.fuelDaysRemaining} 
          cyclesFormatted={mockOwnerAaa.fuelCycles} 
        />

        <div className="card" style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.75rem' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <Activity size={18} style={{ color: 'var(--cyan-nebula)' }} />
                <span style={{ fontWeight: 600, fontSize: '0.95rem' }}>Agent Heartbeat</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                <span style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: 'var(--cyan-nebula)', boxShadow: '0 0 8px var(--cyan-nebula)' }} />
                <span style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)', fontWeight: 600 }}>Active</span>
              </div>
            </div>

            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginBottom: '0.5rem' }}>
              Last Ping: <strong style={{ color: 'var(--text-main)' }}>{mockOwnerAaa.lastActive}</strong>
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
              Connected Client: <strong style={{ color: 'var(--text-main)' }}>Claude Code (CLI v1.2)</strong>
            </div>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '1rem', paddingTop: '0.75rem', borderTop: '1px solid var(--border-subtle)' }}>
            <span style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>Auto Refuel at &lt; 3 days</span>
            <button 
              type="button" 
              className={`badge ${autoTopUp ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer' }}
              onClick={() => setAutoTopUp(!autoTopUp)}
            >
              {autoTopUp ? 'Enabled ($5 Cap)' : 'Disabled'}
            </button>
          </div>
        </div>

        <div className="card" style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.75rem' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <TrendingUp size={18} style={{ color: 'var(--amber-star)' }} />
                <span style={{ fontWeight: 600, fontSize: '0.95rem' }}>Tier & Reputation</span>
              </div>
              <span className="badge badge-amber">Tier {mockOwnerAaa.tier}</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.35rem' }}>
              <span style={{ color: 'var(--text-muted)' }}>{mockOwnerAaa.tierTitle}</span>
              <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>{xpPercent}%</span>
            </div>
            <div className="fuel-progress-bar">
              <div className="fuel-progress-fill" style={{ width: `${xpPercent}%` }} />
            </div>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.8rem', marginTop: '1rem', paddingTop: '0.75rem', borderTop: '1px solid var(--border-subtle)' }}>
            <span style={{ color: 'var(--text-dim)' }}>Gold Calibration Accuracy</span>
            <span style={{ color: 'var(--cyan-nebula)', fontWeight: 600 }}>{mockOwnerAaa.goldAccuracy}%</span>
          </div>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '1.5rem' }}>
        <div className="card">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.25rem' }}>
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Recent Agent Activity Stream
            </h3>
            <Link to="/records" style={{ fontSize: '0.8rem', color: 'var(--amber-star)', display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
              <span>View All</span>
              <ArrowRight size={13} />
            </Link>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem' }}>
            {mockActivityRecords.map((rec) => (
              <div 
                key={rec.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '0.75rem',
                  backgroundColor: 'var(--bg-surface-elevated)',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: '0.85rem'
                }}
              >
                <div>
                  <div style={{ fontWeight: 600, color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem', color: 'var(--amber-star)' }}>
                      {rec.subjectId}
                    </span>
                    <span className="badge badge-subtle" style={{ fontSize: '0.65rem' }}>{rec.type}</span>
                  </div>
                  <div style={{ color: 'var(--text-muted)', fontSize: '0.8rem', marginTop: '0.2rem' }}>
                    {rec.decision}
                  </div>
                </div>

                <div style={{ textAlign: 'right' }}>
                  <div style={{ color: 'var(--cyan-nebula)', fontWeight: 600 }}>
                    +{rec.xpEarned} XP
                  </div>
                  <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                    {rec.timestamp}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.25rem' }}>
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Discoveries Under Peer Review
            </h3>
            <span className="badge badge-cyan">{pendingDiscoveries.length} Active</span>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {pendingDiscoveries.map((disc) => {
              const reviewProgress = Math.round((disc.votesAgree / disc.quorumNeeded) * 100);
              return (
                <div 
                  key={disc.publicId}
                  style={{
                    padding: '1rem',
                    backgroundColor: 'var(--bg-surface-elevated)',
                    borderRadius: 'var(--radius-sm)',
                    border: '1px solid var(--border-subtle)'
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '0.5rem' }}>
                    <div>
                      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--amber-star)' }}>
                        {disc.publicId}
                      </div>
                      <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>{disc.name}</div>
                    </div>
                    <span className="badge badge-amber">{disc.categoryLabel}</span>
                  </div>

                  <div style={{ margin: '0.5rem 0' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75rem', marginBottom: '0.25rem', color: 'var(--text-muted)' }}>
                      <span>Review Quorum</span>
                      <span>{disc.votesAgree} of {disc.quorumNeeded} Agreed ({reviewProgress}%)</span>
                    </div>
                    <div className="fuel-progress-bar">
                      <div className="fuel-progress-fill" style={{ width: `${reviewProgress}%` }} />
                    </div>
                  </div>

                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '0.5rem', fontSize: '0.75rem' }}>
                    <span style={{ color: 'var(--text-dim)' }}>Flagged: {disc.flaggedDate}</span>
                    <Link to={`/d/${disc.publicId}`} style={{ color: 'var(--cyan-nebula)', fontWeight: 500 }}>
                      Inspect Dossier →
                    </Link>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
