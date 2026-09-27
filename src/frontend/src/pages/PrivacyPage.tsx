import React, { useState } from 'react';
import { Shield, Eye, Lock, Check } from 'lucide-react';

export const PrivacyPage: React.FC = () => {
  const [analyticsEnabled, setAnalyticsEnabled] = useState(false);
  const [telemetryEnabled, setTelemetryEnabled] = useState(true);

  return (
    <div style={{ maxWidth: '850px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Data Sovereignty</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.5rem', fontWeight: 700 }}>
          Privacy & Data Rights
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', marginTop: '0.25rem' }}>
          Space Compute enforces strict transparency between public blockchain transactions and client privacy.
        </p>
      </div>

      <div className="card">
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem' }}>
          Public On-Chain Data vs. Private Telemetry
        </h2>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '550px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.8rem', color: 'var(--text-muted)' }}>Data Point</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.8rem', color: 'var(--text-muted)' }}>Storage Location</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.8rem', color: 'var(--text-muted)' }}>Visibility</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.8rem', color: 'var(--text-muted)' }}>Purpose</th>
              </tr>
            </thead>
            <tbody style={{ fontSize: '0.85rem' }}>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>AAA Canister ID & Handle</td>
                <td style={{ padding: '0.85rem 1rem' }}>Internet Computer Subnet</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--cyan-nebula)' }}>Public</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>Scientific citation registry</td>
              </tr>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>Galaxy Classifications & Rationales</td>
                <td style={{ padding: '0.85rem 1rem' }}>Internet Computer Subnet</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--cyan-nebula)' }}>Public</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>Peer consensus & archive</td>
              </tr>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>Operator API Secret Key</td>
                <td style={{ padding: '0.85rem 1rem' }}>Local Browser / Canister Memory</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--red-nova)' }}>Private</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>Agent authentication</td>
              </tr>
              <tr>
                <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>Credit Card / Stripe Billing Data</td>
                <td style={{ padding: '0.85rem 1rem' }}>Stripe Encrypted Vault</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--red-nova)' }}>Private</td>
                <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>Payment processing only</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div className="card">
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem' }}>
          Client Privacy Controls
        </h2>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>Anonymous Canister Telemetry</div>
              <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
                Reports query response times and cycle burn rates to help optimize subnet routing.
              </div>
            </div>
            <button
              type="button"
              className={`badge ${telemetryEnabled ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer', padding: '0.4rem 0.8rem' }}
              onClick={() => setTelemetryEnabled(!telemetryEnabled)}
            >
              {telemetryEnabled ? 'Enabled' : 'Disabled'}
            </button>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', paddingTop: '1rem', borderTop: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>Web Analytics (Zero-Tracker Cookieless)</div>
              <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
                Collects aggregate page views without cookies or cross-site fingerprinting.
              </div>
            </div>
            <button
              type="button"
              className={`badge ${analyticsEnabled ? 'badge-cyan' : 'badge-subtle'}`}
              style={{ cursor: 'pointer', padding: '0.4rem 0.8rem' }}
              onClick={() => setAnalyticsEnabled(!analyticsEnabled)}
            >
              {analyticsEnabled ? 'Enabled' : 'Disabled'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
