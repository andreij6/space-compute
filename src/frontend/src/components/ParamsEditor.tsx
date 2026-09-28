import { useState } from 'react';
import { ConfirmAction } from './ConfirmAction';
import { parseRecordEdits, recordDiff, textFromRecord } from '../lib/admin';

export interface ParamsEditorProps<T extends object> {
  base: T;
  onSave: (params: T) => void;
  saving: boolean;
  error?: string;
}

export function ParamsEditor<T extends object>({ base, onSave, saving, error }: ParamsEditorProps<T>) {
  const [edited, setEdited] = useState(() => textFromRecord(base));
  const baseText = textFromRecord(base);
  const diff = recordDiff(baseText, edited);
  const parsed = parseRecordEdits(base, edited);
  return (
    <div>
      {Object.keys(edited).map((key) => (
        <p key={key}>
          <label>
            {key}
            <input value={edited[key]} onChange={(e) => setEdited({ ...edited, [key]: e.target.value })} />
          </label>
        </p>
      ))}
      {diff.length > 0 && (
        <ul aria-label="Params diff preview">
          {diff.map(([key, before, after]) => (
            <li key={key}>
              {key}: {before} → {after}
            </li>
          ))}
        </ul>
      )}
      {!parsed.ok && (
        <ul role="alert" aria-label="Params errors">
          {parsed.errors.map((e) => (
            <li key={e}>{e}</li>
          ))}
        </ul>
      )}
      <ConfirmAction
        label="save params"
        phrase="save params"
        disabled={saving || diff.length === 0 || !parsed.ok}
        onConfirm={() => parsed.ok && onSave(parsed.value)}
      />
      {error && <p role="alert">{error}</p>}
    </div>
  );
}
