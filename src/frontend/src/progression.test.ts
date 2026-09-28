import { describe, expect, it } from 'vitest';
import { BADGES, hasBadge, tierName, unlockedBadges } from './progression';

describe('tierName', () => {
  it('labels every spec tier', () => {
    expect(tierName(1)).toBe('Stargazer');
    expect(tierName(2)).toBe('Observer');
    expect(tierName(3)).toBe('Astronomer');
    expect(tierName(4)).toBe('Senior Astronomer');
    expect(tierName(5)).toBe('Principal Investigator');
  });

  it('falls back to a generic label for an unknown tier', () => {
    expect(tierName(9)).toBe('Tier 9');
  });
});

describe('badge decoding', () => {
  it('decodes each badge bit on its own', () => {
    for (const badge of BADGES) {
      const bits = 1n << BigInt(badge.bit);
      expect(hasBadge(bits, badge.bit)).toBe(true);
      expect(unlockedBadges(bits)).toEqual([badge]);
    }
  });

  it('decodes no badges from an empty bitset', () => {
    expect(unlockedBadges(0n)).toEqual([]);
  });

  it('decodes every badge from a full bitset', () => {
    const allBits = BADGES.reduce((acc, b) => acc | (1n << BigInt(b.bit)), 0n);
    expect(unlockedBadges(allBits)).toEqual(BADGES);
  });

  it('decodes a mixed bitset without leaking unset bits', () => {
    const bits = (1n << 0n) | (1n << 2n) | (1n << 4n);
    const ids = unlockedBadges(bits).map((b) => b.id);
    expect(ids).toEqual(['first_light', 'confirmed_discoverer', 'sharp_eye']);
  });
});
