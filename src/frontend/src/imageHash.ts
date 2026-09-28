export async function verifyImageHash(url: string, expectedSha256: Uint8Array): Promise<boolean> {
  const res = await fetch(url);
  if (!res.ok) return false;
  const bytes = new Uint8Array(await res.arrayBuffer());
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return digest.length === expectedSha256.length && digest.every((v, i) => v === expectedSha256[i]);
}
