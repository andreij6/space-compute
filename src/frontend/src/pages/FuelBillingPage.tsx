import React, { useState } from 'react';
import { 
  
  
  
  
  
  
  Sliders 
  
} from 'lucide-react';
import { mockOwnerAaa } from '../mockData';
import { FuelCellGauge } from '../components/FuelCellGauge';
import { MultiCurrencyPayment } from '../components/MultiCurrencyPayment';

export const FuelBillingPage: React.FC = () => {
  const [autoRefuelActive, setAutoRefuelActive] = useState(true);
  const [spendCap, setSpendCap] = useState(15);
  const [thresholdDays, setThresholdDays] = useState(3);

  const history = [
    { id: 'tx-8812', date: '2026-09-01', method: 'ICP Direct', amount: '0.45 ICP', cycles: '20 TCycles', status: 'Completed' },
    { id: 'tx-7740', date: '2026-08-01', method: 'Card (Stripe)', amount: '$5.00 USD', cycles: '20 TCycles', status: 'Completed' },
    { id: 'tx-6602', date: '2026-07-01', method: 'Sponsored Beta', amount: 'Free Grant', cycles: '30 TCycles', status: 'Completed' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Cycles & Ledger Management</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Fuel & Billing
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Manage your AAA canister's cycle fuel, configure automated top-up rules, and review on-chain payment history.
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '1.5rem' }}>
        <FuelCellGauge 
          daysRemaining={mockOwnerAaa.fuelDaysRemaining} 
          cyclesFormatted={mockOwnerAaa.fuelCycles}
          showRefuelButton={false}
        />

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <Sliders size={18} style={{ color: 'var(--amber-star)' }} />
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Automated Refuel Safeguards
            </h3>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Auto Top-Up Protection</div>
                <div style={{ color: 'var(--text-dim)', fontSize: '0.8rem' }}>Prevents canister freezing</div>
              </div>
              <button
                type="button"
                className={`badge ${autoRefuelActive ? 'badge-cyan' : 'badge-subtle'}`}
                style={{ cursor: 'pointer' }}
                onClick={() => setAutoRefuelActive(!autoRefuelActive)}
              >
                {autoRefuelActive ? 'Active' : 'Disabled'}
              </button>
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.35rem' }}>
                <span style={{ color: 'var(--text-muted)' }}>Trigger when runway drops below:</span>
                <span style={{ fontWeight: 600 }}>{thresholdDays} Days</span>
              </div>
              <input
                type="range"
                min="1"
                max="7"
                value={thresholdDays}
                onChange={(e) => setThresholdDays(Number(e.target.value))}
                style={{ width: '100%', accentColor: 'var(--amber-star)' }}
              />
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.35rem' }}>
                <span style={{ color: 'var(--text-muted)' }}>Monthly spending cap:</span>
                <span style={{ fontWeight: 600 }}>${spendCap}.00 USD</span>
              </div>
              <input
                type="range"
                min="5"
                max="50"
                step="5"
                value={spendCap}
                onChange={(e) => setSpendCap(Number(e.target.value))}
                style={{ width: '100%', accentColor: 'var(--amber-star)' }}
              />
            </div>
          </div>
        </div>
      </div>

      <MultiCurrencyPayment />

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Payment & Top-Up History
        </h3>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Date</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Tx ID</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Method</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Amount</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Cycles Added</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {history.map((tx) => (
                <tr key={tx.id} style={{ borderBottom: '1px solid var(--border-subtle)', fontSize: '0.85rem' }}>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>{tx.date}</td>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{tx.id}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>{tx.method}</td>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{tx.amount}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--amber-star)', fontWeight: 600 }}>{tx.cycles}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>
                    <span className="badge badge-cyan">{tx.status}</span>
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
