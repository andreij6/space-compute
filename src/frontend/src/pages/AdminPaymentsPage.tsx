import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ParamsEditor } from '../components/ParamsEditor';
import { useAuth } from '../auth';
import { paymentsActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import type { Params } from '../bindings/payments';

export const AdminPaymentsPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();

  const overview = useQuery({
    queryKey: ['admin', 'payments', 'overview'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_overview()),
  });
  const ops = useQuery({
    queryKey: ['admin', 'payments', 'ops'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_list_ops({}, null, 50)),
  });

  const setParams = useMutation({
    mutationFn: async (params: Params) => unwrapAdmin(await paymentsActor(identity!).admin_set_params(params)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'payments', 'overview'] }),
  });

  return (
    <div>
      <h1>Payments & ledgers (ICP only)</h1>
      <AdminNav />

      {overview.isPending && <p>Loading payments overview…</p>}
      {overview.isError && <p role="alert">{overview.error.message}</p>}

      {overview.data && (
        <dl>
          <dt>Treasury ICP balance (e8s)</dt>
          <dd>{overview.data.treasury_icp_balance_e8s.toString()}</dd>
          <dt>Main ICP balance (e8s)</dt>
          <dd>{overview.data.main_icp_balance_e8s.toString()}</dd>
          <dt>Failed ops</dt>
          <dd>{overview.data.failed_ops.toString()}</dd>
          <dt>Stuck ops</dt>
          <dd>{overview.data.stuck_ops.toString()}</dd>
          <dt>Card packs</dt>
          <dd>{overview.data.features.card ? 'enabled' : 'disabled (ICP-only MVP)'}</dd>
        </dl>
      )}

      <section aria-label="Recent payment ops">
        <h2>Recent payment ops</h2>
        {ops.isPending && <p>Loading ops…</p>}
        {ops.isError && <p role="alert">{ops.error.message}</p>}
        {ops.data && (
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Kind</th>
                <th>State</th>
                <th>Amount (e8s)</th>
              </tr>
            </thead>
            <tbody>
              {ops.data.items.map((op) => (
                <tr key={op.id.toString()}>
                  <td>{op.id.toString()}</td>
                  <td>{op.kind.__kind__}</td>
                  <td>{op.state.__kind__}</td>
                  <td>{op.amount_e8s.toString()}</td>
                </tr>
              ))}
              {ops.data.items.length === 0 && (
                <tr>
                  <td colSpan={4}>No ops recorded.</td>
                </tr>
              )}
            </tbody>
          </table>
        )}
      </section>

      {overview.data && (
        <section aria-label="Payments params">
          <h2>Params</h2>
          <ParamsEditor
            base={overview.data.params}
            onSave={(params) => setParams.mutate(params)}
            saving={setParams.isPending}
            error={setParams.error?.message}
          />
        </section>
      )}
    </div>
  );
};
