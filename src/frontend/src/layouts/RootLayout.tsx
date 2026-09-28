import { Outlet } from 'react-router-dom';
import { PageShell } from '../components/PageShell';

export function RootLayout() {
  return (
    <PageShell>
      <Outlet />
    </PageShell>
  );
}
