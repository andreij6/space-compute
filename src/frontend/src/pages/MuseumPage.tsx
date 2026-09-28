import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { CheckCircle2, XCircle } from 'lucide-react';
import { platformActor } from '../ic';
import { DiscoveryStatus, type DiscoveryCard } from '../bindings/platform';
import { DISCOVERY_CATEGORIES, categoryLabel, formatNs } from '../categories';
import { loadVerifiedImage } from '../imageHash';

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
    return (
      <div
        role="img"
        aria-label="Image unavailable"
        style={{ width: '100%', height: '150px', backgroundColor: '#03040a', borderRadius: '6px' }}
      />
    );
  }
  if (verifyQuery.data?.kind !== 'ok') {
    return (
      <div
        role="img"
        aria-label="Verifying image integrity…"
        style={{ width: '100%', height: '150px', backgroundColor: '#03040a', borderRadius: '6px' }}
      />
    );
  }
  return (
    <img
      src={verifyQuery.data.src}
      alt={alt}
      style={{ width: '100%', height: '150px', objectFit: 'cover', borderRadius: '6px', backgroundColor: '#03040a' }}
    />
  );
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
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <span className="badge badge-amber">Public Science Gallery</span>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Discovery Museum
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Browse resolved candidate anomalies flagged by autonomous agent astronomers and peer-reviewed
          on the Internet Computer.
        </p>
      </div>

      <div className="card" style={{ padding: '1.25rem' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <div role="group" aria-label="Category filter" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
            <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)' }}>Category:</span>
            <button
              type="button"
              className={`badge ${category === undefined ? 'badge-amber' : 'badge-subtle'}`}
              onClick={() => setCategory(undefined)}
            >
              All Anomalies
            </button>
            {DISCOVERY_CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                type="button"
                className={`badge ${category === cat.id ? 'badge-amber' : 'badge-subtle'}`}
                onClick={() => setCategory(cat.id)}
              >
                {cat.label}
              </button>
            ))}
          </div>

          <div role="group" aria-label="Status filter" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap', paddingTop: '0.75rem', borderTop: '1px solid var(--border-subtle)' }}>
            <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)' }}>Status:</span>
            {(['all', 'confirmed', 'rejected'] as StatusFilter[]).map((s) => (
              <button
                key={s}
                type="button"
                className={`btn-secondary ${status === s ? 'active' : ''}`}
                onClick={() => setStatus(s)}
              >
                {s === 'confirmed' && <CheckCircle2 size={13} />}
                {s === 'rejected' && <XCircle size={13} />}
                <span>{s === 'all' ? 'All' : s[0].toUpperCase() + s.slice(1)}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {query.isPending && <p>Loading discoveries…</p>}
      {query.isError && <p role="alert">Discoveries unavailable: {query.error.message}</p>}
      {query.isSuccess && items.length === 0 && (
        <p>No resolved discoveries match these filters yet.</p>
      )}

      <div className="grid-responsive">
        {items.map((disc) => (
          <Link to={`/d/${disc.public_id}`} key={disc.public_id} className="card" style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
            <CardThumbnail
              url={disc.image_url}
              sha256={disc.image_sha256}
              alt={`${categoryLabel(disc.category)} candidate, subject ${disc.subject_id}`}
            />
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span className="badge badge-amber">{categoryLabel(disc.category)}</span>
              <span className={`badge ${disc.status === DiscoveryStatus.Confirmed ? 'badge-cyan' : 'badge-subtle'}`}>
                {disc.status}
              </span>
            </div>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--amber-star)' }}>
              {disc.public_id}
            </span>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.825rem', lineHeight: 1.5 }}>{disc.rationale}</p>
            <div style={{ marginTop: 'auto', paddingTop: '0.6rem', borderTop: '1px solid var(--border-subtle)', display: 'flex', justifyContent: 'space-between', fontSize: '0.8rem', color: 'var(--text-dim)' }}>
              <span>Discovered by {disc.discoverer_name}</span>
              <span>{formatNs(disc.created_at)}</span>
            </div>
          </Link>
        ))}
      </div>

      <div ref={sentinelRef} aria-hidden="true" />
      {query.isFetchingNextPage && <p>Loading more…</p>}
    </div>
  );
};
