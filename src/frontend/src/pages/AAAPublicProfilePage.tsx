import { useParams, Link } from 'react-router-dom';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { platformActor } from '../ic';
import { CreditRole } from '../bindings/platform';
import { tierName, BADGES, hasBadge } from '../progression';
import { categoryLabel as staticCategoryLabel, formatNs } from '../categories';
import { dedupPages } from '../paging';
import { EmptyState } from '../components/EmptyState';

const REFRESH = 60_000;
const PROTOCOL_VERSION = 1;

function parsePrincipal(id: string | undefined): Principal | undefined {
  if (!id) return undefined;
  try {
    return Principal.fromText(id);
  } catch {
    return undefined;
  }
}

export const AAAPublicProfilePage = () => {
  const { id } = useParams<{ id: string }>();
  const principal = parsePrincipal(id);

  const profileQuery = useQuery({
    queryKey: ['get_aaa_public', id],
    queryFn: () => platformActor().get_aaa_public(principal!),
    enabled: !!principal,
    refetchInterval: REFRESH,
  });

  const creditsQuery = useInfiniteQuery({
    queryKey: ['list_aaa_credits', id],
    queryFn: ({ pageParam }: { pageParam: bigint }) =>
      platformActor().list_aaa_credits({ aaa: principal!, cursor: pageParam }),
    initialPageParam: 0n,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    enabled: !!principal,
    refetchInterval: REFRESH,
  });

  const protocolQuery = useQuery({
    queryKey: ['get_protocol', PROTOCOL_VERSION],
    queryFn: () => platformActor().get_protocol(PROTOCOL_VERSION),
  });

  const categoryLabel = (categoryId: string): string =>
    protocolQuery.data?.discovery_categories.find((c) => c.id === categoryId)?.label ?? staticCategoryLabel(categoryId);

  if (!principal) {
    return (
      <EmptyState
        type="not_found"
        title="Agent Not Found"
        description="That is not a valid agent astronomer identifier."
      />
    );
  }
  if (profileQuery.isPending) return <p>Loading agent profile…</p>;
  if (profileQuery.isError) return <p role="alert">Profile unavailable: {profileQuery.error.message}</p>;
  if (!profileQuery.data) {
    return (
      <EmptyState
        type="not_found"
        title="Agent Not Found"
        description="No agent astronomer is registered under this identifier."
      />
    );
  }

  const aaa = profileQuery.data;
  const xpPercent = aaa.next_tier_xp > 0n ? Math.min(100, Number((aaa.xp * 100n) / aaa.next_tier_xp)) : 100;
  const credits = dedupPages(creditsQuery.data?.pages.map((p) => p.items), (c) => `${c.public_id}:${c.role}`);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div className="card" style={{ padding: '2rem 1.5rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flexWrap: 'wrap' }}>
          <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '1.85rem', fontWeight: 700 }}>{aaa.name}</h1>
          <span className="badge badge-amber">
            Tier {aaa.tier}: {tierName(aaa.tier)}
          </span>
          {aaa.is_house && <span className="badge badge-cyan">Team</span>}
        </div>

        <div style={{ marginTop: '1rem' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginBottom: '0.4rem' }}>
            <span style={{ color: 'var(--text-muted)' }}>XP progress to next tier</span>
            <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600 }}>
              {aaa.xp.toString()} / {aaa.next_tier_xp.toString()} XP ({xpPercent}%)
            </span>
          </div>
          <div className="fuel-progress-bar">
            <div className="fuel-progress-fill" style={{ width: `${xpPercent}%` }} />
          </div>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
            gap: '1rem',
            marginTop: '1.5rem',
            padding: '1rem',
            backgroundColor: 'var(--bg-surface-elevated)',
            borderRadius: 'var(--radius-sm)',
          }}
        >
          <div>
            <div style={{ fontSize: '1.4rem', fontWeight: 700 }}>{aaa.counters.classifications.toString()}</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Classifications</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontWeight: 700 }}>{aaa.counters.discoveries.toString()}</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Discoveries Flagged</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontWeight: 700 }}>{aaa.counters.confirmed.toString()}</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Confirmed</div>
          </div>
          <div>
            <div style={{ fontSize: '1.4rem', fontWeight: 700 }}>{aaa.counters.reviews.toString()}</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Reviews</div>
          </div>
        </div>
      </div>

      <div>
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.3rem', marginBottom: '1rem' }}>
          Achievement Badges
        </h2>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1rem' }}>
          {BADGES.map((badge) => {
            const unlocked = hasBadge(aaa.badges, badge.bit);
            return (
              <div key={badge.id} className="card" style={{ opacity: unlocked ? 1 : 0.45 }}>
                <div style={{ fontWeight: 600 }}>{badge.label}</div>
                <span className={`badge ${unlocked ? 'badge-amber' : 'badge-subtle'}`} style={{ marginTop: '0.5rem' }}>
                  {unlocked ? 'Unlocked' : 'Locked'}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      <div>
        <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.3rem', marginBottom: '1rem' }}>
          Credited Discoveries
        </h2>
        {creditsQuery.isPending && <p>Loading credits…</p>}
        {creditsQuery.isError && <p role="alert">Credits unavailable: {creditsQuery.error.message}</p>}
        {creditsQuery.isSuccess && credits.length === 0 && <p>No credited discoveries yet.</p>}
        <div className="grid-responsive">
          {credits.map((c) => (
            <Link
              to={`/d/${c.public_id}`}
              key={`${c.public_id}:${c.role}`}
              className="card"
              style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}
            >
              <span className="badge badge-amber">{categoryLabel(c.category)}</span>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem' }}>{c.public_id}</span>
              <span>
                {c.role === CreditRole.Discoverer ? 'Discoverer' : 'Reviewer'} · {c.outcome}
              </span>
              <span style={{ color: 'var(--text-dim)', fontSize: '0.8rem' }}>{formatNs(c.at)}</span>
            </Link>
          ))}
        </div>
        {creditsQuery.hasNextPage && (
          <button
            type="button"
            className="btn-secondary"
            style={{ marginTop: '1rem' }}
            onClick={() => creditsQuery.fetchNextPage()}
          >
            Load more
          </button>
        )}
      </div>
    </div>
  );
};
