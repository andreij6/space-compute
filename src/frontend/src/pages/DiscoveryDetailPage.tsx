import React, { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { 
  ArrowLeft, 
  
  Download, 
  
  
  Sparkles, 
  
  Compass, 
  ZoomIn, 
  ZoomOut,
  RotateCcw
} from 'lucide-react';
import { mockDiscoveries } from '../mockData';
import { CertifiedCitationBlock } from '../components/CertifiedCitationBlock';
import { EmptyState } from '../components/EmptyState';

export const DiscoveryDetailPage: React.FC = () => {
  const { publicId } = useParams<{ publicId: string }>();
  const [zoom, setZoom] = useState(1);
  const discovery = mockDiscoveries.find((d) => d.publicId.toLowerCase() === publicId?.toLowerCase()) || mockDiscoveries[0];

  if (!discovery) {
    return <EmptyState type="not_found" title="Discovery Not Found" />;
  }

  const handleZoomIn = () => setZoom((z) => Math.min(3, z + 0.5));
  const handleZoomOut = () => setZoom((z) => Math.max(1, z - 0.5));
  const handleResetZoom = () => setZoom(1);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
        <Link to="/discoveries" className="btn-secondary" style={{ padding: '0.4rem 0.75rem' }}>
          <ArrowLeft size={16} />
          <span>Back to Museum</span>
        </Link>
        <span className="badge badge-amber">{discovery.publicId}</span>
        <span className="badge badge-cyan">{discovery.categoryLabel}</span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: 'clamp(1.8rem, 4vw, 2.5rem)', fontWeight: 700 }}>
          {discovery.name}
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1.05rem' }}>
          Observational discovery candidate cataloged in {discovery.survey} ({discovery.field}).
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '1.5rem' }}>
        <div className="card" style={{ padding: '0', overflow: 'hidden', position: 'relative' }}>
          <div style={{ 
            height: '420px', 
            backgroundColor: '#03040a', 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center', 
            overflow: 'hidden',
            position: 'relative'
          }}>
            <img 
              src={discovery.imageUrl} 
              alt={discovery.name}
              style={{ 
                width: '100%', 
                height: '100%', 
                objectFit: 'contain',
                transform: `scale(${zoom})`,
                transition: 'transform 0.25s ease'
              }}
            />

            <div style={{ 
              position: 'absolute', 
              bottom: '1rem', 
              right: '1rem', 
              display: 'flex', 
              gap: '0.5rem',
              backgroundColor: 'rgba(7, 9, 19, 0.75)',
              padding: '0.35rem 0.6rem',
              borderRadius: 'var(--radius-sm)',
              backdropFilter: 'blur(8px)'
            }}>
              <button 
                type="button" 
                onClick={handleZoomIn} 
                style={{ color: 'var(--text-main)', padding: '0.2rem' }}
                title="Zoom In"
              >
                <ZoomIn size={18} />
              </button>
              <button 
                type="button" 
                onClick={handleZoomOut} 
                style={{ color: 'var(--text-main)', padding: '0.2rem' }}
                title="Zoom Out"
              >
                <ZoomOut size={18} />
              </button>
              <button 
                type="button" 
                onClick={handleResetZoom} 
                style={{ color: 'var(--text-main)', padding: '0.2rem' }}
                title="Reset Zoom"
              >
                <RotateCcw size={18} />
              </button>
            </div>

            <div style={{ 
              position: 'absolute', 
              top: '1rem', 
              left: '1rem', 
              fontFamily: 'var(--font-mono)', 
              fontSize: '0.75rem',
              backgroundColor: 'rgba(7, 9, 19, 0.75)',
              padding: '0.35rem 0.65rem',
              borderRadius: 'var(--radius-sm)',
              backdropFilter: 'blur(8px)',
              color: 'var(--cyan-nebula)'
            }}>
              JWST / NIRCam RGB Composite (F150W / F277W / F444W)
            </div>
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1.5rem' }}>
          <div className="card">
            <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Compass size={18} style={{ color: 'var(--amber-star)' }} />
              <span>Astrometric & Physical Parameters</span>
            </h3>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', fontSize: '0.9rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-subtle)' }}>
                <span style={{ color: 'var(--text-muted)' }}>Right Ascension (R.A.)</span>
                <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>{discovery.ra}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-subtle)' }}>
                <span style={{ color: 'var(--text-muted)' }}>Declination (Dec.)</span>
                <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>{discovery.dec}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-subtle)' }}>
                <span style={{ color: 'var(--text-muted)' }}>Photometric Redshift (z)</span>
                <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--cyan-nebula)' }}>{discovery.redshift}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-subtle)' }}>
                <span style={{ color: 'var(--text-muted)' }}>Estimated Stellar Mass</span>
                <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>{discovery.stellarMass}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-subtle)' }}>
                <span style={{ color: 'var(--text-muted)' }}>Observational Program</span>
                <span style={{ fontWeight: 500 }}>{discovery.survey}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--text-muted)' }}>Catalog Target Field</span>
                <span style={{ fontWeight: 500 }}>{discovery.field}</span>
              </div>
            </div>

            <div style={{ marginTop: '1.25rem', paddingTop: '1rem', borderTop: '1px solid var(--border-subtle)' }}>
              <a 
                href={discovery.fitsUrl} 
                className="btn-secondary" 
                style={{ width: '100%', justifyContent: 'center', fontSize: '0.85rem' }}
                download
              >
                <Download size={15} />
                <span>Download Scientific FITS File (18.4 MB)</span>
              </a>
            </div>
          </div>

          <div className="card">
            <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.75rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Sparkles size={18} style={{ color: 'var(--amber-star)' }} />
              <span>AI Agent Rationale & Detection</span>
            </h3>

            <p style={{ color: 'var(--text-main)', fontSize: '0.9rem', lineHeight: 1.6, marginBottom: '1.25rem' }}>
              {discovery.rationale}
            </p>

            <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)' }}>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.5rem' }}>
                Discovering Agent Details
              </div>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <Link to={`/aaa/${discovery.discovererAaa}`} style={{ fontWeight: 600, color: 'var(--amber-star)' }}>
                  {discovery.discovererAaa}
                </Link>
                <span className="badge badge-subtle">Tier {discovery.discovererTier}</span>
              </div>
              <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
                Flagged Date: {discovery.flaggedDate}
              </div>
            </div>
          </div>
        </div>

        <CertifiedCitationBlock discovery={discovery} />
      </div>
    </div>
  );
};
