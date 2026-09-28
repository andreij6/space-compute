import type { Op } from '../bindings/payments';
import { CheckNameResult } from '../bindings/platform';
import { opPhase } from './paymentOps';

export const NAME_MIN = 3;
export const NAME_MAX = 32;
const NAME_CHARS = /^[A-Za-z0-9 _.-]+$/;

export type NameValidation = { ok: true; name: string } | { ok: false; message: string };

export function validateNameLocally(raw: string): NameValidation {
  const name = raw.trim();
  if (name.length === 0) return { ok: false, message: 'Enter a name for your agent.' };
  const len = [...name].length;
  if (len < NAME_MIN || len > NAME_MAX) {
    return { ok: false, message: `Name must be ${NAME_MIN}-${NAME_MAX} characters.` };
  }
  if (!NAME_CHARS.test(name)) {
    return { ok: false, message: 'Name may only contain letters, digits, space, _ . -' };
  }
  return { ok: true, name };
}

export function checkNameResultMessage(result: CheckNameResult): string | null {
  switch (result) {
    case CheckNameResult.Ok:
      return null;
    case CheckNameResult.Taken:
      return 'That name is already taken.';
    case CheckNameResult.Invalid:
    default:
      return "That name isn't available.";
  }
}

const AVATAR_SEED_MAX = 999_999_999;

export function randomAvatarSeed(): bigint {
  return BigInt(Math.floor(Math.random() * AVATAR_SEED_MAX));
}

export function avatarGrid(seed: bigint, size = 5): boolean[][] {
  let s = (seed < 0n ? -seed : seed) & 0xffffffffffffffffn;
  if (s === 0n) s = 0x9e3779b97f4a7c15n;
  const next = (): number => {
    s = (s * 6364136223846793005n + 1442695040888963407n) & 0xffffffffffffffffn;
    return Number((s >> 33n) & 0xffffn);
  };
  const half = Math.ceil(size / 2);
  const grid: boolean[][] = [];
  for (let y = 0; y < size; y++) {
    const row: boolean[] = [];
    for (let x = 0; x < half; x++) row.push(next() % 2 === 0);
    for (let x = half; x < size; x++) row.push(row[size - 1 - x]);
    grid.push(row);
  }
  return grid;
}

export type SpawnState =
  | { step: 'name' }
  | { step: 'payment' }
  | { step: 'progress'; opId: bigint }
  | { step: 'done'; opId: bigint };

export type SpawnEvent =
  | { type: 'NAME_CONFIRMED' }
  | { type: 'EDIT_NAME' }
  | { type: 'OP_STARTED'; opId: bigint }
  | { type: 'OP_DONE' }
  | { type: 'RESUME'; state: SpawnState };

export const initialSpawnState: SpawnState = { step: 'name' };

export function spawnReducer(state: SpawnState, event: SpawnEvent): SpawnState {
  switch (event.type) {
    case 'NAME_CONFIRMED':
      return state.step === 'name' ? { step: 'payment' } : state;
    case 'EDIT_NAME':
      return state.step === 'payment' ? { step: 'name' } : state;
    case 'OP_STARTED':
      return state.step === 'payment' || state.step === 'progress'
        ? { step: 'progress', opId: event.opId }
        : state;
    case 'OP_DONE':
      return state.step === 'progress' ? { step: 'done', opId: state.opId } : state;
    case 'RESUME':
      return event.state;
    default:
      return state;
  }
}

export function resumeSpawnState(op: Op | null | undefined): SpawnState | null {
  if (!op) return null;
  return opPhase(op) === 'done' ? { step: 'done', opId: op.id } : { step: 'progress', opId: op.id };
}

export interface KeyValueStore {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export type SavedSpawnOp = { opId: bigint; name: string };

const spawnOpKey = (principal: string) => `sc.spawnOp:${principal}`;

export function browserStore(): KeyValueStore | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

export function saveSpawnOp(store: KeyValueStore | null, principal: string, op: SavedSpawnOp): void {
  try {
    store?.setItem(spawnOpKey(principal), JSON.stringify({ opId: op.opId.toString(), name: op.name }));
  } catch {
    return;
  }
}

export function loadSpawnOp(store: KeyValueStore | null, principal: string): SavedSpawnOp | null {
  try {
    const raw = store?.getItem(spawnOpKey(principal));
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null) return null;
    const { opId, name } = parsed as Record<string, unknown>;
    if (typeof opId !== 'string' || !/^\d+$/.test(opId) || typeof name !== 'string') return null;
    return { opId: BigInt(opId), name };
  } catch {
    return null;
  }
}

export function clearSpawnOp(store: KeyValueStore | null, principal: string): void {
  try {
    store?.removeItem(spawnOpKey(principal));
  } catch {
    return;
  }
}
