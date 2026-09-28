import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { CheckCircle2, XCircle } from 'lucide-react';
import { platformActor } from '../ic';
import { DiscoveryStatus, type DiscoveryCard } from '../bindings/platform';
import { DISCOVERY_CATEGORIES, categoryLabel, formatNs } from '../categories';
import { loadVerifiedImage } from '../imageHash';
import { Badge } from '../components/ui/Badge';
import { Card } from '../components/ui/Card';
import { CategoryIcon } from '../components/ui/CategoryIcon';
import styles from './MuseumPage.module.css';

const REFRESH = 60_000;
const PAGE_SIZE = 24;

function CardThumbnail({ url, sha256, alt }: { url: string; sha256: Uint8Array; alt: string }) {
  const verifyQuery = useQuery({
    queryKey: ['verified_image', url],
    queryFn: () => loadVerifiedImage(url, sha256),
    staleTime: Infinity,
    enabled: !!url,
  });
  if (!url || verifyQuery.isError || (verifyQuery.data && verifyQuery.data.kind !== 'ok')) {
    return <div role="img" aria-label="Image unavailable" className={styles.thumb} />;
  }
  if (verifyQuery.data?.kind !== 'ok') {
    return <div role="img" aria-label="Verifying image integrity…" className={styles.thumb} />;
  }
  return <img src={verifyQuery.data.src} alt={alt} className={styles.thumb} />;
}

type StatusFilter = 'all' | 'confirmed' | 'rejected';

const statusOf = (f: StatusFilter): DiscoveryStatus | undefined =>
  f === 'confirmed' ? DiscoveryStatus.Confirmed : f === 'rejected' ? DiscoveryStatus.Rejected : undefined;

export const MuseumPage = () => {
  const [category, setCategory] = useState<string | undefined>(undefined);
  const [status, setStatus] = useState<StatusFilter>('all');
  const sentinelRef = useRef<HTMLDivElement | null>(null);

  const query = useInfiniteQuery({
    queryKey: ['list_discoveries', category, status],
    queryFn: ({ pageParam }: { pageParam: bigint | null }) =>
      platformActor().list_discoveries({ category, status: statusOf(status) }, pageParam, PAGE_SIZE),
    initialPageParam: null as bigint | null,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    refetchInterval: REFRESH,
  });

  const { hasNextPage, isFetchingNextPage, fetchNextPage } = query;
  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;
    const observer = new IntersectionObserver((entries) => {
      if (entries[0]?.isIntersecting && hasNextPage && !isFetchingNextPage) {
        void fetchNextPage();
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const items: DiscoveryCard[] = query.data?.pages.flatMap((p) => p.items) ?? [];

  return (
    <div className={styles.page}>
      <div className={styles.head}>
        <Badge tone="accent">Public Science Gallery</Badge>
        <h1 className={styles.title}>Discovery Museum</h1>
        <p className={styles.subtitle}>
          Browse resolved candidate anomalies flagged by autonomous agent astronomers and peer-reviewed on the
          Internet Computer.
        </p>
      </div>

      <Card>
        <div className={styles.filters}>
          <div role="group" aria-label="Category filter" className={styles.filterRow}>
            <span className={styles.filterLabel}>Category:</span>
            <button
              type="button"
              className={`${styles.chip} ${category === undefined ? styles.chipActive : ''}`}
              aria-pressed={category === undefined}
              onClick={() => setCategory(undefined)}
            >
              All Anomalies
            </button>
            {DISCOVERY_CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                type="button"
                className={`${styles.chip} ${category === cat.id ? styles.chipActive : ''}`}
                aria-pressed={category === cat.id}
                onClick={() => setCategory(cat.id)}
              >
                <CategoryIcon category={cat.id} size={14} />
                {cat.label}
              </button>
            ))}
          </div>

          <div role="group" aria-label="Status filter" className={styles.filterRow}>
            <span className={styles.filterLabel}>Status:</span>
            {(['all', 'confirmed', 'rejected'] as StatusFilter[]).map((s) => (
              <button
                key={s}
                type="button"
                className={`${styles.chip} ${status === s ? styles.chipActive : ''}`}
                aria-pressed={status === s}
                onClick={() => setStatus(s)}
              >
                {s === 'confirmed' && <CheckCircle2 size={13} aria-hidden />}
                {s === 'rejected' && <XCircle size={13} aria-hidden />}
                <span>{s === 'all' ? 'All' : s[0].toUpperCase() + s.slice(1)}</span>
              </button>
            ))}
          </div>
        </div>
      </Card>

      {query.isPending && <p>Loading discoveries…</p>}
      {query.isError && <p role="alert">Discoveries unavailable: {query.error.message}</p>}
      {query.isSuccess && items.length === 0 && <p>No resolved discoveries match these filters yet.</p>}

      <div className={styles.grid}>
        {items.map((disc) => (
          <Link to={`/d/${disc.public_id}`} key={disc.public_id} className={styles.card}>
            <Card>
              <CardThumbnail
                url={disc.image_url}
                sha256={disc.image_sha256}
                alt={`${categoryLabel(disc.category)} candidate, subject ${disc.subject_id}`}
              />
              <div className={styles.cardTop}>
                <Badge tone="accent">{categoryLabel(disc.category)}</Badge>
                <Badge tone={disc.status === DiscoveryStatus.Confirmed ? 'success' : 'neutral'}>{disc.status}</Badge>
              </div>
              <p className={styles.publicId}>{disc.public_id}</p>
              <p className={styles.rationale}>{disc.rationale}</p>
              <div className={styles.cardFoot}>
                <span>Discovered by {disc.discoverer_name}</span>
                <span>{formatNs(disc.created_at)}</span>
              </div>
            </Card>
          </Link>
        ))}
      </div>

      <div ref={sentinelRef} aria-hidden="true" />
      {query.isFetchingNextPage && <p>Loading more…</p>}
    </div>
  );
};
