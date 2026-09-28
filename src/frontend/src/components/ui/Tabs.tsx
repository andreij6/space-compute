import { useId, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import styles from './Tabs.module.css';

export interface TabItem {
  id: string;
  label: ReactNode;
  content: ReactNode;
  disabled?: boolean;
}

export function nextEnabledIndex(items: { disabled?: boolean }[], from: number, step: 1 | -1): number {
  for (let k = 1; k <= items.length; k++) {
    const i = (from + step * k + items.length * k) % items.length;
    if (!items[i].disabled) return i;
  }
  return from;
}

export function Tabs({ label, items, defaultId }: { label: string; items: TabItem[]; defaultId?: string }) {
  const [active, setActive] = useState(defaultId ?? items.find((t) => !t.disabled)?.id);
  const base = useId();
  const refs = useRef<(HTMLButtonElement | null)[]>([]);

  const onKeyDown = (e: KeyboardEvent, i: number) => {
    const step = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
    if (!step) return;
    e.preventDefault();
    const n = nextEnabledIndex(items, i, step);
    setActive(items[n].id);
    refs.current[n]?.focus();
  };

  return (
    <div className={styles.tabs}>
      <div role="tablist" aria-label={label} className={styles.list}>
        {items.map((t, i) => (
          <button
            key={t.id}
            ref={(el) => {
              refs.current[i] = el;
            }}
            id={`${base}-tab-${t.id}`}
            type="button"
            role="tab"
            aria-selected={t.id === active}
            aria-controls={`${base}-panel-${t.id}`}
            tabIndex={t.id === active ? 0 : -1}
            disabled={t.disabled}
            className={styles.tab}
            onClick={() => setActive(t.id)}
            onKeyDown={(e) => onKeyDown(e, i)}
          >
            {t.label}
          </button>
        ))}
      </div>
      {items
        .filter((t) => t.id === active)
        .map((t) => (
          <div key={t.id} id={`${base}-panel-${t.id}`} role="tabpanel" aria-labelledby={`${base}-tab-${t.id}`} tabIndex={0} className={styles.panel}>
            {t.content}
          </div>
        ))}
    </div>
  );
}
