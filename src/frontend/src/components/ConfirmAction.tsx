import { useState } from 'react';
import { typedConfirmReady } from '../lib/admin';
import { Button } from './ui/Button';
import styles from './ConfirmAction.module.css';

export interface ConfirmActionProps {
  label: string;
  phrase: string;
  onConfirm: () => void;
  disabled?: boolean;
}

export function ConfirmAction({ label, phrase, onConfirm, disabled }: ConfirmActionProps) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState('');
  const close = () => {
    setOpen(false);
    setText('');
  };

  if (!open) {
    return (
      <Button variant="danger" size="sm" disabled={disabled} onClick={() => setOpen(true)}>
        {label}
      </Button>
    );
  }

  return (
    <span className={styles.confirm}>
      <label className={styles.field}>
        Type &quot;{phrase}&quot; to confirm
        <input className={styles.input} value={text} onChange={(e) => setText(e.target.value)} autoComplete="off" />
      </label>
      <Button
        variant="danger"
        size="sm"
        disabled={disabled || !typedConfirmReady(text, phrase)}
        onClick={() => {
          onConfirm();
          close();
        }}
      >
        Confirm {label}
      </Button>
      <Button variant="ghost" size="sm" onClick={close}>
        Cancel
      </Button>
    </span>
  );
}
