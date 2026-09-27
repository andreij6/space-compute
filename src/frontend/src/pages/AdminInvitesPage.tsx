import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { Ticket, Download, Plus, CheckCircle2 } from 'lucide-react';

export const AdminInvitesPage: React.FC = () => {
  const [batches, setBatches] = useState([
    { id: 'batch-01', codePrefix: 'BETA-STAR', total: 100, redeemed: 64, sponsorBudget: '2,000 TCycles', created: '2026-09-01' },
    { id: 'batch-02', codePrefix: 'ASTRONOMY-CONF', total: 50, redeemed: 12, sponsorBudget: '1,000 TCycles', created: '2026-09-15' },
  ]);

  const [createdNotice, setCreatedNotice] = useState(false);

  const handleMintBatch = () => {
    setCreatedNotice(true);
    setTimeout(() => setCreatedNotice(false), 2500);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Beta Access</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Sponsored Invites Generator
        </h1>
      </div>

      <AdminNav />

      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem', flexWrap: 'wrap', gap: '1rem' }}>
          <div>
            <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Mint New Sponsored Code Batch
            </h3>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Subsidize AAA canister provisioning for educational, academic, or research partners.
            </p>
          </div>

          <button type="button" className="btn-primary" onClick={handleMintBatch}>
            <Plus size={16} />
            <span>Mint Batch of 50 Codes</span>
          </button>
        </div>

        {createdNotice && (
          <div style={{ color: 'var(--cyan-nebula)', fontSize: '0.85rem', marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
            <CheckCircle2 size={16} /> Generated 50 new single-use invite codes subsidized by Treasury!
          </div>
        )}

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Batch ID</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Prefix</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Redeemed / Total</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Sponsor Budget</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Export</th>
              </tr>
            </thead>
            <tbody>
              {batches.map((b) => (
                <tr key={b.id} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{b.id}</td>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{b.codePrefix}-****</td>
                  <td style={{ padding: '0.85rem 1rem' }}>
                    <span style={{ color: 'var(--amber-star)', fontWeight: 600 }}>{b.redeemed}</span> / {b.total}
                  </td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--cyan-nebula)' }}>{b.sponsorBudget}</td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <button type="button" className="btn-secondary" style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}>
                      <Download size={12} />
                      <span>CSV</span>
                    </button>
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
