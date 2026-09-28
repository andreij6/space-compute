import { useQuery } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { useAuth } from '../auth';
import { platformActor, paymentsActor } from '../ic';
import { mergeAuditLogs, unwrapAdmin } from '../lib/admin';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';

export const AdminAuditPage: React.FC = () => {
  const { identity } = useAuth();

  const platformLog = useQuery({
    queryKey: ['admin', 'audit', 'platform'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_audit_log(null, 100)),
  });
  const paymentsLog = useQuery({
    queryKey: ['admin', 'audit', 'payments'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_audit_log(null, 100)),
  });

  const merged = platformLog.data && paymentsLog.data ? mergeAuditLogs(platformLog.data, paymentsLog.data.items) : null;

  return (
    <AdminPageShell title="Audit log">
      <p className={shared.hint}>Merged, append-only trail of every admin mutation on the platform and payments canisters.</p>

      {(platformLog.isPending || paymentsLog.isPending) && <p>Loading audit log…</p>}
      {platformLog.isError && <p role="alert" className={shared.alert}>{platformLog.error.message}</p>}
      {paymentsLog.isError && <p role="alert" className={shared.alert}>{paymentsLog.error.message}</p>}

      {merged && (
        <div className={table.wrap}>
          <table className={table.table}>
            <thead>
              <tr>
                <th scope="col">At</th>
                <th scope="col">Admin</th>
                <th scope="col">Method</th>
                <th scope="col">Summary</th>
              </tr>
            </thead>
            <tbody>
              {merged.map((entry, i) => (
                <tr key={`${entry.admin.toText()}-${entry.at.toString()}-${i}`}>
                  <td>{new Date(Number(entry.at / 1_000_000n)).toISOString()}</td>
                  <td>{entry.admin.toText()}</td>
                  <td>{entry.method}</td>
                  <td>{entry.summary}</td>
                </tr>
              ))}
              {merged.length === 0 && (
                <tr>
                  <td colSpan={4} className={table.empty}>
                    No audit entries yet.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </AdminPageShell>
  );
};
