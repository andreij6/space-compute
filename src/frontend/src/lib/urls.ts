export function safeHref(raw: string, localOrigin?: string): string | null {
  try {
    const url = new URL(raw);
    if (url.protocol === 'https:') return url.href;
    if (localOrigin && url.origin === new URL(localOrigin).origin) return url.href;
  } catch {
    return null;
  }
  return null;
}
