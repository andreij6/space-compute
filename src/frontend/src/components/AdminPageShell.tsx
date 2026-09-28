import type { ReactNode } from 'react';
import { AdminNav } from './AdminNav';
import styles from './AdminPageShell.module.css';

export function AdminPageShell({ title, children }: { title: ReactNode; children: ReactNode }) {
  return (
    <div className={styles.layout}>
      <AdminNav />
      <div className={styles.content}>
        <h1>{title}</h1>
        {children}
      </div>
    </div>
  );
}
