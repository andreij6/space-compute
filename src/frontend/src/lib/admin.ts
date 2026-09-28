import { Principal } from '@icp-sdk/core/principal';
import { formatIcp } from './paymentOps';

export type AdminGate = 'loading' | 'denied' | 'allowed';

export function adminGateDecision(
  ready: boolean,
  hasPrincipal: boolean,
  admin: { isPending: boolean; data: boolean | undefined },
): AdminGate {
  if (!ready || (hasPrincipal && admin.isPending)) return 'loading';
  return admin.data ? 'allowed' : 'denied';
}

export interface ApiErrLike {
  __kind__: string;
  [k: string]: unknown;
}

export type ResultLike<T> = { __kind__: 'Ok'; Ok: T } | { __kind__: 'Err'; Err: ApiErrLike };

export function humanAdminError(err: ApiErrLike): string {
  switch (err.__kind__) {
    case 'Unauthorized':
      return 'You are not authorized to do that.';
    case 'NotFound':
      return 'Not found.';
    case 'InvalidInput':
      return (err.InvalidInput as string) || 'That input is not valid.';
    case 'Suspended':
      return 'This account is suspended.';
    case 'Conflict':
      return (err.Conflict as string) || 'That conflicts with existing state.';
    case 'RateLimited':
      return `Too many attempts. Try again in ${(err.RateLimited as { retry_after_secs: number }).retry_after_secs}s.`;
    case 'NotEligible':
      return (err.NotEligible as string) || 'You are not eligible for this.';
    case 'FeatureDisabled':
      return 'This feature is not available yet.';
    case 'NotRegistered':
      return 'This is not registered yet.';
    case 'LeaseExpired':
      return 'The lease expired.';
    case 'LeaseNotFound':
      return 'No active lease was found.';
    case 'InsufficientFee':
      return `The ledger fee changed; it now requires ${(err.InsufficientFee as { required: bigint }).required} e8s.`;
    case 'Internal':
      return (err.Internal as string) || 'Something went wrong on our side. Please try again.';
    default:
      return 'Something went wrong.';
  }
}

export class AdminApiError extends Error {
  constructor(public readonly apiError: ApiErrLike) {
    super(humanAdminError(apiError));
    this.name = 'AdminApiError';
  }
}

export function unwrapAdmin<T>(result: ResultLike<T>): T {
  if (result.__kind__ === 'Err') throw new AdminApiError(result.Err);
  return result.Ok;
}

export function typedConfirmReady(input: string, phrase: string): boolean {
  return input.trim() === phrase;
}

export function canRemoveAdmin(admins: { toText(): string }[]): boolean {
  return admins.length > 1;
}

export interface AuditLike {
  at: bigint;
}

export function mergeAuditLogs<T extends AuditLike>(a: T[], b: T[]): T[] {
  return [...a, ...b].sort((x, y) => (y.at > x.at ? 1 : y.at < x.at ? -1 : 0));
}

export function inviteCodesToCsv(codes: string[]): string {
  return ['code', ...codes].join('\n');
}

export const MIN_SPONSOR_CYCLES = 1_000_000_000_000n;

export function parseCyclesInput(input: string): bigint | null {
  try {
    return BigInt(input || '0');
  } catch {
    return null;
  }
}

export function belowMinSponsorCycles(input: string): boolean {
  const parsed = parseCyclesInput(input);
  return parsed === null || parsed < MIN_SPONSOR_CYCLES;
}

export function textFromRecord(record: object): Record<string, string> {
  return Object.fromEntries(Object.entries(record).map(([k, v]) => [k, String(v)]));
}

export function recordDiff(base: Record<string, string>, edited: Record<string, string>): [string, string, string][] {
  return Object.keys(base)
    .filter((k) => base[k] !== edited[k])
    .map((k): [string, string, string] => [k, base[k], edited[k]]);
}

export type Parsed<T> = { ok: true; value: T } | { ok: false; error: string };

const NAT32_MAX = 4_294_967_295n;

export function parseRecordEdits<T extends object>(
  base: T,
  text: Record<string, string>,
): { ok: true; value: T } | { ok: false; errors: string[] } {
  const out = { ...base } as Record<string, unknown>;
  const errors: string[] = [];
  for (const [k, v] of Object.entries(base as Record<string, unknown>)) {
    const raw = (text[k] ?? '').trim();
    const n = /^\d+$/.test(raw) ? BigInt(raw) : null;
    if (n === null || (typeof v !== 'bigint' && n > NAT32_MAX)) {
      errors.push(`${k} must be a whole number ≥ 0.`);
      continue;
    }
    out[k] = typeof v === 'bigint' ? n : Number(n);
  }
  return errors.length ? { ok: false, errors } : { ok: true, value: out as T };
}

export function parsePrincipal(text: string): Parsed<Principal> {
  try {
    const trimmed = text.trim();
    if (!trimmed) throw new Error('empty');
    return { ok: true, value: Principal.fromText(trimmed) };
  } catch {
    return { ok: false, error: 'Enter a valid principal.' };
  }
}

export function parseNat(text: string, label: string, max: bigint): Parsed<bigint> {
  const raw = text.trim();
  const n = /^\d+$/.test(raw) ? BigInt(raw) : null;
  if (n === null || n > max) return { ok: false, error: `${label} must be a whole number from 0 to ${max}.` };
  return { ok: true, value: n };
}

export type ProposalActionLike =
  | { __kind__: 'Withdraw'; Withdraw: { to: { toText(): string }; amount_e8s: bigint } }
  | { __kind__: 'SetConfig'; SetConfig: object };

export function proposalDetails(action: ProposalActionLike, currentConfig: object | undefined): string[] {
  if (action.__kind__ === 'Withdraw') {
    const { to, amount_e8s } = action.Withdraw;
    return [`Withdraw ${amount_e8s} e8s (${formatIcp(amount_e8s)} ICP) to ${to.toText()}`];
  }
  const proposed = textFromRecord(action.SetConfig);
  const current = currentConfig
    ? textFromRecord(currentConfig)
    : Object.fromEntries(Object.keys(proposed).map((k) => [k, '?']));
  return recordDiff(current, proposed).map(([k, before, after]) => `${k}: ${before} → ${after}`);
}

export function inviteStatus(invite: { used: boolean; expires_at: bigint }, nowNs: bigint): 'used' | 'expired' | 'active' {
  if (invite.used) return 'used';
  if (invite.expires_at <= nowNs) return 'expired';
  return 'active';
}
