import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Home, Compass, PlusCircle, LayoutDashboard, Trophy } from 'lucide-react';

export const MobileBottomBar: React.FC = () => {
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === '/' && location.pathname === '/') return true;
    if (path !== '/' && location.pathname.startsWith(path)) return true;
    return false;
  };

  return (
    <nav className="mobile-bottom-bar">
      <Link to="/" className={`bottom-bar-item ${isActive('/') ? 'active' : ''}`}>
        <Home size={20} />
        <span>Home</span>
      </Link>

      <Link to="/discoveries" className={`bottom-bar-item ${isActive('/discoveries') ? 'active' : ''}`}>
        <Compass size={20} />
        <span>Museum</span>
      </Link>

      <Link to="/spawn" className={`bottom-bar-item ${isActive('/spawn') ? 'active' : ''}`}>
        <PlusCircle size={22} style={{ color: 'var(--amber-star)' }} />
        <span>Spawn</span>
      </Link>

      <Link to="/dashboard" className={`bottom-bar-item ${isActive('/dashboard') ? 'active' : ''}`}>
        <LayoutDashboard size={20} />
        <span>Agent</span>
      </Link>

      <Link to="/leaderboard" className={`bottom-bar-item ${isActive('/leaderboard') ? 'active' : ''}`}>
        <Trophy size={20} />
        <span>Ranks</span>
      </Link>
    </nav>
  );
};
