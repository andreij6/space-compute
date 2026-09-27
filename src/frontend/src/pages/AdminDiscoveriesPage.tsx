import React, { useState } from 'react';
import { AdminNav } from '../components/AdminNav';
import { Compass, Upload, CheckCircle2, Clock, AlertTriangle } from 'lucide-react';
import { mockDiscoveries } from '../mockData';

export const AdminDiscoveriesPage: React.FC = () => {
  const [seedSuccess, setSeedSuccess] = useState(false);

  const handleSeed = () => {
    setSeedSuccess(true);
    setTimeout(() => setSeedSuccess(false), 3000);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Consensus Engine</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Discoveries & Review Queue
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1.25rem' }}>
        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Pending Reviews</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--amber-star)' }}>
            24
          </div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Starving (&gt;7 Days)</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--red-nova)' }}>
            2
          </div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Avg. Quorum Time</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--cyan-nebula)' }}>
            4.8 Hours
          </div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Upload size={18} style={{ color: 'var(--amber-star)' }} />
          <span>Honeypot Gold Target Seed Tool</span>
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1.25rem' }}>
          Inject verified ground-truth subjects to continuously test agent accuracy and calibrate consensus weights.
        </p>

        <div style={{ 
          border: '1.5px dashed var(--border-subtle)', 
          borderRadius: 'var(--radius-md)', 
          padding: '2rem 1.5rem', 
          textAlign: 'center',
          backgroundColor: 'var(--bg-surface-elevated)'
        }}>
          <Upload size={32} style={{ color: 'var(--text-dim)', margin: '0 auto 0.75rem' }} />
          <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>
            Drop Honeypot Batch JSON file here
          </div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: '0.25rem', marginBottom: '1rem' }}>
            Schema: Array of &#123; subject_id, ground_truth, coordinates, fits_hash &#125;
          </div>
          <button type="button" className="btn-primary" onClick={handleSeed}>
            <span>Seed 50 Gold Targets</span>
          </button>
        </div>

        {seedSuccess && (
          <div style={{ marginTop: '1rem', color: 'var(--cyan-nebula)', fontSize: '0.85rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <CheckCircle2 size={16} /> Successfully injected and committed 50 honeypot subjects to the platform pool.
          </div>
        )}
      </div>
    </div>
  );
};
