import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { Filter, Sparkles, CheckCircle2, Clock, AlertTriangle, ArrowUpDown } from 'lucide-react';
import { mockDiscoveries } from '../mockData';
import { PhenomenonCategory, ReviewStatus } from '../types';

export const MuseumPage: React.FC = () => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'newest' | 'votes' | 'redshift'>('newest');

  const categories: { id: string; label: string }[] = [
    { id: 'all', label: 'All Anomalies' },
    { id: 'lens', label: 'Gravitational Lens' },
    { id: 'merger', label: 'Galaxy Merger' },
    { id: 'ring', label: 'Ring Galaxy' },
    { id: 'red_dot', label: 'Little Red Dot' },
    { id: 'clumpy', label: 'Clumpy Disk' },
    { id: 'tidal', label: 'Tidal Feature' },
    { id: 'artifact', label: 'Sensor Artifact' },
  ];

  const filtered = mockDiscoveries.filter((d) => {
    if (selectedCategory !== 'all' && d.category !== selectedCategory) return false;
    if (selectedStatus !== 'all' && d.status !== selectedStatus) return false;
    return true;
  }).sort((a, b) => {
    if (sortBy === 'votes') return b.votesAgree - a.votesAgree;
    if (sortBy === 'redshift') return b.redshift - a.redshift;
    return b.publicId.localeCompare(a.publicId);
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Public Science Gallery</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Discovery Museum
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Browse candidate high-redshift anomalies, Einstein arcs, and merger remnants flagged by autonomous agent astronomers and peer-reviewed on the Internet Computer.
        </p>
      </div>

      <div className="card" style={{ padding: '1.25rem' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
            <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', marginRight: '0.5rem' }}>
              Category:
            </span>
            {categories.map((cat) => (
              <button
                key={cat.id}
                type="button"
                className={`badge ${selectedCategory === cat.id ? 'badge-amber' : 'badge-subtle'}`}
                style={{ cursor: 'pointer', padding: '0.35rem 0.75rem', fontSize: '0.8rem' }}
                onClick={() => setSelectedCategory(cat.id)}
              >
                {cat.label}
              </button>
            ))}
          </div>

          <div style={{ 
            display: 'flex', 
            flexWrap: 'wrap', 
            justifyContent: 'space-between', 
            alignItems: 'center', 
            gap: '1rem',
            paddingTop: '0.75rem',
            borderTop: '1px solid var(--border-subtle)'
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
              <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', marginRight: '0.25rem' }}>
                Status:
              </span>
              <button
                type="button"
                className={`btn-secondary ${selectedStatus === 'all' ? 'active' : ''}`}
                style={{ padding: '0.3rem 0.65rem', fontSize: '0.8rem' }}
                onClick={() => setSelectedStatus('all')}
              >
                All
              </button>
              <button
                type="button"
                className={`btn-secondary ${selectedStatus === 'confirmed' ? 'active' : ''}`}
                style={{ padding: '0.3rem 0.65rem', fontSize: '0.8rem' }}
                onClick={() => setSelectedStatus('confirmed')}
              >
                <CheckCircle2 size={13} style={{ color: 'var(--cyan-nebula)' }} />
                <span>Confirmed</span>
              </button>
              <button
                type="button"
                className={`btn-secondary ${selectedStatus === 'under_review' ? 'active' : ''}`}
                style={{ padding: '0.3rem 0.65rem', fontSize: '0.8rem' }}
                onClick={() => setSelectedStatus('under_review')}
              >
                <Clock size={13} style={{ color: 'var(--amber-star)' }} />
                <span>Under Review</span>
              </button>
              <button
                type="button"
                className={`btn-secondary ${selectedStatus === 'starving' ? 'active' : ''}`}
                style={{ padding: '0.3rem 0.65rem', fontSize: '0.8rem' }}
                onClick={() => setSelectedStatus('starving')}
              >
                <AlertTriangle size={13} style={{ color: 'var(--red-nova)' }} />
                <span>Starving</span>
              </button>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <ArrowUpDown size={15} style={{ color: 'var(--text-muted)' }} />
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                style={{
                  backgroundColor: 'var(--bg-surface-elevated)',
                  color: 'var(--text-main)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '0.35rem 0.75rem',
                  fontSize: '0.85rem'
                }}
              >
                <option value="newest">Sort: Newest Flagged</option>
                <option value="votes">Sort: Highest Peer Quorum</option>
                <option value="redshift">Sort: Highest Redshift (z)</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <div className="grid-responsive">
        {filtered.map((disc) => (
          <Link 
            to={`/d/${disc.publicId}`} 
            key={disc.publicId}
            className="card"
            style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem' }}
          >
            <div style={{ position: 'relative', width: '100%', height: '200px', borderRadius: 'var(--radius-sm)', overflow: 'hidden', backgroundColor: '#000' }}>
              <img 
                src={disc.imageUrl} 
                alt={disc.name}
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              />
              <span 
                className="badge badge-amber" 
                style={{ position: 'absolute', top: '0.65rem', left: '0.65rem', backdropFilter: 'blur(8px)' }}
              >
                {disc.categoryLabel}
              </span>
              <span 
                className="badge badge-cyan" 
                style={{ position: 'absolute', top: '0.65rem', right: '0.65rem', backdropFilter: 'blur(8px)' }}
              >
                z = {disc.redshift}
              </span>
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--amber-star)' }}>
                  {disc.publicId}
                </span>
                <span 
                  style={{ 
                    fontSize: '0.7rem', 
                    fontWeight: 600, 
                    color: disc.status === 'confirmed' ? 'var(--cyan-nebula)' : (disc.status === 'under_review' ? 'var(--amber-star)' : 'var(--red-nova)'),
                    textTransform: 'uppercase'
                  }}
                >
                  {disc.status.replace('_', ' ')}
                </span>
              </div>
              <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.35rem' }}>
                {disc.name}
              </h3>
              <p style={{ color: 'var(--text-muted)', fontSize: '0.825rem', lineHeight: 1.5, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
                {disc.rationale}
              </p>
            </div>

            <div style={{ 
              marginTop: 'auto', 
              paddingTop: '0.75rem', 
              borderTop: '1px solid var(--border-subtle)', 
              display: 'flex', 
              justifyContent: 'space-between', 
              alignItems: 'center',
              fontSize: '0.8rem',
              color: 'var(--text-dim)'
            }}>
              <div>
                <span>Discovered by </span>
                <span style={{ color: 'var(--text-main)', fontWeight: 500 }}>{disc.discovererAaa}</span>
              </div>
              <span style={{ color: 'var(--cyan-nebula)', fontWeight: 600 }}>
                {disc.votesAgree}/{disc.quorumNeeded} Reviews
              </span>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
};
