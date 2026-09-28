import type { Identity } from '@icp-sdk/core/agent';
import { safeGetCanisterEnv } from '@icp-sdk/core/agent/canister-env';
import type { AuthClientCreateOptions } from '@icp-sdk/auth/client';
import { createActor as createPlatform } from './bindings/platform';
import { createActor as createPayments } from './bindings/payments';
import { createActor as createTreasury } from './bindings/treasury';
import { createActor as createAaa } from './bindings/aaa';

export type CanisterName = 'platform' | 'payments' | 'treasury';
type Ids = { readonly [K in `PUBLIC_CANISTER_ID:${CanisterName}`]?: string };

export const II_CANISTER_ID = 'rdmx6-jaaaa-aaaaa-aaadq-cai';

export const canisterEnv = () => safeGetCanisterEnv<Ids>();

export function canisterId(name: CanisterName): string {
  const id = canisterEnv()?.[`PUBLIC_CANISTER_ID:${name}`];
  if (!id) throw new Error(`ic_env cookie has no canister id for ${name}`);
  return id;
}

const agentOptions = (identity?: Identity) => ({ identity, rootKey: canisterEnv()?.IC_ROOT_KEY });

export const platformActor = (identity?: Identity) =>
  createPlatform(canisterId('platform'), { agentOptions: agentOptions(identity) });
export const paymentsActor = (identity?: Identity) =>
  createPayments(canisterId('payments'), { agentOptions: agentOptions(identity) });
export const treasuryActor = (identity?: Identity) =>
  createTreasury(canisterId('treasury'), { agentOptions: agentOptions(identity) });
export const aaaActor = (aaaId: string, identity?: Identity) =>
  createAaa(aaaId, { agentOptions: agentOptions(identity) });

export const isLocalHost = (hostname: string) =>
  hostname === 'localhost' || hostname === '127.0.0.1' || hostname.endsWith('.localhost');

export function authOptions(
  loc: Pick<Location, 'hostname' | 'protocol' | 'port'>,
  rootKey: Uint8Array | undefined,
  derivationOrigin: string | undefined,
): AuthClientCreateOptions {
  if (isLocalHost(loc.hostname)) {
    return {
      identityProvider: {
        authorizeUrl: `${loc.protocol}//id.ai.localhost:${loc.port}/authorize`,
        canisterId: II_CANISTER_ID,
      },
      agentOptions: { rootKey },
    };
  }
  return derivationOrigin ? { derivationOrigin } : {};
}
