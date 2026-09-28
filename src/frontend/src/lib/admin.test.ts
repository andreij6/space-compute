import { describe, expect, it } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import {
  AdminApiError,
  adminGateDecision,
  belowMinSponsorCycles,
  canRemoveAdmin,
  humanAdminError,
  inviteCodesToCsv,
  inviteStatus,
  mergeAuditLogs,
  MIN_SPONSOR_CYCLES,
  parseNat,
  parsePrincipal,
  parseRecordEdits,
  proposalDetails,
  recordDiff,
  textFromRecord,
  typedConfirmReady,
  unwrapAdmin,
} from './admin';

describe('adminGateDecision (05 §2b: /admin/* gate)', () => {
  it('waits for auth before deciding', () => {
    expect(adminGateDecision(false, false, { isPending: false, data: undefined })).toBe('loading');
  });
  it('waits for the is-admin query once a principal is known', () => {
    expect(adminGateDecision(true, true, { isPending: true, data: undefined })).toBe('loading');
  });
  it('denies a non-admin principal (renders 404)', () => {
    expect(adminGateDecision(true, true, { isPending: false, data: false })).toBe('denied');
  });
  it('allows an admin principal into the console', () => {
    expect(adminGateDecision(true, true, { isPending: false, data: true })).toBe('allowed');
  });
  it('does not wait on the admin query for a signed-out visitor', () => {
    expect(adminGateDecision(true, false, { isPending: true, data: undefined })).toBe('denied');
  });
});

describe('humanAdminError / unwrapAdmin (Unauthorized error rendering)', () => {
  it('renders Unauthorized in plain language', () => {
    expect(humanAdminError({ __kind__: 'Unauthorized' })).toBe('You are not authorized to do that.');
  });
  it('unwraps Ok', () => {
    expect(unwrapAdmin({ __kind__: 'Ok', Ok: 42 })).toBe(42);
  });
  it('throws an AdminApiError with the human message on Err', () => {
    expect(() => unwrapAdmin({ __kind__: 'Err', Err: { __kind__: 'Unauthorized' } })).toThrow(AdminApiError);
    try {
      unwrapAdmin({ __kind__: 'Err', Err: { __kind__: 'Unauthorized' } });
    } catch (e) {
      expect((e as Error).message).toBe('You are not authorized to do that.');
    }
  });
  it('falls back to a generic message for an unknown variant', () => {
    expect(humanAdminError({ __kind__: 'Weird' })).toBe('Something went wrong.');
  });
});

describe('typedConfirmReady (destructive mutation confirm flow)', () => {
  it('requires an exact match, trimmed', () => {
    expect(typedConfirmReady('suspend', 'suspend')).toBe(true);
    expect(typedConfirmReady('  suspend  ', 'suspend')).toBe(true);
    expect(typedConfirmReady('Suspend', 'suspend')).toBe(false);
    expect(typedConfirmReady('', 'suspend')).toBe(false);
  });
});

describe('canRemoveAdmin (the last admin is protected)', () => {
  const p1 = Principal.fromText('aaaaa-aa');
  const p2 = Principal.fromText('rrkah-fqaaa-aaaaa-aaaaq-cai');
  it('forbids removing the last admin', () => {
    expect(canRemoveAdmin([p1])).toBe(false);
  });
  it('allows removal when more than one admin remains', () => {
    expect(canRemoveAdmin([p1, p2])).toBe(true);
  });
});

describe('mergeAuditLogs (merged audit view across platform + payments)', () => {
  it('merges and sorts newest first', () => {
    const a = [{ at: 1n, method: 'platform_x' }];
    const b = [{ at: 3n, method: 'payments_y' }, { at: 2n, method: 'payments_z' }];
    expect(mergeAuditLogs(a, b).map((e) => e.method)).toEqual(['payments_y', 'payments_z', 'platform_x']);
  });
});

describe('inviteCodesToCsv', () => {
  it('renders one code per line with a header', () => {
    expect(inviteCodesToCsv(['ABC', 'DEF'])).toBe('code\nABC\nDEF');
  });
});

describe('belowMinSponsorCycles (payments rejects sponsor_cycles under 1T with InvalidInput)', () => {
  it('flags 0.5T as below the minimum', () => {
    expect(belowMinSponsorCycles('500000000000')).toBe(true);
  });
  it('accepts exactly the minimum, 1T', () => {
    expect(belowMinSponsorCycles(MIN_SPONSOR_CYCLES.toString())).toBe(false);
  });
  it('accepts more than the minimum', () => {
    expect(belowMinSponsorCycles('2000000000000')).toBe(false);
  });
  it('treats unparseable input as below the minimum', () => {
    expect(belowMinSponsorCycles('not-a-number')).toBe(true);
  });
});

