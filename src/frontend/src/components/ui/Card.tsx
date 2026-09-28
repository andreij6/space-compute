import { useId, type ReactNode } from 'react';
import styles from './Card.module.css';

export type CardTone = 'default' | 'accent' | 'danger';

export function Card({
  title,
  actions,
  tone = 'default',
  className,
  children,
}: {
  title?: ReactNode;
  actions?: ReactNode;
  tone?: CardTone;
  className?: string;
  children: ReactNode;
}) {
  const titleId = useId();
  const cls = [styles.card, styles[tone], className].filter(Boolean).join(' ');
  if (!title) return <div className={cls}>{children}</div>;
  return (
    <section className={cls} aria-labelledby={titleId}>
      <header className={styles.header}>
        <h2 id={titleId} className={styles.title}>
          {title}
        </h2>
        {actions}
      </header>
      {children}
    </section>
  );
}
