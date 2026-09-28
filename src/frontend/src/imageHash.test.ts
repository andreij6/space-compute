import { afterEach, describe, expect, it, vi } from 'vitest';
import { loadVerifiedImage } from './imageHash';

const bytes = new TextEncoder().encode('a jwst rgb composite');
const sha = async (b: Uint8Array) => new Uint8Array(await crypto.subtle.digest('SHA-256', new Uint8Array(b)));

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('loadVerifiedImage', () => {
  it('returns a data URL of exactly the verified bytes (no second fetch)', async () => {
    const fetchMock = vi.fn(async () => new Response(bytes, { status: 200, headers: { 'Content-Type': 'image/png' } }));
    vi.stubGlobal('fetch', fetchMock);
    const result = await loadVerifiedImage('https://x/rgb.png', await sha(bytes));
    expect(result).toEqual({ kind: 'ok', src: `data:image/png;base64,${btoa('a jwst rgb composite')}` });
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('never labels the data URL with a non-image content type', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(bytes, { status: 200, headers: { 'Content-Type': 'text/html' } })));
    const result = await loadVerifiedImage('https://x/rgb.png', await sha(bytes));
    expect(result.kind === 'ok' && result.src.startsWith('data:image/png;base64,')).toBe(true);
  });

  it('fails closed when the hash does not match (tampered or wrong image)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(bytes, { status: 200 })));
    await expect(loadVerifiedImage('https://x/rgb.png', new Uint8Array(32))).resolves.toEqual({ kind: 'mismatch' });
  });

  it('reports unavailable when the image fails to load', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 404 })));
    await expect(loadVerifiedImage('https://x/missing.png', await sha(bytes))).resolves.toEqual({ kind: 'unavailable' });
  });

  it('reports unavailable instead of throwing when the fetch itself rejects (network, CORS, CSP)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => Promise.reject(new TypeError('Failed to fetch'))));
    await expect(loadVerifiedImage('https://x/rgb.png', await sha(bytes))).resolves.toEqual({ kind: 'unavailable' });
  });

  it('encodes large images without overflowing the call stack', async () => {
    const big = new Uint8Array(300_000).map((_, i) => i % 251);
    vi.stubGlobal('fetch', vi.fn(async () => new Response(big, { status: 200, headers: { 'Content-Type': 'image/png' } })));
    const result = await loadVerifiedImage('https://x/big.png', await sha(big));
    expect(result.kind).toBe('ok');
  });
});
