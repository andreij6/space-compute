import React from 'react';
import { Link } from 'react-router-dom';
import { Compass, AlertTriangle, WifiOff, ImageOff, ArrowLeft, Zap, Telescope } from 'lucide-react';

interface EmptyStateProps {
  type: 'not_found' | 'canister_paused' | 'agent_offline' | 'image_unavailable';
  title?: string;
  description?: string;
}

export const EmptyState: React.FC<EmptyStateProps> = ({ type, title, description }) => {
  switch (type) {
    case 'not_found':
      return (
        <div className="card" style={{ textAlign: 'center', padding: '3.5rem 1.5rem', maxWidth: '600px', margin: '3rem auto' }}>
          <Compass size={48} style={{ color: 'var(--amber-star)', margin: '0 auto 1.25rem' }} />
          <h2 style={{ fontSize: '1.75rem', fontFamily: 'var(--font-display)', marginBottom: '0.75rem' }}>
            {title || '404: Lost in Deep Space'}
          </h2>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.95rem', lineHeight: 1.6, marginBottom: '2rem' }}>
            {description || 'The celestial coordinates you requested do not correspond to any known catalog entry or canister route.'}
          </p>
          <div style={{ display: 'flex', justifyContent: 'center', gap: '1rem' }}>
            <Link to="/" className="btn-primary">
              <ArrowLeft size={16} />
              <span>Return to Base Orbit</span>
            </Link>
            <Link to="/discoveries" className="btn-secondary">
              <Compass size={16} />
              <span>Explore Museum</span>
            </Link>
          </div>
        </div>
      );

    case 'canister_paused':
      return (
        <div className="card" style={{ borderColor: 'var(--red-nova)', textAlign: 'center', padding: '2.5rem 1.5rem' }}>
          <AlertTriangle size={40} style={{ color: 'var(--red-nova)', margin: '0 auto 1rem' }} />
          <h3 style={{ fontSize: '1.25rem', fontFamily: 'var(--font-display)', marginBottom: '0.5rem', color: 'var(--red-nova)' }}>
            {title || 'AAA Canister Suspended: Fuel Depleted'}
          </h3>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem', maxWidth: '500px', margin: '0 auto 1.5rem' }}>
            {description || 'Your agent canister has consumed its cycle balance and entered freezing protection. Stable memory is safe. Refuel with cycles or fiat to resume autonomous observation.'}
          </p>
          <Link to="/fuel" className="btn-primary" style={{ display: 'inline-flex', margin: '0 auto' }}>
            <Zap size={16} />
            <span>Refuel Canister Now</span>
          </Link>
        </div>
      );

    case 'agent_offline':
      return (
        <div className="card" style={{ borderColor: 'var(--amber-star)', textAlign: 'center', padding: '2.5rem 1.5rem' }}>
          <WifiOff size={40} style={{ color: 'var(--amber-star)', margin: '0 auto 1rem' }} />
          <h3 style={{ fontSize: '1.25rem', fontFamily: 'var(--font-display)', marginBottom: '0.5rem' }}>
            {title || 'Local Agent Offline'}
          </h3>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem', maxWidth: '500px', margin: '0 auto 1.5rem' }}>
            {description || 'No heartbeat detected from your local Claude Code or Python agent in the last 30 minutes. Your canister is waiting for instructions.'}
          </p>
          <Link to="/connect" className="btn-secondary" style={{ display: 'inline-flex', margin: '0 auto' }}>
            <Telescope size={16} />
            <span>View Connect Instructions</span>
          </Link>
        </div>
      );

    case 'image_unavailable':
      return (
        <div style={{ 
          backgroundColor: 'var(--bg-surface-elevated)', 
          border: '1px dashed var(--border-subtle)', 
          borderRadius: 'var(--radius-md)',
          padding: '2.5rem 1.5rem',
          textAlign: 'center',
          color: 'var(--text-dim)'
        }}>
          <ImageOff size={32} style={{ margin: '0 auto 0.75rem' }} />
          <div style={{ fontSize: '0.9rem', fontWeight: 500, color: 'var(--text-muted)' }}>
            {title || 'Telescope Cutout Unavailable'}
          </div>
          <div style={{ fontSize: '0.8rem', marginTop: '0.25rem' }}>
            {description || 'Archive FITS tile pending photometric recalibration.'}
          </div>
        </div>
      );
  }
};
