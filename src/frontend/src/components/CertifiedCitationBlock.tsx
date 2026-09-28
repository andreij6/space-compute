import { useState } from 'react';
import { Link } from 'react-router-dom';
import { ShieldCheck, ShieldAlert, ShieldQuestion, Copy, Check, Award } from 'lucide-react';
import type { Citation } from '../bindings/platform';

interface CertifiedCitationBlockProps {
  citation: Citation;
  verified: boolean | undefined;
}

export const CertifiedCitationBlock = ({ citation, verified }: CertifiedCitationBlockProps) => {
  const [copied, setCopied] = useState(false);
  const copyText = () => {
    void navigator.clipboard.writeText(citation.text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="card" style={{ borderColor: 'var(--border-active)' }}>
      <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: '0.75rem', marginBottom: '1.25rem' }}>
        <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
          Certified Cryptographic Citation
        </h3>
        {verified === undefined && (
          <span className="badge badge-subtle" role="status">
            <ShieldQuestion size={14} /> Checking certificate…
          </span>
        )}
        {verified === true && (
          <span className="badge badge-cyan" role="status">
            <ShieldCheck size={14} /> Verified
          </span>
        )}
        {verified === false && (
          <span className="badge" role="status" style={{ color: 'var(--red-nova)' }}>
            <ShieldAlert size={14} /> Unverified
          </span>
        )}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1rem', marginBottom: '1.25rem' }}>
        <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '0.85rem', borderRadius: 'var(--radius-sm)' }}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
            Discoverer
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Award size={18} style={{ color: 'var(--amber-star)' }} />
            <Link to={`/aaa/${citation.discoverer.aaa.toText()}`} style={{ fontWeight: 600 }}>
              {citation.discoverer.aaa_name_at_time}
            </Link>
          </div>
        </div>

        <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '0.85rem', borderRadius: 'var(--radius-sm)' }}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
            Reviewers
          </div>
          <ul style={{ margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
            {citation.reviewers.map((r) => (
              <li key={r.credit.aaa.toText()}>
                <Link to={`/aaa/${r.credit.aaa.toText()}`}>{r.credit.aaa_name_at_time}</Link> — {r.vote}
              </li>
            ))}
          </ul>
        </div>
      </div>

      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
          <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)' }}>Citation</span>
          <button type="button" className="btn-secondary" onClick={copyText}>
            {copied ? <Check size={12} /> : <Copy size={12} />}
            <span>{copied ? 'Copied' : 'Copy Citation'}</span>
          </button>
        </div>
        <div className="citation-block">{citation.text}</div>
      </div>
    </div>
  );
};
