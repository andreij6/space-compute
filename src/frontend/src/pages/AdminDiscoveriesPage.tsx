import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AdminNav } from '../components/AdminNav';
import { ConfirmAction } from '../components/ConfirmAction';
import { useAuth } from '../auth';
import { platformActor } from '../ic';
import { unwrapAdmin } from '../lib/admin';
import { DiscoveryStatus, Vote, type HoneypotSpec } from '../bindings/platform';

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
    <div>
      <h1>Discoveries & review queue</h1>
      <AdminNav />

      <label>
        <input type="checkbox" checked={starvingOnly} onChange={(e) => setStarvingOnly(e.target.checked)} />
        Starving only (past the review-starvation window)
      </label>

      {discoveries.isPending && <p>Loading discoveries…</p>}
      {discoveries.isError && <p role="alert">{discoveries.error.message}</p>}

      {discoveries.data && (
        <table>
          <thead>
            <tr>
              <th>Public ID</th>
              <th>Field</th>
              <th>Category</th>
              <th>Reviews</th>
              <th>Corroborations</th>
              <th>Honeypot</th>
              <th>Starving</th>
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
                <td colSpan={7}>No discoveries match.</td>
              </tr>
            )}
          </tbody>
        </table>
      )}

      <section aria-label="Honeypot accuracy per reviewer">
        <h2>Honeypot accuracy per reviewer</h2>
        {honeypotStats.isPending && <p>Loading honeypot stats…</p>}
        {honeypotStats.isError && <p role="alert">{honeypotStats.error.message}</p>}
        {honeypotStats.data && (
          <table>
            <thead>
              <tr>
                <th>Reviewer AAA</th>
                <th>Hits</th>
                <th>Trials</th>
              </tr>
            </thead>
            <tbody>
              {honeypotStats.data.map((s) => (
                <tr key={s.reviewer_aaa.toText()}>
                  <td>{s.reviewer_aaa.toText()}</td>
                  <td>{s.hits}</td>
                  <td>{s.trials}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section aria-label="Add honeypots">
        <h2>Add honeypots (JSON upload)</h2>
        <p>There is deliberately no edit or delete for confirmed citations.</p>
        <textarea
          value={honeypotJson}
          onChange={(e) => setHoneypotJson(e.target.value)}
          rows={6}
          aria-label="Honeypot batch JSON"
        />
        {parseError && <p role="alert">{parseError}</p>}
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
        {addHoneypots.isError && <p role="alert">{addHoneypots.error.message}</p>}
        {addHoneypots.isSuccess && <p role="status">Added {addHoneypots.data} honeypots.</p>}
      </section>
    </div>
  );
};
