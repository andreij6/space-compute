import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import type { Protocol } from '../bindings/platform';

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
    <div>
      <h1>Subjects & protocols</h1>
      <AdminNav />

      <section aria-label="Subjects">
        <h2>Subjects</h2>
        {subjects.isPending && <p>Loading subjects…</p>}
        {subjects.isError && <p role="alert">{subjects.error.message}</p>}
        {subjects.data && (
          <table>
            <thead>
              <tr>
                <th>Subject ID</th>
                <th>Field</th>
                <th>Active</th>
                <th>Has gold</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {subjects.data.map((s) => (
                <tr key={s.ref_.subject_id}>
                  <td>{s.ref_.subject_id}</td>
                  <td>{s.ref_.field}</td>
                  <td>{s.active ? 'yes' : 'no'}</td>
                  <td>{s.gold ? 'yes' : 'no'}</td>
                  <td>
                    <button
                      type="button"
                      disabled={setSubjectActive.isPending}
                      onClick={() => setSubjectActive.mutate({ subjectId: s.ref_.subject_id, active: !s.active })}
                    >
                      {s.active ? 'Deactivate' : 'Activate'}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {setSubjectActive.isError && <p role="alert">{setSubjectActive.error.message}</p>}
      </section>

      <section aria-label="Protocols">
        <h2>Protocol versions</h2>
        {protocols.isPending && <p>Loading protocols…</p>}
        {protocols.isError && <p role="alert">{protocols.error.message}</p>}
        {protocols.data && (
          <table>
            <thead>
              <tr>
                <th>Version</th>
                <th>Questions</th>
                <th>Categories</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {protocols.data.map((p) => (
                <tr key={p.version}>
                  <td>{p.version}</td>
                  <td>{p.questions.length}</td>
                  <td>{p.discovery_categories.length}</td>
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
        )}
        {setCurrentProtocol.isError && <p role="alert">{setCurrentProtocol.error.message}</p>}

        <h3>Add a protocol version (JSON)</h3>
        <textarea
          value={protocolJson}
          onChange={(e) => setProtocolJson(e.target.value)}
          rows={6}
          aria-label="Protocol JSON"
          placeholder='{"version":2,"guidance_md":"...","questions":[],"discovery_categories":[]}'
        />
        {parseError && <p role="alert">{parseError}</p>}
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
        {addProtocol.isError && <p role="alert">{addProtocol.error.message}</p>}
      </section>
    </div>
  );
};
