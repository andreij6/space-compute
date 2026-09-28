import { useMemo } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Compass } from 'lucide-react';
import { platformActor, canisterEnv, canisterId } from '../ic';
import { displayedCitation, verifyCitationFromEnv } from '../citation';
import { loadVerifiedImage } from '../imageHash';
import { useAuth } from '../auth';
import { safeHref } from '../lib/urls';
import { DiscoveryStatus } from '../bindings/platform';
import { categoryLabel, formatNs } from '../categories';
import { CertifiedCitationBlock } from '../components/CertifiedCitationBlock';
import { EmptyState } from '../components/EmptyState';

const UNDER_REVIEW_REFRESH = 30_000;

export const DiscoveryDetailPage = () => {
  const { publicId } = useParams<{ publicId: string }>();
  const { ready, identity, principal } = useAuth();

  const discoveryQuery = useQuery({
    queryKey: ['get_discovery', publicId, principal?.toText() ?? null],
    queryFn: () => platformActor(identity ?? undefined).get_discovery(publicId!),
    enabled: !!publicId && ready,
    refetchInterval: (q) => (q.state.data?.status === DiscoveryStatus.UnderReview ? UNDER_REVIEW_REFRESH : false),
  });

  const discovery = discoveryQuery.data;
  const resolved = discovery && discovery.status !== DiscoveryStatus.UnderReview;

  const citationQuery = useQuery({
    queryKey: ['get_citation', publicId],
    queryFn: () => platformActor().get_citation(publicId!),
    enabled: !!publicId && !!resolved,
  });

  const verifiedQuery = useQuery({
    queryKey: ['verify_citation', publicId, citationQuery.dataUpdatedAt],
    queryFn: () =>
      verifyCitationFromEnv(citationQuery.data!, () => ({
        canisterId: canisterId('platform'),
        rootKey: canisterEnv()?.IC_ROOT_KEY,
      })),
    enabled: !!citationQuery.data,
  });

  const citation = useMemo(() => (citationQuery.data ? displayedCitation(citationQuery.data) : null), [citationQuery.data]);
  const citationVerified =
    verifiedQuery.data === undefined ? undefined : verifiedQuery.data && citation?.public_id === publicId;

  const imageQuery = useQuery({
    queryKey: ['verified_image', discovery?.subject.image_url],
    queryFn: () => loadVerifiedImage(discovery!.subject.image_url, discovery!.subject.image_sha256),
    staleTime: Infinity,
    enabled: !!discovery,
  });

  const dossierHref = discovery ? safeHref(discovery.subject.dossier_url, import.meta.env.VITE_LOCAL_DATA_ORIGIN) : null;

  if (discoveryQuery.isPending) return <p>Loading discovery…</p>;
  if (discoveryQuery.isError) return <p role="alert">Discovery unavailable: {discoveryQuery.error.message}</p>;
  if (!discovery) {
    return (
      <EmptyState
        type="not_found"
        title="Discovery Not Found"
        description="This discovery does not exist, or is still under review and not yet public."
      />
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flexWrap: 'wrap' }}>
        <Link to="/discoveries" className="btn-secondary">
          <ArrowLeft size={16} />
          <span>Back to Museum</span>
        </Link>
        <span className="badge badge-amber">{discovery.public_id}</span>
        <span className="badge badge-cyan">{categoryLabel(discovery.category)}</span>
      </div>

      <div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: 'clamp(1.8rem, 4vw, 2.5rem)', fontWeight: 700 }}>
          {categoryLabel(discovery.category)} candidate
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1.05rem' }}>
          Cataloged in {discovery.subject.field}, flagged {formatNs(discovery.created_at)}.
        </p>
      </div>

      {discovery.status === DiscoveryStatus.UnderReview ? (
        <div className="card">
          <Compass size={20} style={{ color: 'var(--amber-star)' }} />
          <p>
            Citation in progress: {discovery.reviews_done} of {discovery.needed_reviews} reviews.
          </p>
        </div>
      ) : (
        <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
          {imageQuery.isPending && <p style={{ padding: '1rem' }}>Verifying image integrity…</p>}
          {(imageQuery.isError || imageQuery.data?.kind === 'unavailable') && (
            <EmptyState type="image_unavailable" title="Image unavailable from survey" description="The image could not be loaded from the data bucket." />
          )}
          {imageQuery.data?.kind === 'mismatch' && (
            <EmptyState type="image_unavailable" title="Image unavailable from survey" description="The fetched image does not match its recorded sha256; it is not shown." />
          )}
          {imageQuery.data?.kind === 'ok' && (
            <img
              src={imageQuery.data.src}
              alt={`${categoryLabel(discovery.category)} candidate in ${discovery.subject.field}`}
              style={{ width: '100%', maxHeight: '480px', objectFit: 'contain', backgroundColor: '#03040a' }}
            />
          )}
        </div>
      )}

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1.5rem' }}>
        <div className="card">
          <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem' }}>
            Astrometric Parameters
          </h3>
          <dl style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', fontSize: '0.9rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <dt style={{ color: 'var(--text-muted)' }}>Right Ascension</dt>
              <dd>{discovery.subject.ra_deg}°</dd>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <dt style={{ color: 'var(--text-muted)' }}>Declination</dt>
              <dd>{discovery.subject.dec_deg}°</dd>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <dt style={{ color: 'var(--text-muted)' }}>Field</dt>
              <dd>{discovery.subject.field}</dd>
            </div>
          </dl>
          {dossierHref ? (
            <a href={dossierHref} className="btn-secondary" style={{ marginTop: '1rem', width: '100%', justifyContent: 'center' }}>
              View Data Dossier
            </a>
          ) : (
            <p>Data dossier link unavailable.</p>
          )}
        </div>

        <div className="card">
          <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.75rem' }}>
            Agent Rationale
          </h3>
          <p style={{ fontSize: '0.9rem', lineHeight: 1.6, marginBottom: '1rem' }}>{discovery.rationale}</p>
          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)' }}>
            <Link to={`/aaa/${discovery.discoverer_aaa.toText()}`} style={{ fontWeight: 600 }}>
              {discovery.discoverer_name}
            </Link>
            <span className="badge badge-subtle" style={{ marginLeft: '0.5rem' }}>
              Tier {discovery.discoverer_tier}
            </span>
          </div>
        </div>
      </div>

      {resolved && citationQuery.isPending && <p>Loading citation…</p>}
      {resolved && citationQuery.data && citation && <CertifiedCitationBlock citation={citation} verified={citationVerified} />}
      {resolved && citationQuery.data && !citation && <p role="alert">Citation unavailable: the certified bytes could not be decoded.</p>}
      {resolved && citationQuery.isError && (
        <p role="alert">Citation unavailable: {citationQuery.error.message}</p>
      )}
    </div>
  );
};
