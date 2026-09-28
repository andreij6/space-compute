import React from 'react';
import { AdminNav } from '../components/AdminNav';
import { UploadCloud, FileArchive } from 'lucide-react';

export const AdminReleasesPage: React.FC = () => {
  const versions = [
    { version: 'v1.2.0', hash: '0x9812bf3d...44a1', size: '1.42 MiB (gz)', budgetOk: true, adoption: '88%', status: 'Active Release' },
    { version: 'v1.1.0', hash: '0x32cc4b11...99ef', size: '1.38 MiB (gz)', budgetOk: true, adoption: '12%', status: 'Deprecated' },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Canister Upgrades</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          AAA Wasm Releases & Budget
        </h1>
      </div>

      <AdminNav />

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <UploadCloud size={18} style={{ color: 'var(--amber-star)' }} />
          <span>Upload New AAA Canister Binary</span>
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1.25rem' }}>
          Verify Wasm fits within the strict 1.5 MiB gzipped budget before signing release hash.
        </p>

        <div style={{ 
          border: '1.5px dashed var(--border-subtle)', 
          borderRadius: 'var(--radius-md)', 
          padding: '2rem 1.5rem', 
          textAlign: 'center',
          backgroundColor: 'var(--bg-surface-elevated)'
        }}>
          <FileArchive size={32} style={{ color: 'var(--cyan-nebula)', margin: '0 auto 0.75rem' }} />
          <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>
            Select `aaa.wasm.gz` to stage release
          </div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: '0.25rem', marginBottom: '1rem' }}>
            Built using `cargo build --target wasm32-unknown-unknown --release`
          </div>
          <button type="button" className="btn-secondary">
            <span>Choose Binary File</span>
          </button>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
          Release Registry
        </h3>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px', fontSize: '0.85rem' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Version</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Sha256 Hash</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Size Budget</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)' }}>Adoption %</th>
                <th style={{ padding: '0.75rem 1rem', color: 'var(--text-muted)', textAlign: 'right' }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {versions.map((v) => (
                <tr key={v.version} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{v.version}</td>
                  <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)', color: 'var(--text-dim)' }}>{v.hash}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--cyan-nebula)', fontWeight: 500 }}>
                    {v.size} (PASS &lt;= 1.5M)
                  </td>
                  <td style={{ padding: '0.85rem 1rem' }}>{v.adoption}</td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right' }}>
                    <span className={`badge ${v.status === 'Active Release' ? 'badge-cyan' : 'badge-subtle'}`}>
                      {v.status}
                    </span>
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
