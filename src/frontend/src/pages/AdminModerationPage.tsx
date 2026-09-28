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

export const AdminModerationPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [namePrefix, setNamePrefix] = useState('');
  const [renameTarget, setRenameTarget] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [reason, setReason] = useState('');

  const aaas = useQuery({
    queryKey: ['admin', 'moderation', 'aaas', namePrefix],
    queryFn: async () =>
      unwrapAdmin(await platformActor(identity!).admin_list_aaas(namePrefix ? { name_prefix: namePrefix } : {}, null, 50)),
    enabled: namePrefix.length > 0,
  });

  const rename = useMutation({
    mutationFn: async () =>
      unwrapAdmin(await platformActor(identity!).admin_rename_aaa(Principal.fromText(renameTarget!), newName, reason)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'moderation', 'aaas'] });
      setRenameTarget(null);
      setNewName('');
      setReason('');
    },
  });

  return (
    <AdminPageShell title="Moderation">
      <p className={shared.hint}>
        There is no automated flag feed yet; search by name prefix to find names that violate the blocklist or were
        reported out of band, then force-rename. The action is audit-logged and confirmed citations keep the name used
        at the time.
      </p>

      <label className={shared.field}>
        Search AAA names
        <input className={shared.input} value={namePrefix} onChange={(e) => setNamePrefix(e.target.value)} />
      </label>

      {aaas.isPending && namePrefix && <p>Searching…</p>}
      {aaas.isError && <p role="alert" className={shared.alert}>{aaas.error.message}</p>}

      {aaas.data && (
        <div className={table.wrap} role="region" aria-label="AAA names" tabIndex={0}>
          <table className={table.table}>
            <thead>
              <tr>
                <th scope="col">Name</th>
                <th scope="col">Owner</th>
                <th scope="col" />
              </tr>
            </thead>
            <tbody>
              {aaas.data.items.map((a) => (
                <tr key={a.owner.toText()}>
                  <td>{a.name}</td>
                  <td>{a.owner.toText()}</td>
                  <td>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => {
                        setRenameTarget(a.owner.toText());
                        setNewName(a.name);
                      }}
                    >
                      Rename
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {renameTarget && (
        <section aria-label="Force rename" className={card.card}>
          <h2 className={card.title}>Force-rename {renameTarget}</h2>
          <div className={shared.formRow}>
            <label className={shared.field}>
              New name
              <input className={shared.input} value={newName} onChange={(e) => setNewName(e.target.value)} />
            </label>
            <label className={shared.field}>
              Reason
              <input className={shared.input} value={reason} onChange={(e) => setReason(e.target.value)} />
            </label>
          </div>
          <ConfirmAction
            label="rename"
            phrase={newName}
            disabled={rename.isPending || !newName.trim() || !reason.trim()}
            onConfirm={() => rename.mutate()}
          />
          {rename.isError && <p role="alert" className={shared.alert}>{rename.error.message}</p>}
        </section>
      )}
    </AdminPageShell>
  );
};
