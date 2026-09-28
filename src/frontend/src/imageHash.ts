export type VerifiedImage = { kind: 'ok'; src: string } | { kind: 'mismatch' } | { kind: 'unavailable' };

function base64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary);
}

export async function loadVerifiedImage(url: string, expectedSha256: Uint8Array): Promise<VerifiedImage> {
  let res: Response;
  let bytes: Uint8Array;
  try {
    res = await fetch(url);
    if (!res.ok) return { kind: 'unavailable' };
    bytes = new Uint8Array(await res.arrayBuffer());
  } catch {
    return { kind: 'unavailable' };
  }
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new Uint8Array(bytes)));
  const matches = digest.length === expectedSha256.length && digest.every((v, i) => v === expectedSha256[i]);
  if (!matches) return { kind: 'mismatch' };
  const type = res.headers.get('Content-Type')?.split(';')[0].trim() ?? '';
  const mime = /^image\/[a-z0-9.+-]+$/i.test(type) ? type : 'image/png';
  return { kind: 'ok', src: `data:${mime};base64,${base64(bytes)}` };
}
