import React from 'react';
import { AdminNav } from '../components/AdminNav';
import { AlertTriangle, RefreshCw, RotateCcw } from 'lucide-react';

export const AdminPaymentsPage: React.FC = () => {
  const sagas = [
    { id: 'saga-1044', user: '2vxsx...cai', method: 'ckBTC', amount: '0.00015 BTC', error: 'Callback timeout on CMC minting', status: 'stalled' },
    { id: 'saga-1042', user: 'rrkah...cai', method: 'Stripe', amount: '$5.00 USD', error: 'Webhook signature verification retry', status: 'retrying' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Financial Sagas</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Payments & Token Ledgers
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1.25rem' }}>
        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Treasury ICP</div>
          <div style={{ fontSize: '1.6rem', fontFamily: 'var(--font-mono)', fontWeight: 700, color: 'var(--amber-star)' }}>
            4,280.5 ICP
          </div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>ckBTC Ledger Balance</div>
          <div style={{ fontSize: '1.6rem', fontFamily: 'var(--font-mono)', fontWeight: 700, color: 'var(--cyan-nebula)' }}>
            0.425 BTC
          </div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>ckETH Ledger Balance</div>
          <div style={{ fontSize: '1.6rem', fontFamily: 'var(--font-mono)', fontWeight: 700, color: 'var(--blue-cosmic)' }}>
            8.120 ETH
          </div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <AlertTriangle size={18} style={{ color: 'var(--amber-star)' }} />
          <span>Stalled Saga Operations</span>
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1rem' }}>
          Inter-canister payment sagas requiring manual administrative intervention or rollback.
        </p>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Saga ID</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Target User</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Method</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Failure Detail</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {sagas.map((s) => (
                <tr key={s.id} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{s.id}</td>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{s.user}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>{s.method} ({s.amount})</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--red-nova)' }}>{s.error}</td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <div style={{ display: 'inline-flex', gap: '0.4rem' }}>
                      <button type="button" className="btn-secondary" style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}>
                        <RefreshCw size={12} />
                        <span>Resume</span>
                      </button>
                      <button type="button" className="btn-secondary" style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem', color: 'var(--red-nova)' }}>
                        <RotateCcw size={12} />
                        <span>Refund</span>
                      </button>
                    </div>
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
