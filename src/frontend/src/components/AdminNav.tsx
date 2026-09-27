import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  ShieldCheck, 
  Users, 
  Compass, 
  Database, 
  CreditCard, 
  UploadCloud, 
  Settings, 
  Ticket, 
  Vault, 
  AlertOctagon, 
  ScrollText 
} from 'lucide-react';

export const AdminNav: React.FC = () => {
  const location = useLocation();

  const links = [
    { to: '/admin', label: 'Overview', icon: <ShieldCheck size={16} /> },
    { to: '/admin/aaas', label: 'AAAs Directory', icon: <Users size={16} /> },
    { to: '/admin/discoveries', label: 'Review Queue', icon: <Compass size={16} /> },
    { to: '/admin/data', label: 'Data Catalog', icon: <Database size={16} /> },
    { to: '/admin/payments', label: 'Payments & Sagas', icon: <CreditCard size={16} /> },
    { to: '/admin/releases', label: 'Wasm Releases', icon: <UploadCloud size={16} /> },
    { to: '/admin/settings', label: 'Global Flags', icon: <Settings size={16} /> },
    { to: '/admin/invites', label: 'Invites Mint', icon: <Ticket size={16} /> },
    { to: '/admin/treasury', label: 'Treasury Ops', icon: <Vault size={16} /> },
    { to: '/admin/moderation', label: 'Moderation', icon: <AlertOctagon size={16} /> },
    { to: '/admin/audit', label: 'Audit Log', icon: <ScrollText size={16} /> },
  ];

  return (
    <div style={{ 
      display: 'flex', 
      gap: '0.5rem', 
      overflowX: 'auto', 
      paddingBottom: '0.75rem', 
      marginBottom: '1.5rem',
      borderBottom: '1px solid var(--border-subtle)'
    }}>
      {links.map((link) => {
        const isActive = location.pathname === link.to;
        return (
          <Link
            key={link.to}
            to={link.to}
            className={`btn-secondary ${isActive ? 'active' : ''}`}
            style={{
              padding: '0.4rem 0.8rem',
              fontSize: '0.8rem',
              whiteSpace: 'nowrap',
              borderColor: isActive ? 'var(--amber-star)' : undefined,
              backgroundColor: isActive ? 'var(--amber-glow)' : undefined
            }}
          >
            {link.icon}
            <span>{link.label}</span>
          </Link>
        );
      })}
    </div>
  );
};
