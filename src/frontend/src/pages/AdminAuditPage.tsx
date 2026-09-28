import { useQuery } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { useAuth } from '../auth';
import { platformActor, paymentsActor } from '../ic';
import { mergeAuditLogs, unwrapAdmin } from '../lib/admin';

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
    <div>
      <h1>Audit log</h1>
      <AdminNav />
      <p>Merged, append-only trail of every admin mutation on the platform and payments canisters.</p>

      {(platformLog.isPending || paymentsLog.isPending) && <p>Loading audit log…</p>}
      {platformLog.isError && <p role="alert">{platformLog.error.message}</p>}
      {paymentsLog.isError && <p role="alert">{paymentsLog.error.message}</p>}

      {merged && (
        <table>
          <thead>
            <tr>
              <th>At</th>
              <th>Admin</th>
              <th>Method</th>
              <th>Summary</th>
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
                <td colSpan={4}>No audit entries yet.</td>
              </tr>
            )}
          </tbody>
        </table>
      )}
    </div>
  );
};
