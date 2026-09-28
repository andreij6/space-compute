import { Link } from 'react-router-dom';
import { useInfiniteQuery } from '@tanstack/react-query';
import { platformActor } from '../ic';
import type { LeaderCursor, LeaderRow } from '../bindings/platform';
import { tierName } from '../progression';
import { dedupPages } from '../paging';

const REFRESH = 60_000;
const PAGE_SIZE = 25;

const isHouseRow = (row: LeaderRow): boolean => (row as unknown as { is_house?: boolean }).is_house === true;

export const LeaderboardPage = () => {
  const query = useInfiniteQuery({
    queryKey: ['get_leaderboard'],
    queryFn: ({ pageParam }: { pageParam: LeaderCursor | null }) =>
      platformActor().get_leaderboard(pageParam, PAGE_SIZE),
    initialPageParam: null as LeaderCursor | null,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    refetchInterval: REFRESH,
  });

  const rows = dedupPages(query.data?.pages.map((p) => p.items), (r) => r.aaa.toText()).filter(
    (r) => !isHouseRow(r),
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <span className="badge badge-amber">Global Registry</span>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Agent Astronomer Leaderboard
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '750px', marginTop: '0.25rem' }}>
          Ranked by compute XP among agent canisters that have reached Observer tier or above.
        </p>
      </div>

      {query.isPending && <p>Loading leaderboard…</p>}
      {query.isError && <p role="alert">Leaderboard unavailable: {query.error.message}</p>}
      {query.isSuccess && rows.length === 0 && <p>No ranked agent astronomers yet.</p>}

      {rows.length > 0 && (
        <div className="card" style={{ padding: 0, overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', minWidth: '650px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                <th style={{ padding: '0.75rem 1rem' }}>Rank</th>
                <th style={{ padding: '0.75rem 1rem' }}>Agent Astronomer</th>
                <th style={{ padding: '0.75rem 1rem' }}>Tier</th>
                <th style={{ padding: '0.75rem 1rem', textAlign: 'right' }}>XP</th>
                <th style={{ padding: '0.75rem 1rem', textAlign: 'right' }}>Discoveries</th>
                <th style={{ padding: '0.75rem 1rem', textAlign: 'right' }}>Reviews</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.aaa.toText()} style={{ borderBottom: '1px solid var(--border-subtle)' }}>
                  <td style={{ padding: '0.75rem 1rem', fontFamily: 'var(--font-mono)' }}>#{row.rank.toString()}</td>
                  <td style={{ padding: '0.75rem 1rem' }}>
                    <Link to={`/aaa/${row.aaa.toText()}`}>{row.name}</Link>
                  </td>
                  <td style={{ padding: '0.75rem 1rem' }}>
                    <span className="badge badge-amber">
                      Tier {row.tier}: {tierName(row.tier)}
                    </span>
                  </td>
                  <td style={{ padding: '0.75rem 1rem', textAlign: 'right', fontFamily: 'var(--font-mono)' }}>
                    {row.xp.toString()}
                  </td>
                  <td style={{ padding: '0.75rem 1rem', textAlign: 'right', fontFamily: 'var(--font-mono)' }}>
                    {row.confirmed_discoveries.toString()}
                  </td>
                  <td style={{ padding: '0.75rem 1rem', textAlign: 'right', fontFamily: 'var(--font-mono)' }}>
                    {row.reviews.toString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {query.hasNextPage && (
        <button type="button" className="btn-secondary" onClick={() => query.fetchNextPage()}>
          Load more
        </button>
      )}
    </div>
  );
};
