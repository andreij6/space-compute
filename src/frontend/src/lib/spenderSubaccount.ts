import type { Principal } from '@icp-sdk/core/principal';

export type SubaccountPurpose = 'spawn' | 'topup' | 'auto';

const PREFIX = new TextEncoder().encode('sc-spender');
const TAGS: Record<SubaccountPurpose, Uint8Array> = {
  spawn: new TextEncoder().encode('spawn'),
  topup: new TextEncoder().encode('topup'),
  auto: new TextEncoder().encode('auto'),
};

export async function spenderSubaccount(purpose: SubaccountPurpose, beneficiary: Principal): Promise<Uint8Array> {
  const tag = TAGS[purpose];
  const beneficiaryBytes = beneficiary.toUint8Array();
  const input = new Uint8Array(PREFIX.length + tag.length + beneficiaryBytes.length);
  input.set(PREFIX, 0);
  input.set(tag, PREFIX.length);
  input.set(beneficiaryBytes, PREFIX.length + tag.length);
  const digest = await crypto.subtle.digest('SHA-256', input);
  return new Uint8Array(digest);
}
