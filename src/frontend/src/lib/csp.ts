export type HeadersEnv = { VITE_SC_DOMAIN?: string; VITE_LOCAL_DATA_ORIGIN?: string };

const DOMAIN = /^(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$/i;

export function dataOrigins(env: HeadersEnv): string[] {
  const origins: string[] = [];
  if (env.VITE_SC_DOMAIN) {
    if (!DOMAIN.test(env.VITE_SC_DOMAIN)) throw new Error(`VITE_SC_DOMAIN is not a domain: ${env.VITE_SC_DOMAIN}`);
    origins.push(`https://data.${env.VITE_SC_DOMAIN}`);
  }
  if (env.VITE_LOCAL_DATA_ORIGIN) {
    const url = new URL(env.VITE_LOCAL_DATA_ORIGIN);
    if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || url.origin !== env.VITE_LOCAL_DATA_ORIGIN) {
      throw new Error(`VITE_LOCAL_DATA_ORIGIN must be an http://127.0.0.1:<port> origin: ${env.VITE_LOCAL_DATA_ORIGIN}`);
    }
    origins.push(url.origin);
  }
  return origins;
}

export function renderHeaders(template: string, env: HeadersEnv): string {
  const origins = dataOrigins(env).join(' ');
  return template.replaceAll(' {{DATA_ORIGINS}}', origins ? ` ${origins}` : '');
}
