import { useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Compass, Minus, Plus, RotateCcw } from 'lucide-react';
import { platformActor, canisterEnv, canisterId } from '../ic';
import { displayedCitation, verifyCitationFromEnv } from '../citation';
import { loadVerifiedImage } from '../imageHash';
import { fetchDossier } from '../lib/dossier';
import { RESET_VIEW, ZOOM_MAX, panBy, toggleZoom, viewerTransform, zoomIn, zoomOut, type ViewerState } from '../lib/imageViewer';
import { useAuth } from '../auth';
import { safeHref } from '../lib/urls';
import { DiscoveryStatus } from '../bindings/platform';
import { categoryLabel, formatNs } from '../categories';
import { CertifiedCitationBlock } from '../components/CertifiedCitationBlock';
import { EmptyState } from '../components/EmptyState';
import { Badge } from '../components/ui/Badge';
import { Button, ButtonLink, buttonClass } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import styles from './DiscoveryDetailPage.module.css';

const UNDER_REVIEW_REFRESH = 30_000;

function ImageViewer({ src, alt }: { src: string; alt: string }) {
  const [view, setView] = useState<ViewerState>(RESET_VIEW);
  const [dragging, setDragging] = useState(false);
  const dragRef = useRef<{ x: number; y: number } | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      setView(e.deltaY < 0 ? zoomIn : zoomOut);
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  }, []);

  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (view.scale <= 1) return;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    dragRef.current = { x: e.clientX, y: e.clientY };
    setDragging(true);
  };
  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) return;
    const rect = containerRef.current?.getBoundingClientRect();
    const w = rect?.width || 1;
    const h = rect?.height || 1;
    const dxPercent = ((e.clientX - dragRef.current.x) / w) * 100;
    const dyPercent = ((e.clientY - dragRef.current.y) / h) * 100;
    dragRef.current = { x: e.clientX, y: e.clientY };
    setView((v) => panBy(v, dxPercent, dyPercent));
  };
  const endDrag = () => {
    dragRef.current = null;
    setDragging(false);
  };

  const zoomed = view.scale > 1;
  return (
    <div
      ref={containerRef}
      className={[styles.viewer, zoomed && styles.viewerZoomed, dragging && styles.viewerDragging].filter(Boolean).join(' ')}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={endDrag}
      onPointerCancel={endDrag}
      onPointerLeave={endDrag}
      onDoubleClick={() => setView(toggleZoom(view))}
    >
      <img src={src} alt={alt} className={styles.viewerImg} style={{ transform: viewerTransform(view) }} draggable={false} />
      <span className={styles.viewerHint}>Double-tap, Ctrl+scroll or the buttons zoom; drag to pan</span>
      <div className={styles.viewerControls}>
        <Button size="sm" onClick={() => setView(zoomOut)} aria-label="Zoom out" disabled={!zoomed}>
          <Minus size={16} aria-hidden />
        </Button>
        <Button size="sm" onClick={() => setView(RESET_VIEW)} aria-label="Reset zoom" disabled={!zoomed}>
          <RotateCcw size={16} aria-hidden />
        </Button>
        <Button size="sm" onClick={() => setView(zoomIn)} aria-label="Zoom in" disabled={view.scale >= ZOOM_MAX}>
          <Plus size={16} aria-hidden />
        </Button>
      </div>
    </div>
  );
}

