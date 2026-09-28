import React, { useState } from 'react';
import { Search, X } from 'lucide-react';
import { mockActivityRecords } from '../mockData';
import { ActivityRecord } from '../types';

export const ScientificRecordsPage: React.FC = () => {
  const [filterType, setFilterType] = useState<string>('all');
  const [search, setSearch] = useState('');
  const [selectedRecord, setSelectedRecord] = useState<ActivityRecord | null>(mockActivityRecords[0]);

  const filtered = mockActivityRecords.filter(r => {
    if (filterType !== 'all' && r.type !== filterType) return false;
    if (search && !r.subjectId.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Immutable Audit Trail</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Activity & Scientific Records
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Full cryptographic provenance and decision audit log for every JWST target evaluated by your AAA canister.
        </p>
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'center', gap: '1rem' }}>
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          <button
            type="button"
            className={`btn-secondary ${filterType === 'all' ? 'active' : ''}`}
            onClick={() => setFilterType('all')}
          >
            All Activity
          </button>
          <button
            type="button"
            className={`btn-secondary ${filterType === 'classification' ? 'active' : ''}`}
            onClick={() => setFilterType('classification')}
          >
            Classifications
          </button>
          <button
            type="button"
            className={`btn-secondary ${filterType === 'discovery' ? 'active' : ''}`}
            onClick={() => setFilterType('discovery')}
          >
            Discoveries Flagged
          </button>
          <button
            type="button"
            className={`btn-secondary ${filterType === 'review' ? 'active' : ''}`}
            onClick={() => setFilterType('review')}
          >
            Peer Reviews
          </button>
        </div>

        <div style={{ position: 'relative', width: '260px' }}>
          <Search size={16} style={{ position: 'absolute', left: '0.75rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input
            type="text"
            placeholder="Search Subject ID..."
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

      <div style={{ display: 'grid', gridTemplateColumns: selectedRecord ? '1fr 360px' : '1fr', gap: '1.5rem' }}>
        <div className="card" style={{ padding: '0', overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '600px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
                <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Timestamp</th>
                <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Type</th>
                <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Subject ID</th>
                <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Agent Decision</th>
                <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>XP</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((record) => {
                const isSelected = selectedRecord?.id === record.id;
                return (
                  <tr
                    key={record.id}
                    onClick={() => setSelectedRecord(record)}
                    style={{
                      borderBottom: '1px solid var(--border-subtle)',
                      backgroundColor: isSelected ? 'var(--amber-glow)' : undefined,
                      cursor: 'pointer',
                      transition: 'background-color 0.15s ease'
                    }}
                  >
                    <td style={{ padding: '0.85rem 1rem', fontSize: '0.85rem', color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>
                      {record.timestamp}
                    </td>
                    <td style={{ padding: '0.85rem 1rem' }}>
                      <span className={`badge ${record.type === 'discovery' ? 'badge-amber' : (record.type === 'review' ? 'badge-cyan' : 'badge-subtle')}`}>
                        {record.type}
                      </span>
                    </td>
                    <td style={{ padding: '0.85rem 1rem', fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--text-main)' }}>
                      {record.subjectId}
                    </td>
                    <td style={{ padding: '0.85rem 1rem', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
                      {record.decision}
                    </td>
                    <td style={{ padding: '0.85rem 1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--cyan-nebula)' }}>
                      +{record.xpEarned}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {selectedRecord && (
          <div className="card" style={{ padding: '1.25rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span className="badge badge-amber">{selectedRecord.subjectId}</span>
              <button 
                type="button" 
                onClick={() => setSelectedRecord(null)}
                style={{ color: 'var(--text-muted)' }}
              >
                <X size={18} />
              </button>
            </div>

            <div style={{ height: '160px', backgroundColor: '#020308', borderRadius: 'var(--radius-sm)', overflow: 'hidden', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <img 
                src="https://images.unsplash.com/photo-1462331940025-496dfbfc7564?auto=format&fit=crop&w=400&q=80" 
                alt="Galaxy cutout"
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              />
            </div>

            <div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>
                Classification Decision
              </div>
              <div style={{ fontWeight: 600, fontSize: '1rem', marginTop: '0.2rem' }}>
                {selectedRecord.decision}
              </div>
              <div style={{ fontSize: '0.8rem', color: 'var(--cyan-nebula)', marginTop: '0.2rem' }}>
                Confidence: {(selectedRecord.confidence * 100).toFixed(1)}%
              </div>
            </div>

            <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '0.85rem', borderRadius: 'var(--radius-sm)' }}>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
                AI Model Rationale
              </div>
              <p style={{ fontSize: '0.85rem', color: 'var(--text-main)', lineHeight: 1.5 }}>
                {selectedRecord.rationale}
              </p>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.8rem', color: 'var(--text-dim)', paddingTop: '0.5rem', borderTop: '1px solid var(--border-subtle)' }}>
              <span>Earned: +{selectedRecord.xpEarned} Compute XP</span>
              <span style={{ color: 'var(--cyan-nebula)' }}>BLS Witness Valid</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
