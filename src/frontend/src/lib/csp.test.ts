import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { dataOrigins, renderHeaders } from './csp';

const template = readFileSync(new URL('../../headers.template', import.meta.url), 'utf8');
const cspOf = (headers: string) => headers.match(/Content-Security-Policy: (.*)/)?.[1] ?? '';
const directive = (csp: string, name: string) =>
  csp
    .split(';')
    .map((d) => d.trim())
    .find((d) => d.startsWith(`${name} `)) ?? '';

describe('build-time CSP (05 §1)', () => {
  it('narrows img-src to self, the data bucket and data: URLs', () => {
    const csp = cspOf(renderHeaders(template, { VITE_SC_DOMAIN: 'spacecompute.app' }));
    expect(directive(csp, 'img-src')).toBe("img-src 'self' https://data.spacecompute.app data:");
  });

  it('lets the frontend fetch the data bucket (image hash verification, dossiers)', () => {
    const csp = cspOf(renderHeaders(template, { VITE_SC_DOMAIN: 'spacecompute.app' }));
    expect(directive(csp, 'connect-src')).toContain(' https://data.spacecompute.app');
    expect(csp).not.toContain('127.0.0.1');
  });

  it('adds the local bucket only for local builds that set it', () => {
    const csp = cspOf(
      renderHeaders(template, { VITE_SC_DOMAIN: 'spacecompute.app', VITE_LOCAL_DATA_ORIGIN: 'http://127.0.0.1:8765' }),
    );
    expect(directive(csp, 'img-src')).toContain('http://127.0.0.1:8765');
    expect(directive(csp, 'connect-src')).toContain('http://127.0.0.1:8765');
  });

  it('fails closed (no data host) when no domain is configured, and leaves no placeholder', () => {
    const headers = renderHeaders(template, {});
    expect(directive(cspOf(headers), 'img-src')).toBe("img-src 'self' data:");
    expect(headers).not.toContain('{{');
  });

  it('rejects a malformed domain or a non-loopback local origin', () => {
    expect(() => dataOrigins({ VITE_SC_DOMAIN: 'evil.com; script-src *' })).toThrow();
    expect(() => dataOrigins({ VITE_LOCAL_DATA_ORIGIN: 'https://evil.com' })).toThrow();
    expect(() => dataOrigins({ VITE_LOCAL_DATA_ORIGIN: 'http://127.0.0.1:8765/path' })).toThrow();
  });

  it('keeps frame-ancestors none, script-src self and the IC/II connect-src', () => {
    const csp = cspOf(renderHeaders(template, { VITE_SC_DOMAIN: 'spacecompute.app' }));
    expect(csp).toContain("frame-ancestors 'none'");
    expect(directive(csp, 'script-src')).toBe("script-src 'self'");
    for (const host of ['https://icp-api.io', 'https://id.ai']) expect(directive(csp, 'connect-src')).toContain(host);
  });
});
