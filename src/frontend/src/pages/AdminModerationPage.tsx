import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';

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
    <div>
      <h1>Moderation</h1>
      <AdminNav />
      <p>
        There is no automated flag feed yet; search by name prefix to find names that violate the blocklist or were
        reported out of band, then force-rename. The action is audit-logged and confirmed citations keep the name used
        at the time.
      </p>

      <label>
        Search AAA names
        <input value={namePrefix} onChange={(e) => setNamePrefix(e.target.value)} />
      </label>

      {aaas.isPending && namePrefix && <p>Searching…</p>}
      {aaas.isError && <p role="alert">{aaas.error.message}</p>}

      {aaas.data && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Owner</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {aaas.data.map((a) => (
              <tr key={a.owner.toText()}>
                <td>{a.name}</td>
                <td>{a.owner.toText()}</td>
                <td>
                  <button
                    type="button"
                    onClick={() => {
                      setRenameTarget(a.owner.toText());
                      setNewName(a.name);
                    }}
                  >
                    Rename
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {renameTarget && (
        <section aria-label="Force rename">
          <h2>Force-rename {renameTarget}</h2>
          <label>
            New name
            <input value={newName} onChange={(e) => setNewName(e.target.value)} />
          </label>
          <label>
            Reason
            <input value={reason} onChange={(e) => setReason(e.target.value)} />
          </label>
          <ConfirmAction
            label="rename"
            phrase={newName}
            disabled={rename.isPending || !newName.trim() || !reason.trim()}
            onConfirm={() => rename.mutate()}
          />
          {rename.isError && <p role="alert">{rename.error.message}</p>}
        </section>
      )}
    </div>
  );
};
