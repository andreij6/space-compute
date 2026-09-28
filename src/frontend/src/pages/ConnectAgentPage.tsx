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
  newIdentityCommand,
  operatorIdentityName,
  parseOperatorPrincipal,
  removeOperator,
} from '../lib/connect';
import { EmptyState } from '../components/EmptyState';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import styles from './ConnectAgentPage.module.css';

const REFRESH_MS = 10_000;
const SKILL_INSTALL_CMD = 'cp -r agent-kit/skills/space-compute-astronomer ~/.claude/skills/';

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function CopyCommand({ command }: { command: string }) {
  return (
    <span className={styles.cmdRow}>
      <code className={styles.cmd}>{command}</code>
      <Button type="button" variant="ghost" size="sm" onClick={() => copyToClipboard(command)}>
        Copy
      </Button>
    </span>
  );
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

  const [revokeError, setRevokeError] = useState<string | null>(null);
  const removeMutation = useMutation({
    mutationFn: async (principal: Principal) => {
      const result = await removeOperator(aaa!, principal);
      if (!result.ok) throw new Error(result.error);
    },
    onMutate: () => setRevokeError(null),
    onSuccess: invalidateStatus,
    onError: (e: unknown) => setRevokeError(e instanceof Error ? e.message : 'Something went wrong.'),
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
  const newIdentityCmd = newIdentityCommand(identityName);
  const principalCmd = `icp identity principal --identity ${identityName}`;
  const firstContactCmd = firstContactCommand(aaaIdText, identityName, net);

  return (
    <div className={styles.page}>
      <h1>Connect your agent</h1>
      <p className={styles.meta}>
        AAA canister: <code>{aaaIdText}</code>
      </p>

      <Card title="Status">
        <p className={`${styles.status} ${connected ? styles.connected : ''}`}>
          <span className={styles.dot} aria-hidden="true" />
          {connected ? 'Connected' : 'Not connected yet'}
        </p>
      </Card>

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

      <Card title="Setup steps">
        <ol className={styles.steps}>
          <li>
            Create an operator identity:
            <CopyCommand command={newIdentityCmd} />
          </li>
          <li>
            Print its principal:
            <CopyCommand command={principalCmd} />
          </li>
          <li>Paste the principal below and add it as an operator.</li>
          <li>
            Install the skill:
            <CopyCommand command={SKILL_INSTALL_CMD} />
          </li>
          <li>
            Run the first-contact command:
            <CopyCommand command={firstContactCmd} />
          </li>
        </ol>
      </Card>

      <Card title="Add an operator">
        <form
          className={styles.form}
          onSubmit={(e) => {
            e.preventDefault();
            setFormError(null);
            addMutation.mutate();
          }}
        >
          <div>
            <label htmlFor="operator-principal">Operator principal</label>
            <input
              id="operator-principal"
              type="text"
              value={principalText}
              onChange={(e) => setPrincipalText(e.target.value)}
              autoComplete="off"
            />
          </div>
          <div>
            <label htmlFor="operator-label">Label</label>
            <input id="operator-label" type="text" value={label} onChange={(e) => setLabel(e.target.value)} autoComplete="off" />
          </div>
          <Button type="submit" variant="primary" busy={addMutation.isPending}>
            {addMutation.isPending ? 'Adding…' : 'Add operator'}
          </Button>
        </form>
        {formError && <p role="alert">{formError}</p>}
      </Card>

      <Card title="Operators" actions={<Badge tone="neutral">{operators.length}</Badge>}>
        {operators.length === 0 && <p>No operators yet.</p>}
        {operators.length > 0 && (
          <ul>
            {operators.map(([principal, operator]) => (
              <li key={principal.toText()} className={styles.operatorRow}>
                <span className={styles.operatorMeta}>
                  <code>{principal.toText()}</code>
                  <span className={styles.operatorLabel}>
                    {operator.label}
                    {operator.last_used_at != null && <> · last used {formatNs(operator.last_used_at)}</>}
                  </span>
                </span>
                <Button
                  type="button"
                  variant="danger"
                  size="sm"
                  busy={removeMutation.isPending && removeMutation.variables?.toText() === principal.toText()}
                  onClick={() => removeMutation.mutate(principal)}
                >
                  {removeMutation.isPending && removeMutation.variables?.toText() === principal.toText() ? 'Revoking…' : 'Revoke'}
                </Button>
              </li>
            ))}
          </ul>
        )}
        {revokeError && <p role="alert">Could not revoke the operator: {revokeError}</p>}
      </Card>
    </div>
  );
}
