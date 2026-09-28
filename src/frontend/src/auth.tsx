import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { AuthClient } from '@icp-sdk/auth/client';
import type { Identity } from '@icp-sdk/core/agent';
import type { Principal } from '@icp-sdk/core/principal';
import { useQuery } from '@tanstack/react-query';
import { authOptions, canisterEnv, paymentsActor, platformActor } from './ic';

type AuthState = {
  ready: boolean;
  identity: Identity | null;
  principal: Principal | null;
  signIn: () => Promise<Identity>;
  signOut: () => Promise<void>;
};

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [client] = useState(
    () =>
      new AuthClient(
        authOptions(window.location, canisterEnv()?.IC_ROOT_KEY, import.meta.env.VITE_II_DERIVATION_ORIGIN),
      ),
  );
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const sync = () =>
      (client.isAuthenticated() ? client.getIdentity() : Promise.resolve(null))
        .catch(() => null)
        .then((id) => {
          setIdentity(id);
          setReady(true);
        });
    void sync();
    return client.subscribe(() => void sync());
  }, [client]);

  const signIn = useCallback(async () => {
    const id = await client.signIn();
    setIdentity(id);
    return id;
  }, [client]);
  const signOut = useCallback(() => client.signOut(), [client]);

  const value = useMemo(
    () => ({ ready, identity, principal: identity?.getPrincipal() ?? null, signIn, signOut }),
    [ready, identity, signIn, signOut],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthState {
  const auth = useContext(AuthContext);
  if (!auth) throw new Error('useAuth outside AuthProvider');
  return auth;
}

type AaaLookup = { aaa_by_owner(owner: Principal): Promise<Principal | null> };

export async function homeRoute(platform: AaaLookup, me: Principal): Promise<'/spawn' | '/dashboard'> {
  return (await platform.aaa_by_owner(me)) ? '/dashboard' : '/spawn';
}

export type Guard = 'loading' | 'error' | 'ok' | '/signin' | '/spawn';

export function guardDecision(g: {
  ready: boolean;
  signedIn: boolean;
  needAaa: boolean;
  aaa: { isPending: boolean; isError: boolean; data: Principal | null | undefined };
}): Guard {
  if (!g.ready) return 'loading';
  if (!g.signedIn) return '/signin';
  if (!g.needAaa) return 'ok';
  if (g.aaa.isError) return 'error';
  if (g.aaa.isPending) return 'loading';
  return g.aaa.data ? 'ok' : '/spawn';
}

export function useMyAaa() {
  const { identity, principal } = useAuth();
  return useQuery({
    queryKey: ['aaa_by_owner', principal?.toText()],
    queryFn: () => platformActor(identity!).aaa_by_owner(principal!),
    enabled: !!principal,
  });
}

type AdminList = { __kind__: 'Ok'; Ok: Principal[] } | { __kind__: 'Err'; Err: unknown };

export const listsMe = (r: PromiseSettledResult<AdminList>, me: string) =>
  r.status === 'fulfilled' && r.value.__kind__ === 'Ok' && r.value.Ok.some((p) => p.toText() === me);

export function useIsAdmin() {
  const { identity, principal } = useAuth();
  return useQuery({
    queryKey: ['is_admin', principal?.toText()],
    queryFn: async () => {
      const me = principal!.toText();
      const lists = await Promise.allSettled([
        platformActor(identity!).admin_list_admins(),
        paymentsActor(identity!).admin_list_admins(),
      ]);
      return lists.some((r) => listsMe(r, me));
    },
    enabled: !!principal,
  });
}