function DataPanel({ dossierHref }: { dossierHref: string | null }) {
  const dossierQuery = useQuery({
    queryKey: ['dossier', dossierHref],
    queryFn: () => fetchDossier(dossierHref!),
    enabled: !!dossierHref,
    staleTime: Infinity,
  });
  const d = dossierQuery.data;
  if (!dossierHref) return <p>Data dossier link unavailable.</p>;
  if (dossierQuery.isPending) return <p>Loading data panel…</p>;
  if (!d) return <p>Data panel unavailable from the survey archive.</p>;
  return (
    <>
      <dl className={styles.dataList}>
        <div className={styles.dataRow}>
          <dt>Field / program</dt>
          <dd>
            {d.field} · {d.programs}
          </dd>
        </div>
        <div className={styles.dataRow}>
          <dt>Color composition</dt>
          <dd>{d.filterComposition}</dd>
        </div>
        <div className={styles.dataRow}>
          <dt>R.A. / Dec.</dt>
          <dd>
            {d.raDeg.toFixed(5)}° / {d.decDeg.toFixed(5)}°
          </dd>
        </div>
        <div className={styles.dataRow}>
          <dt>Redshift</dt>
          <dd>{d.redshiftLabel}</dd>
        </div>
        <div className={styles.dataRow}>
          <dt>Stellar mass</dt>
          <dd>{d.stellarMassLabel}</dd>
        </div>
        <div className={styles.dataRow}>
          <dt>Magnification</dt>
          <dd>{d.magnificationLabel}</dd>
        </div>
      </dl>
      {d.fits.length > 0 && (
        <div className={styles.fitsList}>
          {d.fits.map((f) => (
            <a key={f.filter} href={f.url} className={buttonClass({ variant: 'secondary', size: 'sm' })}>
              {f.filter.toUpperCase()} FITS
            </a>
          ))}
        </div>
      )}
      {d.acknowledgment && <p className={styles.acknowledgment}>{d.acknowledgment}</p>}
    </>
  );
}

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
    <div className={styles.page}>
      <div className={styles.crumbs}>
        <ButtonLink to="/discoveries">
          <ArrowLeft size={16} aria-hidden />
          <span>Back to Museum</span>
        </ButtonLink>
        <Badge tone="accent">{discovery.public_id}</Badge>
        <Badge tone="success">{categoryLabel(discovery.category)}</Badge>
      </div>

      <div>
        <h1 className={styles.title}>{categoryLabel(discovery.category)} candidate</h1>
        <p className={styles.subtitle}>
          Cataloged in {discovery.subject.field}, flagged {formatNs(discovery.created_at)}.
        </p>
      </div>

      {discovery.status === DiscoveryStatus.UnderReview ? (
        <Card>
          <div className={styles.reviewCard}>
            <Compass size={20} style={{ color: 'var(--color-accent)' }} aria-hidden />
            <p>
              Citation in progress: {discovery.reviews_done} of {discovery.needed_reviews} reviews.
            </p>
          </div>
        </Card>
      ) : (
        <Card className={styles.imageCard}>
          {imageQuery.isPending && <p style={{ padding: 'var(--space-4)' }}>Verifying image integrity…</p>}
          {(imageQuery.isError || imageQuery.data?.kind === 'unavailable') && (
            <EmptyState type="image_unavailable" title="Image unavailable from survey" description="The image could not be loaded from the data bucket." />
          )}
          {imageQuery.data?.kind === 'mismatch' && (
            <EmptyState type="image_unavailable" title="Image unavailable from survey" description="The fetched image does not match its recorded sha256; it is not shown." />
          )}
          {imageQuery.data?.kind === 'ok' && (
            <ImageViewer src={imageQuery.data.src} alt={`${categoryLabel(discovery.category)} candidate in ${discovery.subject.field}`} />
          )}
        </Card>
      )}

      <div className={styles.grid}>
        <Card>
          <h2 className={styles.cardTitle}>Scientific Data Panel</h2>
          <DataPanel dossierHref={dossierHref} />
        </Card>

        <Card>
          <h2 className={styles.cardTitle}>Agent Rationale</h2>
          <p className={styles.rationale}>{discovery.rationale}</p>
          <div className={styles.discovererRow}>
            <Link to={`/aaa/${discovery.discoverer_aaa.toText()}`} className={styles.discovererName}>
              {discovery.discoverer_name}
            </Link>
            <Badge>Tier {discovery.discoverer_tier}</Badge>
          </div>
        </Card>
      </div>

      {resolved && citationQuery.isPending && <p>Loading citation…</p>}
      {resolved && citationQuery.data && citation && <CertifiedCitationBlock citation={citation} verified={citationVerified} />}
      {resolved && citationQuery.data && !citation && <p role="alert">Citation unavailable: the certified bytes could not be decoded.</p>}
      {resolved && citationQuery.isError && <p role="alert">Citation unavailable: {citationQuery.error.message}</p>}
    </div>
  );
};
