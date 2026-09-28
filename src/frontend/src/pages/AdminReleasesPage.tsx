import { useRef, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';
import confirmStyles from '../components/ConfirmAction.module.css';

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export const AdminReleasesPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [version, setVersion] = useState('');
  const [staged, setStaged] = useState<{ bytes: Uint8Array; sha256: Uint8Array } | null>(null);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const wasms = useQuery({
    queryKey: ['admin', 'wasms'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_list_wasms()),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['admin', 'wasms'] });

  const upload = useMutation({
    mutationFn: async () => {
      if (!staged) throw new Error('Choose a wasm file first.');
      return unwrapAdmin(await platformActor(identity!).admin_upload_wasm(Number(version), staged.bytes, staged.sha256));
    },
    onSuccess: invalidate,
  });
  const approve = useMutation({
    mutationFn: async (v: number) => unwrapAdmin(await platformActor(identity!).admin_approve_wasm(v)),
    onSuccess: invalidate,
  });

  async function onFileChosen(file: File) {
    setUploadError(null);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const sha256 = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
      setStaged({ bytes, sha256 });
    } catch (e) {
      setUploadError(e instanceof Error ? e.message : 'Could not read the file.');
    }
  }

  return (
    <AdminPageShell title="AAA wasm releases">
      <section aria-label="Upload wasm" className={card.card}>
        <h2 className={card.title}>Upload a new AAA wasm binary</h2>
        <div className={shared.formRow}>
          <label className={confirmStyles.field}>
            Version
            <input className={confirmStyles.input} type="number" value={version} onChange={(e) => setVersion(e.target.value)} />
          </label>
          <label className={confirmStyles.field}>
            Wasm file
            <input
              ref={fileRef}
              type="file"
              accept=".wasm,.gz"
              onChange={(e) => e.target.files?.[0] && onFileChosen(e.target.files[0])}
            />
          </label>
        </div>
        {staged && (
          <p className={shared.hint}>
            Computed sha256: {toHex(staged.sha256)} ({staged.bytes.length} bytes)
          </p>
        )}
        {uploadError && <p role="alert" className={shared.alert}>{uploadError}</p>}
        <ConfirmAction
          label="upload wasm"
          phrase={version || 'version'}
          disabled={upload.isPending || !staged || !version}
          onConfirm={() => upload.mutate()}
        />
        {upload.isError && <p role="alert" className={shared.alert}>{upload.error.message}</p>}
      </section>

      <section aria-label="Release registry" className={card.card}>
        <h2 className={card.title}>Release registry</h2>
        {wasms.isPending && <p>Loading releases…</p>}
        {wasms.isError && <p role="alert" className={shared.alert}>{wasms.error.message}</p>}
        {wasms.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">Version</th>
                  <th scope="col">Sha256</th>
                  <th scope="col">Size (bytes)</th>
                  <th scope="col">Approved</th>
                  <th scope="col" />
                </tr>
              </thead>
              <tbody>
                {wasms.data.map(([v, meta]) => (
                  <tr key={v}>
                    <td>{v}</td>
                    <td>{toHex(meta.sha256)}</td>
                    <td className={table.numeric}>{meta.size.toString()}</td>
                    <td>{meta.approved ? 'yes' : 'no'}</td>
                    <td>
                      {!meta.approved && (
                        <ConfirmAction
                          label="approve"
                          phrase={toHex(meta.sha256).slice(0, 8)}
                          disabled={approve.isPending}
                          onConfirm={() => approve.mutate(v)}
                        />
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {approve.isError && <p role="alert" className={shared.alert}>{approve.error.message}</p>}
      </section>
    </AdminPageShell>
  );
};
