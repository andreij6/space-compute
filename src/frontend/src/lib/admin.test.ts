import { describe, expect, it } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import {
  AdminApiError,
  adminGateDecision,
  belowMinSponsorCycles,
  canRemoveAdmin,
  humanAdminError,
  inviteCodesToCsv,
  mergeAuditLogs,
  MIN_SPONSOR_CYCLES,
  recordDiff,
  recordFromText,
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
    expect(recordFromText(params, text)).toEqual(params);
  });
  it('diffs only the changed fields', () => {
    const text = textFromRecord(params);
    const edited = { ...text, quorum: '7' };
    expect(recordDiff(text, edited)).toEqual([['quorum', '5', '7']]);
  });
});
