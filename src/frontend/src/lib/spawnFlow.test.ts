import { describe, expect, it } from 'vitest';
import type { Op } from '../bindings/payments';
import { CheckNameResult } from '../bindings/platform';
import {
  avatarGrid,
  checkNameResultMessage,
  clearSpawnOp,
  deriveAvatarSeeds,
  initialSpawnState,
  loadSpawnOp,
  resumeSpawnState,
  saveSpawnOp,
  spawnReducer,
  validateNameLocally,
  type KeyValueStore,
} from './spawnFlow';

function fakeStore(): KeyValueStore {
  const map = new Map<string, string>();
  return {
    getItem: (k) => map.get(k) ?? null,
    setItem: (k, v) => void map.set(k, v),
    removeItem: (k) => void map.delete(k),
  };
}

function op(state: Op['state'], id = 7n): Op {
  return { id, state } as Op;
}

describe('validateNameLocally (04 §4 pre-check / sc-types name limits)', () => {
  it('rejects an empty name', () => {
    expect(validateNameLocally('   ')).toEqual({ ok: false, message: 'Enter a name for your agent.' });
  });
  it('rejects a name under the minimum length', () => {
    expect(validateNameLocally('ab')).toEqual({ ok: false, message: 'Name must be 3-32 characters.' });
  });
  it('rejects a name over the maximum length', () => {
    expect(validateNameLocally('a'.repeat(33))).toEqual({ ok: false, message: 'Name must be 3-32 characters.' });
  });
  it('rejects characters outside letters, digits, space, _ . -', () => {
    expect(validateNameLocally('bad@name')).toEqual({
      ok: false,
      message: 'Name may only contain letters, digits, space, _ . -',
    });
  });
  it('accepts a valid trimmed name', () => {
    expect(validateNameLocally('  Nova Explorer  ')).toEqual({ ok: true, name: 'Nova Explorer' });
  });
  it('accepts the boundary lengths', () => {
    expect(validateNameLocally('abc').ok).toBe(true);
    expect(validateNameLocally('a'.repeat(32)).ok).toBe(true);
  });
});

