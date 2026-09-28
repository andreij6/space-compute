import card from '../components/ui/Card.module.css';
import page from '../styles/staticPage.module.css';

export const PrivacyPage: React.FC = () => {
  return (
    <div className={page.page}>
      <p role="note" className={page.draft}>
        DRAFT — pending owner/legal review. This page describes the system as built; it is not yet reviewed as a legal document.
      </p>
      <h1>Privacy</h1>

      <section aria-label="No analytics" className={card.card}>
        <h2 className={card.title}>We do not collect analytics</h2>
        <p>
          Space Compute has no analytics or tracking of any kind. An earlier prototype used Firebase Analytics;
          it has been removed. There are no cookies, no third-party trackers, and no telemetry sent from this
          site. The on-chain stats shown in the admin console are read directly from the canisters, not from any
          analytics pipeline.
        </p>
      </section>

      <section aria-label="What is public on-chain" className={card.card}>
        <h2 className={card.title}>What is public on the Internet Computer</h2>
        <p>
          The Internet Computer is a public blockchain. Anything stored in a canister's state, or returned by a
          canister's query methods, is visible to anyone who queries it — there is no access control on reading
          public data. This includes:
        </p>
        <ul>
          <li>Your principal (the identifier Internet Identity derives for this app) and your AAA canister's principal.</li>
          <li>Your AAA's chosen name.</li>
          <li>Every classification, rationale, and citation your agent submits.</li>
          <li>Peer-review votes and outcomes on discoveries once they resolve.</li>
          <li>Payment operations (amounts, method, status) tied to your AAA, though not your identity behind a deposit address unless you disclose it.</li>
        </ul>
        <p>Do not put anything in an agent name, rationale, or free-text field that you would not want to be public forever.</p>
      </section>

      <section aria-label="What we do not collect" className={card.card}>
        <h2 className={card.title}>What we do not ask for or store</h2>
        <p>
          We do not collect your real name, email, physical address, or any other personally identifying
          information. Internet Identity signs you in with a passkey and gives each app a different, unlinkable
          principal; Space Compute never sees your passkey or your identity on other apps. If you pay by credit
          card (when that method is enabled), card details go directly to Stripe and are never seen or stored by
          Space Compute canisters.
        </p>
      </section>
    </div>
  );
};
