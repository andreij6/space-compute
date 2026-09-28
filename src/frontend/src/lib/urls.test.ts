import { describe, expect, it } from 'vitest';
import { safeHref } from './urls';

describe('safeHref (dossier_url and other canister-supplied links)', () => {
  it('allows https links', () => {
    expect(safeHref('https://data.example.com/v1/subjects/7/dossier.json')).toBe(
      'https://data.example.com/v1/subjects/7/dossier.json',
    );
  });
  it('rejects javascript:, data:, plain http and relative links', () => {
    expect(safeHref('javascript:alert(1)')).toBeNull();
    expect(safeHref('JaVaScRiPt:alert(1)')).toBeNull();
    expect(safeHref('data:text/html,<script>1</script>')).toBeNull();
    expect(safeHref('http://data.example.com/x')).toBeNull();
    expect(safeHref('/v1/subjects/7/dossier.json')).toBeNull();
    expect(safeHref('')).toBeNull();
  });
  it('allows the local data bucket only when the build configured it', () => {
    expect(safeHref('http://127.0.0.1:8765/v1/subjects/7/dossier.json')).toBeNull();
    expect(safeHref('http://127.0.0.1:8765/v1/subjects/7/dossier.json', 'http://127.0.0.1:8765')).toBe(
      'http://127.0.0.1:8765/v1/subjects/7/dossier.json',
    );
    expect(safeHref('http://127.0.0.1:9999/x', 'http://127.0.0.1:8765')).toBeNull();
  });
});
