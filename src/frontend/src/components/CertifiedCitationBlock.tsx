import React, { useState } from 'react';
import { ShieldCheck, Copy, Check, Award, FileCode } from 'lucide-react';
import { Discovery } from '../types';

interface CertifiedCitationBlockProps {
  discovery: Discovery;
}

export const CertifiedCitationBlock: React.FC<CertifiedCitationBlockProps> = ({ discovery }) => {
  const [copiedBibtex, setCopiedBibtex] = useState(false);
  const [copiedHarvard, setCopiedHarvard] = useState(false);

  const bibtex = `@article{${discovery.publicId.toLowerCase()},
  author = {${discovery.discovererAaa} and Space Compute Community Reviewers},
  title = {${discovery.name}: A Candidate ${discovery.categoryLabel} Identified in ${discovery.survey}},
  journal = {Space Compute Canonical Citizen-Science Archive},
  year = {2026},
  volume = {1},
  pages = {${discovery.publicId}},
  note = {Internet Computer BLS Certified Witness: ${discovery.blsSignature}}
}`;

  const harvard = `${discovery.discovererAaa} et al., 2026. ${discovery.name} (${discovery.publicId}). Space Compute Verified Archive, BLS Sig: ${discovery.blsSignature}.`;

  const copyToClipboard = (text: string, type: 'bibtex' | 'harvard') => {
    navigator.clipboard.writeText(text);
    if (type === 'bibtex') {
      setCopiedBibtex(true);
      setTimeout(() => setCopiedBibtex(false), 2000);
    } else {
      setCopiedHarvard(true);
      setTimeout(() => setCopiedHarvard(false), 2000);
    }
  };

  return (
    <div className="card" style={{ borderColor: 'var(--border-active)' }}>
      <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: '0.75rem', marginBottom: '1.25rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <ShieldCheck size={22} style={{ color: 'var(--cyan-nebula)' }} />
          <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
            Certified Cryptographic Citation
          </h3>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <span className="badge badge-cyan">
            BLS Verified on IC Subnet
          </span>
          <span className="badge badge-amber">
            Quorum: {discovery.votesAgree}/{discovery.quorumNeeded}
          </span>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1rem', marginBottom: '1.25rem' }}>
        <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '0.85rem', borderRadius: 'var(--radius-sm)' }}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
            Primary Discoverer
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Award size={18} style={{ color: 'var(--amber-star)' }} />
            <span style={{ fontWeight: 600, color: 'var(--text-main)' }}>{discovery.discovererAaa}</span>
            <span className="badge badge-subtle">Tier {discovery.discovererTier}</span>
          </div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
            Owner: @{discovery.discovererOwner}
          </div>
        </div>

        <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '0.85rem', borderRadius: 'var(--radius-sm)' }}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
            Peer Consensus State
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ color: 'var(--cyan-nebula)', fontWeight: 600 }}>
              +{discovery.votesAgree} Agree
            </span>
            <span style={{ color: 'var(--red-nova)', fontWeight: 600 }}>
              -{discovery.votesDisagree} Disagree
            </span>
          </div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
            Status: {discovery.status.toUpperCase()}
          </div>
        </div>
      </div>

      <div style={{ marginBottom: '1.25rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
          <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)' }}>
            Harvard Scientific Citation
          </span>
          <button 
            className="btn-secondary" 
            style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}
            onClick={() => copyToClipboard(harvard, 'harvard')}
          >
            {copiedHarvard ? <Check size={12} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={12} />}
            <span>{copiedHarvard ? 'Copied' : 'Copy Citation'}</span>
          </button>
        </div>
        <div className="citation-block">
          {harvard}
        </div>
      </div>

      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
          <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
            <FileCode size={14} /> BibTeX Entry
          </span>
          <button 
            className="btn-secondary" 
            style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}
            onClick={() => copyToClipboard(bibtex, 'bibtex')}
          >
            {copiedBibtex ? <Check size={12} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={12} />}
            <span>{copiedBibtex ? 'Copied' : 'Copy BibTeX'}</span>
          </button>
        </div>
        <pre className="citation-block" style={{ whiteSpace: 'pre-wrap', maxHeight: '180px', overflowY: 'auto' }}>
          {bibtex}
        </pre>
      </div>
    </div>
  );
};
