import { describe, expect, it, vi } from 'vitest';
import { fetchDossier, summarizeDossier } from './dossier';

const RAW = {
  schema: 'sc-dossier/1',
  target: { ra_deg: 214.9568704, dec_deg: 52.853008, field: 'ceers' },
  images: {
    fits: [{ filter: 'f115w', url: 'f115w.fits', sha256: 'a'.repeat(64) }],
    rgb: { channels: { r: 'f444w', g: 'f277w', b: 'f150w' }, url: 'rgb.png', sha256: 'b'.repeat(64) },
  },
  redshift: { z_phot: 0.068, z_phot_p16: 0.061, z_phot_p84: 0.073, z_spec: null, z_spec_grade: null },
  physical: { log_mstar: 8.02 },
  lensing: { magnification: null },
  provenance: {
    programs: [{ name: 'CEERS', pid: 1345 }],
    acknowledgment: 'This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope.',
  },
};

describe('summarizeDossier', () => {
  it('extracts the 05 §3 Data panel fields from a sc-dossier/1 document', () => {
    const out = summarizeDossier(RAW, 'https://data.example.com/v1/subjects/10021379/dossier.json');
    expect(out).toEqual({
      field: 'ceers',
      programs: 'CEERS',
      filterComposition: 'R f444w · G f277w · B f150w',
      raDeg: 214.9568704,
      decDeg: 52.853008,
      redshiftLabel: 'z_phot = 0.068 (+0.005/-0.007)',
      stellarMassLabel: 'log M* = 8.02 (M☉)',
      magnificationLabel: 'None (unlensed)',
      fits: [{ filter: 'f115w', url: 'https://data.example.com/v1/subjects/10021379/f115w.fits' }],
      acknowledgment: RAW.provenance.acknowledgment,
    });
  });

  it('prefers z_spec over z_phot when a spectroscopic redshift is present', () => {
    const out = summarizeDossier({ ...RAW, redshift: { ...RAW.redshift, z_spec: 6.9, z_spec_grade: 'A' } }, 'https://x/dossier.json');
    expect(out?.redshiftLabel).toBe('z = 6.900 (spectroscopic, grade A)');
  });

  it('shows a magnification when lensed', () => {
    const out = summarizeDossier({ ...RAW, lensing: { magnification: 3.2 } }, 'https://x/dossier.json');
    expect(out?.magnificationLabel).toBe('3.20×');
  });

  it('returns null for malformed input', () => {
    expect(summarizeDossier(null, 'https://x/dossier.json')).toBeNull();
    expect(summarizeDossier({}, 'https://x/dossier.json')).toBeNull();
  });
});

describe('fetchDossier', () => {
  it('returns null when the fetch fails or the response is not ok', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }));
    expect(await fetchDossier('https://x/dossier.json')).toBeNull();
    vi.stubGlobal(
      'fetch',
      vi.fn().mockRejectedValue(new Error('network')),
    );
    expect(await fetchDossier('https://x/dossier.json')).toBeNull();
    vi.unstubAllGlobals();
  });

  it('parses a successful response into a summary', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => RAW }));
    const out = await fetchDossier('https://data.example.com/v1/subjects/10021379/dossier.json');
    expect(out?.field).toBe('ceers');
    vi.unstubAllGlobals();
  });
});
