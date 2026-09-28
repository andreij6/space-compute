import type { ApiError, Op, OpState } from '../bindings/payments';

export function formatIcp(e8s: bigint): string {
  const negative = e8s < 0n;
  const abs = negative ? -e8s : e8s;
  const whole = abs / 100_000_000n;
  const frac = (abs % 100_000_000n).toString().padStart(8, '0').replace(/0+$/, '');
  const text = frac ? `${whole}.${frac}` : `${whole}`;
  return negative ? `-${text}` : text;
}

export class PaymentApiError extends Error {
  constructor(public readonly apiError: ApiError) {
    super(humanApiError(apiError));
    this.name = 'PaymentApiError';
  }
}

export function unwrapResult<T>(result: { __kind__: 'Ok'; Ok: T } | { __kind__: 'Err'; Err: ApiError }): T {
  if (result.__kind__ === 'Err') throw new PaymentApiError(result.Err);
  return result.Ok;
}

export function humanApiError(err: ApiError): string {
  switch (err.__kind__) {
    case 'Internal':
      return 'Something went wrong on our side. Please try again.';
    case 'NotRegistered':
      return 'This agent is not registered yet.';
    case 'InvalidInput':
      return err.InvalidInput || 'That input is not valid.';
    case 'InsufficientFee':
      return `The ledger fee changed; it now requires ${err.InsufficientFee.required} e8s.`;
    case 'NotFound':
      return 'Not found.';
    case 'Suspended':
      return 'This account is suspended.';
    case 'LeaseExpired':
      return 'Your lease on this task expired.';
    case 'LeaseNotFound':
      return 'No active lease was found.';
    case 'Unauthorized':
      return 'You are not authorized to do that.';
    case 'RateLimited':
      return `Too many attempts. Try again in ${err.RateLimited.retry_after_secs}s.`;
    case 'NotEligible':
      return err.NotEligible || 'You are not eligible for this.';
    case 'FeatureDisabled':
      return 'This payment method is not available yet.';
    case 'Conflict':
      return err.Conflict || 'That conflicts with existing state.';
    default:
      return 'Something went wrong.';
  }
}

export type OpPhase = 'pending' | 'done' | 'failed';

export function opPhase(op: Op | null | undefined): OpPhase {
  if (!op) return 'pending';
  switch (op.state.__kind__) {
    case 'Done':
      return 'done';
    case 'Failed':
    case 'Refunded':
      return 'failed';
    default:
      return 'pending';
  }
}

export function opStatusLabel(state: OpState): string {
  switch (state.__kind__) {
    case 'Pending':
      return 'Waiting for your payment…';
    case 'Pulled':
      return 'Payment received, forwarding to the Cycles Minting Canister…';
    case 'Notified':
      return 'Cycles minted, finishing up…';
    case 'Registered':
      return 'Registering your agent…';
    case 'Credited':
      return 'Payment credited…';
    case 'TreasuryPaid':
      return 'Treasury is funding your top-up…';
    case 'Done':
      return 'Done.';
    case 'Failed':
      return state.Failed.reason || 'The payment failed.';
    case 'Refunded':
      return `Refunded at ledger block ${state.Refunded.block}.`;
    default:
      return 'Working…';
  }
}

export const OP_POLL_INTERVAL_MS = 3_000;

export function nextPollDelayMs(op: Op | null | undefined): number | false {
  return opPhase(op) === 'pending' ? OP_POLL_INTERVAL_MS : false;
}

export const MIN_TOPUP_E8S = 10_000_000n;

export function e8sToCycles(e8s: bigint, rateXdrPermyriadPerIcp: bigint): bigint {
  return e8s * rateXdrPermyriadPerIcp;
}

export function parseIcpToE8s(input: string): bigint | null {
  const trimmed = input.trim();
  if (!/^\d+(\.\d+)?$/.test(trimmed)) return null;
  const [whole, frac = ''] = trimmed.split('.');
  if (frac.length > 8) return null;
  const paddedFrac = frac.padEnd(8, '0');
  return BigInt(whole) * 100_000_000n + BigInt(paddedFrac || '0');
}

export function walletErrorMessage(e: unknown): string {
  if (e instanceof PaymentApiError) return humanApiError(e.apiError);
  if (e && typeof e === 'object' && 'code' in e) return 'The wallet could not complete that action.';
  if (e instanceof Error) return e.message;
  return 'Something went wrong.';
}
