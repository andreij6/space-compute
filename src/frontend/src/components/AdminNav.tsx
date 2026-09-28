import { Link, useLocation } from 'react-router-dom';

const links: [string, string][] = [
  ['/admin', 'Overview'],
  ['/admin/aaas', 'AAAs'],
  ['/admin/discoveries', 'Discoveries'],
  ['/admin/data', 'Subjects & protocols'],
  ['/admin/payments', 'Payments'],
  ['/admin/releases', 'Releases'],
  ['/admin/settings', 'Settings'],
  ['/admin/invites', 'Invites'],
  ['/admin/treasury', 'Treasury'],
  ['/admin/moderation', 'Moderation'],
  ['/admin/audit', 'Audit'],
];

export function AdminNav() {
  const location = useLocation();
  return (
    <nav aria-label="Admin">
      {links.map(([to, label]) => (
        <Link key={to} to={to} aria-current={location.pathname === to ? 'page' : undefined}>
          {label}
        </Link>
      ))}
    </nav>
  );
}
