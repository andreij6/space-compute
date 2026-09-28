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

  it('serves a strict CSP with frame-ancestors none and IC/II connect-src', () => {
    const headers = readFileSync(join(root, '../public/_headers'), 'utf8');
    const csp = headers.match(/Content-Security-Policy: (.*)/)?.[1] ?? '';
    expect(csp).toContain("frame-ancestors 'none'");
    expect(csp).toContain("script-src 'self';");
    for (const host of ['https://icp-api.io', 'https://id.ai', 'http://localhost:*']) expect(csp).toContain(host);
  });
});
