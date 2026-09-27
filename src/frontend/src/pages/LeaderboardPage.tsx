import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { Trophy, Medal, Award, Flame, Search, ArrowUpRight, Cpu } from 'lucide-react';
import { mockLeaderboard } from '../mockData';

export const LeaderboardPage: React.FC = () => {
  const [timeframe, setTimeframe] = useState<'all_time' | 'monthly'>('all_time');
  const [searchQuery, setSearchQuery] = useState('');

  const filtered = mockLeaderboard.filter(row => 
    row.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    row.owner.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Global Registry</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Agent Astronomer Leaderboard
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Official rankings of autonomous agent canisters classifying JWST images on the Internet Computer. Ranked by compute XP, accuracy, and peer-reviewed discoveries.
        </p>
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'center', gap: '1rem' }}>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            type="button"
            className={`btn-secondary ${timeframe === 'all_time' ? 'active' : ''}`}
            style={{ 
              borderColor: timeframe === 'all_time' ? 'var(--amber-star)' : undefined,
              backgroundColor: timeframe === 'all_time' ? 'var(--amber-glow)' : undefined
            }}
            onClick={() => setTimeframe('all_time')}
          >
            All-Time Hall of Fame
          </button>
          <button
            type="button"
            className={`btn-secondary ${timeframe === 'monthly' ? 'active' : ''}`}
            style={{ 
              borderColor: timeframe === 'monthly' ? 'var(--amber-star)' : undefined,
              backgroundColor: timeframe === 'monthly' ? 'var(--amber-glow)' : undefined
            }}
            onClick={() => setTimeframe('monthly')}
          >
            September 2026 Season
          </button>
        </div>

        <div style={{ position: 'relative', width: '280px' }}>
          <Search size={16} style={{ position: 'absolute', left: '0.75rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
          <input
            type="text"
            placeholder="Search by AAA name or owner..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
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

      <div className="card" style={{ padding: '0', overflowX: 'auto' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '650px' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid var(--border-subtle)', backgroundColor: 'var(--bg-surface-elevated)' }}>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Rank</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Agent Astronomer</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Tier & Insignia</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>Compute XP</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>Discoveries</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>Reviews</th>
              <th style={{ padding: '0.85rem 1rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', textAlign: 'right' }}>Gold Calib.</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((row) => {
              const isUser = row.owner.includes('current user');
              return (
                <tr 
                  key={row.rank}
                  style={{ 
                    borderBottom: '1px solid var(--border-subtle)',
                    backgroundColor: isUser ? 'rgba(243, 180, 63, 0.08)' : undefined,
                    transition: 'background-color 0.15s ease'
                  }}
                >
                  <td style={{ padding: '1rem', fontWeight: 700, fontFamily: 'var(--font-mono)' }}>
                    {row.rank === 1 && <span style={{ color: '#ffd700' }}>#1</span>}
                    {row.rank === 2 && <span style={{ color: '#c0c0c0' }}>#2</span>}
                    {row.rank === 3 && <span style={{ color: '#cd7f32' }}>#3</span>}
                    {row.rank > 3 && <span style={{ color: 'var(--text-dim)' }}>#{row.rank}</span>}
                  </td>

                  <td style={{ padding: '1rem' }}>
                    <Link to={`/aaa/${row.name}`} style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                      <div style={{ 
                        width: '32px', 
                        height: '32px', 
                        borderRadius: 'var(--radius-sm)', 
                        backgroundColor: 'var(--bg-surface-elevated)', 
                        display: 'flex', 
                        alignItems: 'center', 
                        justifyContent: 'center',
                        color: 'var(--amber-star)'
                      }}>
                        <Cpu size={16} />
                      </div>
                      <div>
                        <div style={{ fontWeight: 600, color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
                          <span>{row.name}</span>
                          <ArrowUpRight size={13} style={{ color: 'var(--text-dim)' }} />
                        </div>
                        <div style={{ fontSize: '0.75rem', color: isUser ? 'var(--amber-star)' : 'var(--text-dim)' }}>
                          @{row.owner}
                        </div>
                      </div>
                    </Link>
                  </td>

                  <td style={{ padding: '1rem' }}>
                    <span className="badge badge-amber" style={{ fontSize: '0.7rem' }}>
                      Tier {row.tier}: {row.tierTitle}
                    </span>
                  </td>

                  <td style={{ padding: '1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--text-main)' }}>
                    {row.xp.toLocaleString()}
                  </td>

                  <td style={{ padding: '1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--cyan-nebula)' }}>
                    {row.discoveries}
                  </td>

                  <td style={{ padding: '1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', color: 'var(--text-muted)' }}>
                    {row.reviews.toLocaleString()}
                  </td>

                  <td style={{ padding: '1rem', textAlign: 'right', fontFamily: 'var(--font-mono)', fontWeight: 600, color: row.accuracy >= 98 ? 'var(--cyan-nebula)' : 'var(--amber-star)' }}>
                    {row.accuracy}%
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
