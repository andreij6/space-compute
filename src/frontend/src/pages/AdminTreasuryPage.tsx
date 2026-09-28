import React from 'react';
import { AdminNav } from '../components/AdminNav';
import { mockTreasury } from '../mockData';

export const AdminTreasuryPage: React.FC = () => {
  const canisters = [
    { name: 'platform', id: 'qhbym-qaaaa-aaaaa-aaafq-cai', balance: '48.5 TCycles', dailyBurn: '420 GCycles', status: 'Healthy' },
    { name: 'payments', id: 'qda4v-eyaaa-aaaaa-aaaha-cai', balance: '32.1 TCycles', dailyBurn: '180 GCycles', status: 'Healthy' },
    { name: 'treasury', id: 'qjdve-lqaaa-aaaaa-aaaeq-cai', balance: '85.0 TCycles', dailyBurn: '50 GCycles', status: 'Healthy' },
    { name: 'frontend', id: 'q42bv-ciaaa-aaaaa-aaajq-cai', balance: '18.2 TCycles', dailyBurn: '95 GCycles', status: 'Healthy' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Treasury Operations</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Treasury & Cycles Float
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1.25rem' }}>
        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Treasury ICP Balance</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-mono)', fontWeight: 700, color: 'var(--amber-star)' }}>
            {mockTreasury.icpBalance} ICP
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)', marginTop: '0.25rem' }}>
            ~{mockTreasury.runwayMonths} Months Runway
          </div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Platform Daily Burn</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-mono)', fontWeight: 700, color: 'var(--cyan-nebula)' }}>
            {mockTreasury.dailyBurnCycles}
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.25rem' }}>
            Across all 4 core canisters
          </div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Core Infrastructure Canister Balances
        </h3>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Canister</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Canister ID</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Cycle Balance</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Daily Velocity</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {canisters.map((c) => (
                <tr key={c.name} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{c.name}</td>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)', color: 'var(--text-dim)' }}>{c.id}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--amber-star)', fontWeight: 600 }}>{c.balance}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>{c.dailyBurn}</td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <span className="badge badge-cyan">{c.status}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
