import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { RootLayout } from './layouts/RootLayout';
import { LandingPage } from './pages/LandingPage';
import { MuseumPage } from './pages/MuseumPage';
import { DiscoveryDetailPage } from './pages/DiscoveryDetailPage';
import { AAAPublicProfilePage } from './pages/AAAPublicProfilePage';
import { LeaderboardPage } from './pages/LeaderboardPage';
import { SpawnWizardPage } from './pages/SpawnWizardPage';
import { OwnerDashboardPage } from './pages/OwnerDashboardPage';
import { ConnectAgentPage } from './pages/ConnectAgentPage';
import { ScientificRecordsPage } from './pages/ScientificRecordsPage';
import { FuelBillingPage } from './pages/FuelBillingPage';
import { AboutPage } from './pages/AboutPage';
import { PracticePage } from './pages/PracticePage';
import { TermsPage } from './pages/TermsPage';
import { PrivacyPage } from './pages/PrivacyPage';
import { CreditsPage } from './pages/CreditsPage';
import { AdminOverviewPage } from './pages/AdminOverviewPage';
import { AdminAAAsPage } from './pages/AdminAAAsPage';
import { AdminDiscoveriesPage } from './pages/AdminDiscoveriesPage';
import { AdminCatalogPage } from './pages/AdminCatalogPage';
import { AdminPaymentsPage } from './pages/AdminPaymentsPage';
import { AdminReleasesPage } from './pages/AdminReleasesPage';
import { AdminSettingsPage } from './pages/AdminSettingsPage';
import { AdminInvitesPage } from './pages/AdminInvitesPage';
import { AdminTreasuryPage } from './pages/AdminTreasuryPage';
import { AdminModerationPage } from './pages/AdminModerationPage';
import { AdminAuditPage } from './pages/AdminAuditPage';
import { NotFoundPage } from './pages/NotFoundPage';

export const App: React.FC = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<RootLayout />}>
          <Route index element={<LandingPage />} />
          <Route path="discoveries" element={<MuseumPage />} />
          <Route path="d/:publicId" element={<DiscoveryDetailPage />} />
          <Route path="aaa/:id" element={<AAAPublicProfilePage />} />
          <Route path="leaderboard" element={<LeaderboardPage />} />

          <Route path="spawn" element={<SpawnWizardPage />} />
          <Route path="dashboard" element={<OwnerDashboardPage />} />
          <Route path="connect" element={<ConnectAgentPage />} />
          <Route path="records" element={<ScientificRecordsPage />} />
          <Route path="fuel" element={<FuelBillingPage />} />

          <Route path="about" element={<AboutPage />} />
          <Route path="practice" element={<PracticePage />} />
          <Route path="terms" element={<TermsPage />} />
          <Route path="privacy" element={<PrivacyPage />} />
          <Route path="credits" element={<CreditsPage />} />

          <Route path="admin" element={<AdminOverviewPage />} />
          <Route path="admin/aaas" element={<AdminAAAsPage />} />
          <Route path="admin/discoveries" element={<AdminDiscoveriesPage />} />
          <Route path="admin/data" element={<AdminCatalogPage />} />
          <Route path="admin/payments" element={<AdminPaymentsPage />} />
          <Route path="admin/releases" element={<AdminReleasesPage />} />
          <Route path="admin/settings" element={<AdminSettingsPage />} />
          <Route path="admin/invites" element={<AdminInvitesPage />} />
          <Route path="admin/treasury" element={<AdminTreasuryPage />} />
          <Route path="admin/moderation" element={<AdminModerationPage />} />
          <Route path="admin/audit" element={<AdminAuditPage />} />

          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
};
