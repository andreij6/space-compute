import React from 'react';
import { ExternalLink, Award, BookOpen, Telescope } from 'lucide-react';

export const CreditsPage: React.FC = () => {
  const surveys = [
    {
      name: 'CEERS (Cosmic Evolution Early Release Science)',
      pi: 'Steven L. Finkelstein et al.',
      desc: 'NIRCam and MIRI deep fields covering the Extended Groth Strip (EGS).',
      link: 'https://ceers.github.io'
    },
    {
      name: 'COSMOS-Web',
      pi: 'Jeyhan S. Kartaltepe, Caitlin M. Casey et al.',
      desc: 'Contiguous 0.54 deg² deep survey in COSMOS field with NIRCam.',
      link: 'https://cosmos.astro.caltech.edu'
    },
    {
      name: 'JADES (JWST Advanced Deep Extragalactic Survey)',
      pi: 'Daniel Eisenstein, Roberto Maiolino et al.',
      desc: 'Deep and medium NIRCam/NIRSpec imaging in GOODS-South and GOODS-North.',
      link: 'https://jades-survey.github.io'
    },
    {
      name: 'DJA (DAWN JWST Archive v7)',
      pi: 'Cosmic Dawn Center (Niels Bohr Institute)',
      desc: 'Uniformly reduced, calibrated and photometered mosaics and cutouts.',
      link: 'https://dawn-cph.github.io/dja/'
    }
  ];

  return (
    <div style={{ maxWidth: '850px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Scientific Attribution</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.5rem', fontWeight: 700 }}>
          Credits & Acknowledgements
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', marginTop: '0.25rem' }}>
          Formal astronomical survey acknowledgements and open science data providers.
        </p>
      </div>

      <div className="card">
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.75rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Telescope size={18} style={{ color: 'var(--amber-star)' }} />
          <span>Observatory & Space Agency Attribution</span>
        </h2>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem', lineHeight: 1.7 }}>
          This research is based on observations made with the NASA/ESA/CSA James Webb Space Telescope. 
          The data were obtained from the Mikulski Archive for Space Telescopes (MAST) at the Space Telescope 
          Science Institute (STScI), which is operated by the Association of Universities for Research in Astronomy, 
          Inc., under NASA contract NAS 5-03127.
        </p>
      </div>

      <div className="card">
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <BookOpen size={18} style={{ color: 'var(--cyan-nebula)' }} />
          <span>Survey Programs & Reduction Pipelines</span>
        </h2>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {surveys.map((survey) => (
            <div 
              key={survey.name}
              style={{
                padding: '1rem',
                backgroundColor: 'var(--bg-surface-elevated)',
                borderRadius: 'var(--radius-sm)',
                border: '1px solid var(--border-subtle)'
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ fontWeight: 600, fontSize: '0.95rem', color: 'var(--text-main)' }}>
                  {survey.name}
                </div>
                <a 
                  href={survey.link} 
                  target="_blank" 
                  rel="noreferrer"
                  style={{ color: 'var(--amber-star)', display: 'flex', alignItems: 'center', gap: '0.25rem', fontSize: '0.8rem' }}
                >
                  <span>Portal</span>
                  <ExternalLink size={12} />
                </a>
              </div>
              <div style={{ fontSize: '0.8rem', color: 'var(--cyan-nebula)', marginTop: '0.2rem' }}>
                {survey.pi}
              </div>
              <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.4rem', lineHeight: 1.5 }}>
                {survey.desc}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
