import { afterEach, describe, expect, it, vi } from 'vitest';
import { verifyImageHash } from './imageHash';

const bytes = new TextEncoder().encode('a jwst rgb composite');

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('verifyImageHash', () => {
  it('passes when the fetched bytes hash to the expected sha256', async () => {
    const expected = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
    vi.stubGlobal('fetch', vi.fn(async () => new Response(bytes, { status: 200 })));
    await expect(verifyImageHash('https://x/rgb.png', expected)).resolves.toBe(true);
  });

  it('fails closed when the hash does not match (tampered or wrong image)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(bytes, { status: 200 })));
    await expect(verifyImageHash('https://x/rgb.png', new Uint8Array(32))).resolves.toBe(false);
  });

  it('fails closed when the image fails to load', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 404 })));
    const expected = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
    await expect(verifyImageHash('https://x/missing.png', expected)).resolves.toBe(false);
  });
});
