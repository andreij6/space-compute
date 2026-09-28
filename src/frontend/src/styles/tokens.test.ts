import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('./tokens.css', import.meta.url), 'utf8');
const tokens = Object.fromEntries([...css.matchAll(/--(color-[a-z-]+):\s*(#[0-9a-f]{6});/gi)].map((m) => [m[1], m[2]]));

const luminance = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};

const contrast = (a: string, b: string) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

const SURFACES = ['color-bg', 'color-surface', 'color-surface-raised'];
const TEXT_ON_SURFACES = ['color-text', 'color-text-muted', 'color-text-dim', 'color-accent', 'color-success', 'color-danger', 'color-info'];
const PAIRS: [string, string][] = [
  ...TEXT_ON_SURFACES.flatMap((fg) => SURFACES.map((bg): [string, string] => [fg, bg])),
  ['color-on-accent', 'color-accent'],
  ['color-on-accent', 'color-accent-strong'],
  ['color-accent', 'color-accent-soft'],
  ['color-success', 'color-success-soft'],
  ['color-danger', 'color-danger-soft'],
  ['color-info', 'color-info-soft'],
  ['color-text', 'color-danger-soft'],
  ['color-text-muted', 'color-surface-raised'],
];

describe('design tokens (05 §3 WCAG AA)', () => {
  it('computes the WCAG reference ratios', () => {
    expect(contrast('#000000', '#ffffff')).toBeCloseTo(21, 5);
    expect(contrast('#777777', '#ffffff')).toBeCloseTo(4.48, 2);
  });

  it.each(PAIRS)('%s on %s meets AA for normal text (≥ 4.5:1)', (fg, bg) => {
    expect(tokens[fg], fg).toBeDefined();
    expect(tokens[bg], bg).toBeDefined();
    expect(contrast(tokens[fg], tokens[bg])).toBeGreaterThanOrEqual(4.5);
  });

  it('declares every token group the components rely on', () => {
    for (const name of ['--text-md', '--space-4', '--radius-md', '--shadow-card', '--duration-fast', '--font-body']) {
      expect(css).toContain(`${name}:`);
    }
    expect(css).toMatch(/color-scheme:\s*dark/);
    expect(css).toMatch(/prefers-reduced-motion/);
  });
});
