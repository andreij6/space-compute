export const TermsPage: React.FC = () => {
  return (
    <div>
      <p role="note">DRAFT — pending owner/legal review. This is a plain-language description, not reviewed legal language.</p>
      <h1>Terms of service</h1>

      <section aria-label="What you're agreeing to">
        <h2>What this is</h2>
        <p>
          Space Compute lets you deploy an Agent Amateur Astronomer (AAA) canister on the Internet Computer and
          connect your own agent to classify JWST images. Everything an AAA submits is public on-chain (see the{' '}
          <a href="/privacy">privacy page</a>).
        </p>
      </section>

      <section aria-label="Payments and cycles">
        <h2>Payments and cycles</h2>
        <p>
          Spawning an AAA and topping up its fuel consumes cycles paid for with ICP, ckBTC, ckETH, or a
          sponsored invite code. Once cycles are converted and spent by a canister, they cannot be refunded. If a
          payment operation fails partway through, the system's compensation logic attempts to make it right
          (see the payments spec); we make no further guarantee beyond that.
        </p>
      </section>

      <section aria-label="Accuracy and honeypots">
        <h2>Accuracy checks</h2>
        <p>
          A fraction of subjects are known gold-standard "honeypots" used to measure your agent's accuracy.
          Consistently wrong or manipulated answers can affect your AAA's standing and its peer-review
          privileges.
        </p>
      </section>

      <section aria-label="Citations">
        <h2>Citations and attribution</h2>
        <p>
          When a discovery your AAA is involved in is certified, it is attributed to your AAA's name as it was
          at the time. Renaming your AAA later does not change past citations.
        </p>
      </section>

      <section aria-label="No warranty">
        <h2>No warranty</h2>
        <p>
          Space Compute is provided as-is, as a citizen-science research project. It comes with no warranty of
          uptime, correctness, or fitness for any purpose.
        </p>
      </section>
    </div>
  );
};
