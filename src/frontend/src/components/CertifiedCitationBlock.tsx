import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Award, Check, Copy, ShieldAlert, ShieldCheck, ShieldQuestion } from 'lucide-react';
import type { Citation } from '../bindings/platform';
import { Badge } from './ui/Badge';
import { Button } from './ui/Button';
import { Card } from './ui/Card';
import styles from './CertifiedCitationBlock.module.css';

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
    <Card tone="accent">
      <div className={styles.head}>
        <h3 className={styles.title}>Certified Cryptographic Citation</h3>
        {verified === undefined && (
          <Badge tone="neutral">
            <ShieldQuestion size={14} aria-hidden /> Checking certificate…
          </Badge>
        )}
        {verified === true && (
          <Badge tone="success">
            <ShieldCheck size={14} aria-hidden /> Verified
          </Badge>
        )}
        {verified === false && (
          <Badge tone="danger">
            <ShieldAlert size={14} aria-hidden /> Unverified
          </Badge>
        )}
      </div>

      <div className={styles.parties}>
        <div className={styles.partyBox}>
          <div className={styles.partyLabel}>Discoverer</div>
          <div className={styles.discoverer}>
            <Award size={18} style={{ color: 'var(--color-accent)' }} aria-hidden />
            <Link to={`/aaa/${citation.discoverer.aaa.toText()}`}>{citation.discoverer.aaa_name_at_time}</Link>
          </div>
        </div>

        <div className={styles.partyBox}>
          <div className={styles.partyLabel}>Reviewers</div>
          <ul className={styles.reviewers}>
            {citation.reviewers.map((r) => (
              <li key={r.credit.aaa.toText()}>
                <Link to={`/aaa/${r.credit.aaa.toText()}`}>{r.credit.aaa_name_at_time}</Link> — {r.vote}
              </li>
            ))}
          </ul>
        </div>
      </div>

      <div>
        <div className={styles.citationHead}>
          <span className={styles.citationLabel}>Citation</span>
          <Button size="sm" onClick={copyText}>
            {copied ? <Check size={12} aria-hidden /> : <Copy size={12} aria-hidden />}
            <span>{copied ? 'Copied' : 'Copy Citation'}</span>
          </Button>
        </div>
        <div className={styles.citationText}>{citation.text}</div>
      </div>
    </Card>
  );
};
