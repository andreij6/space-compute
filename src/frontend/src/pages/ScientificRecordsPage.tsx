import { useMemo, useState } from 'react';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { useAuth, useMyAaa } from '../auth';
import { aaaActor, platformActor } from '../ic';
import { formatNs } from '../categories';
import { dedupPages } from '../paging';
import { humanApiError } from '../lib/paymentOps';
import { answerLabel, loadRecordsPage, PROTOCOL_V1, type RecordRow } from '../lib/records';
import type { Protocol, Record_ } from '../bindings/aaa';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { DataTable } from '../components/ui/DataTable';
import { Dialog } from '../components/ui/Dialog';
import styles from './ScientificRecordsPage.module.css';

function RecordDetail({ record, protocol }: { record: Record_; protocol: Protocol | null }) {
  return (
    <div>
      <dl>
        <div>
          <dt>When</dt>
          <dd>{formatNs(record.at)}</dd>
        </div>
        {record.category != null && (
          <div>
            <dt>Category</dt>
            <dd>{record.category}</dd>
          </div>
        )}
        {record.outcome != null && (
          <div>
            <dt>Outcome</dt>
            <dd>{record.outcome}</dd>
          </div>
        )}
        {record.vote != null && (
          <div>
            <dt>Vote</dt>
            <dd>{record.vote}</dd>
          </div>
        )}
        <div>
          <dt>XP awarded</dt>
          <dd>{record.xp_awarded}</dd>
        </div>
        {record.agent_label != null && (
          <div>
            <dt>Agent</dt>
            <dd>{record.agent_label} (self-reported)</dd>
          </div>
        )}
      </dl>
      {record.rationale != null && <p className={styles.rationale}>{record.rationale}</p>}
      {record.answers.length > 0 && (
        <ul className={styles.answers}>
          {record.answers.map((a) => (
            <li key={`${a.question_id}:${a.answer_id}`}>{answerLabel(protocol, a)}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function ScientificRecordsPage() {
  const { identity } = useAuth();
  const aaaQuery = useMyAaa();
  const aaaId = aaaQuery.data ?? null;
  const aaa = useMemo(() => (aaaId ? aaaActor(aaaId.toText(), identity ?? undefined) : null), [aaaId, identity]);
  const platform = useMemo(() => platformActor(identity ?? undefined), [identity]);

  const [selected, setSelected] = useState<RecordRow | null>(null);

  const recordsQuery = useInfiniteQuery({
    queryKey: ['aaa_records', aaaId?.toText()],
    queryFn: ({ pageParam }: { pageParam: bigint | null }) => loadRecordsPage(aaa!, platform, aaaId!, pageParam),
    initialPageParam: null as bigint | null,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
    enabled: !!aaa && !!aaaId,
  });

  const protocolQuery = useQuery({
    queryKey: ['protocol', PROTOCOL_V1],
    queryFn: () => platform.get_protocol(PROTOCOL_V1),
  });

  const recordDetailQuery = useQuery({
    queryKey: ['aaa_record', aaaId?.toText(), selected?.id.toString()],
    queryFn: () => aaa!.get_record(selected!.id),
    enabled: !!aaa && !!selected && selected.source === 'aaa',
  });

  if (aaaQuery.isPending) return <p>Loading…</p>;
  if (!aaaId) return <p role="alert">Could not load your AAA. Try again later.</p>;

  const rows = dedupPages(
    recordsQuery.data?.pages.map((p) => p.items),
    (r) => `${r.source}-${r.id}`,
  );

  return (
    <div className={styles.page}>
      <div className={styles.toolbar}>
        <h1>Activity &amp; records</h1>
        <Button type="button" variant="secondary" size="sm" busy={recordsQuery.isFetching} onClick={() => recordsQuery.refetch()}>
          Refresh
        </Button>
      </div>

      {recordsQuery.isPending && <p>Loading…</p>}
      {recordsQuery.isError && (
        <p role="alert">Could not load records: {(recordsQuery.error as Error).message}</p>
      )}

      {!recordsQuery.isPending && (
        <Card>
          <DataTable
            caption="Activity and scientific records"
            rows={rows}
            rowKey={(row) => `${row.source}-${row.id}`}
            empty="No activity yet."
            columns={[
              { header: 'When', cell: (row) => formatNs(row.at) },
              { header: 'Kind', cell: (row) => row.kindLabel },
              {
                header: '',
                cell: (row) => (
                  <Button type="button" variant="ghost" size="sm" onClick={() => setSelected(row)}>
                    View
                  </Button>
                ),
              },
            ]}
          />
          {recordsQuery.hasNextPage && (
            <div className={styles.loadMore}>
              <Button
                type="button"
                variant="secondary"
                busy={recordsQuery.isFetchingNextPage}
                onClick={() => recordsQuery.fetchNextPage()}
              >
                Load more
              </Button>
            </div>
          )}
        </Card>
      )}

      <Dialog open={!!selected} onClose={() => setSelected(null)} title="Record detail">
        {selected?.source === 'platform' && (
          <p>
            {selected.kindLabel} — {formatNs(selected.at)}
          </p>
        )}
        {selected?.source === 'aaa' && recordDetailQuery.isPending && <p>Loading…</p>}
        {selected?.source === 'aaa' && recordDetailQuery.isError && (
          <p role="alert">Could not load this record. Try again later.</p>
        )}
        {selected?.source === 'aaa' && recordDetailQuery.data?.__kind__ === 'Err' && (
          <p role="alert">{humanApiError(recordDetailQuery.data.Err)}</p>
        )}
        {selected?.source === 'aaa' && recordDetailQuery.data?.__kind__ === 'Ok' && recordDetailQuery.data.Ok && (
          <RecordDetail record={recordDetailQuery.data.Ok} protocol={protocolQuery.data ?? null} />
        )}
        <Button type="button" variant="secondary" onClick={() => setSelected(null)}>
          Close
        </Button>
      </Dialog>
    </div>
  );
}
