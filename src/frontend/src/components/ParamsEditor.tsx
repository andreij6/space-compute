import { useState } from 'react';
import { ConfirmAction } from './ConfirmAction';
import { parseRecordEdits, recordDiff, textFromRecord } from '../lib/admin';
import confirmStyles from './ConfirmAction.module.css';
import shared from '../styles/adminShared.module.css';

export interface ParamsEditorProps<T extends object> {
  base: T;
  onSave: (params: T) => void;
  saving: boolean;
  error?: string;
  floatKeys?: readonly string[];
}

export function ParamsEditor<T extends object>({ base, onSave, saving, error, floatKeys }: ParamsEditorProps<T>) {
  const [edited, setEdited] = useState(() => textFromRecord(base));
  const baseText = textFromRecord(base);
  const diff = recordDiff(baseText, edited);
  const parsed = parseRecordEdits(base, edited, floatKeys);
  return (
    <div>
      <div className={shared.formRow}>
        {Object.keys(edited).map((key) => (
          <label key={key} className={confirmStyles.field}>
            {key}
            <input className={confirmStyles.input} value={edited[key]} onChange={(e) => setEdited({ ...edited, [key]: e.target.value })} />
          </label>
        ))}
      </div>
      {diff.length > 0 && (
        <ul aria-label="Params diff preview" className={shared.detailList}>
          {diff.map(([key, before, after]) => (
            <li key={key}>
              {key}: {before} → {after}
            </li>
          ))}
        </ul>
      )}
      {!parsed.ok && (
        <div role="alert" className={shared.alert}>
          <ul aria-label="Params errors">
            {parsed.errors.map((e) => (
              <li key={e}>{e}</li>
            ))}
          </ul>
        </div>
      )}
      <ConfirmAction
        label="save params"
        phrase="save params"
        disabled={saving || diff.length === 0 || !parsed.ok}
        onConfirm={() => parsed.ok && onSave(parsed.value)}
      />
      {error && <p role="alert" className={shared.alert}>{error}</p>}
    </div>
  );
}
