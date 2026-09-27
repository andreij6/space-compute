import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { Search, ShieldAlert, RefreshCw, X, Ban, ShieldCheck } from 'lucide-react';
import { mockLeaderboard } from '../mockData';

export const AdminAAAsPage: React.FC = () => {
  const [search, setSearch] = useState('');
  const [selectedAaa, setSelectedAaa] = useState<any>(mockLeaderboard[0]);

  const filtered = mockLeaderboard.filter(a => 
    a.name.toLowerCase().includes(search.toLowerCase()) ||
    a.owner.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Canister Registry</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          AAAs Directory & Operations
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ position: 'relative', width: '300px' }}>
          <Search size={16} style={{ position: 'absolute', left: '0.75rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input
            type="text"
            placeholder="Search AAAs..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              width: '100%',
              backgroundColor: 'var(--bg-surface-elevated)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-sm)',
              padding: '0.5rem 0.75rem 0.5rem 2.25rem',
              color: 'var(--text-main)',
              fontSize: '0.85rem'
            }}
          />
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: selectedAaa ? '1fr 340px' : '1fr', gap: '1.5rem' }}>
        <div className="card" style={{ padding: '0', overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>AAA Name</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Owner</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Tier</th>
                <th style={{ padding: '0.75rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>Accuracy</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((item) => (
                <tr
                  key={item.name}
                  onClick={() => setSelectedAaa(item)}
                  style={{
                    borderBottom: '1px solid var(--border-subtle)',
                    backgroundColor: selectedAaa?.name === item.name ? 'var(--amber-glow)' : undefined,
                    cursor: 'pointer',
                    fontSize: '0.85rem'
                  }}
                >
                  <td style={{ padding: '0.85rem 1rem', fontWeight: 600 }}>{item.name}</td>
                  <td style={{ padding: '0.85rem 1rem', color: 'var(--text-muted)' }}>@{item.owner}</td>
                  <td style={{ padding: '0.85rem 1rem' }}>
                    <span className="badge badge-amber">Tier {item.tier}</span>
                  </td>
                  <td style={{ padding: '0.85rem 1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', color: 'var(--cyan-nebula)' }}>
                    {item.accuracy}%
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {selectedAaa && (
          <div className="card" style={{ padding: '1.25rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
                {selectedAaa.name}
              </h3>
              <button type="button" onClick={() => setSelectedAaa(null)} style={{ color: 'var(--text-muted)' }}>
                <X size={18} />
              </button>
            </div>

            <div style={{ fontSize: '0.85rem', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Owner Principal:</span>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--text-dim)', wordBreak: 'break-all' }}>
                  2vxsx-fae7a-3x67w-5o67o-4a4g6-p46a2-yquaa-aaaaa-cai
                </div>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Registered Wasm Hash:</span>
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--cyan-nebula)' }}>
                  0x9812bf...44a1 (v1.2.0)
                </div>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Canister Cycles:</span>
                <div style={{ fontWeight: 600, color: 'var(--amber-star)' }}>
                  12.4 TCycles (~21 Days)
                </div>
              </div>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: 'auto', paddingTop: '1rem', borderTop: '1px solid var(--border-subtle)' }}>
              <button type="button" className="btn-secondary" style={{ justifyContent: 'center', fontSize: '0.8rem' }}>
                <RefreshCw size={14} />
                <span>Force Wasm Upgrade</span>
              </button>
              <button type="button" className="btn-secondary" style={{ justifyContent: 'center', fontSize: '0.8rem', color: 'var(--red-nova)' }}>
                <Ban size={14} />
                <span>Suspend AAA Canister</span>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
