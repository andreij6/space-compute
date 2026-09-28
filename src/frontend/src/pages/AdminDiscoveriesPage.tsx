import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import { DiscoveryStatus, Vote, type HoneypotSpec } from '../bindings/platform';
import card from '../components/ui/Card.module.css';
import table from '../components/ui/DataTable.module.css';
import shared from '../styles/adminShared.module.css';
import confirmStyles from '../components/ConfirmAction.module.css';

function parseHoneypotJson(text: string): HoneypotSpec[] {
  const parsed = JSON.parse(text);
  if (!Array.isArray(parsed)) throw new Error('Expected a JSON array of honeypot specs.');
  return parsed.map((row) => ({
    subject_id: Number(row.subject_id),
    category: String(row.category),
    rationale: String(row.rationale),
    truth: row.truth === 'Agree' ? Vote.Agree : Vote.Disagree,
  }));
}

export const AdminDiscoveriesPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [starvingOnly, setStarvingOnly] = useState(false);
  const [honeypotJson, setHoneypotJson] = useState('[]');
  const [parseError, setParseError] = useState<string | null>(null);

  const discoveries = useQuery({
    queryKey: ['admin', 'discoveries', starvingOnly],
    queryFn: async () =>
      unwrapAdmin(
        await platformActor(identity!).admin_list_discoveries(
          { status: DiscoveryStatus.UnderReview, starving: starvingOnly || undefined },
          null,
          50,
        ),
      ),
  });

  const honeypotStats = useQuery({
    queryKey: ['admin', 'honeypot_stats'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_honeypot_stats()),
  });

  const addHoneypots = useMutation({
    mutationFn: async (specs: HoneypotSpec[]) => unwrapAdmin(await platformActor(identity!).admin_add_honeypots(specs)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin', 'honeypot_stats'] }),
  });

  return (
    <AdminPageShell title="Discoveries & review queue">
      <label className={shared.row}>
        <input type="checkbox" checked={starvingOnly} onChange={(e) => setStarvingOnly(e.target.checked)} />
        Starving only (past the review-starvation window)
      </label>

      {discoveries.isPending && <p>Loading discoveries…</p>}
      {discoveries.isError && <p role="alert" className={shared.alert}>{discoveries.error.message}</p>}

      {discoveries.data && (
        <div className={table.wrap}>
          <table className={table.table}>
            <thead>
              <tr>
                <th scope="col">Public ID</th>
                <th scope="col">Field</th>
                <th scope="col">Category</th>
                <th scope="col">Reviews</th>
                <th scope="col">Corroborations</th>
                <th scope="col">Honeypot</th>
                <th scope="col">Starving</th>
              </tr>
            </thead>
            <tbody>
              {discoveries.data.items.map((d) => (
                <tr key={d.public_id}>
                  <td>{d.public_id}</td>
                  <td>{d.field}</td>
                  <td>{d.category}</td>
                  <td>
                    {d.reviews_done}/{d.needed_reviews}
                  </td>
                  <td>{d.corroborations}</td>
                  <td>{d.is_honeypot ? 'yes' : 'no'}</td>
                  <td>{d.starving ? 'yes' : 'no'}</td>
                </tr>
              ))}
              {discoveries.data.items.length === 0 && (
                <tr>
                  <td colSpan={7} className={table.empty}>
                    No discoveries match.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      <section aria-label="Honeypot accuracy per reviewer" className={card.card}>
        <h2 className={card.title}>Honeypot accuracy per reviewer</h2>
        {honeypotStats.isPending && <p>Loading honeypot stats…</p>}
        {honeypotStats.isError && <p role="alert" className={shared.alert}>{honeypotStats.error.message}</p>}
        {honeypotStats.data && (
          <div className={table.wrap}>
            <table className={table.table}>
              <thead>
                <tr>
                  <th scope="col">Reviewer AAA</th>
                  <th scope="col">Hits</th>
                  <th scope="col">Trials</th>
                </tr>
              </thead>
              <tbody>
                {honeypotStats.data.map((s) => (
                  <tr key={s.reviewer_aaa.toText()}>
                    <td>{s.reviewer_aaa.toText()}</td>
                    <td className={table.numeric}>{s.hits}</td>
                    <td className={table.numeric}>{s.trials}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section aria-label="Add honeypots" className={card.card}>
        <h2 className={card.title}>Add honeypots (JSON upload)</h2>
        <p className={shared.hint}>There is deliberately no edit or delete for confirmed citations.</p>
        <textarea
          className={confirmStyles.input}
          value={honeypotJson}
          onChange={(e) => setHoneypotJson(e.target.value)}
          rows={6}
          aria-label="Honeypot batch JSON"
        />
        {parseError && <p role="alert" className={shared.alert}>{parseError}</p>}
        <ConfirmAction
          label="add honeypots"
          phrase="add honeypots"
          disabled={addHoneypots.isPending}
          onConfirm={() => {
            try {
              setParseError(null);
              addHoneypots.mutate(parseHoneypotJson(honeypotJson));
            } catch (e) {
              setParseError(e instanceof Error ? e.message : 'Invalid JSON.');
            }
          }}
        />
        {addHoneypots.isError && <p role="alert" className={shared.alert}>{addHoneypots.error.message}</p>}
        {addHoneypots.isSuccess && <p role="status" className={shared.status}>Added {addHoneypots.data} honeypots.</p>}
      </section>
    </AdminPageShell>
  );
};
