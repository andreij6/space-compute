import React from 'react';
import { AdminNav } from '../components/AdminNav';
import { FileCode } from 'lucide-react';

export const AdminCatalogPage: React.FC = () => {
  const schema = `{
  "$schema": "https://space-compute.org/schemas/question-tree-v2.json",
  "name": "JWST Galaxy Morphology Tree",
  "root": "Q01_SmoothOrFeatured",
  "questions": {
    "Q01_SmoothOrFeatured": {
      "prompt": "Is the galaxy completely smooth and rounded, or does it have features/disk?",
      "options": ["Smooth", "Featured / Disk", "Star / Artifact"]
    },
    "Q02_DiskFeatures": {
      "prompt": "Does the disk feature spiral arms, bars, or clumpiness?",
      "options": ["Spiral Arms", "Central Bar", "Clumpy", "Edge-On Ring"]
    }
  }
}`;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.35rem' }}>
          <span className="badge badge-amber">Dataset Administration</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Data Catalog & Protocols
        </h1>
      </div>

      <AdminNav />

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1.25rem' }}>
        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>CEERS Field</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--amber-star)' }}>
            48,200
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)' }}>98% Classified (Retired)</div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>COSMOS-Web Field</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--cyan-nebula)' }}>
            64,150
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--amber-star)' }}>45% Classified (Active)</div>
        </div>

        <div className="card" style={{ padding: '1.25rem' }}>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>JADES Deep Field</div>
          <div style={{ fontSize: '1.8rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--blue-cosmic)' }}>
            30,500
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)' }}>Queued for Next Season</div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <FileCode size={18} style={{ color: 'var(--cyan-nebula)' }} />
          <span>Active Classification Question Tree Schema</span>
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '1rem' }}>
          JSON decision tree delivered to connected AI agents for standardized morphological classification.
        </p>

        <pre className="citation-block" style={{ whiteSpace: 'pre-wrap', maxHeight: '260px', overflowY: 'auto' }}>
          {schema}
        </pre>
      </div>
    </div>
  );
};
