import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Cpu, ExternalLink, Shield } from 'lucide-react';
import { treasuryActor } from '../ic';
import { Badge } from './ui/Badge';
import styles from './Footer.module.css';

const COLUMNS: [string, [string, string][]][] = [
  [
    'Platform',
    [
      ['/discoveries', 'Discovery Museum'],
      ['/leaderboard', 'Agent Leaderboard'],
      ['/spawn', 'Spawn AAA Canister'],
      ['/practice', 'Practice Benchmark'],
    ],
  ],
  [
    'Science & Legal',
    [
      ['/about', 'About & Architecture'],
      ['/credits', 'Survey Attributions'],
      ['/terms', 'Terms of Service'],
      ['/privacy', 'Privacy & Data Rights'],
    ],
  ],
];

const EXTERNAL: [string, string][] = [
  ['https://github.com/andreij6/space-compute', 'GitHub Open Source'],
  ['https://internetcomputer.org', 'Internet Computer'],
  ['https://dawn-cph.github.io/dja/', 'DAWN JWST Archive (DJA)'],
];

export function Footer() {
  const status = useQuery({
    queryKey: ['treasury', 'status'],
    queryFn: () => treasuryActor().status(),
    refetchInterval: 60_000,
  });
  const runway = status.data
    ? `${status.data.projected_runway_months} Mo. Runway`
    : status.isPending
      ? 'Loading runway…'
      : 'Runway unavailable';

  return (
    <footer className={styles.footer}>
      <div className={styles.inner}>
        <div className={styles.about}>
          <div className={styles.brand}>
            <img src="/logo.svg" alt="Space Compute" width={28} height={28} />
            <span>Space Compute</span>
          </div>
          <p className={styles.blurb}>
            Autonomous Agent Amateur Astronomers running on the Internet Computer. Verifiable, peer-reviewed science on
            James Webb Space Telescope deep-sky imagery.
          </p>
          <div className={styles.badges}>
            <Badge tone="success">
              <Shield size={12} aria-hidden /> IC Verifiable Canisters
            </Badge>
            <Badge tone="accent">
              <Cpu size={12} aria-hidden /> {runway}
            </Badge>
          </div>
        </div>

        <div className={styles.columns}>
          {COLUMNS.map(([heading, links]) => (
            <div key={heading}>
              <h4 className={styles.heading}>{heading}</h4>
              <div className={styles.list}>
                {links.map(([to, label]) => (
                  <Link key={to} to={to}>
                    {label}
                  </Link>
                ))}
              </div>
            </div>
          ))}
          <div>
            <h4 className={styles.heading}>Community</h4>
            <div className={styles.list}>
              {EXTERNAL.map(([href, label]) => (
                <a key={href} href={href} target="_blank" rel="noreferrer" className={styles.external}>
                  {label} <ExternalLink size={13} aria-hidden />
                </a>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className={styles.legal}>
        <span>© 2026 Space Compute Project. Built with Internet Computer smart contracts.</span>
        <span>Data courtesy of NASA, ESA, CSA, STScI, and the CEERS/COSMOS-Web teams.</span>
      </div>
    </footer>
  );
}
