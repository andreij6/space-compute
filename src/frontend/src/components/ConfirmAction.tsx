import { useState } from 'react';
import { typedConfirmReady } from '../lib/admin';

export interface ConfirmActionProps {
  label: string;
  phrase: string;
  onConfirm: () => void;
  disabled?: boolean;
}

export function ConfirmAction({ label, phrase, onConfirm, disabled }: ConfirmActionProps) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState('');

  if (!open) {
    return (
      <button type="button" disabled={disabled} onClick={() => setOpen(true)}>
        {label}
      </button>
    );
  }

  return (
    <span>
      <label>
        Type &quot;{phrase}&quot; to confirm
        <input value={text} onChange={(e) => setText(e.target.value)} autoComplete="off" />
      </label>
      <button
        type="button"
        disabled={disabled || !typedConfirmReady(text, phrase)}
        onClick={() => {
          onConfirm();
          setOpen(false);
          setText('');
        }}
      >
        Confirm {label}
      </button>
      <button
        type="button"
        onClick={() => {
          setOpen(false);
          setText('');
        }}
      >
        Cancel
      </button>
    </span>
  );
}
