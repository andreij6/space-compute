import { useRef, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';

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
    <div>
      <h1>AAA wasm releases</h1>
      <AdminNav />

      <section aria-label="Upload wasm">
        <h2>Upload a new AAA wasm binary</h2>
        <label>
          Version
          <input type="number" value={version} onChange={(e) => setVersion(e.target.value)} />
        </label>
        <label>
          Wasm file
          <input
            ref={fileRef}
            type="file"
            accept=".wasm,.gz"
            onChange={(e) => e.target.files?.[0] && onFileChosen(e.target.files[0])}
          />
        </label>
        {staged && <p>Computed sha256: {toHex(staged.sha256)} ({staged.bytes.length} bytes)</p>}
        {uploadError && <p role="alert">{uploadError}</p>}
        <ConfirmAction
          label="upload wasm"
          phrase={version || 'version'}
          disabled={upload.isPending || !staged || !version}
          onConfirm={() => upload.mutate()}
        />
        {upload.isError && <p role="alert">{upload.error.message}</p>}
      </section>

      <section aria-label="Release registry">
        <h2>Release registry</h2>
        {wasms.isPending && <p>Loading releases…</p>}
        {wasms.isError && <p role="alert">{wasms.error.message}</p>}
        {wasms.data && (
          <table>
            <thead>
              <tr>
                <th>Version</th>
                <th>Sha256</th>
                <th>Size (bytes)</th>
                <th>Approved</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {wasms.data.map(([v, meta]) => (
                <tr key={v}>
                  <td>{v}</td>
                  <td>{toHex(meta.sha256)}</td>
                  <td>{meta.size.toString()}</td>
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
        )}
        {approve.isError && <p role="alert">{approve.error.message}</p>}
      </section>
    </div>
  );
};
