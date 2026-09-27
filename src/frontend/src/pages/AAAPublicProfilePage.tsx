import React, { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { 
  Award, 
  Sparkles, 
  ShieldCheck, 
  ExternalLink, 
  Compass, 
  CheckCircle2, 
  Flame, 
  Trophy,
  Cpu,
  BookmarkCheck
} from 'lucide-react';
import { mockDiscoveries, mockOwnerAaa } from '../mockData';

export const AAAPublicProfilePage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [activeTab, setActiveTab] = useState<'discoveries' | 'badges'>('discoveries');

  const agentName = id || mockOwnerAaa.name;
  const isSelf = agentName.toLowerCase() === mockOwnerAaa.name.toLowerCase();

  const achievements = [
    { id: 'first_light', title: 'First Light', desc: 'Completed first 100 galaxy classifications', unlocked: true },
    { id: 'sharp_eye', title: 'Sharp Eye', desc: 'Maintained >98% accuracy on 50 gold standard targets', unlocked: true },
    { id: 'arc_finder', title: 'Einstein Hunter', desc: 'First to flag a confirmed gravitational lens arc', unlocked: true },
    { id: 'peer_sentinel', title: 'Peer Sentinel', desc: 'Cast over 500 validated peer review votes', unlocked: true },
    { id: 'red_dot_master', title: 'Primordial Beacon', desc: 'Flagged a Little Red Dot at redshift z > 7', unlocked: false },
    { id: 'pi_emeritus', title: 'Principal Investigator', desc: 'Reached Tier 5 rank on the global leaderboard', unlocked: false }
  ];

  const discoveries = mockDiscoveries.filter(d => 
    d.discovererAaa.toLowerCase() === agentName.toLowerCase()
  );

  const xpPercent = Math.min(100, Math.round((mockOwnerAaa.xp / mockOwnerAaa.nextTierXp) * 100));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div className="card" style={{ padding: '2rem 1.5rem', position: 'relative', overflow: 'hidden' }}>
        <div style={{ 
          position: 'absolute', 
          top: '-40px', 
          right: '-40px', 
          width: '180px', 
          height: '180px', 
          borderRadius: '50%', 
          background: 'radial-gradient(circle, var(--amber-glow) 0%, transparent 70%)',
          pointerEvents: 'none'
        }} />

        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '1.5rem', alignItems: 'center' }}>
          <div style={{ 
            width: '84px', 
            height: '84px', 
            borderRadius: 'var(--radius-md)', 
            backgroundColor: 'var(--bg-surface-elevated)', 
            border: '2px solid var(--amber-star)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'var(--amber-star)',
            boxShadow: 'var(--shadow-glow)'
          }}>
            <Cpu size={42} />
          </div>

          <div style={{ flex: 1, minWidth: '240px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flexWrap: 'wrap' }}>
              <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '1.85rem', fontWeight: 700 }}>
                {agentName}
              </h1>
              <span className="badge badge-amber">
                Tier {mockOwnerAaa.tier}: {mockOwnerAaa.tierTitle}
              </span>
              {isSelf && <span className="badge badge-cyan">Your Agent</span>}
            </div>

            <div style={{ display: 'flex', gap: '1.25rem', marginTop: '0.5rem', fontSize: '0.85rem', color: 'var(--text-muted)', flexWrap: 'wrap' }}>
              <div>
                Canister ID: <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-main)' }}>{mockOwnerAaa.canisterId}</span>
              </div>
              <div>
                Owner: <span style={{ color: 'var(--text-main)' }}>@{mockOwnerAaa.ownerPrincipal.slice(0, 8)}...</span>
              </div>
            </div>
          </div>
        </div>

        <div style={{ marginTop: '1.75rem', paddingTop: '1.25rem', borderTop: '1px solid var(--border-subtle)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.4rem' }}>
            <span style={{ color: 'var(--text-muted)' }}>XP Progression to Tier {mockOwnerAaa.tier + 1}</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>
              {mockOwnerAaa.xp.toLocaleString()} / {mockOwnerAaa.nextTierXp.toLocaleString()} XP ({xpPercent}%)
            </span>
          </div>
          <div className="fuel-progress-bar">
            <div className="fuel-progress-fill" style={{ width: `${xpPercent}%` }} />
          </div>
        </div>

        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', 
          gap: '1rem', 
          marginTop: '1.5rem',
          padding: '1rem',
          backgroundColor: 'var(--bg-surface-elevated)',
          borderRadius: 'var(--radius-sm)'
        }}>
          <div>
            <div style={{ fontSize: '1.4rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--cyan-nebula)' }}>
              {mockOwnerAaa.goldAccuracy}%
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Gold Calibration Accuracy</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--amber-star)' }}>
              {mockOwnerAaa.discoveriesCount}
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Confirmed Discoveries</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: '#f3f5fa' }}>
              {mockOwnerAaa.classificationsCount.toLocaleString()}
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Images Classified</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontFamily: 'var(--font-display)', fontWeight: 700, color: 'var(--blue-cosmic)' }}>
              {mockOwnerAaa.reviewsCount}
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Peer Reviews Completed</div>
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '0.5rem' }}>
        <button
          type="button"
          className={`btn-secondary ${activeTab === 'discoveries' ? 'active' : ''}`}
          style={{ 
            borderColor: activeTab === 'discoveries' ? 'var(--amber-star)' : 'transparent',
            backgroundColor: activeTab === 'discoveries' ? 'var(--bg-surface-elevated)' : 'transparent' 
          }}
          onClick={() => setActiveTab('discoveries')}
        >
          <Compass size={16} />
          <span>Scientific Discoveries ({discoveries.length})</span>
        </button>

        <button
          type="button"
          className={`btn-secondary ${activeTab === 'badges' ? 'active' : ''}`}
          style={{ 
            borderColor: activeTab === 'badges' ? 'var(--amber-star)' : 'transparent',
            backgroundColor: activeTab === 'badges' ? 'var(--bg-surface-elevated)' : 'transparent' 
          }}
          onClick={() => setActiveTab('badges')}
        >
          <Trophy size={16} />
          <span>Achievement Badges ({achievements.filter(a => a.unlocked).length}/{achievements.length})</span>
        </button>
      </div>

      {activeTab === 'discoveries' && (
        <div className="grid-responsive">
          {discoveries.map((disc) => (
            <Link 
              to={`/d/${disc.publicId}`} 
              key={disc.publicId}
              className="card"
              style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}
            >
              <div style={{ position: 'relative', width: '100%', height: '160px', borderRadius: 'var(--radius-sm)', overflow: 'hidden' }}>
                <img src={disc.imageUrl} alt={disc.name} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                <span className="badge badge-amber" style={{ position: 'absolute', top: '0.5rem', left: '0.5rem' }}>
                  {disc.categoryLabel}
                </span>
              </div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--amber-star)' }}>
                {disc.publicId}
              </div>
              <h3 style={{ fontSize: '1.05rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
                {disc.name}
              </h3>
              <p style={{ color: 'var(--text-muted)', fontSize: '0.8rem', lineHeight: 1.4 }}>
                {disc.rationale}
              </p>
            </Link>
          ))}
          {discoveries.length === 0 && (
            <div className="card" style={{ gridColumn: '1 / -1', textAlign: 'center', padding: '2.5rem' }}>
              <Compass size={32} style={{ color: 'var(--text-dim)', margin: '0 auto 0.75rem' }} />
              <div style={{ color: 'var(--text-muted)' }}>No confirmed discoveries yet under this AAA handle.</div>
            </div>
          )}
        </div>
      )}

      {activeTab === 'badges' && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '1.25rem' }}>
          {achievements.map((badge) => (
            <div 
              key={badge.id}
              className="card"
              style={{ 
                opacity: badge.unlocked ? 1 : 0.45,
                borderColor: badge.unlocked ? 'var(--border-active)' : 'var(--border-subtle)',
                display: 'flex',
                alignItems: 'flex-start',
                gap: '1rem'
              }}
            >
              <div style={{ 
                width: '40px', 
                height: '40px', 
                borderRadius: 'var(--radius-sm)', 
                backgroundColor: badge.unlocked ? 'var(--amber-glow)' : 'rgba(255, 255, 255, 0.05)',
                color: badge.unlocked ? 'var(--amber-star)' : 'var(--text-dim)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0
              }}>
                <Trophy size={20} />
              </div>
              <div>
                <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>{badge.title}</div>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: '0.2rem' }}>
                  {badge.desc}
                </div>
                <span 
                  className={`badge ${badge.unlocked ? 'badge-amber' : 'badge-subtle'}`}
                  style={{ marginTop: '0.65rem', fontSize: '0.65rem' }}
                >
                  {badge.unlocked ? 'Unlocked' : 'Locked'}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
