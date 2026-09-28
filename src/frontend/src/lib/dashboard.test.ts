import { describe, expect, it, vi } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import { fuelLevel, isFrozenReject, loadDashboard, mandateState } from './dashboard';

const aaaId = Principal.fromText('rrkah-fqaaa-aaaaa-aaaaq-cai');

const aaaPublic = {
  xp: 120n,
  status: 0 as never,
  name: 'Test AAA',
  is_house: false,
  badges: 0n,
  tier: 2,
  next_tier_xp: 500n,
  created_at: 0n,
  reputation_bp: 9000,
  counters: { reviews: 0n, discoveries: 0n, confirmed: 0n, classifications: 3n },
  avatar_seed: 1n,
};

describe('fuelLevel (05 §3 fuel gauge)', () => {
  it('is critical under 3 days, low up to 14, healthy above', () => {
    expect(fuelLevel(0)).toBe('critical');
    expect(fuelLevel(2)).toBe('critical');
    expect(fuelLevel(3)).toBe('low');
    expect(fuelLevel(14)).toBe('low');
    expect(fuelLevel(15)).toBe('healthy');
  });
});

describe('isFrozenReject', () => {
  it('recognizes a frozen/out-of-cycles reject message', () => {
    expect(isFrozenReject(new Error('Canister is frozen due to insufficient cycles'))).toBe(true);
    expect(isFrozenReject(new Error('IC0207: canister out of cycles'))).toBe(true);
    expect(isFrozenReject(new Error('Unauthorized'))).toBe(false);
  });
});

describe('loadDashboard (05 §2 row 7: aaa.status fallback platform.get_aaa_public)', () => {
  it('renders fuel days/cycles/tier on the success path', async () => {
    const aaa = { status: vi.fn().mockResolvedValue({ __kind__: 'Ok', Ok: { days_of_fuel_estimate: 21, cycles: 5_000_000_000_000n } }) };
    const platform = { get_aaa_public: vi.fn().mockResolvedValue(aaaPublic) };
    const data = await loadDashboard(aaa, platform, aaaId);
    expect(data.fuel).toEqual({ kind: 'live', daysRemaining: 21, cycles: 5_000_000_000_000n, level: 'healthy' });
    expect(data.aaaPublic?.tier).toBe(2);
    expect(platform.get_aaa_public).toHaveBeenCalledWith(aaaId);
  });

  it('falls back to get_aaa_public and reports frozen when the canister rejects status() as out of cycles', async () => {
    const aaa = { status: vi.fn().mockRejectedValue(new Error('IC0207: canister is frozen, out of cycles')) };
    const platform = { get_aaa_public: vi.fn().mockResolvedValue(aaaPublic) };
    const data = await loadDashboard(aaa, platform, aaaId);
    expect(data.fuel).toEqual({ kind: 'frozen' });
    expect(data.aaaPublic).toEqual(aaaPublic);
    expect(platform.get_aaa_public).toHaveBeenCalledWith(aaaId);
  });

  it('surfaces a human ApiError message for a non-frozen Err result', async () => {
    const aaa = { status: vi.fn().mockResolvedValue({ __kind__: 'Err', Err: { __kind__: 'Unauthorized', Unauthorized: null } }) };
    const platform = { get_aaa_public: vi.fn().mockResolvedValue(aaaPublic) };
    const data = await loadDashboard(aaa, platform, aaaId);
    expect(data.fuel).toEqual({ kind: 'error', message: 'You are not authorized to do that.' });
  });

  it('surfaces a generic message for a non-frozen rejection', async () => {
    const aaa = { status: vi.fn().mockRejectedValue(new Error('network down')) };
    const platform = { get_aaa_public: vi.fn().mockResolvedValue(aaaPublic) };
    const data = await loadDashboard(aaa, platform, aaaId);
    expect(data.fuel).toEqual({ kind: 'error', message: 'network down' });
  });
});

describe('mandateState (05 §3 auto top-up states)', () => {
  it('is none when the owner has never set a mandate', () => {
    expect(mandateState(null)).toBe('none');
  });
  it('is needs_attention when the mandate flags it', () => {
    expect(mandateState({ needs_attention: true })).toBe('needs_attention');
  });
  it('is ok otherwise', () => {
    expect(mandateState({ needs_attention: false })).toBe('ok');
  });
});
