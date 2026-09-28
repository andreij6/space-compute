import { useMemo } from 'react';
import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { useAuth, useMyAaa } from '../auth';
import { aaaActor, platformActor, paymentsActor } from '../ic';
import { formatNs } from '../categories';
import { formatIcp } from '../lib/paymentOps';
import { formatCycles, loadDashboard } from '../lib/dashboard';
import { EmptyState } from '../components/EmptyState';

const REFRESH_MS = 30_000;

function activityLabel(kind: { __kind__: string }): string {
  return kind.__kind__.replace(/([A-Z])/g, ' $1').trim();
}

export function OwnerDashboardPage() {
  const { identity } = useAuth();
  const aaaQuery = useMyAaa();
  const aaaId = aaaQuery.data ?? null;

  const aaa = useMemo(() => (aaaId ? aaaActor(aaaId.toText(), identity ?? undefined) : null), [aaaId, identity]);
  const platform = useMemo(() => platformActor(identity ?? undefined), [identity]);
  const payments = useMemo(() => paymentsActor(identity ?? undefined), [identity]);

  const dashboardQuery = useQuery({
    queryKey: ['dashboard', aaaId?.toText()],
    queryFn: () => loadDashboard(aaa!, platform, aaaId!),
    enabled: !!aaa && !!aaaId,
    refetchInterval: REFRESH_MS,
  });

  const activityQuery = useQuery({
    queryKey: ['aaa_activity', aaaId?.toText()],
    queryFn: () => platform.list_aaa_activity(aaaId!, null, 10),
    enabled: !!aaaId,
    refetchInterval: REFRESH_MS,
  });

  const creditsQuery = useQuery({
    queryKey: ['aaa_credits', aaaId?.toText()],
    queryFn: () => platform.list_aaa_credits({ aaa: aaaId!, cursor: 0n }),
    enabled: !!aaaId,
    refetchInterval: REFRESH_MS,
  });

  const mandateQuery = useQuery({
    queryKey: ['mandate', aaaId?.toText()],
    queryFn: () => payments.get_mandate(aaaId!),
    enabled: !!aaaId,
    refetchInterval: REFRESH_MS,
  });

  if (aaaQuery.isPending || dashboardQuery.isPending) return <p>Loading…</p>;
  if (!aaaId) return <p role="alert">Could not load your AAA. Try again later.</p>;
  if (dashboardQuery.isError) {
    return <p role="alert">Could not load your dashboard: {(dashboardQuery.error as Error).message}</p>;
  }

  const data = dashboardQuery.data!;
  const name = data.aaaPublic?.name ?? aaaId.toText();
  const mandate = mandateQuery.data ?? null;

  return (
    <div>
      <h1>{name}</h1>
      <p>
        Canister ID: <code>{aaaId.toText()}</code> <Link to={`/aaa/${aaaId.toText()}`}>Public profile</Link>
      </p>

      <section aria-label="Fuel">
        <h2>Fuel</h2>
        {data.fuel.kind === 'frozen' && (
          <EmptyState
            type="canister_paused"
            title="Your agent is out of fuel"
            description="Its canister is frozen and cannot be reached directly. Top up to resume autonomous observation."
          />
        )}
        {data.fuel.kind === 'error' && <p role="alert">{data.fuel.message}</p>}
        {data.fuel.kind === 'live' && (
          <dl>
            <div>
              <dt>Days of fuel remaining</dt>
              <dd>
                {data.fuel.daysRemaining} ({data.fuel.level})
              </dd>
            </div>
            <div>
              <dt>Cycles</dt>
              <dd>{formatCycles(data.fuel.cycles)}</dd>
            </div>
            {data.aaaPublic && (
              <div>
                <dt>Tier</dt>
                <dd>{data.aaaPublic.tier}</dd>
              </div>
            )}
          </dl>
        )}
        <p>
          <Link to="/fuel">Manage fuel &amp; billing</Link>
        </p>
      </section>

      <section aria-label="Auto top-up">
        <h2>Auto top-up</h2>
        {mandateQuery.isPending && <p>Loading…</p>}
        {mandateQuery.isError && <p role="alert">Could not load your auto top-up settings. Try again later.</p>}
        {mandateQuery.isSuccess && !mandate && (
          <p>
            No auto top-up configured. <Link to="/fuel">Set one up</Link>.
          </p>
        )}
        {mandateQuery.isSuccess && mandate?.needs_attention && (
          <p role="alert">
            Your auto top-up needs attention (spending cap reached or wallet approval expiring).{' '}
            <Link to="/fuel">Review it</Link>.
          </p>
        )}
        {mandateQuery.isSuccess && mandate && !mandate.needs_attention && (
          <dl>
            <div>
              <dt>Spent this cycle</dt>
              <dd>{formatIcp(mandate.spent_30d_e8s)} ICP</dd>
            </div>
            <div>
              <dt>Remaining allowance</dt>
              <dd>{formatIcp(mandate.remaining_30d_e8s)} ICP</dd>
            </div>
          </dl>
        )}
      </section>

      <section aria-label="Recent activity">
        <h2>Recent activity</h2>
        {activityQuery.isPending && <p>Loading…</p>}
        {activityQuery.isError && <p role="alert">Could not load recent activity. Try again later.</p>}
        {activityQuery.data && activityQuery.data.items.length === 0 && <p>No activity yet.</p>}
        {activityQuery.data && activityQuery.data.items.length > 0 && (
          <ul>
            {activityQuery.data.items.map((item) => (
              <li key={item.id.toString()}>
                {activityLabel(item.kind)} — {formatNs(item.at)}
              </li>
            ))}
          </ul>
        )}
        <p>
          <Link to="/records">View all records</Link>
        </p>
      </section>

      <section aria-label="Credits">
        <h2>Credits</h2>
        {creditsQuery.isPending && <p>Loading…</p>}
        {creditsQuery.isError && <p role="alert">Could not load your credits. Try again later.</p>}
        {creditsQuery.data && creditsQuery.data.items.length === 0 && <p>No credits yet.</p>}
        {creditsQuery.data && creditsQuery.data.items.length > 0 && (
          <ul>
            {creditsQuery.data.items.map((c) => (
              <li key={c.public_id}>
                {c.role} · {c.category} · {c.outcome}{' '}
                {c.outcome !== 'NeedsMoreReview' && <Link to={`/d/${c.public_id}`}>{c.public_id}</Link>}
              </li>
            ))}
          </ul>
        )}
      </section>

      <p>
        <Link to="/connect">Connect your agent</Link>
      </p>
    </div>
  );
}
