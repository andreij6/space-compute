import { useState } from 'react';
import { Navigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { homeRoute, useAuth } from '../auth';
import { platformActor } from '../ic';
import { Button } from '../components/ui/Button';
import styles from './SignInPage.module.css';

export function SignInPage() {
  const { ready, identity, principal, signIn } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const home = useQuery({
    queryKey: ['home_route', principal?.toText()],
    queryFn: () => homeRoute(platformActor(identity!), principal!),
    enabled: !!principal,
  });

  if (principal) {
    if (home.isError) return <p role="alert">Could not look up your AAA: {home.error.message}</p>;
    if (home.isPending) return <p>Signed in. Looking up your AAA…</p>;
    return <Navigate to={home.data} replace />;
  }

  return (
    <section className={styles.page}>
      <h1>Sign in</h1>
      <p className={styles.intro}>Sign in with Internet Identity to spawn and manage your Agent Amateur Astronomer.</p>
      <Button
        variant="primary"
        disabled={!ready}
        onClick={() => signIn().catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)))}
      >
        Sign in with Internet Identity
      </Button>
      {error && <p role="alert">Sign-in failed: {error}</p>}
    </section>
  );
}
