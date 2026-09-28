import { useMemo, useState } from 'react';
import type { Principal } from '@icp-sdk/core/principal';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuth, useMyAaa } from '../auth';
import { aaaActor } from '../ic';
import { formatNs } from '../categories';
import { isFrozenReject } from '../lib/dashboard';
import { humanApiError } from '../lib/paymentOps';
import {
  addOperator,
  firstContactCommand,
  isConnected,
  netFlag,
  operatorIdentityName,
  parseOperatorPrincipal,
  removeOperator,
} from '../lib/connect';
import { EmptyState } from '../components/EmptyState';

const REFRESH_MS = 10_000;
const SKILL_INSTALL_CMD = 'cp -r agent-kit/skills/space-compute-astronomer ~/.claude/skills/';

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

export function ConnectAgentPage() {
  const { identity } = useAuth();
  const aaaQuery = useMyAaa();
  const aaaId = aaaQuery.data ?? null;
  const aaa = useMemo(() => (aaaId ? aaaActor(aaaId.toText(), identity ?? undefined) : null), [aaaId, identity]);
  const queryClient = useQueryClient();

  const [principalText, setPrincipalText] = useState('');
  const [label, setLabel] = useState('');
  const [formError, setFormError] = useState<string | null>(null);

  const statusQueryKey = ['aaa_status_operators', aaaId?.toText()];
  const statusQuery = useQuery({
    queryKey: statusQueryKey,
    queryFn: () => aaa!.status(),
    enabled: !!aaa,
    refetchInterval: REFRESH_MS,
  });

  const invalidateStatus = () => queryClient.invalidateQueries({ queryKey: statusQueryKey });

  const addMutation = useMutation({
    mutationFn: async () => {
      const parsed = parseOperatorPrincipal(principalText);
      if (!parsed.ok) throw new Error(parsed.error);
      const result = await addOperator(aaa!, parsed.principal, label.trim() || 'operator');
      if (!result.ok) throw new Error(result.error);
    },
    onSuccess: () => {
      setPrincipalText('');
      setLabel('');
      setFormError(null);
      invalidateStatus();
    },
    onError: (e: unknown) => setFormError(e instanceof Error ? e.message : 'Something went wrong.'),
  });

  const removeMutation = useMutation({
    mutationFn: async (principal: Principal) => {
      const result = await removeOperator(aaa!, principal);
      if (!result.ok) throw new Error(result.error);
    },
    onSuccess: invalidateStatus,
  });

  if (aaaQuery.isPending) return <p>Loading…</p>;
  if (!aaaId) return <p role="alert">Could not load your AAA. Try again later.</p>;

  const aaaIdText = aaaId.toText();
  const frozen = statusQuery.isError && isFrozenReject(statusQuery.error);
  const status = statusQuery.data;
  const operators = status?.__kind__ === 'Ok' ? status.Ok.operators : [];
  const connected = isConnected(operators, BigInt(statusQuery.dataUpdatedAt) * 1_000_000n);
  const identityName = operatorIdentityName();
  const net = netFlag(window.location.hostname);
  const newIdentityCmd = `icp identity new ${identityName} --storage plaintext`;
  const principalCmd = `icp identity principal --identity ${identityName}`;
  const firstContactCmd = firstContactCommand(aaaIdText, identityName, net);

  return (
    <div>
      <h1>Connect your agent</h1>
      <p>
        AAA canister: <code>{aaaIdText}</code>
      </p>

      <section aria-label="Connection status">
        <h2>Status</h2>
        <p>{connected ? 'Connected' : 'Not connected yet'}</p>
      </section>

      {frozen && (
        <EmptyState
          type="canister_paused"
          title="Your agent is out of fuel"
          description="Its canister is frozen and cannot be reached directly. Top up to resume autonomous observation."
        />
      )}
      {statusQuery.isError && !frozen && (
        <p role="alert">Could not load operator status: {(statusQuery.error as Error).message}</p>
      )}
      {status?.__kind__ === 'Err' && <p role="alert">{humanApiError(status.Err)}</p>}

      <section aria-label="Setup steps">
        <h2>Setup steps</h2>
        <ol>
          <li>
            Create an operator identity: <code>{newIdentityCmd}</code>{' '}
            <button type="button" onClick={() => copyToClipboard(newIdentityCmd)}>
              Copy
            </button>
          </li>
          <li>
            Print its principal: <code>{principalCmd}</code>{' '}
            <button type="button" onClick={() => copyToClipboard(principalCmd)}>
              Copy
            </button>
          </li>
          <li>Paste the principal below and add it as an operator.</li>
          <li>
            Install the skill: <code>{SKILL_INSTALL_CMD}</code>{' '}
            <button type="button" onClick={() => copyToClipboard(SKILL_INSTALL_CMD)}>
              Copy
            </button>
          </li>
          <li>
            Run the first-contact command: <code>{firstContactCmd}</code>{' '}
            <button type="button" onClick={() => copyToClipboard(firstContactCmd)}>
              Copy
            </button>
          </li>
        </ol>
      </section>

      <section aria-label="Add operator">
        <h2>Add an operator</h2>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setFormError(null);
            addMutation.mutate();
          }}
        >
          <label htmlFor="operator-principal">Operator principal</label>
          <input
            id="operator-principal"
            type="text"
            value={principalText}
            onChange={(e) => setPrincipalText(e.target.value)}
            autoComplete="off"
          />
          <label htmlFor="operator-label">Label</label>
          <input id="operator-label" type="text" value={label} onChange={(e) => setLabel(e.target.value)} autoComplete="off" />
          <button type="submit" disabled={addMutation.isPending}>
            {addMutation.isPending ? 'Adding…' : 'Add operator'}
          </button>
        </form>
        {formError && <p role="alert">{formError}</p>}
      </section>

      <section aria-label="Operators">
        <h2>Operators</h2>
        {operators.length === 0 && <p>No operators yet.</p>}
        {operators.length > 0 && (
          <ul>
            {operators.map(([principal, operator]) => (
              <li key={principal.toText()}>
                <code>{principal.toText()}</code> — {operator.label}
                {operator.last_used_at != null && <> · last used {formatNs(operator.last_used_at)}</>}{' '}
                <button type="button" onClick={() => removeMutation.mutate(principal)} disabled={removeMutation.isPending}>
                  {removeMutation.isPending && removeMutation.variables?.toText() === principal.toText() ? 'Revoking…' : 'Revoke'}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
