import { useEffect, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { HttpAgent } from '@icp-sdk/core/agent';
import { Signer } from '@icp-sdk/signer';
import { PostMessageTransport } from '@icp-sdk/signer/web';
import { SignerAgent } from '@icp-sdk/signer/agent';
import { IcrcLedgerCanister, toCandidAccount, type IcrcAccount } from '@icp-sdk/canisters/ledger/icrc';
import type { Account, Op, PayPath, Quote } from '../bindings/payments';
import { canisterEnv, canisterId } from '../ic';
import { formatIcp, nextPollDelayMs, opPhase, opStatusLabel, walletErrorMessage } from '../lib/paymentOps';
import { spenderSubaccount, type SubaccountPurpose } from '../lib/spenderSubaccount';
import { Button } from './ui/Button';
import { Card } from './ui/Card';
import { Tabs, type TabItem } from './ui/Tabs';
import { svgAsset } from './ui/assets';
import styles from './PaymentPanel.module.css';

const ICP_LEDGER_ID = Principal.fromText('ryjl3-tyaaa-aaaaa-aaaba-cai');
const OISY_SIGNER_URL = 'https://oisy.com/sign';

export type PaymentPurpose = 'spawn' | 'topup';

export interface PaymentPanelProps {
  purpose: PaymentPurpose;
  beneficiary: Principal;
  sponsoredSpawnEnabled: boolean;
  fetchQuote: () => Promise<Quote>;
  fetchDepositAccount: () => Promise<[string, Account]>;
  fetchOp: (opId: bigint) => Promise<Op | null>;
  submitOp: (path: PayPath) => Promise<bigint>;
  onPaid: (opId: bigint) => void;
  initialOpId?: bigint | null;
  quoteKey?: string;
  onOpChange?: (opId: bigint | null) => void;
}

function PaymentIcon({ name }: { name: string }) {
  return <img className={styles.icon} src={svgAsset('payments', name)} alt="" />;
}

export function PaymentPanel({
  purpose,
  beneficiary,
  sponsoredSpawnEnabled,
  fetchQuote,
  fetchDepositAccount,
  fetchOp,
  submitOp,
  onPaid,
  initialOpId = null,
  quoteKey = '',
  onOpChange,
}: PaymentPanelProps) {
  const beneficiaryKey = beneficiary.toText();
  const [opId, setOpId] = useState<bigint | null>(initialOpId);
  const [inviteCode, setInviteCode] = useState('');
  const [confirming, setConfirming] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const paidNotified = useRef(false);

  const quoteQuery = useQuery({
    queryKey: ['paymentPanelQuote', purpose, beneficiaryKey, quoteKey],
    queryFn: fetchQuote,
    refetchInterval: 15_000,
  });
  const depositQuery = useQuery({
    queryKey: ['paymentPanelDeposit', purpose, beneficiaryKey],
    queryFn: fetchDepositAccount,
    staleTime: Infinity,
  });
  const opQuery = useQuery({
    queryKey: ['paymentPanelOp', opId?.toString()],
    queryFn: () => fetchOp(opId as bigint),
    enabled: opId !== null,
    refetchInterval: (query) => nextPollDelayMs(query.state.data ?? null),
  });

  useEffect(() => {
    onOpChange?.(opId);
  }, [opId, onOpChange]);

  const op = opQuery.data ?? null;
  const phase = opId === null ? null : opPhase(op);

  useEffect(() => {
    if (phase === 'done' && op && !paidNotified.current) {
      paidNotified.current = true;
      onPaid(op.id);
    }
  }, [phase, op, onPaid]);

  async function runSubmit(path: PayPath) {
    setSubmitError(null);
    setConfirming(true);
    try {
      const id = await submitOp(path);
      setOpId(id);
    } catch (e) {
      setSubmitError(walletErrorMessage(e));
    } finally {
      setConfirming(false);
    }
  }

  async function handleWalletApprove() {
    setSubmitError(null);
    setConfirming(true);
    try {
      const quote = quoteQuery.data;
      if (!quote) throw new Error('The quote has not loaded yet.');
      const signer = new Signer({ transport: new PostMessageTransport({ url: OISY_SIGNER_URL }) });
      const accounts = await signer.getAccounts();
      if (accounts.length === 0) throw new Error('The wallet shared no account.');
      const account: IcrcAccount = accounts[0];
      const agent = HttpAgent.createSync({ rootKey: canisterEnv()?.IC_ROOT_KEY });
      const signerAgent = await SignerAgent.create({ signer, account: account.owner, agent });
      const subaccountPurpose: SubaccountPurpose = purpose;
      const spenderSub = await spenderSubaccount(subaccountPurpose, beneficiary);
      const ledger = IcrcLedgerCanister.create({ agent: signerAgent, canisterId: ICP_LEDGER_ID });
      await ledger.approve({
        spender: toCandidAccount({ owner: Principal.fromText(canisterId('payments')), subaccount: spenderSub }),
        amount: quote.total_e8s,
        from_subaccount: account.subaccount,
      });
      const id = await submitOp({ __kind__: 'Wallet', Wallet: { payer: account } });
      setOpId(id);
    } catch (e) {
      setSubmitError(walletErrorMessage(e));
    } finally {
      setConfirming(false);
    }
  }

  if (opId !== null) {
    return (
      <Card>
        <div className={styles.status}>
          <p role="status" className={styles.statusText}>
            {op ? opStatusLabel(op.state) : 'Loading…'}
          </p>
          {phase === 'failed' && (
            <Button variant="secondary" onClick={() => setOpId(null)}>
              Try again
            </Button>
          )}
          {phase === 'done' && purpose === 'topup' && (
            <Button
              variant="secondary"
              onClick={() => {
                paidNotified.current = false;
                setOpId(null);
              }}
            >
              Make another top-up
            </Button>
          )}
        </div>
      </Card>
    );
  }

  const tabs: TabItem[] = [
    {
      id: 'wallet',
      label: (
        <span className={styles.tabLabel}>
          <PaymentIcon name="payment_oisy" />
          Wallet
        </span>
      ),
      content: (
        <div>
          <p className={styles.hint}>
            Approve the amount above to spender (payments) from your OISY wallet, then it pays automatically.
          </p>
          <Button variant="primary" onClick={handleWalletApprove} disabled={!quoteQuery.data} busy={confirming}>
            Connect wallet &amp; approve
          </Button>
        </div>
      ),
    },
    {
      id: 'deposit',
      label: (
        <span className={styles.tabLabel}>
          <PaymentIcon name="payment_icp" />
          Deposit
        </span>
      ),
      content: (
        <div>
          {depositQuery.isPending && <p>Loading deposit address…</p>}
          {depositQuery.isError && <p role="alert">Could not load the deposit address. Try again later.</p>}
          {depositQuery.data && (
            <div>
              <div className={styles.field}>
                <label htmlFor="deposit-account-id">Send ICP to this account</label>
                <div className={styles.depositRow}>
                  <input id="deposit-account-id" type="text" readOnly value={depositQuery.data[0]} />
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    onClick={() => navigator.clipboard?.writeText(depositQuery.data![0])}
                  >
                    Copy account id
                  </Button>
                </div>
              </div>
              <p className={styles.hint}>
                Once your transfer confirms, click below. We&apos;ll sweep the balance and finish the payment.
              </p>
              <Button
                variant="primary"
                onClick={() => runSubmit({ __kind__: 'Deposit', Deposit: null })}
                busy={confirming}
              >
                I&apos;ve sent it
              </Button>
            </div>
          )}
        </div>
      ),
    },
  ];

  if (purpose === 'spawn' && sponsoredSpawnEnabled) {
    tabs.push({
      id: 'invite',
      label: (
        <span className={styles.tabLabel}>
          <PaymentIcon name="payment_invite_code" />
          Invite code
        </span>
      ),
      content: (
        <div>
          <div className={styles.field}>
            <label htmlFor="invite-code">Invite code</label>
            <input
              id="invite-code"
              type="text"
              value={inviteCode}
              onChange={(e) => setInviteCode(e.target.value)}
              autoComplete="off"
            />
          </div>
          <Button
            variant="primary"
            onClick={() => runSubmit({ __kind__: 'Invite', Invite: { code: inviteCode.trim() } })}
            disabled={inviteCode.trim().length === 0}
            busy={confirming}
          >
            Redeem invite
          </Button>
        </div>
      ),
    });
  }

  return (
    <Card>
      {quoteQuery.isPending && <p>Loading quote…</p>}
      {quoteQuery.isError && <p role="alert">Could not load the quote. Try again later.</p>}
      {quoteQuery.data && (
        <div className={styles.field}>
          <p className={styles.amount}>
            Amount due: <span className={styles.amountValue}>{formatIcp(quoteQuery.data.total_e8s)} ICP</span>
            {purpose === 'spawn' ? ' (creation + starter fuel)' : ' (top-up)'}
          </p>
          <p className={styles.fee}>Includes {formatIcp(quoteQuery.data.fee_e8s)} ICP network fee. No hidden costs.</p>
        </div>
      )}

      <Tabs label="Payment method" items={tabs} defaultId="deposit" />

      {submitError && <p role="alert">{submitError}</p>}
    </Card>
  );
}
