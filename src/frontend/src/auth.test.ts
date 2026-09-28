import { describe, expect, it, vi } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import { guardDecision, homeRoute, listsMe } from './auth';

const me = Principal.fromText('aaaaa-aa');
const aaa = Principal.fromText('rrkah-fqaaa-aaaaa-aaaaq-cai');

describe('sign-in routing (05 §2)', () => {
  it('routes an owner without an AAA to /spawn', async () => {
    const platform = { aaa_by_owner: vi.fn().mockResolvedValue(null) };
    expect(await homeRoute(platform, me)).toBe('/spawn');
    expect(platform.aaa_by_owner).toHaveBeenCalledWith(me);
  });

  it('routes an owner with an AAA to /dashboard', async () => {
    const platform = { aaa_by_owner: vi.fn().mockResolvedValue(aaa) };
    expect(await homeRoute(platform, me)).toBe('/dashboard');
  });

  it('propagates a failed lookup instead of guessing a route', async () => {
    const platform = { aaa_by_owner: vi.fn().mockRejectedValue(new Error('down')) };
    await expect(homeRoute(platform, me)).rejects.toThrow('down');
  });
});

describe('owner route guard', () => {
  const loaded = (data: Principal | null) => ({ isPending: false, isError: false, data });
  const pending = { isPending: true, isError: false, data: undefined };

  it('waits for the auth client', () => {
    expect(guardDecision({ ready: false, signedIn: false, needAaa: true, aaa: pending })).toBe('loading');
  });
  it('sends signed-out visitors to sign-in', () => {
    expect(guardDecision({ ready: true, signedIn: false, needAaa: false, aaa: pending })).toBe('/signin');
    expect(guardDecision({ ready: true, signedIn: false, needAaa: true, aaa: loaded(aaa) })).toBe('/signin');
  });
  it('lets a signed-in owner into /spawn without an AAA', () => {
    expect(guardDecision({ ready: true, signedIn: true, needAaa: false, aaa: loaded(null) })).toBe('ok');
  });
  it('sends an owner without an AAA from AAA routes to /spawn', () => {
    expect(guardDecision({ ready: true, signedIn: true, needAaa: true, aaa: pending })).toBe('loading');
    expect(guardDecision({ ready: true, signedIn: true, needAaa: true, aaa: loaded(null) })).toBe('/spawn');
    expect(guardDecision({ ready: true, signedIn: true, needAaa: true, aaa: loaded(aaa) })).toBe('ok');
  });
  it('shows an error when the AAA lookup fails', () => {
    expect(
      guardDecision({ ready: true, signedIn: true, needAaa: true, aaa: { isPending: false, isError: true, data: undefined } }),
    ).toBe('error');
  });
});

describe('admin gate (cosmetic)', () => {
  it('is admin only when an admin list contains the caller', () => {
    const ok = (ps: Principal[]) => ({ status: 'fulfilled' as const, value: { __kind__: 'Ok' as const, Ok: ps } });
    expect(listsMe(ok([aaa, me]), me.toText())).toBe(true);
    expect(listsMe(ok([aaa]), me.toText())).toBe(false);
    expect(listsMe({ status: 'fulfilled', value: { __kind__: 'Err', Err: { Unauthorized: null } } }, me.toText())).toBe(false);
    expect(listsMe({ status: 'rejected', reason: new Error('x') }, me.toText())).toBe(false);
  });
});
