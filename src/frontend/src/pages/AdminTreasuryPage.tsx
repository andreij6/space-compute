import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { paymentsActor, treasuryActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';

export const AdminTreasuryPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [watchTarget, setWatchTarget] = useState('');
  const [watchMinDays, setWatchMinDays] = useState('7');
  const [watchTargetDays, setWatchTargetDays] = useState('30');
  const [withdrawTo, setWithdrawTo] = useState('');
  const [withdrawAmount, setWithdrawAmount] = useState('');

  const status = useQuery({
    queryKey: ['admin', 'treasury', 'status'],
    queryFn: () => treasuryActor(identity!).status(),
  });
  const health = useQuery({
    queryKey: ['admin', 'treasury', 'health'],
    queryFn: () => treasuryActor(identity!).health(),
  });
  const history = useQuery({
    queryKey: ['admin', 'treasury', 'history'],
    queryFn: () => treasuryActor(identity!).history(null, 20),
  });
  const proposals = useQuery({
    queryKey: ['admin', 'treasury', 'proposals'],
    queryFn: async () => unwrapAdmin(await treasuryActor(identity!).admin_proposals()),
  });
  const paymentsOverview = useQuery({
    queryKey: ['admin', 'payments', 'overview'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_overview()),
  });

  const invalidateTreasury = () => {
    queryClient.invalidateQueries({ queryKey: ['admin', 'treasury'] });
  };

  const topupNow = useMutation({
    mutationFn: async (canister: string) =>
      unwrapAdmin(await treasuryActor(identity!).admin_topup_now(canister ? Principal.fromText(canister) : null)),
    onSuccess: invalidateTreasury,
  });
  const watch = useMutation({
    mutationFn: async () =>
      unwrapAdmin(
        await treasuryActor(identity!).admin_watch(Principal.fromText(watchTarget), Number(watchMinDays), Number(watchTargetDays)),
      ),
    onSuccess: invalidateTreasury,
  });
  const unwatch = useMutation({
    mutationFn: async (canister: Principal) => unwrapAdmin(await treasuryActor(identity!).admin_unwatch(canister)),
    onSuccess: invalidateTreasury,
  });
  const withdraw = useMutation({
    mutationFn: async () =>
      unwrapAdmin(await treasuryActor(identity!).admin_withdraw(Principal.fromText(withdrawTo), BigInt(withdrawAmount))),
    onSuccess: invalidateTreasury,
  });
  const approve = useMutation({
    mutationFn: async (proposalId: bigint) => unwrapAdmin(await treasuryActor(identity!).admin_approve(proposalId)),
    onSuccess: invalidateTreasury,
  });
  const toggleNonIcpIntake = useMutation({
    mutationFn: async () =>
      unwrapAdmin(
        await paymentsActor(identity!).admin_pause({
          ...paymentsOverview.data!.paused,
          non_icp: !paymentsOverview.data!.paused.non_icp,
        }),
      ),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'payments', 'overview'] }),
  });

  return (
    <div>
      <h1>Treasury</h1>
      <AdminNav />

      {status.isPending && <p>Loading treasury status…</p>}
      {status.data && (
        <dl>
          <dt>ICP balance (e8s)</dt>
          <dd>{status.data.icp_balance_e8s.toString()}</dd>
          <dt>Reserve floor (e8s)</dt>
          <dd>{status.data.reserve_e8s.toString()}</dd>
          <dt>Daily burn (cycles)</dt>
          <dd>{status.data.daily_burn_cycles.toString()}</dd>
          <dt>Projected runway (months)</dt>
          <dd>{status.data.projected_runway_months}</dd>
        </dl>
      )}

      {health.data && (
        <p role={health.data.reserve_breached ? 'alert' : 'status'}>
          Health: {health.data.reserve_breached ? 'reserve breached' : 'ok'}, min runway {health.data.min_runway_days} days
          {health.data.worst_canister ? ` (worst: ${health.data.worst_canister.toText()})` : ''}
        </p>
      )}

      {paymentsOverview.data && (
        <p>
          Non-ICP intake: {paymentsOverview.data.paused.non_icp ? 'paused' : 'open'}{' '}
          <ConfirmAction
            label={paymentsOverview.data.paused.non_icp ? 'open intake' : 'pause intake'}
            phrase="non_icp"
            disabled={toggleNonIcpIntake.isPending}
            onConfirm={() => toggleNonIcpIntake.mutate()}
          />
        </p>
      )}

      {status.data && (
        <table>
          <caption>Per-canister cycles runway</caption>
          <thead>
            <tr>
              <th>Canister</th>
              <th>Cycles</th>
              <th>Burn/day</th>
              <th>Runway (days)</th>
            </tr>
          </thead>
          <tbody>
            {status.data.canisters.map((c) => (
              <tr key={c.canister.toText()}>
                <td>{c.canister.toText()}</td>
                <td>{c.cycles.toString()}</td>
                <td>{c.burn_per_day.toString()}</td>
                <td>{c.runway_days}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <section aria-label="Top up now">
        <h2>Top up now</h2>
        <ConfirmAction label="top up all watched" phrase="top up" disabled={topupNow.isPending} onConfirm={() => topupNow.mutate('')} />
        {topupNow.isError && <p role="alert">{topupNow.error.message}</p>}
      </section>

      <section aria-label="Watch list">
        <h2>Watch a canister</h2>
        <label>
          Canister principal
          <input value={watchTarget} onChange={(e) => setWatchTarget(e.target.value)} />
        </label>
        <label>
          Min runway days
          <input value={watchMinDays} onChange={(e) => setWatchMinDays(e.target.value)} />
        </label>
        <label>
          Target runway days
          <input value={watchTargetDays} onChange={(e) => setWatchTargetDays(e.target.value)} />
        </label>
        <button type="button" disabled={watch.isPending || !watchTarget} onClick={() => watch.mutate()}>
          Watch
        </button>
        <ConfirmAction
          label="unwatch"
          phrase={watchTarget || 'canister'}
          disabled={unwatch.isPending || !watchTarget}
          onConfirm={() => unwatch.mutate(Principal.fromText(watchTarget))}
        />
        {(watch.isError || unwatch.isError) && <p role="alert">{watch.error?.message ?? unwatch.error?.message}</p>}
      </section>

      <section aria-label="Withdraw (two-admin approval)">
        <h2>Withdraw (proposal + second-admin approval)</h2>
        <label>
          To principal
          <input value={withdrawTo} onChange={(e) => setWithdrawTo(e.target.value)} />
        </label>
        <label>
          Amount (e8s)
          <input value={withdrawAmount} onChange={(e) => setWithdrawAmount(e.target.value)} />
        </label>
        <ConfirmAction
          label="propose withdrawal"
          phrase="withdraw"
          disabled={withdraw.isPending || !withdrawTo || !withdrawAmount}
          onConfirm={() => withdraw.mutate()}
        />
        {withdraw.isError && <p role="alert">{withdraw.error.message}</p>}

        {proposals.data && (
          <table>
            <caption>Pending proposals</caption>
            <thead>
              <tr>
                <th>ID</th>
                <th>Proposer</th>
                <th>Action</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {proposals.data.map(([id, p]) => (
                <tr key={id.toString()}>
                  <td>{id.toString()}</td>
                  <td>{p.proposer.toText()}</td>
                  <td>{p.action.__kind__}</td>
                  <td>
                    <ConfirmAction
                      label="approve"
                      phrase={id.toString()}
                      disabled={approve.isPending}
                      onConfirm={() => approve.mutate(id)}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {approve.isError && <p role="alert">{approve.error.message}</p>}
      </section>

      <section aria-label="Deposit history">
        <h2>History</h2>
        {history.data && (
          <table>
            <thead>
              <tr>
                <th>Seq</th>
                <th>At</th>
                <th>Actor</th>
                <th>Kind</th>
              </tr>
            </thead>
            <tbody>
              {history.data[0].map((h) => (
                <tr key={h.seq.toString()}>
                  <td>{h.seq.toString()}</td>
                  <td>{new Date(Number(h.at / 1_000_000n)).toISOString()}</td>
                  <td>{h.actor.toText()}</td>
                  <td>{h.kind.__kind__}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
};
