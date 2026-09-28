import { useState, type ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { BookOpen, Compass, LayoutDashboard, Menu, PlusCircle, ShieldCheck, Telescope, Trophy, Wallet, X } from 'lucide-react';
import { useAuth } from '../auth';
import { Badge, type BadgeTone } from './ui/Badge';
import { buttonClass } from './ui/Button';
import styles from './Navbar.module.css';

const PRIMARY: [string, string, ReactNode][] = [
  ['/discoveries', 'Museum', <Compass key="i" size={17} aria-hidden />],
  ['/leaderboard', 'Leaderboard', <Trophy key="i" size={17} aria-hidden />],
  ['/dashboard', 'Dashboard', <LayoutDashboard key="i" size={17} aria-hidden />],
  ['/about', 'About', <BookOpen key="i" size={17} aria-hidden />],
  ['/admin', 'Admin', <ShieldCheck key="i" size={17} aria-hidden />],
];

const DRAWER: { to: string; label: string; icon?: ReactNode; badge?: [string, BadgeTone] }[] = [
  { to: '/discoveries', label: 'Discovery Museum', icon: <Compass size={18} aria-hidden />, badge: ['Gallery', 'accent'] },
  { to: '/leaderboard', label: 'Agent Leaderboard', icon: <Trophy size={18} aria-hidden />, badge: ['Ranks', 'success'] },
  { to: '/dashboard', label: 'Owner Command Center', icon: <LayoutDashboard size={18} aria-hidden />, badge: ['Online', 'neutral'] },
  { to: '/connect', label: 'Connect Local Agent', icon: <Telescope size={18} aria-hidden /> },
  { to: '/records', label: 'Activity & Audit Records' },
  { to: '/fuel', label: 'Cycle Fuel & Top-Up' },
  { to: '/about', label: 'About & Treasury Runway' },
  { to: '/practice', label: 'Practice & Benchmarks' },
  { to: '/admin', label: 'Admin Console', icon: <ShieldCheck size={18} aria-hidden /> },
];

export function Navbar() {
  const [open, setOpen] = useState(false);
  const { principal, signOut } = useAuth();
  const { pathname } = useLocation();
  const close = () => setOpen(false);

  return (
    <header className={styles.navbar}>
      <div className={styles.inner}>
        <Link to="/" className={styles.brand} onClick={close}>
          <img src="/logo.svg" alt="Space Compute Logo" width={32} height={32} />
          <span>Space Compute</span>
        </Link>

        <nav aria-label="Primary" className={styles.links}>
          {PRIMARY.map(([to, label, icon]) => (
            <Link key={to} to={to} className={styles.link} aria-current={pathname === to ? 'page' : undefined}>
              {icon}
              <span>{label}</span>
            </Link>
          ))}
        </nav>

        <div className={styles.actions}>
          <Link to="/spawn" className={buttonClass({ variant: 'primary' })}>
            <PlusCircle size={16} aria-hidden />
            <span>Spawn AAA</span>
          </Link>

          {principal ? (
            <button type="button" className={buttonClass({})} onClick={() => void signOut()} title={principal.toText()}>
              <Wallet size={15} aria-hidden />
              <span className={styles.principal}>{principal.toText().slice(0, 5)}… Sign out</span>
            </button>
          ) : (
            <Link to="/signin" className={buttonClass({})}>
              <Wallet size={15} aria-hidden />
              <span>Sign in</span>
            </Link>
          )}

          <button
            type="button"
            className={styles.menuButton}
            onClick={() => setOpen(!open)}
            aria-label="Toggle navigation menu"
            aria-expanded={open}
            aria-controls="mobile-drawer"
          >
            {open ? <X size={22} aria-hidden /> : <Menu size={22} aria-hidden />}
          </button>
        </div>
      </div>

      {open && (
        <nav id="mobile-drawer" aria-label="Menu" className={styles.drawer}>
          {DRAWER.map(({ to, label, icon, badge }) => (
            <Link key={to} to={to} className={styles.drawerLink} onClick={close}>
              <span className={styles.drawerLabel}>
                {icon} {label}
              </span>
              {badge && <Badge tone={badge[1]}>{badge[0]}</Badge>}
            </Link>
          ))}
        </nav>
      )}
    </header>
  );
}
