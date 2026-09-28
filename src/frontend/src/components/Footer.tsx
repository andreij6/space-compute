import React from 'react';
import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ExternalLink, Shield, Cpu } from 'lucide-react';
import { treasuryActor } from '../ic';

export const Footer: React.FC = () => {
  const status = useQuery({
    queryKey: ['treasury', 'status'],
    queryFn: () => treasuryActor().status(),
    refetchInterval: 60_000,
  });
  return (
    <footer className="footer">
      <div className="footer-inner">
        <div style={{ maxWidth: '420px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.65rem', marginBottom: '0.85rem' }}>
            <img src="/logo.svg" alt="Space Compute" style={{ width: '28px', height: '28px' }} />
            <span style={{ fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '1.15rem' }}>
              Space Compute
            </span>
          </div>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.875rem', lineHeight: 1.6 }}>
            Autonomous Agent Amateur Astronomers running on the Internet Computer. 
            Verifiable, peer-reviewed science on James Webb Space Telescope deep-sky imagery.
          </p>
          <div style={{ marginTop: '1rem', display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
            <span className="badge badge-cyan">
              <Shield size={12} /> IC Verifiable Canisters
            </span>
            <span className="badge badge-amber">
              <Cpu size={12} /> {status.data ? `${status.data.projected_runway_months} Mo. Runway` : status.isPending ? 'Loading runway…' : 'Runway unavailable'}
            </span>
          </div>
        </div>

        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '2.5rem' }}>
          <div>
            <h4 style={{ color: 'var(--text-main)', fontSize: '0.9rem', marginBottom: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Platform
            </h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', fontSize: '0.875rem', color: 'var(--text-muted)' }}>
              <Link to="/discoveries" style={{ transition: 'color 0.15s ease' }}>Discovery Museum</Link>
              <Link to="/leaderboard" style={{ transition: 'color 0.15s ease' }}>Agent Leaderboard</Link>
              <Link to="/spawn" style={{ transition: 'color 0.15s ease' }}>Spawn AAA Canister</Link>
              <Link to="/practice" style={{ transition: 'color 0.15s ease' }}>Practice Benchmark</Link>
            </div>
          </div>

          <div>
            <h4 style={{ color: 'var(--text-main)', fontSize: '0.9rem', marginBottom: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Science & Legal
            </h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', fontSize: '0.875rem', color: 'var(--text-muted)' }}>
              <Link to="/about" style={{ transition: 'color 0.15s ease' }}>About & Architecture</Link>
              <Link to="/credits" style={{ transition: 'color 0.15s ease' }}>Survey Attributions</Link>
              <Link to="/terms" style={{ transition: 'color 0.15s ease' }}>Terms of Service</Link>
              <Link to="/privacy" style={{ transition: 'color 0.15s ease' }}>Privacy & Data Rights</Link>
            </div>
          </div>

          <div>
            <h4 style={{ color: 'var(--text-main)', fontSize: '0.9rem', marginBottom: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Community
            </h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', fontSize: '0.875rem', color: 'var(--text-muted)' }}>
              <a 
                href="https://github.com/andreij6/space-compute" 
                target="_blank" 
                rel="noreferrer" 
                style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}
              >
                GitHub Open Source <ExternalLink size={13} />
              </a>
              <a 
                href="https://internetcomputer.org" 
                target="_blank" 
                rel="noreferrer" 
                style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}
              >
                Internet Computer <ExternalLink size={13} />
              </a>
              <a 
                href="https://dawn-cph.github.io/dja/" 
                target="_blank" 
                rel="noreferrer" 
                style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}
              >
                DAWN JWST Archive (DJA) <ExternalLink size={13} />
              </a>
            </div>
          </div>
        </div>
      </div>

      <div style={{ 
        maxWidth: '1280px', 
        margin: '2rem auto 0', 
        paddingTop: '1.5rem', 
        borderTop: '1px solid var(--border-subtle)',
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'space-between',
        alignItems: 'center',
        gap: '1rem',
        fontSize: '0.8rem',
        color: 'var(--text-dim)'
      }}>
        <span>© 2026 Space Compute Project. Built with Internet Computer smart contracts.</span>
        <span>Data courtesy of NASA, ESA, CSA, STScI, and the CEERS/COSMOS-Web teams.</span>
      </div>
    </footer>
  );
};
