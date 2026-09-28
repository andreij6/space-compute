const FIELDS: { field: string; programs: string }[] = [
  { field: 'CEERS', programs: 'JWST program 1345 (Finkelstein et al.)' },
  { field: 'JADES (GOODS-South)', programs: 'JWST programs 1180, 1210, 3215 — JADES / JADES Origins Field' },
  { field: 'JADES (GOODS-North)', programs: 'JWST program 1181 — JADES' },
  { field: 'PRIMER (UDS)', programs: 'JWST program 1837 — PRIMER' },
  { field: 'PRIMER (COSMOS)', programs: 'JWST program 1837 — PRIMER' },
  { field: 'Abell 2744', programs: 'JWST programs 2561 (UNCOVER), 3516 (MegaScience)' },
];

export const CreditsPage: React.FC = () => {
  return (
    <div>
      <h1>Credits & acknowledgements</h1>

      <section aria-label="JWST acknowledgment">
        <h2>Observatory acknowledgment</h2>
        <p>
          This work is based on observations made with the NASA/ESA/CSA James Webb Space Telescope, obtained
          from the Mikulski Archive for Space Telescopes at the Space Telescope Science Institute, which is
          operated by the Association of Universities for Research in Astronomy, Inc., under NASA contract NAS
          5-03127.
        </p>
      </section>

      <section aria-label="Data reduction">
        <h2>Data reduction and photometry</h2>
        <ul>
          <li>Dawn JWST Archive v7 (Valentino et al. 2023; grizli, Brammer) — mosaics, cutouts, and photometric catalogs.</li>
          <li>EAZY photometric redshifts (Brammer et al. 2008).</li>
        </ul>
      </section>

      <section aria-label="Survey programs">
        <h2>Survey programs by field</h2>
        <table>
          <thead>
            <tr>
              <th>Field</th>
              <th>Programs</th>
            </tr>
          </thead>
          <tbody>
            {FIELDS.map((f) => (
              <tr key={f.field}>
                <td>{f.field}</td>
                <td>{f.programs}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <p>Every discovery page carries the program credits and this acknowledgment for the field it came from.</p>
    </div>
  );
};
