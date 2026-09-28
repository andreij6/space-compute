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

export function recordFromText<T extends object>(base: T, text: Record<string, string>): T {
  const out = { ...base } as Record<string, unknown>;
  for (const k of Object.keys(base as Record<string, unknown>)) {
    out[k] = typeof (base as Record<string, unknown>)[k] === 'bigint' ? BigInt(text[k]) : Number(text[k]);
  }
  return out as T;
}

export function recordDiff(base: Record<string, string>, edited: Record<string, string>): [string, string, string][] {
  return Object.keys(base)
    .filter((k) => base[k] !== edited[k])
    .map((k): [string, string, string] => [k, base[k], edited[k]]);
}
