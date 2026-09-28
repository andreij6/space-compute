import type { Principal } from '@icp-sdk/core/principal';
import type { AaaPublic } from '../bindings/platform';
import type { Result_7 as StatusResult } from '../bindings/aaa';
import { humanApiError } from './paymentOps';

export type FuelLevel = 'healthy' | 'low' | 'critical';

export function fuelLevel(daysRemaining: number): FuelLevel {
  if (daysRemaining < 3) return 'critical';
  if (daysRemaining <= 14) return 'low';
  return 'healthy';
}

export function isFrozenReject(e: unknown): boolean {
  const msg = e instanceof Error ? e.message : String(e);
  return /frozen|out of cycles|canister is stopped/i.test(msg);
}

export function formatCycles(cycles: bigint): string {
  return `${(Number(cycles) / 1e12).toFixed(2)} T Cycles`;
}

export type DashboardFuelView =
  | { kind: 'live'; daysRemaining: number; cycles: bigint; level: FuelLevel }
  | { kind: 'frozen' }
  | { kind: 'error'; message: string };

export interface AaaStatusActor {
  status(): Promise<StatusResult>;
}

export interface PlatformAaaPublicActor {
  get_aaa_public(owner: Principal): Promise<AaaPublic | null>;
}

export interface DashboardData {
  aaaPublic: AaaPublic | null;
  fuel: DashboardFuelView;
}

export async function loadDashboard(
  aaa: AaaStatusActor,
  platform: PlatformAaaPublicActor,
  aaaId: Principal,
): Promise<DashboardData> {
  const aaaPublic = await platform.get_aaa_public(aaaId);
  try {
    const result = await aaa.status();
    if (result.__kind__ === 'Ok') {
      const daysRemaining = result.Ok.days_of_fuel_estimate;
      return {
        aaaPublic,
        fuel: { kind: 'live', daysRemaining, cycles: result.Ok.cycles, level: fuelLevel(daysRemaining) },
      };
    }
    return { aaaPublic, fuel: { kind: 'error', message: humanApiError(result.Err) } };
  } catch (e) {
    if (isFrozenReject(e)) return { aaaPublic, fuel: { kind: 'frozen' } };
    return { aaaPublic, fuel: { kind: 'error', message: e instanceof Error ? e.message : 'Something went wrong.' } };
  }
}

export type MandateState = 'none' | 'needs_attention' | 'ok';

export function mandateState(mandate: { needs_attention: boolean } | null): MandateState {
  if (!mandate) return 'none';
  return mandate.needs_attention ? 'needs_attention' : 'ok';
}

export type MandateUiState = 'none' | 'enabled' | 'disabled' | 'needs_attention' | 'cap_reached';

export interface MandateLike {
  enabled: boolean;
  needs_attention: boolean;
  remaining_30d_e8s: bigint;
}

export function mandateUiState(mandate: MandateLike | null): MandateUiState {
  if (!mandate) return 'none';
  if (mandate.needs_attention) return 'needs_attention';
  if (!mandate.enabled) return 'disabled';
  if (mandate.remaining_30d_e8s <= 0n) return 'cap_reached';
  return 'enabled';
}

export function mandateStatusMessage(state: MandateUiState): string {
  switch (state) {
    case 'none':
      return 'No auto top-up configured.';
    case 'enabled':
      return 'Auto top-up is enabled.';
    case 'disabled':
      return 'Auto top-up is set up but disabled.';
    case 'needs_attention':
      return 'Auto top-up needs attention: allowance revoked or insufficient.';
    case 'cap_reached':
      return 'Auto top-up has reached its monthly limit for this 30-day cycle.';
    default:
      return 'Auto top-up status unknown.';
  }
}

export function mandateApproveAmountE8s(topupE8s: bigint, capE8s: bigint): bigint {
  return (topupE8s > capE8s ? topupE8s : capE8s) * 12n;
}
