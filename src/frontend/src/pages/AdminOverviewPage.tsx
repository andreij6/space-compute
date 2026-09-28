import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor, paymentsActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import type { PauseFlags as PlatformPauseFlags } from '../bindings/platform';
import type { PauseFlags as PaymentsPauseFlags } from '../bindings/payments';

export const AdminOverviewPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();

  const platformOverview = useQuery({
    queryKey: ['admin', 'platform', 'overview'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_overview()),
  });
  const paymentsOverview = useQuery({
    queryKey: ['admin', 'payments', 'overview'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_overview()),
  });

  const platformPause = useMutation({
    mutationFn: async (flags: PlatformPauseFlags) => unwrapAdmin(await platformActor(identity!).admin_pause(flags)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'platform', 'overview'] }),
  });
  const paymentsPause = useMutation({
    mutationFn: async (flags: PaymentsPauseFlags) => unwrapAdmin(await paymentsActor(identity!).admin_pause(flags)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'payments', 'overview'] }),
  });

  const platform = platformOverview.data;
  const payments = paymentsOverview.data;

  return (
    <div>
      <h1>Admin overview</h1>
      <AdminNav />

      {(platformOverview.isPending || paymentsOverview.isPending) && <p>Loading overview…</p>}
      {platformOverview.isError && <p role="alert">{platformOverview.error.message}</p>}
      {paymentsOverview.isError && <p role="alert">{paymentsOverview.error.message}</p>}

      {platform && (
        <dl>
          <dt>Active AAAs</dt>
          <dd>{platform.total_aaas.toString()}</dd>
          <dt>Platform cycles</dt>
          <dd>{platform.cycles.toString()}</dd>
          <dt>Events (24h)</dt>
          <dd>{platform.events_24h.toString()}</dd>
          <dt>Platform audit entries</dt>
          <dd>{platform.audit_entries.toString()}</dd>
        </dl>
      )}

      {payments && (
        <dl>
          <dt>Failed payment ops</dt>
          <dd>{payments.failed_ops.toString()}</dd>
          <dt>Stuck payment ops</dt>
          <dd>{payments.stuck_ops.toString()}</dd>
          <dt>Treasury ICP balance (e8s)</dt>
          <dd>{payments.treasury_icp_balance_e8s.toString()}</dd>
        </dl>
      )}

      {platform && (
        <section aria-label="Platform circuit breakers">
          <h2>Platform circuit breakers</h2>
          {(['tasks', 'reviews', 'spawns'] as const).map((flag) => (
            <p key={flag}>
              {flag}: {platform.paused[flag] ? 'paused' : 'active'}{' '}
              <ConfirmAction
                label={platform.paused[flag] ? `unpause ${flag}` : `pause ${flag}`}
                phrase={flag}
                disabled={platformPause.isPending}
                onConfirm={() => platformPause.mutate({ ...platform.paused, [flag]: !platform.paused[flag] })}
              />
            </p>
          ))}
          {platformPause.isError && <p role="alert">{platformPause.error.message}</p>}
        </section>
      )}

      {payments && (
        <section aria-label="Payments circuit breakers">
          <h2>Payments circuit breakers (non-ICP flows stay pause-only)</h2>
          {(['topup', 'auto_topup', 'spawn', 'non_icp'] as const).map((flag) => (
            <p key={flag}>
              {flag}: {payments.paused[flag] ? 'paused' : 'active'}{' '}
              <ConfirmAction
                label={payments.paused[flag] ? `unpause ${flag}` : `pause ${flag}`}
                phrase={flag}
                disabled={paymentsPause.isPending}
                onConfirm={() => paymentsPause.mutate({ ...payments.paused, [flag]: !payments.paused[flag] })}
              />
            </p>
          ))}
          {paymentsPause.isError && <p role="alert">{paymentsPause.error.message}</p>}
        </section>
      )}
    </div>
  );
};
