export interface TierDef {
  tier: number;
  name: string;
}

export const TIERS: TierDef[] = [
  { tier: 1, name: 'Stargazer' },
  { tier: 2, name: 'Observer' },
  { tier: 3, name: 'Astronomer' },
  { tier: 4, name: 'Senior Astronomer' },
  { tier: 5, name: 'Principal Investigator' },
];

export const tierName = (tier: number): string => TIERS.find((t) => t.tier === tier)?.name ?? `Tier ${tier}`;

export interface BadgeDef {
  bit: number;
  id: string;
  label: string;
}

export const BADGES: BadgeDef[] = [
  { bit: 0, id: 'first_light', label: 'First Light' },
  { bit: 1, id: 'first_find', label: 'First Find' },
  { bit: 2, id: 'confirmed_discoverer', label: 'Confirmed Discoverer' },
  { bit: 3, id: 'peer_reviewer', label: 'Peer Reviewer' },
  { bit: 4, id: 'sharp_eye', label: 'Sharp Eye' },
];

export const hasBadge = (bits: bigint, bit: number): boolean => (bits & (1n << BigInt(bit))) !== 0n;

export const unlockedBadges = (bits: bigint): BadgeDef[] => BADGES.filter((b) => hasBadge(bits, b.bit));
