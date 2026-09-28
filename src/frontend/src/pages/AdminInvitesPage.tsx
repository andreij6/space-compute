import { useState } from 'react';
import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { paymentsActor } from '../ic';
import { belowMinSponsorCycles, inviteCodesToCsv, inviteStatus, MIN_SPONSOR_CYCLES, unwrapAdmin } from '../lib/admin';
import { dedupPages } from '../paging';

export const AdminInvitesPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [count, setCount] = useState('50');
  const [sponsorCycles, setSponsorCycles] = useState(MIN_SPONSOR_CYCLES.toString());
  const [expiresAt, setExpiresAt] = useState('');
  const belowMinCycles = belowMinSponsorCycles(sponsorCycles);

  const invites = useInfiniteQuery({
    queryKey: ['admin', 'invites', 'list'],
    queryFn: async ({ pageParam }: { pageParam: Uint8Array | null }) =>
      unwrapAdmin(await paymentsActor(identity!).admin_list_invites(pageParam, 100)),
    initialPageParam: null as Uint8Array | null,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
  });
  const inviteItems = dedupPages(invites.data?.pages.map((p) => p.items), (i) => i.code_hash);
  const nowNs = BigInt(invites.dataUpdatedAt) * 1_000_000n;
  const expiresAtMs = expiresAt ? Date.parse(expiresAt) : NaN;

  const mint = useMutation({
    mutationFn: async () =>
      unwrapAdmin(
        await paymentsActor(identity!).admin_mint_invites({
          count: Number(count),
          sponsor_cycles: BigInt(sponsorCycles || '0'),
          expires_at: BigInt(expiresAtMs) * 1_000_000n,
        }),
      ),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'invites', 'list'] }),
  });

  function downloadCsv(codes: string[]) {
    const blob = new Blob([inviteCodesToCsv(codes)], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'invite-codes.csv';
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div>
      <h1>Sponsored invites</h1>
      <AdminNav />

      <section aria-label="Mint invite batch">
        <h2>Mint a new batch</h2>
        <label>
          Count
          <input type="number" value={count} onChange={(e) => setCount(e.target.value)} />
        </label>
        <label>
          Sponsor cycles (minimum {MIN_SPONSOR_CYCLES.toString()}; below that an AAA can&apos;t install)
          <input value={sponsorCycles} onChange={(e) => setSponsorCycles(e.target.value)} />
        </label>
        {belowMinCycles && (
          <p role="alert">Sponsor cycles must be at least {MIN_SPONSOR_CYCLES.toString()}; payments rejects less with InvalidInput.</p>
        )}
        <label>
          Expires at
          <input type="datetime-local" value={expiresAt} onChange={(e) => setExpiresAt(e.target.value)} />
        </label>
        <ConfirmAction
          label="mint batch"
          phrase="mint batch"
          disabled={mint.isPending || !/^[1-9]\d*$/.test(count) || belowMinCycles || !Number.isFinite(expiresAtMs)}
          onConfirm={() => mint.mutate()}
        />
        {mint.isError && <p role="alert">{mint.error.message}</p>}
      </section>

      {mint.isSuccess && (
        <section aria-label="Minted codes">
          <h2>Codes (shown once)</h2>
          <ul>
            {mint.data.map((code) => (
              <li key={code}>{code}</li>
            ))}
          </ul>
          <button type="button" onClick={() => downloadCsv(mint.data)}>
            Download CSV
          </button>
        </section>
      )}

      <section aria-label="Invite batches">
        <h2>Minted, used and expired invites</h2>
        {invites.isPending && <p>Loading invites…</p>}
        {invites.isError && <p role="alert">{invites.error.message}</p>}
        {invites.isSuccess && inviteItems.length === 0 && <p>No invites minted yet.</p>}
        {inviteItems.length > 0 && (
          <table>
            <caption className="sr-only">Minted invite codes by hash, sponsor cycles, and status</caption>
            <thead>
              <tr>
                <th scope="col">Code hash</th>
                <th scope="col">Sponsor cycles</th>
                <th scope="col">Minted at</th>
                <th scope="col">Expires at</th>
                <th scope="col">Status</th>
                <th scope="col">Used by</th>
              </tr>
            </thead>
            <tbody>
              {inviteItems.map((invite) => (
                <tr key={invite.code_hash}>
                  <td>{invite.code_hash.slice(0, 12)}…</td>
                  <td>{invite.sponsor_cycles.toString()}</td>
                  <td>{new Date(Number(invite.minted_at / 1_000_000n)).toLocaleString()}</td>
                  <td>{new Date(Number(invite.expires_at / 1_000_000n)).toLocaleString()}</td>
                  <td>{inviteStatus(invite, nowNs)}</td>
                  <td>{invite.used_by?.toText() ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {invites.hasNextPage && (
          <button type="button" onClick={() => void invites.fetchNextPage()} disabled={invites.isFetchingNextPage}>
            {invites.isFetchingNextPage ? 'Loading…' : 'Load more'}
          </button>
        )}
        <p>Revoking unused invites is not available yet: payments has no revoke method.</p>
      </section>
    </div>
  );
};
