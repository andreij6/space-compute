import React from 'react';
import { AdminNav } from '../components/AdminNav';
import { Download } from 'lucide-react';

export const AdminAuditPage: React.FC = () => {
  const events = [
    { timestamp: '2026-09-27 01:14:02 UTC', principal: '2vxsx...cai', action: 'SPAWN_CANISTER', target: 'OrionSurveyor-01', status: 'SUCCESS' },
    { timestamp: '2026-09-26 23:45:11 UTC', principal: 'qhbym...cai', action: 'DISCOVERY_QUORUM_REACHED', target: 'SC-2026-000142', status: 'SUCCESS' },
    { timestamp: '2026-09-26 21:30:00 UTC', principal: 'admin_ops', action: 'TOGGLE_FEATURE_FLAG', target: 'CARD_PACKS', status: 'SUCCESS' },
    { timestamp: '2026-09-26 18:22:15 UTC', principal: 'qda4v...cai', action: 'STRIPE_WEBHOOK_PROCESSED', target: 'tx-8812', status: 'SUCCESS' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Immutable Event Stream</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Administrative Audit Log
        </h1>
      </div>

      <AdminNav />

      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem', flexWrap: 'wrap', gap: '1rem' }}>
          <div>
            <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              System Mutations & Governance Actions
            </h3>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Append-only audit trail verified on the Internet Computer consensus layer.
            </p>
          </div>

          <button type="button" className="btn-secondary">
            <Download size={15} />
            <span>Export Audit CSV</span>
          </button>
        </div>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '650px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Timestamp</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Actor Principal</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Action Type</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Target Entity</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {events.map((e, idx) => (
                <tr key={idx} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)', color: 'var(--text-dim)' }}>{e.timestamp}</td>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{e.principal}</td>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{e.action}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--amber-star)' }}>{e.target}</td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <span className="badge badge-cyan">{e.status}</span>
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
