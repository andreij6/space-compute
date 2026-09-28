import { safeHref } from './urls';

export interface DossierFitsLink {
  filter: string;
  url: string;
}

export interface DossierSummary {
  field: string;
  programs: string;
  filterComposition: string;
  raDeg: number;
  decDeg: number;
  redshiftLabel: string;
  stellarMassLabel: string;
  magnificationLabel: string;
  fits: DossierFitsLink[];
  acknowledgment: string;
}

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null;

function resolveUrl(relative: string, base: string): string | null {
  try {
    return safeHref(new URL(relative, base).href, base);
  } catch {
    return null;
  }
}

function redshiftLabel(redshift: Record<string, unknown>): string {
  const zSpec = redshift.z_spec;
  if (typeof zSpec === 'number') {
    const grade = redshift.z_spec_grade;
    return `z = ${zSpec.toFixed(3)} (spectroscopic${typeof grade === 'string' ? `, grade ${grade}` : ''})`;
  }
  const zPhot = redshift.z_phot;
  if (typeof zPhot !== 'number') return 'z unavailable';
  const p16 = redshift.z_phot_p16;
  const p84 = redshift.z_phot_p84;
  if (typeof p16 === 'number' && typeof p84 === 'number') {
    return `z_phot = ${zPhot.toFixed(3)} (+${(p84 - zPhot).toFixed(3)}/-${(zPhot - p16).toFixed(3)})`;
  }
  return `z_phot = ${zPhot.toFixed(3)}`;
}

function stellarMassLabel(physical: Record<string, unknown>): string {
  const logMstar = physical.log_mstar;
  return typeof logMstar === 'number' ? `log M* = ${logMstar.toFixed(2)} (M☉)` : 'Stellar mass unavailable';
}

function magnificationLabel(lensing: Record<string, unknown>): string {
  const mag = lensing.magnification;
  return typeof mag === 'number' ? `${mag.toFixed(2)}×` : 'None (unlensed)';
}

function filterComposition(rgb: Record<string, unknown>): string {
  const channels = rgb.channels;
  if (!isRecord(channels)) return 'unavailable';
  const { r, g, b } = channels;
  return `R ${r ?? '?'} · G ${g ?? '?'} · B ${b ?? '?'}`;
}

export function summarizeDossier(raw: unknown, baseUrl: string): DossierSummary | null {
  if (!isRecord(raw)) return null;
  const target = raw.target;
  const images = raw.images;
  const redshift = raw.redshift;
  const physical = raw.physical;
  const lensing = raw.lensing;
  const provenance = raw.provenance;
  if (!isRecord(target) || !isRecord(images) || !isRecord(redshift) || !isRecord(physical) || !isRecord(lensing) || !isRecord(provenance)) {
    return null;
  }
  const rgb = images.rgb;
  const fitsRaw = images.fits;
  const fits: DossierFitsLink[] = Array.isArray(fitsRaw)
    ? fitsRaw
        .filter(isRecord)
        .map((f) => ({ filter: String(f.filter ?? '?'), url: resolveUrl(String(f.url ?? ''), baseUrl) }))
        .filter((f): f is DossierFitsLink => f.url !== null)
    : [];
  const programs = Array.isArray(provenance.programs)
    ? provenance.programs
        .filter(isRecord)
        .map((p) => String(p.name ?? ''))
        .filter(Boolean)
        .join(', ')
    : '';
  return {
    field: typeof target.field === 'string' ? target.field : 'unknown',
    programs: programs || 'unavailable',
    filterComposition: isRecord(rgb) ? filterComposition(rgb) : 'unavailable',
    raDeg: typeof target.ra_deg === 'number' ? target.ra_deg : 0,
    decDeg: typeof target.dec_deg === 'number' ? target.dec_deg : 0,
    redshiftLabel: redshiftLabel(redshift),
    stellarMassLabel: stellarMassLabel(physical),
    magnificationLabel: magnificationLabel(lensing),
    fits,
    acknowledgment: typeof provenance.acknowledgment === 'string' ? provenance.acknowledgment : '',
  };
}

export async function fetchDossier(url: string): Promise<DossierSummary | null> {
  try {
    const res = await fetch(url);
    if (!res.ok) return null;
    return summarizeDossier(await res.json(), url);
  } catch {
    return null;
  }
}
