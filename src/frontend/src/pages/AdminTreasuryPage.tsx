import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { Button } from '../components/ui/Button';
import { useAuth } from '../auth';
import { paymentsActor, treasuryActor } from '../ic';
import { parseNat, parsePrincipal, proposalDetails, unwrapAdmin } from '../lib/admin';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';
import confirmStyles from '../components/ConfirmAction.module.css';

const U8_MAX = 255n;
const U32_MAX = 4_294_967_295n;
const U64_MAX = 18_446_744_073_709_551_615n;

export const AdminTreasuryPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [watchTarget, setWatchTarget] = useState('');
  const [watchPriority, setWatchPriority] = useState('1');
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
  const config = useQuery({
    queryKey: ['admin', 'treasury', 'config'],
    queryFn: async () => unwrapAdmin(await treasuryActor(identity!).admin_get_config()),
  });
  const paymentsOverview = useQuery({
    queryKey: ['admin', 'payments', 'overview'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_overview()),
  });

  const invalidateTreasury = () => {
    queryClient.invalidateQueries({ queryKey: ['admin', 'treasury'] });
  };

  const watchTargetParsed = parsePrincipal(watchTarget);
  const watchPriorityParsed = parseNat(watchPriority, 'Priority', U8_MAX);
  const watchTargetDaysParsed = parseNat(watchTargetDays, 'Target runway days', U32_MAX);
  const watchErrors = [watchTargetParsed, watchPriorityParsed, watchTargetDaysParsed].flatMap((r) => (r.ok ? [] : [r.error]));
  const withdrawToParsed = parsePrincipal(withdrawTo);
  const withdrawAmountParsed = parseNat(withdrawAmount, 'Amount (e8s)', U64_MAX);
  const withdrawErrors = [withdrawToParsed, withdrawAmountParsed].flatMap((r) => (r.ok ? [] : [r.error]));

  const topupNow = useMutation({
    mutationFn: async () => unwrapAdmin(await treasuryActor(identity!).admin_topup_now(null)),
    onSuccess: invalidateTreasury,
  });
  const watch = useMutation({
    mutationFn: async () => {
      if (!watchTargetParsed.ok || !watchPriorityParsed.ok || !watchTargetDaysParsed.ok) throw new Error(watchErrors.join(' '));
      return unwrapAdmin(
        await treasuryActor(identity!).admin_watch(
          watchTargetParsed.value,
          Number(watchPriorityParsed.value),
          Number(watchTargetDaysParsed.value),
        ),
      );
    },
    onSuccess: invalidateTreasury,
  });
  const unwatch = useMutation({
    mutationFn: async () => {
      if (!watchTargetParsed.ok) throw new Error(watchTargetParsed.error);
      return unwrapAdmin(await treasuryActor(identity!).admin_unwatch(watchTargetParsed.value));
    },
    onSuccess: invalidateTreasury,
  });
  const withdraw = useMutation({
    mutationFn: async () => {
      if (!withdrawToParsed.ok || !withdrawAmountParsed.ok) throw new Error(withdrawErrors.join(' '));
      return unwrapAdmin(await treasuryActor(identity!).admin_withdraw(withdrawToParsed.value, withdrawAmountParsed.value));
    },
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
    <AdminPageShell title="Treasury">
      {status.isPending && <p>Loading treasury status…</p>}
      {status.data && (
        <dl className={card.card}>
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
        <p role={health.data.reserve_breached ? 'alert' : 'status'} className={health.data.reserve_breached ? shared.alert : shared.status}>
          Health: {health.data.reserve_breached ? 'reserve breached' : 'ok'}, min runway {health.data.min_runway_days} days
          {health.data.worst_canister ? ` (worst: ${health.data.worst_canister.toText()})` : ''}
        </p>
      )}

      {paymentsOverview.data && (
        <p className={shared.row}>
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
        <div className={table.wrap}>
          <table className={table.table}>
            <caption className={table.caption}>Per-canister cycles runway</caption>
            <thead>
              <tr>
                <th scope="col">Canister</th>
                <th scope="col">Cycles</th>
                <th scope="col">Burn/day</th>
                <th scope="col">Runway (days)</th>
              </tr>
            </thead>
            <tbody>
              {status.data.canisters.map((c) => (
                <tr key={c.canister.toText()}>
                  <td>{c.canister.toText()}</td>
                  <td className={table.numeric}>{c.cycles.toString()}</td>
                  <td className={table.numeric}>{c.burn_per_day.toString()}</td>
                  <td className={table.numeric}>{c.runway_days}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <section aria-label="Top up now" className={card.card}>
        <h2 className={card.title}>Top up now</h2>
        <ConfirmAction label="top up all watched" phrase="top up" disabled={topupNow.isPending} onConfirm={() => topupNow.mutate()} />
        {topupNow.isError && <p role="alert" className={shared.alert}>{topupNow.error.message}</p>}
      </section>

      <section aria-label="Watch list" className={card.card}>
        <h2 className={card.title}>Watch a canister</h2>
        <div className={shared.formRow}>
          <label className={confirmStyles.field}>
            Canister principal
            <input className={confirmStyles.input} value={watchTarget} onChange={(e) => setWatchTarget(e.target.value)} />
          </label>
          <label className={confirmStyles.field}>
            Priority (0-255)
            <input className={confirmStyles.input} value={watchPriority} onChange={(e) => setWatchPriority(e.target.value)} />
          </label>
          <label className={confirmStyles.field}>
            Target runway days
            <input className={confirmStyles.input} value={watchTargetDays} onChange={(e) => setWatchTargetDays(e.target.value)} />
          </label>
        </div>
        {watchTarget && watchErrors.length > 0 && <p role="alert" className={shared.alert}>{watchErrors.join(' ')}</p>}
        <div className={shared.actions}>
          <Button variant="secondary" disabled={watch.isPending || watchErrors.length > 0} onClick={() => watch.mutate()}>
            Watch
          </Button>
          <ConfirmAction
            label="unwatch"
            phrase={watchTarget.trim() || 'canister'}
            disabled={unwatch.isPending || !watchTargetParsed.ok}
            onConfirm={() => unwatch.mutate()}
          />
        </div>
        {(watch.isError || unwatch.isError) && <p role="alert" className={shared.alert}>{watch.error?.message ?? unwatch.error?.message}</p>}
      </section>

      <section aria-label="Withdraw (two-admin approval)" className={card.card}>
        <h2 className={card.title}>Withdraw (proposal + second-admin approval)</h2>
        <div className={shared.formRow}>
          <label className={confirmStyles.field}>
            To principal
            <input className={confirmStyles.input} value={withdrawTo} onChange={(e) => setWithdrawTo(e.target.value)} />
          </label>
          <label className={confirmStyles.field}>
            Amount (e8s)
            <input className={confirmStyles.input} value={withdrawAmount} onChange={(e) => setWithdrawAmount(e.target.value)} />
          </label>
        </div>
        {(withdrawTo || withdrawAmount) && withdrawErrors.length > 0 && <p role="alert" className={shared.alert}>{withdrawErrors.join(' ')}</p>}
        <ConfirmAction
          label="propose withdrawal"
          phrase="withdraw"
          disabled={withdraw.isPending || withdrawErrors.length > 0}
          onConfirm={() => withdraw.mutate()}
        />
        {withdraw.isError && <p role="alert" className={shared.alert}>{withdraw.error.message}</p>}

        {proposals.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <caption className={table.caption}>Pending proposals</caption>
              <thead>
                <tr>
                  <th scope="col">ID</th>
                  <th scope="col">Proposer</th>
                  <th scope="col">Action</th>
                  <th scope="col" />
                </tr>
              </thead>
              <tbody>
                {proposals.data.map(([id, p]) => (
                  <tr key={id.toString()}>
                    <td>{id.toString()}</td>
                    <td>{p.proposer.toText()}</td>
                    <td>
                      {p.action.__kind__}
                      <ul aria-label={`Proposal ${id.toString()} details`} className={shared.detailList}>
                        {proposalDetails(p.action, config.data).map((line) => (
                          <li key={line}>{line}</li>
                        ))}
                      </ul>
                    </td>
                    <td>
                      <ConfirmAction
                        label="approve"
                        phrase={id.toString()}
                        disabled={approve.isPending || (p.action.__kind__ === 'SetConfig' && !config.data)}
                        onConfirm={() => approve.mutate(id)}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {approve.isError && <p role="alert" className={shared.alert}>{approve.error.message}</p>}
      </section>

      <section aria-label="Deposit history" className={card.card}>
        <h2 className={card.title}>History</h2>
        {history.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">Seq</th>
                  <th scope="col">At</th>
                  <th scope="col">Actor</th>
                  <th scope="col">Kind</th>
                </tr>
              </thead>
              <tbody>
                {history.data[0].map((h) => (
                  <tr key={h.seq.toString()}>
                    <td className={table.numeric}>{h.seq.toString()}</td>
                    <td>{new Date(Number(h.at / 1_000_000n)).toISOString()}</td>
                    <td>{h.actor.toText()}</td>
                    <td>{h.kind.__kind__}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </AdminPageShell>
  );
};
