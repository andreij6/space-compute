import { lazy, Suspense, type ComponentType } from 'react';
import { BrowserRouter, Navigate, Outlet, Route, Routes, useLocation } from 'react-router-dom';
import { RootLayout } from './layouts/RootLayout';
import { guardDecision, useAuth, useIsAdmin, useMyAaa } from './auth';

const lazyPages = Object.fromEntries(
  Object.entries(import.meta.glob<Record<string, ComponentType>>('./pages/*Page.tsx')).map(([path, load]) => {
    const name = path.slice('./pages/'.length, -'.tsx'.length);
    return [name, lazy(() => load().then((m) => ({ default: m[name] })))];
  }),
);
const page = (name: string) => {
  const Page = lazyPages[name];
  return (
    <Suspense fallback={<p>Loading…</p>}>
      <Page />
    </Suspense>
  );
};

function RequireAuth({ needAaa }: { needAaa: boolean }) {
  const { ready, principal } = useAuth();
  const aaa = useMyAaa();
  const location = useLocation();
  const decision = guardDecision({ ready, signedIn: !!principal, needAaa, aaa });
  if (decision === 'loading') return <p>Loading…</p>;
  if (decision === 'error') return <p role="alert">Could not load your AAA. Try again later.</p>;
  if (decision === 'ok') return <Outlet />;
  return <Navigate to={decision} replace state={{ from: location.pathname }} />;
}

function RequireAdmin() {
  const { ready, principal } = useAuth();
  const admin = useIsAdmin();
  if (!ready || (principal && admin.isPending)) return <p>Loading…</p>;
  return admin.data ? <Outlet /> : page('NotFoundPage');
}

const adminPages: [string, string][] = [
  ['aaas', 'AdminAAAsPage'],
  ['discoveries', 'AdminDiscoveriesPage'],
  ['data', 'AdminCatalogPage'],
  ['payments', 'AdminPaymentsPage'],
  ['releases', 'AdminReleasesPage'],
  ['settings', 'AdminSettingsPage'],
  ['invites', 'AdminInvitesPage'],
  ['treasury', 'AdminTreasuryPage'],
  ['moderation', 'AdminModerationPage'],
  ['audit', 'AdminAuditPage'],
];

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<RootLayout />}>
          <Route index element={page('LandingPage')} />
          <Route path="discoveries" element={page('MuseumPage')} />
          <Route path="d/:publicId" element={page('DiscoveryDetailPage')} />
          <Route path="aaa/:id" element={page('AAAPublicProfilePage')} />
          <Route path="leaderboard" element={page('LeaderboardPage')} />
          <Route path="signin" element={page('SignInPage')} />

          <Route element={<RequireAuth needAaa={false} />}>
            <Route path="spawn" element={page('SpawnWizardPage')} />
          </Route>
          <Route element={<RequireAuth needAaa />}>
            <Route path="dashboard" element={page('OwnerDashboardPage')} />
            <Route path="connect" element={page('ConnectAgentPage')} />
            <Route path="records" element={page('ScientificRecordsPage')} />
            <Route path="fuel" element={page('FuelBillingPage')} />
          </Route>

          <Route path="about" element={page('AboutPage')} />
          <Route path="practice" element={page('PracticePage')} />
          <Route path="terms" element={page('TermsPage')} />
          <Route path="privacy" element={page('PrivacyPage')} />
          <Route path="credits" element={page('CreditsPage')} />

          <Route path="admin" element={<RequireAdmin />}>
            <Route index element={page('AdminOverviewPage')} />
            {adminPages.map(([path, name]) => (
              <Route key={path} path={path} element={page(name)} />
            ))}
          </Route>

          <Route path="*" element={page('NotFoundPage')} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
