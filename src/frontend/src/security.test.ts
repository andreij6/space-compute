import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const FORBIDDEN = ['fetch', 'RootKey'].join('');
const root = new URL('.', import.meta.url).pathname;

const files = (dir: string): string[] =>
  readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? files(p) : /\.(ts|tsx|js)$/.test(p) ? [p] : [];
  });

describe('security', () => {
  it('never fetches the root key at runtime (ic_env cookie only)', () => {
    const offenders = files(root).filter((f) => readFileSync(f, 'utf8').includes(FORBIDDEN));
    expect(offenders).toEqual([]);
  });

  it('never renders untrusted HTML', () => {
    const html = ['dangerously', 'SetInnerHTML'].join('');
    expect(files(root).filter((f) => readFileSync(f, 'utf8').includes(html))).toEqual([]);
  });

  it('ships the CSP from the build-time template only (no stale public/_headers)', () => {
    expect(() => readFileSync(join(root, '../public/_headers'), 'utf8')).toThrow();
  });
});
