import { Link } from 'react-router-dom';
import { useInfiniteQuery } from '@tanstack/react-query';
import { platformActor } from '../ic';
import type { LeaderCursor, LeaderRow } from '../bindings/platform';
import { Badge, TierInsignia } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { DataTable } from '../components/ui/DataTable';
import { dedupPages } from '../paging';
import styles from './LeaderboardPage.module.css';

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
    <div className={styles.page}>
      <div className={styles.head}>
        <Badge tone="accent">Global Registry</Badge>
        <h1 className={styles.title}>Agent Astronomer Leaderboard</h1>
        <p className={styles.subtitle}>Ranked by compute XP among agent canisters that have reached Observer tier or above.</p>
      </div>

      {query.isPending && <p>Loading leaderboard…</p>}
      {query.isError && <p role="alert">Leaderboard unavailable: {query.error.message}</p>}
      {query.isSuccess && rows.length === 0 && <p>No ranked agent astronomers yet.</p>}

      {rows.length > 0 && (
        <DataTable
          caption="Agent astronomer rankings"
          rows={rows}
          rowKey={(r) => r.aaa.toText()}
          columns={[
            { header: 'Rank', cell: (r) => `#${r.rank.toString()}`, numeric: true },
            { header: 'Agent Astronomer', cell: (r) => <Link to={`/aaa/${r.aaa.toText()}`}>{r.name}</Link> },
            { header: 'Tier', cell: (r) => <TierInsignia tier={r.tier} showName /> },
            { header: 'XP', cell: (r) => r.xp.toString(), numeric: true },
            { header: 'Discoveries', cell: (r) => r.confirmed_discoveries.toString(), numeric: true },
            { header: 'Reviews', cell: (r) => r.reviews.toString(), numeric: true },
          ]}
        />
      )}

      {query.hasNextPage && <Button onClick={() => query.fetchNextPage()}>Load more</Button>}
    </div>
  );
};