describe('checkNameResultMessage (04 §4 check_name -> human message)', () => {
  it('has no message when the name is available', () => {
    expect(checkNameResultMessage(CheckNameResult.Ok)).toBeNull();
  });
  it('explains a taken name', () => {
    expect(checkNameResultMessage(CheckNameResult.Taken)).toMatch(/already taken/i);
  });
  it('explains an invalid/blocklisted name', () => {
    expect(checkNameResultMessage(CheckNameResult.Invalid)).toMatch(/isn't available/i);
  });
});

describe('avatarGrid (deterministic seed preview)', () => {
  it('is deterministic for the same seed', () => {
    expect(avatarGrid(42n)).toEqual(avatarGrid(42n));
  });
  it('differs across seeds', () => {
    expect(avatarGrid(1n)).not.toEqual(avatarGrid(2n));
  });
  it('is horizontally symmetric (classic identicon shape)', () => {
    const grid = avatarGrid(123n, 5);
    for (const row of grid) {
      for (let x = 0; x < row.length; x++) expect(row[x]).toBe(row[row.length - 1 - x]);
    }
  });
  it('handles a zero seed without throwing', () => {
    expect(() => avatarGrid(0n)).not.toThrow();
  });
});

describe('deriveAvatarSeeds (avatar picker options)', () => {
  it('is deterministic for the same base seed', () => {
    expect(deriveAvatarSeeds(42n, 6)).toEqual(deriveAvatarSeeds(42n, 6));
  });
  it('returns the requested count of distinct seeds', () => {
    const seeds = deriveAvatarSeeds(1n, 6);
    expect(seeds).toHaveLength(6);
    expect(new Set(seeds).size).toBe(6);
  });
  it('differs across base seeds', () => {
    expect(deriveAvatarSeeds(1n, 6)).not.toEqual(deriveAvatarSeeds(2n, 6));
  });
  it('handles a zero base seed without throwing', () => {
    expect(() => deriveAvatarSeeds(0n, 6)).not.toThrow();
  });
});

describe('spawnReducer (05 §2 row 6 spawn step machine)', () => {
  it('starts at the name step', () => {
    expect(initialSpawnState).toEqual({ step: 'name' });
  });
  it('advances name -> payment on confirmation', () => {
    expect(spawnReducer({ step: 'name' }, { type: 'NAME_CONFIRMED' })).toEqual({ step: 'payment' });
  });
  it('ignores NAME_CONFIRMED once past the name step', () => {
    const state = { step: 'payment' } as const;
    expect(spawnReducer(state, { type: 'NAME_CONFIRMED' })).toBe(state);
  });
  it('goes back to name from payment on EDIT_NAME', () => {
    expect(spawnReducer({ step: 'payment' }, { type: 'EDIT_NAME' })).toEqual({ step: 'name' });
  });
  it('advances payment -> progress when the op starts', () => {
    expect(spawnReducer({ step: 'payment' }, { type: 'OP_STARTED', opId: 5n })).toEqual({
      step: 'progress',
      opId: 5n,
    });
  });
  it('ignores OP_STARTED from the name step', () => {
    const state = { step: 'name' } as const;
    expect(spawnReducer(state, { type: 'OP_STARTED', opId: 5n })).toBe(state);
  });
  it('advances progress -> done on OP_DONE, keeping the opId', () => {
    expect(spawnReducer({ step: 'progress', opId: 9n }, { type: 'OP_DONE' })).toEqual({
      step: 'done',
      opId: 9n,
    });
  });
  it('ignores OP_DONE outside progress', () => {
    const state = { step: 'payment' } as const;
    expect(spawnReducer(state, { type: 'OP_DONE' })).toBe(state);
  });
  it('RESUME jumps straight to any state, bypassing the normal guards', () => {
    const resumed = { step: 'progress', opId: 3n } as const;
    expect(spawnReducer({ step: 'name' }, { type: 'RESUME', state: resumed })).toEqual(resumed);
  });
});

describe('resumeSpawnState (05 §2 row 6 resume after reload)', () => {
  it('is null when there is no op', () => {
    expect(resumeSpawnState(null)).toBeNull();
    expect(resumeSpawnState(undefined)).toBeNull();
  });
  it('resumes to progress for a pending op', () => {
    expect(resumeSpawnState(op({ __kind__: 'Pending' } as never))).toEqual({ step: 'progress', opId: 7n });
  });
  it('resumes to progress for a pulled/notified/registered op', () => {
    expect(resumeSpawnState(op({ __kind__: 'Pulled', Pulled: { block: 1n } } as never))).toEqual({
      step: 'progress',
      opId: 7n,
    });
  });
  it('resumes straight to done for a completed op', () => {
    expect(resumeSpawnState(op({ __kind__: 'Done' } as never))).toEqual({ step: 'done', opId: 7n });
  });
  it('treats a failed op as resumable progress (so the panel can offer retry)', () => {
    expect(resumeSpawnState(op({ __kind__: 'Failed', Failed: { reason: 'x' } } as never))).toEqual({
      step: 'progress',
      opId: 7n,
    });
  });
});

describe('spawn op storage (resume across reload, 05 §2 row 6)', () => {
  const alice = 'aaaaa-aa';
  const bob = '2vxsx-fae';
  it('round-trips the op id together with the agent name', () => {
    const store = fakeStore();
    expect(loadSpawnOp(store, alice)).toBeNull();
    saveSpawnOp(store, alice, { opId: 12345n, name: 'Hubble Jr' });
    expect(loadSpawnOp(store, alice)).toEqual({ opId: 12345n, name: 'Hubble Jr' });
    clearSpawnOp(store, alice);
    expect(loadSpawnOp(store, alice)).toBeNull();
  });
  it('keys storage by principal so another signed-in user never resumes it', () => {
    const store = fakeStore();
    saveSpawnOp(store, alice, { opId: 1n, name: 'Alpha' });
    expect(loadSpawnOp(store, bob)).toBeNull();
  });
  it('ignores a corrupted stored value', () => {
    const store = fakeStore();
    store.setItem(`sc.spawnOp:${alice}`, 'not-json');
    expect(loadSpawnOp(store, alice)).toBeNull();
    store.setItem(`sc.spawnOp:${alice}`, JSON.stringify({ opId: 'x', name: 'A' }));
    expect(loadSpawnOp(store, alice)).toBeNull();
  });
  it('never throws when storage is unavailable or throws', () => {
    const throwing: KeyValueStore = {
      getItem: () => {
        throw new Error('SecurityError');
      },
      setItem: () => {
        throw new Error('QuotaExceededError');
      },
      removeItem: () => {
        throw new Error('SecurityError');
      },
    };
    expect(() => saveSpawnOp(throwing, alice, { opId: 1n, name: 'A' })).not.toThrow();
    expect(loadSpawnOp(throwing, alice)).toBeNull();
    expect(() => clearSpawnOp(throwing, alice)).not.toThrow();
    expect(loadSpawnOp(null, alice)).toBeNull();
    expect(() => saveSpawnOp(null, alice, { opId: 1n, name: 'A' })).not.toThrow();
  });
});
