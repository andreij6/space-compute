import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { AlertOctagon, CheckCircle2, Shield, Edit3, Trash2 } from 'lucide-react';

export const AdminModerationPage: React.FC = () => {
  const [blocklist, setBlocklist] = useState('nsfw_term, offensive_word, scam_bot, test_abuse');
  const [reports, setReports] = useState([
    { id: 'rep-01', targetName: 'FakeNasaOfficial-01', reporter: 'user_9912', reason: 'Impersonation of institutional entity', status: 'Pending Review' }
  ]);
  const [saved, setSaved] = useState(false);

  const handleSaveBlocklist = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Trust & Safety</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Content & Handle Moderation
        </h1>
      </div>

      <AdminNav />

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem' }}>
          Offensive Term Blocklist
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1rem' }}>
          Comma-separated keywords prohibited from being chosen during AAA canister spawning.
        </p>

        <textarea
          value={blocklist}
          onChange={(e) => setBlocklist(e.target.value)}
          rows={3}
          style={{
            width: '100%',
            backgroundColor: 'var(--bg-surface-elevated)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-sm)',
            padding: '0.75rem',
            color: 'var(--text-main)',
            fontFamily: 'var(--font-mono)',
            fontSize: '0.85rem',
            marginBottom: '1rem'
          }}
        />

        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <button type="button" className="btn-primary" onClick={handleSaveBlocklist}>
            <span>Update Keyword Blocklist</span>
          </button>
          {saved && <span style={{ color: 'var(--cyan-nebula)', fontSize: '0.85rem' }}>Blocklist updated on platform canister!</span>}
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Reported AAA Handles Queue
        </h3>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Report ID</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Flagged Name</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Reason</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Status</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {reports.map((r) => (
                <tr key={r.id} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)' }}>{r.id}</td>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600, color: 'var(--amber-star)' }}>{r.targetName}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>{r.reason}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>
                    <span className="badge badge-amber">{r.status}</span>
                  </td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <button type="button" className="btn-secondary" style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}>
                      <Edit3 size={12} />
                      <span>Admin Rename</span>
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
