import { describe, expect, it } from 'vitest';
import type { Op } from '../bindings/payments';
import {
  formatIcp,
  humanApiError,
  nextPollDelayMs,
  OP_POLL_INTERVAL_MS,
  opPhase,
  opStatusLabel,
  PaymentApiError,
  unwrapResult,
} from './paymentOps';

describe('formatIcp (quote e8s → ICP)', () => {
  it('formats whole ICP with no trailing decimal', () => {
    expect(formatIcp(100_000_000n)).toBe('1');
  });
  it('formats fractional e8s and trims trailing zeros', () => {
    expect(formatIcp(31_885_000n)).toBe('0.31885');
  });
  it('keeps the smallest unit', () => {
    expect(formatIcp(1n)).toBe('0.00000001');
  });
  it('formats zero', () => {
    expect(formatIcp(0n)).toBe('0');
  });
  it('formats a negative amount (e.g. a refund delta)', () => {
    expect(formatIcp(-50_000_000n)).toBe('-0.5');
  });
});

describe('humanApiError (ApiError → human message)', () => {
  it('covers every ApiError variant with a non-empty message', () => {
    const cases: Array<[Parameters<typeof humanApiError>[0], RegExp]> = [
      [{ __kind__: 'Internal', Internal: 'x' }, /wrong/i],
      [{ __kind__: 'NotRegistered', NotRegistered: null }, /registered/i],
      [{ __kind__: 'InvalidInput', InvalidInput: 'bad name' }, /bad name/],
      [{ __kind__: 'InsufficientFee', InsufficientFee: { required: 10_000n } }, /10000/],
      [{ __kind__: 'NotFound', NotFound: null }, /not found/i],
      [{ __kind__: 'Suspended', Suspended: null }, /suspended/i],
      [{ __kind__: 'LeaseExpired', LeaseExpired: null }, /expired/i],
      [{ __kind__: 'LeaseNotFound', LeaseNotFound: null }, /lease/i],
      [{ __kind__: 'Unauthorized', Unauthorized: null }, /authorized/i],
      [{ __kind__: 'RateLimited', RateLimited: { retry_after_secs: 30 } }, /30s/],
      [{ __kind__: 'NotEligible', NotEligible: 'no invite left' }, /no invite left/],
      [{ __kind__: 'FeatureDisabled', FeatureDisabled: null }, /not available/i],
      [{ __kind__: 'Conflict', Conflict: 'already spawned' }, /already spawned/],
    ];
    for (const [err, expected] of cases) {
      expect(humanApiError(err)).toMatch(expected);
    }
  });

  it('falls back to the generic message for a blank InvalidInput/NotEligible/Conflict', () => {
    expect(humanApiError({ __kind__: 'InvalidInput', InvalidInput: '' })).toMatch(/not valid/i);
    expect(humanApiError({ __kind__: 'NotEligible', NotEligible: '' })).toMatch(/not eligible/i);
    expect(humanApiError({ __kind__: 'Conflict', Conflict: '' })).toMatch(/conflicts/i);
  });
});

describe('unwrapResult (Result_2 → bigint | throw)', () => {
  it('returns Ok', () => {
    expect(unwrapResult({ __kind__: 'Ok', Ok: 42n })).toBe(42n);
  });
  it('throws a PaymentApiError carrying the ApiError on Err', () => {
    const err = { __kind__: 'Unauthorized' as const, Unauthorized: null };
    try {
      unwrapResult({ __kind__: 'Err', Err: err });
      expect.unreachable();
    } catch (e) {
      expect(e).toBeInstanceOf(PaymentApiError);
      expect((e as PaymentApiError).apiError).toEqual(err);
      expect((e as PaymentApiError).message).toMatch(/authorized/i);
    }
  });
});

describe('op polling state machine (05 §3, poll get_op every 3s until Done/Failed)', () => {
  const op = (state: Op['state']): Op => ({
    v: 1,
    id: 1n,
    updated_at: 0n,
    kind: { __kind__: 'TopUp', TopUp: { aaa: undefined as never } },
    path: { __kind__: 'Deposit', Deposit: null },
    attempts: 0,
    created_at: 0n,
    created_by: undefined as never,
    amount_e8s: 0n,
    state,
  });

  it('treats no op yet as pending (still waiting on the first fetch)', () => {
    expect(opPhase(null)).toBe('pending');
    expect(opPhase(undefined)).toBe('pending');
  });

  it.each([
    ['Pending', { __kind__: 'Pending', Pending: null }] as const,
    ['Pulled', { __kind__: 'Pulled', Pulled: { block: 1n } }] as const,
    ['Notified', { __kind__: 'Notified', Notified: { canister_or_cycles: { __kind__: 'Cycles', Cycles: 1n } } }] as const,
    ['Registered', { __kind__: 'Registered', Registered: null }] as const,
  ])('keeps polling through the %s state', (_name, state) => {
    expect(opPhase(op(state))).toBe('pending');
    expect(nextPollDelayMs(op(state))).toBe(OP_POLL_INTERVAL_MS);
  });

  it('stops polling once Done', () => {
    const done = op({ __kind__: 'Done', Done: null });
    expect(opPhase(done)).toBe('done');
    expect(nextPollDelayMs(done)).toBe(false);
  });

  it('stops polling on Failed and surfaces the reason', () => {
    const failed = op({ __kind__: 'Failed', Failed: { reason: 'ledger fee changed' } });
    expect(opPhase(failed)).toBe('failed');
    expect(nextPollDelayMs(failed)).toBe(false);
    expect(opStatusLabel(failed.state)).toBe('ledger fee changed');
  });

  it('stops polling on Refunded, a terminal failure', () => {
    const refunded = op({ __kind__: 'Refunded', Refunded: { block: 7n } });
    expect(opPhase(refunded)).toBe('failed');
    expect(nextPollDelayMs(refunded)).toBe(false);
    expect(opStatusLabel(refunded.state)).toMatch(/block 7/);
  });
});
