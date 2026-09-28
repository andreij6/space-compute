import { describe, expect, it, vi } from 'vitest';
import { Principal } from '@icp-sdk/core/principal';
import {
  addOperator,
  firstContactCommand,
  isConnected,
  netFlag,
  newIdentityCommand,
  operatorIdentityName,
  parseOperatorPrincipal,
  removeOperator,
  todayStamp,
  withMinDelay,
} from './connect';

const operatorPrincipal = Principal.fromText('rrkah-fqaaa-aaaaa-aaaaq-cai');

describe('parseOperatorPrincipal (05 §3 connect: principal input validation)', () => {
  it('accepts a valid principal', () => {
    const result = parseOperatorPrincipal('rrkah-fqaaa-aaaaa-aaaaq-cai');
    expect(result.ok).toBe(true);
    if (result.ok) expect(result.principal.toText()).toBe('rrkah-fqaaa-aaaaa-aaaaq-cai');
  });

  it('trims surrounding whitespace', () => {
    const result = parseOperatorPrincipal('  rrkah-fqaaa-aaaaa-aaaaq-cai  ');
    expect(result.ok).toBe(true);
  });

  it('rejects an empty string', () => {
    const result = parseOperatorPrincipal('   ');
    expect(result).toEqual({ ok: false, error: 'Enter a principal.' });
  });

  it('rejects text that is not a valid principal', () => {
    const result = parseOperatorPrincipal('not-a-principal');
    expect(result).toEqual({ ok: false, error: 'That is not a valid principal.' });
  });

  it('rejects the anonymous principal', () => {
    const result = parseOperatorPrincipal(Principal.anonymous().toText());
    expect(result).toEqual({ ok: false, error: 'The anonymous principal cannot be an operator.' });
  });
});

describe('isConnected (05 §3: an operator exists with last_used_at < 24h)', () => {
  const now = 1_700_000_000_000_000_000n;
  const day = 86_400_000_000_000n;

  it('is false with no operators', () => {
    expect(isConnected([], now)).toBe(false);
  });

  it('is false when the only operator has never been used', () => {
    const ops: [Principal, { v: number; label: string; added_at: bigint }][] = [
      [operatorPrincipal, { v: 1, label: 'claude-code', added_at: now - day }],
    ];
    expect(isConnected(ops as never, now)).toBe(false);
  });

  it('is true when an operator was used under 24h ago', () => {
    const ops = [[operatorPrincipal, { v: 1, label: 'claude-code', added_at: now - day, last_used_at: now - day / 2n }]];
    expect(isConnected(ops as never, now)).toBe(true);
  });

  it('is false when the only operator was last used 24h or more ago', () => {
    const ops = [[operatorPrincipal, { v: 1, label: 'claude-code', added_at: now - day, last_used_at: now - day }]];
    expect(isConnected(ops as never, now)).toBe(false);
  });
});

describe('withMinDelay (05 §3: every update call shows pending state for >= 300ms)', () => {
  it('does not resolve before the minimum delay even when the call finishes instantly', async () => {
    vi.useFakeTimers();
    let resolved = false;
    withMinDelay(Promise.resolve('done'), 300).then(() => {
      resolved = true;
    });
    await vi.advanceTimersByTimeAsync(299);
    expect(resolved).toBe(false);
    await vi.advanceTimersByTimeAsync(1);
    expect(resolved).toBe(true);
    vi.useRealTimers();
  });

  it('resolves with the underlying value', async () => {
    await expect(withMinDelay(Promise.resolve(42), 0)).resolves.toBe(42);
  });
});

describe('addOperator / removeOperator (05 §3: ApiError messages)', () => {
  it('returns ok on success', async () => {
    const aaa = { add_operator: vi.fn().mockResolvedValue({ __kind__: 'Ok', Ok: null }) };
    const result = await addOperator(aaa, operatorPrincipal, 'claude-code');
    expect(result).toEqual({ ok: true });
    expect(aaa.add_operator).toHaveBeenCalledWith(operatorPrincipal, 'claude-code', null);
  });

  it('maps an ApiError to a human message on add', async () => {
    const aaa = { add_operator: vi.fn().mockResolvedValue({ __kind__: 'Err', Err: { __kind__: 'Unauthorized', Unauthorized: null } }) };
    const result = await addOperator(aaa, operatorPrincipal, 'claude-code');
    expect(result).toEqual({ ok: false, error: 'You are not authorized to do that.' });
  });

  it('maps an ApiError to a human message on remove', async () => {
    const aaa = { remove_operator: vi.fn().mockResolvedValue({ __kind__: 'Err', Err: { __kind__: 'NotFound', NotFound: null } }) };
    const result = await removeOperator(aaa, operatorPrincipal);
    expect(result).toEqual({ ok: false, error: 'Not found.' });
  });

  it('returns ok on successful remove', async () => {
    const aaa = { remove_operator: vi.fn().mockResolvedValue({ __kind__: 'Ok', Ok: null }) };
    const result = await removeOperator(aaa, operatorPrincipal);
    expect(result).toEqual({ ok: true });
  });
});

describe('connect command helpers (05 §3 connect steps)', () => {
  it('formats today as YYYYMMDD', () => {
    expect(todayStamp(new Date(2026, 8, 27))).toBe('20260927');
  });

  it('names the operator identity after today', () => {
    expect(operatorIdentityName(new Date(2026, 8, 27))).toBe('sc-operator-20260927');
  });

  it('uses -e local on a local host and -n ic elsewhere', () => {
    expect(netFlag('localhost')).toBe('-e local');
    expect(netFlag('app.localhost')).toBe('-e local');
    expect(netFlag('spacecompute.app')).toBe('-n ic');
  });

  it('creates the operator identity with the default (encrypted/keyring) storage, never plaintext (05 §3 step 1)', () => {
    expect(newIdentityCommand('sc-operator-20260927')).toBe('icp identity new sc-operator-20260927');
  });

  it('builds the first-contact whoami command', () => {
    expect(firstContactCommand('aaaid-cai', 'sc-operator-20260927', '-e local')).toBe(
      "icp canister call aaaid-cai whoami '()' -e local --identity sc-operator-20260927 --query --candid agent-kit/skills/space-compute-astronomer/reference/aaa.did",
    );
  });
});
