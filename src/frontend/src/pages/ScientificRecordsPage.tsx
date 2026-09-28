import { useMemo, useState } from 'react';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { useAuth, useMyAaa } from '../auth';
import { aaaActor, platformActor } from '../ic';
import { formatNs } from '../categories';
import { dedupPages } from '../paging';
import { humanApiError } from '../lib/paymentOps';
import { answerLabel, loadRecordsPage, PROTOCOL_V1, type RecordRow } from '../lib/records';
import type { Protocol, Record_ } from '../bindings/aaa';

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
      {record.rationale != null && <p>{record.rationale}</p>}
      {record.answers.length > 0 && (
        <ul>
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
    <div>
      <h1>Activity &amp; records</h1>
      <button type="button" onClick={() => recordsQuery.refetch()}>
        Refresh
      </button>

      {recordsQuery.isPending && <p>Loading…</p>}
      {recordsQuery.isError && (
        <p role="alert">Could not load records: {(recordsQuery.error as Error).message}</p>
      )}
      {recordsQuery.isSuccess && rows.length === 0 && <p>No activity yet.</p>}

      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>When</th>
              <th>Kind</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={`${row.source}-${row.id}`}>
                <td>{formatNs(row.at)}</td>
                <td>{row.kindLabel}</td>
                <td>
                  <button type="button" onClick={() => setSelected(row)}>
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {recordsQuery.hasNextPage && (
        <button type="button" onClick={() => recordsQuery.fetchNextPage()}>
          Load more
        </button>
      )}

      {selected && (
        <section aria-label="Record detail">
          <h2>Record detail</h2>
          <button type="button" onClick={() => setSelected(null)}>
            Close
          </button>
          {selected.source === 'platform' && (
            <p>
              {selected.kindLabel} — {formatNs(selected.at)}
            </p>
          )}
          {selected.source === 'aaa' && recordDetailQuery.isPending && <p>Loading…</p>}
          {selected.source === 'aaa' && recordDetailQuery.isError && (
            <p role="alert">Could not load this record. Try again later.</p>
          )}
          {selected.source === 'aaa' && recordDetailQuery.data?.__kind__ === 'Err' && (
            <p role="alert">{humanApiError(recordDetailQuery.data.Err)}</p>
          )}
          {selected.source === 'aaa' && recordDetailQuery.data?.__kind__ === 'Ok' && recordDetailQuery.data.Ok && (
            <RecordDetail record={recordDetailQuery.data.Ok} protocol={protocolQuery.data ?? null} />
          )}
        </section>
      )}
    </div>
  );
}
