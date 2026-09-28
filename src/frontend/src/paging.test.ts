import { describe, expect, it } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import { dedupPages } from './paging';
import { CreditOutcome, CreditRole, type CreditCopy, type LeaderRow } from './bindings/platform';
import type { Op } from './bindings/payments';

const aaa = (n: number) => Principal.fromUint8Array(new Uint8Array([n]));

const leaderRow = (n: number, rank: number): LeaderRow => ({
  xp: BigInt(1000 - n),
  aaa: aaa(n),
  reviews: 10n,
  confirmed_discoveries: 1n,
  name: `Agent-${n}`,
  rank: BigInt(rank),
  tier: 3,
});

describe('leaderboard paging', () => {
  it('accumulates rows across pages without duplicates', () => {
    const page1 = [leaderRow(1, 1), leaderRow(2, 2)];
    const page2 = [leaderRow(3, 3), leaderRow(4, 4)];
    const rows = dedupPages([page1, page2], (r) => r.aaa.toText());
    expect(rows).toHaveLength(4);
    expect(rows.map((r) => r.name)).toEqual(['Agent-1', 'Agent-2', 'Agent-3', 'Agent-4']);
  });

  it('drops a row repeated across pages, keeping the first sighting', () => {
    const page1 = [leaderRow(1, 1)];
    const page2 = [leaderRow(1, 1), leaderRow(2, 2)];
    const rows = dedupPages([page1, page2], (r) => r.aaa.toText());
    expect(rows).toHaveLength(2);
    expect(rows.map((r) => r.name)).toEqual(['Agent-1', 'Agent-2']);
  });

  it('returns an empty list when no pages have loaded yet', () => {
    expect(dedupPages<LeaderRow>(undefined, (r) => r.aaa.toText())).toEqual([]);
  });
});

const credit = (publicId: string, role: CreditRole): CreditCopy => ({
  v: 1,
  at: 1n,
  public_id: publicId,
  role,
  subject_id: 7,
  category: 'lens',
  outcome: CreditOutcome.Confirmed,
});

describe('profile credits paging', () => {
  it('renders credits accumulated from every page', () => {
    const page1 = [credit('SC-2026-000001', CreditRole.Discoverer)];
    const page2 = [credit('SC-2026-000002', CreditRole.Reviewer)];
    const credits = dedupPages([page1, page2], (c) => `${c.public_id}:${c.role}`);
    expect(credits.map((c) => c.public_id)).toEqual(['SC-2026-000001', 'SC-2026-000002']);
  });

  it('keeps distinct roles on the same discovery but drops an exact repeat', () => {
    const page1 = [credit('SC-2026-000001', CreditRole.Discoverer), credit('SC-2026-000001', CreditRole.Reviewer)];
    const page2 = [credit('SC-2026-000001', CreditRole.Discoverer)];
    const credits = dedupPages([page1, page2], (c) => `${c.public_id}:${c.role}`);
    expect(credits).toHaveLength(2);
  });
});

const fuelOp = (id: number): Op => ({
  v: 1,
  id: BigInt(id),
  updated_at: 0n,
  kind: { __kind__: 'TopUp', TopUp: { aaa: aaa(1) } },
  path: { __kind__: 'Deposit', Deposit: null },
  attempts: 0,
  created_at: 0n,
  created_by: aaa(1),
  amount_e8s: 1_000_000n,
  state: { __kind__: 'Done', Done: null },
});

describe('fuel op history paging (list_ops_for_aaa, newest-first pages)', () => {
  it('accumulates ops across pages without duplicates', () => {
    const page1 = [fuelOp(3), fuelOp(2)];
    const page2 = [fuelOp(1)];
    const ops = dedupPages([page1, page2], (op) => op.id.toString());
    expect(ops.map((op) => op.id)).toEqual([3n, 2n, 1n]);
  });

  it('drops an op repeated across pages, keeping the first sighting', () => {
    const page1 = [fuelOp(2)];
    const page2 = [fuelOp(2), fuelOp(1)];
    const ops = dedupPages([page1, page2], (op) => op.id.toString());
    expect(ops.map((op) => op.id)).toEqual([2n, 1n]);
  });
});
