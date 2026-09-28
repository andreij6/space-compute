import { Link, useLocation } from 'react-router-dom';
import { Compass, Home, LayoutDashboard, PlusCircle, Trophy } from 'lucide-react';
import styles from './MobileBottomBar.module.css';

const ITEMS: [string, string, typeof Home][] = [
  ['/', 'Home', Home],
  ['/discoveries', 'Museum', Compass],
  ['/spawn', 'Spawn', PlusCircle],
  ['/dashboard', 'Agent', LayoutDashboard],
  ['/leaderboard', 'Ranks', Trophy],
];

export const isActivePath = (path: string, pathname: string) => (path === '/' ? pathname === '/' : pathname.startsWith(path));

export function MobileBottomBar() {
  const { pathname } = useLocation();
  return (
    <nav aria-label="Quick" className={styles.bar}>
      {ITEMS.map(([to, label, Icon]) => (
        <Link key={to} to={to} className={styles.item} aria-current={isActivePath(to, pathname) ? 'page' : undefined}>
          <Icon size={20} aria-hidden />
          <span>{label}</span>
        </Link>
      ))}
    </nav>
  );
}
