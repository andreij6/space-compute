import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { Button } from '../components/ui/Button';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';

export const AdminAAAsPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [namePrefix, setNamePrefix] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [suspendReason, setSuspendReason] = useState('');

  const list = useQuery({
    queryKey: ['admin', 'aaas', namePrefix],
    queryFn: async () =>
      unwrapAdmin(
        await platformActor(identity!).admin_list_aaas(
          namePrefix ? { name_prefix: namePrefix } : {},
          null,
          50,
        ),
      ),
  });

  const detail = useQuery({
    queryKey: ['admin', 'aaa', selected],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_get_aaa(Principal.fromText(selected!))),
    enabled: !!selected,
  });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['admin', 'aaas'] });
    queryClient.invalidateQueries({ queryKey: ['admin', 'aaa', selected] });
  };

  const suspend = useMutation({
    mutationFn: async () =>
      unwrapAdmin(await platformActor(identity!).admin_suspend_aaa(Principal.fromText(selected!), suspendReason)),
    onSuccess: invalidate,
  });
  const unsuspend = useMutation({
    mutationFn: async () => unwrapAdmin(await platformActor(identity!).admin_unsuspend_aaa(Principal.fromText(selected!))),
    onSuccess: invalidate,
  });
  const retryInstall = useMutation({
    mutationFn: async () => unwrapAdmin(await platformActor(identity!).admin_retry_install(Principal.fromText(selected!))),
    onSuccess: invalidate,
  });
  const setHouse = useMutation({
    mutationFn: async (isHouse: boolean) =>
      unwrapAdmin(await platformActor(identity!).admin_set_house(Principal.fromText(selected!), isHouse)),
    onSuccess: invalidate,
  });

  return (
    <AdminPageShell title="AAAs directory">
      <label className={shared.field}>
        Search by name prefix
        <input className={shared.input} value={namePrefix} onChange={(e) => setNamePrefix(e.target.value)} />
      </label>

      {list.isPending && <p>Loading AAAs…</p>}
      {list.isError && <p role="alert" className={shared.alert}>{list.error.message}</p>}

      {list.data && (
        <div className={table.wrap} role="region" aria-label="AAAs" tabIndex={0}>
          <table className={table.table}>
            <thead>
              <tr>
                <th scope="col">Name</th>
                <th scope="col">Owner</th>
                <th scope="col">Status</th>
                <th scope="col">Suspended</th>
              </tr>
            </thead>
            <tbody>
              {list.data.items.map((aaa) => (
                <tr key={aaa.owner.toText()}>
                  <td>
                    <Button variant="ghost" size="sm" onClick={() => setSelected(aaa.owner.toText())}>
                      {aaa.name}
                    </Button>
                  </td>
                  <td>{aaa.owner.toText()}</td>
                  <td>{aaa.status}</td>
                  <td>{aaa.admin_suspended ? 'yes' : 'no'}</td>
                </tr>
              ))}
              {list.data.items.length === 0 && (
                <tr>
                  <td colSpan={4} className={table.empty}>
                    No AAAs match.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {selected && (
        <section aria-label="AAA detail" className={card.card}>
          <h2 className={card.title}>{selected}</h2>
          {detail.isPending && <p>Loading detail…</p>}
          {detail.isError && <p role="alert" className={shared.alert}>{detail.error.message}</p>}
          {detail.data && (
            <dl>
              <dt>Status</dt>
              <dd>{detail.data.status}</dd>
              <dt>House AAA</dt>
              <dd className={shared.row}>
                {detail.data.is_house ? 'yes' : 'no'}{' '}
                <ConfirmAction
                  label={detail.data.is_house ? 'unset house' : 'set house'}
                  phrase={detail.data.name}
                  disabled={setHouse.isPending}
                  onConfirm={() => setHouse.mutate(!detail.data!.is_house)}
                />
              </dd>
              <dt>Wasm version</dt>
              <dd>{detail.data.wasm_version}</dd>
              <dt>Install attempts</dt>
              <dd>{detail.data.install_attempts}</dd>
              <dt>Admin suspended</dt>
              <dd>{detail.data.admin_suspended ? 'yes' : 'no'}</dd>
            </dl>
          )}

          {detail.data && !detail.data.admin_suspended && (
            <p className={shared.formRow}>
              <label className={shared.field}>
                Suspension reason
                <input className={shared.input} value={suspendReason} onChange={(e) => setSuspendReason(e.target.value)} />
              </label>
              <ConfirmAction
                label="suspend"
                phrase={detail.data.name}
                disabled={suspend.isPending || !suspendReason.trim()}
                onConfirm={() => suspend.mutate()}
              />
            </p>
          )}
          {detail.data && detail.data.admin_suspended && (
            <p className={shared.actions}>
              <ConfirmAction label="unsuspend" phrase={detail.data.name} disabled={unsuspend.isPending} onConfirm={() => unsuspend.mutate()} />
            </p>
          )}
          {detail.data && (
            <p className={shared.actions}>
              <ConfirmAction
                label="retry install"
                phrase={detail.data.name}
                disabled={retryInstall.isPending}
                onConfirm={() => retryInstall.mutate()}
              />
            </p>
          )}
          {(suspend.isError || unsuspend.isError || retryInstall.isError || setHouse.isError) && (
            <p role="alert" className={shared.alert}>
              {suspend.error?.message ?? unsuspend.error?.message ?? retryInstall.error?.message ?? setHouse.error?.message}
            </p>
          )}
        </section>
      )}
    </AdminPageShell>
  );
};
