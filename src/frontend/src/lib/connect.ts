import { Principal } from '@icp-sdk/core/principal';
import type { Operator, Result } from '../bindings/aaa';
import { humanApiError } from './paymentOps';
import { isLocalHost } from '../ic';

export const MIN_PENDING_MS = 300;
const DAY_NS = 86_400_000_000_000n;

export type PrincipalInputResult = { ok: true; principal: Principal } | { ok: false; error: string };

export function parseOperatorPrincipal(text: string): PrincipalInputResult {
  const trimmed = text.trim();
  if (!trimmed) return { ok: false, error: 'Enter a principal.' };
  let principal: Principal;
  try {
    principal = Principal.fromText(trimmed);
  } catch {
    return { ok: false, error: 'That is not a valid principal.' };
  }
  if (principal.isAnonymous()) return { ok: false, error: 'The anonymous principal cannot be an operator.' };
  return { ok: true, principal };
}

export function isConnected(operators: Array<[Principal, Operator]>, nowNs: bigint): boolean {
  return operators.some(([, op]) => op.last_used_at != null && nowNs - op.last_used_at < DAY_NS);
}

export function withMinDelay<T>(promise: Promise<T>, ms: number = MIN_PENDING_MS): Promise<T> {
  return Promise.all([promise, new Promise<void>((resolve) => setTimeout(resolve, ms))]).then(([result]) => result);
}

export interface AddOperatorActor {
  add_operator(principal: Principal, label: string, expiresAt: bigint | null): Promise<Result>;
}
export interface RemoveOperatorActor {
  remove_operator(principal: Principal): Promise<Result>;
}

export type OperatorOpResult = { ok: true } | { ok: false; error: string };

export async function addOperator(aaa: AddOperatorActor, principal: Principal, label: string): Promise<OperatorOpResult> {
  const result = await withMinDelay(aaa.add_operator(principal, label, null));
  return result.__kind__ === 'Err' ? { ok: false, error: humanApiError(result.Err) } : { ok: true };
}

export async function removeOperator(aaa: RemoveOperatorActor, principal: Principal): Promise<OperatorOpResult> {
  const result = await withMinDelay(aaa.remove_operator(principal));
  return result.__kind__ === 'Err' ? { ok: false, error: humanApiError(result.Err) } : { ok: true };
}

export function todayStamp(d: Date = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}${m}${day}`;
}

export function operatorIdentityName(d: Date = new Date()): string {
  return `sc-operator-${todayStamp(d)}`;
}

export function netFlag(hostname: string): string {
  return isLocalHost(hostname) ? '-e local' : '-n ic';
}

export function newIdentityCommand(identityName: string): string {
  return `icp identity new ${identityName}`;
}

export function firstContactCommand(aaaIdText: string, identityName: string, net: string): string {
  return `icp canister call ${aaaIdText} whoami '()' ${net} --identity ${identityName} --query --candid agent-kit/skills/space-compute-astronomer/reference/aaa.did`;
}
