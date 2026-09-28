import { describe, expect, it } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import { spenderSubaccount } from './spenderSubaccount';

const toHex = (bytes: Uint8Array) => Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
const p = (n: number) => Principal.fromUint8Array(new Uint8Array(29).fill(n));

describe('spenderSubaccount (04 §1, S(purpose, beneficiary))', () => {
  it('matches the Rust reference vector for TopUp (t5_2_spender_subaccount_matches_the_spec_formula)', async () => {
    const sub = await spenderSubaccount('topup', p(2));
    expect(toHex(sub)).toBe('372976014c96e69db22e65dafc07cbec131e54d45c46a85d23e99b6b2a27c140');
  });

  it('differs by purpose and by beneficiary', async () => {
    const a = await spenderSubaccount('spawn', p(1));
    const b = await spenderSubaccount('topup', p(1));
    const c = await spenderSubaccount('spawn', p(2));
    expect(toHex(a)).not.toBe(toHex(b));
    expect(toHex(a)).not.toBe(toHex(c));
  });

  it('is deterministic', async () => {
    const a = await spenderSubaccount('auto', p(5));
    const b = await spenderSubaccount('auto', p(5));
    expect(toHex(a)).toBe(toHex(b));
  });
});
