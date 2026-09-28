import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { Button } from '../components/ui/Button';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import type { Protocol } from '../bindings/platform';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';
import confirmStyles from '../components/ConfirmAction.module.css';

export const AdminCatalogPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [protocolJson, setProtocolJson] = useState('');
  const [parseError, setParseError] = useState<string | null>(null);

  const subjects = useQuery({
    queryKey: ['admin', 'subjects'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_list_subjects({}, null, 50)),
  });
  const protocols = useQuery({
    queryKey: ['admin', 'protocols'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_list_protocols()),
  });

  const invalidateSubjects = () => queryClient.invalidateQueries({ queryKey: ['admin', 'subjects'] });
  const invalidateProtocols = () => queryClient.invalidateQueries({ queryKey: ['admin', 'protocols'] });

  const setSubjectActive = useMutation({
    mutationFn: async ({ subjectId, active }: { subjectId: number; active: boolean }) =>
      unwrapAdmin(await platformActor(identity!).admin_set_subject_active(subjectId, active)),
    onSuccess: invalidateSubjects,
  });
  const addProtocol = useMutation({
    mutationFn: async (protocol: Protocol) => unwrapAdmin(await platformActor(identity!).admin_add_protocol(protocol)),
    onSuccess: invalidateProtocols,
  });
  const setCurrentProtocol = useMutation({
    mutationFn: async (version: number) => unwrapAdmin(await platformActor(identity!).admin_set_current_protocol(version)),
    onSuccess: invalidateProtocols,
  });

  return (
    <AdminPageShell title="Subjects & protocols">
      <section aria-label="Subjects" className={card.card}>
        <h2 className={card.title}>Subjects</h2>
        {subjects.isPending && <p>Loading subjects…</p>}
        {subjects.isError && <p role="alert" className={shared.alert}>{subjects.error.message}</p>}
        {subjects.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">Subject ID</th>
                  <th scope="col">Field</th>
                  <th scope="col">Active</th>
                  <th scope="col">Has gold</th>
                  <th scope="col" />
                </tr>
              </thead>
              <tbody>
                {subjects.data.items.map((s) => (
                  <tr key={s.ref_.subject_id}>
                    <td>{s.ref_.subject_id}</td>
                    <td>{s.ref_.field}</td>
                    <td>{s.active ? 'yes' : 'no'}</td>
                    <td>{s.gold ? 'yes' : 'no'}</td>
                    <td>
                      <Button
                        variant="secondary"
                        size="sm"
                        disabled={setSubjectActive.isPending}
                        onClick={() => setSubjectActive.mutate({ subjectId: s.ref_.subject_id, active: !s.active })}
                      >
                        {s.active ? 'Deactivate' : 'Activate'}
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {setSubjectActive.isError && <p role="alert" className={shared.alert}>{setSubjectActive.error.message}</p>}
      </section>

      <section aria-label="Protocols" className={card.card}>
        <h2 className={card.title}>Protocol versions</h2>
        {protocols.isPending && <p>Loading protocols…</p>}
        {protocols.isError && <p role="alert" className={shared.alert}>{protocols.error.message}</p>}
        {protocols.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">Version</th>
                  <th scope="col">Questions</th>
                  <th scope="col">Categories</th>
                  <th scope="col" />
                </tr>
              </thead>
              <tbody>
                {protocols.data.map((p) => (
                  <tr key={p.version}>
                    <td>{p.version}</td>
                    <td className={table.numeric}>{p.questions.length}</td>
                    <td className={table.numeric}>{p.discovery_categories.length}</td>
                    <td>
                      <ConfirmAction
                        label="set current"
                        phrase={String(p.version)}
                        disabled={setCurrentProtocol.isPending}
                        onConfirm={() => setCurrentProtocol.mutate(p.version)}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {setCurrentProtocol.isError && <p role="alert" className={shared.alert}>{setCurrentProtocol.error.message}</p>}

        <h3>Add a protocol version (JSON)</h3>
        <textarea
          className={confirmStyles.input}
          value={protocolJson}
          onChange={(e) => setProtocolJson(e.target.value)}
          rows={6}
          aria-label="Protocol JSON"
          placeholder='{"version":2,"guidance_md":"...","questions":[],"discovery_categories":[]}'
        />
        {parseError && <p role="alert" className={shared.alert}>{parseError}</p>}
        <ConfirmAction
          label="add protocol"
          phrase="add protocol"
          disabled={addProtocol.isPending}
          onConfirm={() => {
            try {
              setParseError(null);
              addProtocol.mutate(JSON.parse(protocolJson));
            } catch (e) {
              setParseError(e instanceof Error ? e.message : 'Invalid JSON.');
            }
          }}
        />
        {addProtocol.isError && <p role="alert" className={shared.alert}>{addProtocol.error.message}</p>}
      </section>
    </AdminPageShell>
  );
};
