import { describe, expect, it, vi } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import { activityKindLabel, answerLabel, loadRecordsPage, recordKindLabel } from './records';
import type { Protocol } from '../bindings/aaa';

const aaaId = Principal.fromText('rrkah-fqaaa-aaaaa-aaaaq-cai');

const protocol: Protocol = {
  version: 1,
  guidance_md: '',
  discovery_categories: [],
  questions: [
    {
      id: 'q1',
      prompt: 'Shape?',
      answers: [
        { id: 'spiral', label: 'Spiral', next: undefined },
        { id: 'elliptical', label: 'Elliptical', next: undefined },
      ],
    },
  ],
};

describe('answerLabel (05 §2 row 9: record detail renders answers against protocol labels)', () => {
  it('resolves the human label for a known question/answer pair', () => {
    expect(answerLabel(protocol, { question_id: 'q1', answer_id: 'spiral' })).toBe('Spiral');
  });

  it('falls back to the raw answer id when the question is unknown', () => {
    expect(answerLabel(protocol, { question_id: 'q9', answer_id: 'mystery' })).toBe('mystery');
  });

  it('falls back to the raw answer id when the answer option is unknown', () => {
    expect(answerLabel(protocol, { question_id: 'q1', answer_id: 'mystery' })).toBe('mystery');
  });

  it('falls back to the raw answer id when there is no protocol at all', () => {
    expect(answerLabel(null, { question_id: 'q1', answer_id: 'spiral' })).toBe('spiral');
  });
});

describe('recordKindLabel / activityKindLabel', () => {
  it('spaces out camelCase record kinds', () => {
    expect(recordKindLabel('TopUp' as never)).toBe('Top Up');
    expect(recordKindLabel('Operator' as never)).toBe('Operator');
  });

  it('spaces out camelCase activity kinds', () => {
    expect(activityKindLabel({ __kind__: 'DiscoveryResolved', DiscoveryResolved: { public_id: 'SC-2026-000001', outcome: 'Confirmed' } } as never)).toBe(
      'Discovery Resolved',
    );
  });
});

describe('loadRecordsPage (05 §2 row 9: aaa.list_records with platform.list_aaa_activity fallback)', () => {
  const record = {
    v: 1,
    at: 5n,
    by: aaaId,
    fee: 0n,
    seq: 7n,
    xp_awarded: 15,
    answers: [],
    kind: 'Classification',
  };

  it('lists from aaa.list_records on the success path', async () => {
    const aaa = { list_records: vi.fn().mockResolvedValue({ __kind__: 'Ok', Ok: { items: [record], next_cursor: 8n } }) };
    const platform = { list_aaa_activity: vi.fn() };
    const page = await loadRecordsPage(aaa as never, platform as never, aaaId, null, 20);
    expect(page.source).toBe('aaa');
    expect(page.nextCursor).toBe(8n);
    expect(page.items).toEqual([{ id: 7n, at: 5n, kindLabel: 'Classification', source: 'aaa', record }]);
    expect(platform.list_aaa_activity).not.toHaveBeenCalled();
  });

  it('passes the cursor through to list_records', async () => {
    const aaa = { list_records: vi.fn().mockResolvedValue({ __kind__: 'Ok', Ok: { items: [], next_cursor: undefined } }) };
    const platform = { list_aaa_activity: vi.fn() };
    await loadRecordsPage(aaa as never, platform as never, aaaId, 3n, 10);
    expect(aaa.list_records).toHaveBeenCalledWith({ cursor: 3n, limit: 10 });
  });

  it('falls back to platform.list_aaa_activity when list_records rejects (frozen canister)', async () => {
    const aaa = { list_records: vi.fn().mockRejectedValue(new Error('IC0207: canister is frozen')) };
    const activityItem = { id: 3n, at: 9n, aaa: aaaId, owner: aaaId, kind: { __kind__: 'Classified', Classified: { fee: 0n } } };
    const platform = { list_aaa_activity: vi.fn().mockResolvedValue({ items: [activityItem], next_cursor: undefined }) };
    const page = await loadRecordsPage(aaa as never, platform as never, aaaId, null, 20);
    expect(page.source).toBe('platform');
    expect(page.items).toEqual([{ id: 3n, at: 9n, kindLabel: 'Classified', source: 'platform', record: null }]);
    expect(platform.list_aaa_activity).toHaveBeenCalledWith(aaaId, null, 20);
  });

  it('falls back to platform.list_aaa_activity when list_records returns an Err result', async () => {
    const aaa = { list_records: vi.fn().mockResolvedValue({ __kind__: 'Err', Err: { __kind__: 'NotRegistered', NotRegistered: null } }) };
    const platform = { list_aaa_activity: vi.fn().mockResolvedValue({ items: [], next_cursor: undefined }) };
    const page = await loadRecordsPage(aaa as never, platform as never, aaaId, null, 20);
    expect(page.source).toBe('platform');
  });
});
