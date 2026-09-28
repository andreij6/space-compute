import { afterEach, describe, expect, it } from 'vitest';
import { authOptions, canisterEnv, canisterId, II_CANISTER_ID } from './ic';

const ROOT_KEY_HEX =
  '308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100' + 'ab'.repeat(96);

const setCookie = (cookie: string) => Object.assign(globalThis, { document: { cookie } });

afterEach(() => {
  delete (globalThis as { document?: unknown }).document;
});

describe('ic_env', () => {
  it('reads canister ids and the root key from the ic_env cookie', () => {
    setCookie(
      `other=1; ic_env=${encodeURIComponent(
        `PUBLIC_CANISTER_ID:platform=4qggx-l3777-77775-aaaca-cai&PUBLIC_CANISTER_ID:payments=4caro-hl777-77775-aaaba-cai&ic_root_key=${ROOT_KEY_HEX}`,
      )}`,
    );
    expect(canisterId('platform')).toBe('4qggx-l3777-77775-aaaca-cai');
    expect(canisterId('payments')).toBe('4caro-hl777-77775-aaaba-cai');
    expect(canisterEnv()?.IC_ROOT_KEY).toHaveLength(ROOT_KEY_HEX.length / 2);
  });

  it('fails loudly when the cookie lacks a canister id', () => {
    setCookie(`ic_env=${encodeURIComponent(`ic_root_key=${ROOT_KEY_HEX}`)}`);
    expect(() => canisterId('treasury')).toThrow(/treasury/);
  });

  it('is undefined without a cookie', () => {
    setCookie('');
    expect(canisterEnv()).toBeUndefined();
  });
});

describe('authOptions', () => {
  const rootKey = new Uint8Array([1, 2, 3]);

  it('uses the local II canister and the ic_env root key on the local network', () => {
    const o = authOptions({ hostname: 'abc-cai.localhost', protocol: 'http:', port: '8000' }, rootKey, 'https://x.io');
    expect(o.identityProvider).toEqual({
      authorizeUrl: 'http://id.ai.localhost:8000/authorize',
      canisterId: II_CANISTER_ID,
    });
    expect(o.agentOptions?.rootKey).toBe(rootKey);
    expect(o.derivationOrigin).toBeUndefined();
  });

  it('uses mainnet II (id.ai default) with the pinned derivation origin in production', () => {
    const o = authOptions({ hostname: 'spacecompute.org', protocol: 'https:', port: '' }, rootKey, 'https://abc.icp.net');
    expect(o).toEqual({ derivationOrigin: 'https://abc.icp.net' });
    expect(authOptions({ hostname: 'abc.icp.net', protocol: 'https:', port: '' }, rootKey, undefined)).toEqual({});
  });
});