describe('params diff preview editing (bigint/number safe round-trip)', () => {
  const params = { fee_e8s: 100n, quorum: 5 };
  it('renders every field as text, then rebuilds original types', () => {
    const text = textFromRecord(params);
    expect(text).toEqual({ fee_e8s: '100', quorum: '5' });
    expect(parseRecordEdits(params, text)).toEqual({ ok: true, value: params });
  });
  it('diffs only the changed fields', () => {
    const text = textFromRecord(params);
    const edited = { ...text, quorum: '7' };
    expect(recordDiff(text, edited)).toEqual([['quorum', '5', '7']]);
  });
});

describe('parseRecordEdits (params editor validates before BigInt, errors shown not thrown)', () => {
  const params = { fee_e8s: 100n, quorum: 5 };
  it('rebuilds typed values from valid text', () => {
    expect(parseRecordEdits(params, { fee_e8s: '250', quorum: '7' })).toEqual({
      ok: true,
      value: { fee_e8s: 250n, quorum: 7 },
    });
  });
  it('reports every invalid field instead of throwing', () => {
    expect(parseRecordEdits(params, { fee_e8s: '1.5', quorum: 'abc' })).toEqual({
      ok: false,
      errors: ['fee_e8s must be a whole number ≥ 0.', 'quorum must be a whole number ≥ 0.'],
    });
  });
  it('accepts decimals only for declared float64 fields (platform claim_cell_arcsec)', () => {
    const withFloat = { claim_cell_arcsec: 1.5, quorum: 5 };
    expect(parseRecordEdits(withFloat, { claim_cell_arcsec: '2.25', quorum: '5' }, ['claim_cell_arcsec'])).toEqual({
      ok: true,
      value: { claim_cell_arcsec: 2.25, quorum: 5 },
    });
    expect(parseRecordEdits(withFloat, textFromRecord(withFloat), ['claim_cell_arcsec']).ok).toBe(true);
    expect(parseRecordEdits(withFloat, { claim_cell_arcsec: '1.5', quorum: '5.5' }, ['claim_cell_arcsec']).ok).toBe(false);
    expect(parseRecordEdits(withFloat, { claim_cell_arcsec: '-1', quorum: '5' }, ['claim_cell_arcsec']).ok).toBe(false);
  });
  it('rejects negatives, blanks and numbers beyond nat32', () => {
    const r = parseRecordEdits(params, { fee_e8s: '-1', quorum: '4294967296' });
    expect(r.ok).toBe(false);
    expect(parseRecordEdits(params, { fee_e8s: '', quorum: '1' }).ok).toBe(false);
  });
});

describe('parsePrincipal / parseNat (treasury forms validate before Principal.fromText/BigInt)', () => {
  it('accepts a textual principal', () => {
    const r = parsePrincipal(' aaaaa-aa ');
    expect(r.ok && r.value.toText()).toBe('aaaaa-aa');
  });
  it('returns an error for a malformed principal', () => {
    expect(parsePrincipal('not a principal')).toEqual({ ok: false, error: 'Enter a valid principal.' });
    expect(parsePrincipal('')).toEqual({ ok: false, error: 'Enter a valid principal.' });
  });
  it('parses bounded naturals', () => {
    expect(parseNat('42', 'Amount (e8s)', 2n ** 64n - 1n)).toEqual({ ok: true, value: 42n });
    expect(parseNat('256', 'Priority', 255n)).toEqual({ ok: false, error: 'Priority must be a whole number from 0 to 255.' });
    expect(parseNat('1e3', 'Priority', 255n)).toEqual({ ok: false, error: 'Priority must be a whole number from 0 to 255.' });
  });
});

describe('proposalDetails (treasury approve shows what is being approved)', () => {
  const to = Principal.fromText('aaaaa-aa');
  it('describes a withdrawal with recipient and amount', () => {
    expect(
      proposalDetails({ __kind__: 'Withdraw', Withdraw: { to, amount_e8s: 150_000_000n } }, undefined),
    ).toEqual(['Withdraw 150000000 e8s (1.5 ICP) to aaaaa-aa']);
  });
  it('shows a SetConfig proposal as a diff against the current config', () => {
    const current = { reserve_e8s: 10n, runway_alert_days: 7, admins: [to] };
    const proposed = { reserve_e8s: 20n, runway_alert_days: 7, admins: [to] };
    expect(proposalDetails({ __kind__: 'SetConfig', SetConfig: proposed }, current)).toEqual([
      'reserve_e8s: 10 → 20',
    ]);
  });
  it('lists every field when the current config is unknown', () => {
    expect(proposalDetails({ __kind__: 'SetConfig', SetConfig: { reserve_e8s: 20n } }, undefined)).toEqual([
      'reserve_e8s: ? → 20',
    ]);
  });
});

describe('inviteStatus (admin invite list)', () => {
  it('prefers used, then expired, else active', () => {
    expect(inviteStatus({ used: true, expires_at: 1n }, 5n)).toBe('used');
    expect(inviteStatus({ used: false, expires_at: 5n }, 5n)).toBe('expired');
    expect(inviteStatus({ used: false, expires_at: 6n }, 5n)).toBe('active');
  });
});
