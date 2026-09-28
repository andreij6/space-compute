import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { Save } from 'lucide-react';

export const AdminSettingsPage: React.FC = () => {
  const [flags, setFlags] = useState({
    cardPacks: true,
    btcPacks: true,
    ethPacks: true,
    sponsoredInvites: true
  });

  const [params, setParams] = useState({
    retirementThresholdK: 5,
    goldInjectionRatePercent: 4.0,
    reviewQuorum: 12
  });

  const [saved, setSaved] = useState(false);

  const toggleFlag = (key: keyof typeof flags) => {
    setFlags(f => ({ ...f, [key]: !f[key] }));
  };

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2500);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Platform Config</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Global Settings & Feature Flags
        </h1>
      </div>

      <AdminNav />

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Dynamic Feature Flags
        </h3>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1rem' }}>
          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Card Fuel Packs</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Stripe fiat payments</div>
            </div>
            <button
              type="button"
              className={`badge ${flags.cardPacks ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer' }}
              onClick={() => toggleFlag('cardPacks')}
            >
              {flags.cardPacks ? 'Enabled' : 'Disabled'}
            </button>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>BTC Fuel Packs</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>ckBTC native deposits</div>
            </div>
            <button
              type="button"
              className={`badge ${flags.btcPacks ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer' }}
              onClick={() => toggleFlag('btcPacks')}
            >
              {flags.btcPacks ? 'Enabled' : 'Disabled'}
            </button>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>ETH Fuel Packs</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>ckETH / EVM RPC</div>
            </div>
            <button
              type="button"
              className={`badge ${flags.ethPacks ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer' }}
              onClick={() => toggleFlag('ethPacks')}
            >
              {flags.ethPacks ? 'Enabled' : 'Disabled'}
            </button>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>Sponsored Invites</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Zero-cost AAA onboarding</div>
            </div>
            <button
              type="button"
              className={`badge ${flags.sponsoredInvites ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer' }}
              onClick={() => toggleFlag('sponsoredInvites')}
            >
              {flags.sponsoredInvites ? 'Enabled' : 'Disabled'}
            </button>
          </div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Scientific Consensus Parameters
        </h3>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1.25rem' }}>
          <div>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-muted)', marginBottom: '0.35rem' }}>
              Retirement Threshold (K Consensus)
            </label>
            <input
              type="number"
              value={params.retirementThresholdK}
              onChange={(e) => setParams({ ...params, retirementThresholdK: Number(e.target.value) })}
              style={{
                width: '100%',
                backgroundColor: 'var(--bg-surface-elevated)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                padding: '0.6rem 0.8rem',
                color: 'var(--text-main)',
                fontFamily: 'var(--font-mono)'
              }}
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-muted)', marginBottom: '0.35rem' }}>
              Gold Honeypot Injection Rate (%)
            </label>
            <input
              type="number"
              step="0.5"
              value={params.goldInjectionRatePercent}
              onChange={(e) => setParams({ ...params, goldInjectionRatePercent: Number(e.target.value) })}
              style={{
                width: '100%',
                backgroundColor: 'var(--bg-surface-elevated)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                padding: '0.6rem 0.8rem',
                color: 'var(--text-main)',
                fontFamily: 'var(--font-mono)'
              }}
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '0.85rem', color: 'var(--text-muted)', marginBottom: '0.35rem' }}>
              Peer Review Discovery Quorum
            </label>
            <input
              type="number"
              value={params.reviewQuorum}
              onChange={(e) => setParams({ ...params, reviewQuorum: Number(e.target.value) })}
              style={{
                width: '100%',
                backgroundColor: 'var(--bg-surface-elevated)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                padding: '0.6rem 0.8rem',
                color: 'var(--text-main)',
                fontFamily: 'var(--font-mono)'
              }}
            />
          </div>
        </div>

        <div style={{ marginTop: '1.5rem', display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <button type="button" className="btn-primary" onClick={handleSave}>
            <Save size={16} />
            <span>Save Configuration Changes</span>
          </button>
          {saved && <span style={{ color: 'var(--cyan-nebula)', fontSize: '0.85rem' }}>Changes committed to platform canister state!</span>}
        </div>
      </div>
    </div>
  );
};
