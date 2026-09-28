import type { ReactNode } from 'react';
import { Navbar } from './Navbar';
import { Footer } from './Footer';
import { MobileBottomBar } from './MobileBottomBar';
import styles from './PageShell.module.css';

export function PageShell({ children }: { children: ReactNode }) {
  return (
    <div className={styles.shell}>
      <a href="#main" className={styles.skip}>
        Skip to content
      </a>
      <Navbar />
      <main id="main" className={styles.main}>
        {children}
      </main>
      <Footer />
      <MobileBottomBar />
    </div>
  );
}
