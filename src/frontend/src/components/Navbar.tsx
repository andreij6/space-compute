import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Telescope, 
  Compass, 
  Trophy, 
  PlusCircle, 
  LayoutDashboard, 
  Menu, 
  X, 
  ShieldCheck, 
  Wallet,
  BookOpen
} from 'lucide-react';
import { useAuth } from '../auth';

export const Navbar: React.FC = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const { principal, signOut } = useAuth();
  const location = useLocation();

  const isActive = (path: string) => location.pathname === path;

  const closeMobileMenu = () => setMobileMenuOpen(false);

  return (
    <header className="navbar">
      <div className="navbar-inner">
        <Link to="/" className="nav-brand" onClick={closeMobileMenu}>
          <img src="/logo.svg" alt="Space Compute Logo" />
          <span>Space Compute</span>
        </Link>

        <nav className="nav-links-desktop">
          <Link 
            to="/discoveries" 
            className={`nav-link ${isActive('/discoveries') ? 'active' : ''}`}
          >
            <Compass size={17} />
            <span>Museum</span>
          </Link>

          <Link 
            to="/leaderboard" 
            className={`nav-link ${isActive('/leaderboard') ? 'active' : ''}`}
          >
            <Trophy size={17} />
            <span>Leaderboard</span>
          </Link>

          <Link 
            to="/dashboard" 
            className={`nav-link ${isActive('/dashboard') ? 'active' : ''}`}
          >
            <LayoutDashboard size={17} />
            <span>Dashboard</span>
          </Link>

          <Link 
            to="/about" 
            className={`nav-link ${isActive('/about') ? 'active' : ''}`}
          >
            <BookOpen size={17} />
            <span>About</span>
          </Link>

          <Link 
            to="/admin" 
            className={`nav-link ${isActive('/admin') ? 'active' : ''}`}
          >
            <ShieldCheck size={17} />
            <span>Admin</span>
          </Link>
        </nav>

        <div className="nav-actions">
          <Link to="/spawn" className="btn-primary" style={{ textDecoration: 'none' }}>
            <PlusCircle size={16} />
            <span>Spawn AAA</span>
          </Link>

          {principal ? (
            <button className="btn-secondary" onClick={() => void signOut()} title={principal.toText()}>
              <Wallet size={15} />
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>
                {principal.toText().slice(0, 5)}… Sign out
              </span>
            </button>
          ) : (
            <Link to="/signin" className="btn-secondary">
              <Wallet size={15} />
              <span>Sign in</span>
            </Link>
          )}

          <button 
            className="mobile-menu-btn" 
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            aria-label="Toggle navigation menu"
          >
            {mobileMenuOpen ? <X size={22} /> : <Menu size={22} />}
          </button>
        </div>
      </div>

      {mobileMenuOpen && (
        <div className="mobile-drawer">
          <Link 
            to="/discoveries" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
              <Compass size={18} /> Discovery Museum
            </span>
            <span className="badge badge-amber">Gallery</span>
          </Link>

          <Link 
            to="/leaderboard" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
              <Trophy size={18} /> Agent Leaderboard
            </span>
            <span className="badge badge-cyan">Ranks</span>
          </Link>

          <Link 
            to="/dashboard" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
              <LayoutDashboard size={18} /> Owner Command Center
            </span>
            <span className="badge badge-subtle">Online</span>
          </Link>

          <Link 
            to="/connect" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
              <Telescope size={18} /> Connect Local Agent
            </span>
          </Link>

          <Link 
            to="/records" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span>Activity & Audit Records</span>
          </Link>

          <Link 
            to="/fuel" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span>Cycle Fuel & Top-Up</span>
          </Link>

          <Link 
            to="/about" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span>About & Treasury Runway</span>
          </Link>

          <Link 
            to="/practice" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span>Practice & Benchmarks</span>
          </Link>

          <Link 
            to="/admin" 
            className="mobile-nav-link" 
            onClick={closeMobileMenu}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.65rem' }}>
              <ShieldCheck size={18} /> Admin Console
            </span>
          </Link>
        </div>
      )}
    </header>
  );
};
