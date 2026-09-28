import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { ParamsEditor } from '../components/ParamsEditor';
import { useAuth } from '../auth';
import { paymentsActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import type { Params } from '../bindings/payments';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';

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
    <AdminPageShell title="Payments & ledgers (ICP only)">
      {overview.isPending && <p>Loading payments overview…</p>}
      {overview.isError && <p role="alert" className={shared.alert}>{overview.error.message}</p>}

      {overview.data && (
        <dl className={card.card}>
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

      <section aria-label="Recent payment ops" className={card.card}>
        <h2 className={card.title}>Recent payment ops</h2>
        {ops.isPending && <p>Loading ops…</p>}
        {ops.isError && <p role="alert" className={shared.alert}>{ops.error.message}</p>}
        {ops.data && (
          <div className={table.wrap} role="region" aria-label="Payment operations" tabIndex={0}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">ID</th>
                  <th scope="col">Kind</th>
                  <th scope="col">State</th>
                  <th scope="col">Amount (e8s)</th>
                </tr>
              </thead>
              <tbody>
                {ops.data.items.map((op) => (
                  <tr key={op.id.toString()}>
                    <td>{op.id.toString()}</td>
                    <td>{op.kind.__kind__}</td>
                    <td>{op.state.__kind__}</td>
                    <td className={table.numeric}>{op.amount_e8s.toString()}</td>
                  </tr>
                ))}
                {ops.data.items.length === 0 && (
                  <tr>
                    <td colSpan={4} className={table.empty}>
                      No ops recorded.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {overview.data && (
        <section aria-label="Payments params" className={card.card}>
          <h2 className={card.title}>Params</h2>
          <ParamsEditor
            base={overview.data.params}
            onSave={(params) => setParams.mutate(params)}
            saving={setParams.isPending}
            error={setParams.error?.message}
          />
        </section>
      )}
    </AdminPageShell>
  );
};
