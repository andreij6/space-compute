import { useEffect, useMemo, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { HttpAgent } from '@icp-sdk/core/agent';
import { Signer } from '@icp-sdk/signer';
import { PostMessageTransport } from '@icp-sdk/signer/web';
import { SignerAgent } from '@icp-sdk/signer/agent';
import { IcrcLedgerCanister, toCandidAccount, type IcrcAccount } from '@icp-sdk/canisters/ledger/icrc';
import { useAuth, useMyAaa } from '../auth';
import { aaaActor, canisterEnv, canisterId, platformActor, paymentsActor } from '../ic';
import { Purpose } from '../bindings/payments';
import { PaymentPanel } from '../components/PaymentPanel';
import { EmptyState } from '../components/EmptyState';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { FuelGauge } from '../components/ui/FuelGauge';
import {
  e8sToCycles,
  formatIcp,
  MIN_TOPUP_E8S,
  opStatusLabel,
  parseIcpToE8s,
  unwrapResult,
  walletErrorMessage,
} from '../lib/paymentOps';
import {
  formatCycles,
  loadDashboard,
  mandateApproveAmountE8s,
  mandateStatusMessage,
  mandateUiState,
} from '../lib/dashboard';
import { spenderSubaccount } from '../lib/spenderSubaccount';
import { dedupPages } from '../paging';
import styles from './FuelBillingPage.module.css';

const ICP_LEDGER_ID = Principal.fromText('ryjl3-tyaaa-aaaaa-aaaba-cai');
const OISY_SIGNER_URL = 'https://oisy.com/sign';
const REFRESH_MS = 5_000;
const NS_PER_YEAR = 365n * 24n * 60n * 60n * 1_000_000_000n;

type Payments = ReturnType<typeof paymentsActor>;

const HISTORY_PAGE_SIZE = 20;

function FuelHistory({ payments, aaaId }: { payments: Payments; aaaId: Principal }) {
  const query = useInfiniteQuery({
    queryKey: ['fuel_history', aaaId.toText()],
    queryFn: ({ pageParam }: { pageParam: bigint | null }) =>
      payments.list_ops_for_aaa(aaaId, pageParam, HISTORY_PAGE_SIZE),
    initialPageParam: null as bigint | null,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
  });
  const ops = dedupPages(query.data?.pages.map((p) => p.items), (op) => op.id.toString());

  return (
    <>
      {query.isPending && <p>Loading…</p>}
      {query.isError && <p role="alert">Could not load fuel history. Try again later.</p>}
      {query.isSuccess && ops.length === 0 && <p>No top-ups yet.</p>}
      {ops.length > 0 && (
        <ul className={styles.list}>
          {ops.map((op) => (
            <li key={op.id.toString()} className={styles.item}>
              <span>#{op.id.toString()} — {opStatusLabel(op.state)}</span>
              <span>{formatIcp(op.pull_e8s ?? op.amount_e8s)} ICP</span>
            </li>
          ))}
        </ul>
      )}
      {query.hasNextPage && (
        <div className={styles.loadMore}>
          <Button type="button" variant="secondary" size="sm" busy={query.isFetchingNextPage} onClick={() => void query.fetchNextPage()}>
            Load more
          </Button>
        </div>
      )}
    </>
  );
}

export function FuelBillingPage() {
  const { identity } = useAuth();
  const aaaQuery = useMyAaa();
  const aaaId = aaaQuery.data ?? null;
  const queryClient = useQueryClient();

  const aaa = useMemo(() => (aaaId ? aaaActor(aaaId.toText(), identity ?? undefined) : null), [aaaId, identity]);
  const platform = useMemo(() => platformActor(identity ?? undefined), [identity]);
  const payments = useMemo(() => paymentsActor(identity ?? undefined), [identity]);

  const dashboardQuery = useQuery({
    queryKey: ['dashboard', aaaId?.toText()],
    queryFn: () => loadDashboard(aaa!, platform, aaaId!),
    enabled: !!aaa && !!aaaId,
    refetchInterval: REFRESH_MS,
  });

  const rateQuery = useQuery({ queryKey: ['payments_rate'], queryFn: () => payments.get_rate() });

  const mandateQuery = useQuery({
    queryKey: ['mandate', aaaId?.toText()],
    queryFn: () => payments.get_mandate(aaaId!),
    enabled: !!aaaId,
    refetchInterval: REFRESH_MS,
  });
  const mandate = mandateQuery.data ?? null;
  const uiState = mandateUiState(mandate);

  const [topupIcp, setTopupIcp] = useState('');
  const [capIcp, setCapIcp] = useState('');
  const [mandateBusy, setMandateBusy] = useState(false);
  const [mandateError, setMandateError] = useState<string | null>(null);
  const prefilled = useRef(false);
  useEffect(() => {
    if (mandate && !prefilled.current) {
      setTopupIcp(formatIcp(mandate.topup_e8s));
      setCapIcp(formatIcp(mandate.cap_30d_e8s));
      prefilled.current = true;
    }
  }, [mandate]);

  const [topupAmountIcp, setTopupAmountIcp] = useState('0.1');
  const [topupOpId, setTopupOpId] = useState<bigint | null>(null);
  const topupLocked = topupOpId !== null;
  const topupAmountE8s = parseIcpToE8s(topupAmountIcp);
  const topupAmountValid = topupAmountE8s !== null && topupAmountE8s >= MIN_TOPUP_E8S;
  const cyclesWanted =
    topupAmountValid && rateQuery.data ? e8sToCycles(topupAmountE8s!, rateQuery.data.xdr_permyriad_per_icp) : null;

  async function invalidateMandate() {
    await queryClient.invalidateQueries({ queryKey: ['mandate', aaaId?.toText()] });
  }

  async function handleMandateApprove() {
    if (!aaaId) return;
    setMandateError(null);
    setMandateBusy(true);
    try {
      const topupE8s = parseIcpToE8s(topupIcp);
      const capE8s = parseIcpToE8s(capIcp);
      if (topupE8s === null || topupE8s < MIN_TOPUP_E8S) {
        throw new Error('Enter a per-top-up amount of at least 0.1 ICP.');
      }
      if (capE8s === null || capE8s < topupE8s) {
        throw new Error('The monthly limit must be at least the per-top-up amount.');
      }
      const approveE8s = mandateApproveAmountE8s(topupE8s, capE8s);
      const signer = new Signer({ transport: new PostMessageTransport({ url: OISY_SIGNER_URL }) });
      const accounts = await signer.getAccounts();
      if (accounts.length === 0) throw new Error('The wallet shared no account.');
      const account: IcrcAccount = accounts[0];
      const agent = HttpAgent.createSync({ rootKey: canisterEnv()?.IC_ROOT_KEY });
      const signerAgent = await SignerAgent.create({ signer, account: account.owner, agent });
      const spenderSub = await spenderSubaccount('auto', aaaId);
      const ledger = IcrcLedgerCanister.create({ agent: signerAgent, canisterId: ICP_LEDGER_ID });
      await ledger.approve({
        spender: toCandidAccount({ owner: Principal.fromText(canisterId('payments')), subaccount: spenderSub }),
        amount: approveE8s,
        from_subaccount: account.subaccount,
        expires_at: BigInt(Date.now()) * 1_000_000n + NS_PER_YEAR,
      });
      await payments
        .set_mandate({
          aaa: aaaId,
          payer: { owner: account.owner, subaccount: account.subaccount },
          topup_e8s: topupE8s,
          cap_30d_e8s: capE8s,
          enabled: true,
        })
        .then(unwrapResult);
      await invalidateMandate();
    } catch (e) {
      setMandateError(walletErrorMessage(e));
    } finally {
      setMandateBusy(false);
    }
  }

  async function setMandateEnabled(next: boolean) {
    if (!aaaId || !mandate) return;
    setMandateError(null);
    setMandateBusy(true);
    try {
      await payments
        .set_mandate({
          aaa: aaaId,
          payer: mandate.payer,
          topup_e8s: mandate.topup_e8s,
          cap_30d_e8s: mandate.cap_30d_e8s,
          enabled: next,
        })
        .then(unwrapResult);
      await invalidateMandate();
    } catch (e) {
      setMandateError(walletErrorMessage(e));
    } finally {
      setMandateBusy(false);
    }
  }

  if (aaaQuery.isPending) return <p>Loading…</p>;
  if (!aaaId) return <p role="alert">Could not load your AAA. Try again later.</p>;

  const data = dashboardQuery.data;

  return (
    <div className={styles.page}>
      <h1>Fuel &amp; billing</h1>
      <p className={styles.meta}>
        Canister ID: <code>{aaaId.toText()}</code>
      </p>

      <Card title="Current fuel">
        {dashboardQuery.isPending && <p>Loading…</p>}
        {dashboardQuery.isError && (
          <p role="alert">Could not load fuel status: {(dashboardQuery.error as Error).message}</p>
        )}
        {data?.fuel.kind === 'frozen' && (
          <EmptyState
            type="canister_paused"
            title="Your agent is out of fuel"
            description="Its canister is frozen and cannot be reached directly. Top up below to resume autonomous observation."
          />
        )}
        {data?.fuel.kind === 'error' && <p role="alert">{data.fuel.message}</p>}
        {data?.fuel.kind === 'live' && (
          <>
            <FuelGauge daysRemaining={data.fuel.daysRemaining} />
            <dl>
              <div>
                <dt>Days of fuel remaining</dt>
                <dd>
                  {data.fuel.daysRemaining} ({data.fuel.level})
                </dd>
              </div>
              <div>
                <dt>Cycles</dt>
                <dd>{formatCycles(data.fuel.cycles)}</dd>
              </div>
            </dl>
          </>
        )}
      </Card>

      <Card title="One-time top-up">
        <div className={styles.field}>
          <label htmlFor="topup-amount">Amount to send (ICP, min 0.1)</label>
          <input
            id="topup-amount"
            type="text"
            inputMode="decimal"
            value={topupAmountIcp}
            onChange={(e) => setTopupAmountIcp(e.target.value)}
            disabled={topupLocked}
          />
        </div>
        {topupLocked && <p className={styles.locked}>The amount is locked while this top-up is in progress.</p>}
        {!topupAmountValid && <p role="alert">Enter an amount of at least 0.1 ICP.</p>}
        {topupAmountValid && rateQuery.isPending && <p>Loading rate…</p>}
        {topupAmountValid && rateQuery.isError && <p role="alert">Could not load the current rate.</p>}
        {topupAmountValid && cyclesWanted !== null && !topupLocked && (
          <p className={styles.rate}>≈ {formatCycles(cyclesWanted)} of cycles at the current rate.</p>
        )}
        {(topupLocked || (topupAmountValid && cyclesWanted !== null)) && (
          <PaymentPanel
            purpose="topup"
            beneficiary={aaaId}
            sponsoredSpawnEnabled={false}
            quoteKey={cyclesWanted?.toString()}
            onOpChange={setTopupOpId}
            fetchQuote={() =>
              cyclesWanted === null
                ? Promise.reject(new Error('Enter an amount of at least 0.1 ICP.'))
                : payments.get_quote_topup(cyclesWanted).then(unwrapResult)
            }
            fetchDepositAccount={() => payments.get_deposit_account(Purpose.TopUp, aaaId)}
            fetchOp={(opId) => payments.get_op(opId)}
            submitOp={(path) => payments.top_up({ aaa: aaaId, path }).then(unwrapResult)}
            onPaid={() => {
              void queryClient.invalidateQueries({ queryKey: ['dashboard', aaaId.toText()] });
              void queryClient.invalidateQueries({ queryKey: ['fuel_history', aaaId.toText()] });
            }}
          />
        )}
      </Card>

      <Card title="Auto top-up">
        <p>{mandateQuery.isPending ? 'Loading…' : mandateStatusMessage(uiState)}</p>
        {mandateQuery.isError && <p role="alert">Could not load your auto top-up settings. Try again later.</p>}

        {mandate && (
          <dl>
            <div>
              <dt>Per top-up amount</dt>
              <dd>{formatIcp(mandate.topup_e8s)} ICP</dd>
            </div>
            <div>
              <dt>Monthly limit</dt>
              <dd>{formatIcp(mandate.cap_30d_e8s)} ICP</dd>
            </div>
            <div>
              <dt>Spent (last 30 days)</dt>
              <dd>{formatIcp(mandate.spent_30d_e8s)} ICP</dd>
            </div>
            <div>
              <dt>Remaining allowance</dt>
              <dd>{formatIcp(mandate.remaining_30d_e8s)} ICP</dd>
            </div>
          </dl>
        )}

        {uiState === 'enabled' && (
          <div className={styles.mandateRow}>
            <Button type="button" variant="secondary" busy={mandateBusy} onClick={() => setMandateEnabled(false)}>
              Disable auto top-up
            </Button>
          </div>
        )}
        {uiState === 'disabled' && (
          <div className={styles.mandateRow}>
            <Button type="button" variant="secondary" busy={mandateBusy} onClick={() => setMandateEnabled(true)}>
              Enable auto top-up
            </Button>
          </div>
        )}

        <h3>{mandate ? 'Update auto top-up' : 'Set up auto top-up'}</h3>
        <div className={styles.fieldsRow}>
          <div>
            <label htmlFor="mandate-topup">Per top-up amount (ICP, min 0.1)</label>
            <input id="mandate-topup" type="text" inputMode="decimal" value={topupIcp} onChange={(e) => setTopupIcp(e.target.value)} />
          </div>
          <div>
            <label htmlFor="mandate-cap">Monthly limit (ICP)</label>
            <input id="mandate-cap" type="text" inputMode="decimal" value={capIcp} onChange={(e) => setCapIcp(e.target.value)} />
          </div>
        </div>
        <p className={styles.hint}>Approve the wallet allowance below to spender (payments) for this AAA, then save.</p>
        <div className={styles.actions}>
          <Button type="button" variant="primary" busy={mandateBusy} onClick={handleMandateApprove}>
            Connect wallet, approve &amp; save
          </Button>
        </div>
        {mandateError && <p role="alert">{mandateError}</p>}
      </Card>

      <Card title="Fuel operation history">
        <FuelHistory payments={payments} aaaId={aaaId} />
      </Card>

      <p className={styles.footer}>
        <Link to="/dashboard">Back to dashboard</Link>
      </p>
    </div>
  );
}
