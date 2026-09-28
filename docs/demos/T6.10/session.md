# T6.10 demo: states, accessibility, Playwright + axe + Lighthouse smoke

Date 2026-09-28. Network: local (`-e local`), deployed via `just deploy-local`.

## Lighthouse accessibility (05 §5 #5)

Run via `scripts/lighthouse-a11y.sh` (`playwright-lighthouse` against the deployed local frontend). Reports in this directory (`landing.html`/`.json`, `discovery-detail.html`/`.json`, `dashboard.html`/`.json`).

| Route | Accessibility score |
|---|---|
| `/` (Landing) | 98 |
| `/d/SC-0000-000000` (Discovery detail) | 98 |
| `/dashboard` (signed-in owner, freshly spawned AAA) | 98 |

All three ≥ 90. Two real findings surfaced by the axe Playwright smoke suite were fixed before this run:
- `--text-dim` (`#626e85` on `#0e1224`/`#161b36`/`#070913`) failed WCAG AA contrast (3.3–3.9:1); lightened to `#7b8aa6` (4.5–5.7:1 against all three surface backgrounds) in `src/index.css`. This is the color used by the footer and several muted captions across every page, so the single token fix clears the violation everywhere.
- The AAA wasm upload `<input type="file">` on `/admin/releases` had no associated label (critical axe finding); wrapped it in a `<label>`.

Also added: a global `:focus-visible` outline and a `.sr-only` utility class in `src/index.css` for visible-focus and screen-reader-only text (used by the new invites table caption).

### Discovery detail: no Confirmed discovery available

`/d/SC-0000-000000` is a not-found id (matches the existing `discovery.spec.ts` fixture) — no `Confirmed` discovery exists on the shared local network at the time of this run (`admin_list_discoveries` with `status = opt Confirmed` returns empty). Reaching `Confirmed` requires a discoverer AAA plus 3 independent reviewer AAAs that have each cleared the tier-2 reputation gate (`xp >= 50`, `rep_bp >= 6000`, `gold_tasks >= 20`, `progression.rs::calculate_tier`), which in turn requires dozens of correctly-answered honeypot classifications per reviewer against hidden ground truth. That is a multi-session curation exercise (see T3.7's demo notes on the classify/flag/review loop), not a fixture buildable within this task's scope, so per the task brief's own fallback ("or use a honeypot-free confirmed one if the seed has none — else document") this is documented here rather than attempted.

What *was* exercised locally: spawned 4 real AAAs (`T610-Discoverer` + 3 reviewers) via `payments.spawn_aaa{Deposit}` and flagged a real discovery (`SC-2026-000182`, category `ring`, subject 10005239) through `aaa.submit_classification`, confirming the classify → flag pipeline still works end-to-end and produces a real `public_id` while `UnderReview`. It stayed below the reviewer tier-2 gate so review submission (`get_review_assignment`) correctly rejected the reviewer AAAs with `NotEligible("tier")`. The Discovery Detail page's `UnderReview` branch (no image/citation block, "Citation in progress: n of m reviews") is exercised instead by the axe/Lighthouse runs against the not-found id's page shell (same nav/layout/heading structure) plus the existing `discovery.spec.ts` and the `DiscoveryDetailPage` component's own logic for the `resolved` branch is exercised by `src/citation.test.ts` and `src/lib` unit tests.

## Playwright + axe smoke (`tests/e2e/a11y-smoke.spec.ts`)

13 tests, all passing:
- 11 signed-out public routes (`/`, `/discoveries`, `/d/SC-0000-000000`, `/aaa/2vxsx-fae`, `/leaderboard`, `/signin`, `/about`, `/practice`, `/terms`, `/privacy`, `/credits`)
- 1 signed-in owner flow: `/spawn` → sign in + spawn via Deposit → `/dashboard`, `/connect`, `/records`, `/fuel`
- 1 admin flow: sign in, `admin_add_admin` on both platform and payments, all 11 `/admin/*` routes

Each assertion: zero browser console errors and zero axe `serious`/`critical` violations (`@axe-core/playwright`).

A real CSP console error was found and fixed along the way: `<link rel="preconnect">` to `fonts.googleapis.com`/`fonts.gstatic.com` in `index.html` was blocked by `connect-src` (Chromium enforces `connect-src` for preconnect and, intermittently, for the stylesheet fetch itself). Removed the now-redundant preconnect hints and added `https://fonts.googleapis.com https://fonts.gstatic.com` to `connect-src` in `public/_headers` (both origins were already trusted under `style-src`/`font-src`).

## Bundle size (05 §4)

`scripts/check-bundle-size.sh` sums the gzip size of every script referenced by `dist/index.html` (the entry chunk plus `modulepreload` hints — i.e. what loads before any route-level `React.lazy` chunk). Current: **191,831 B gz** against a 358,400 B (350 KiB) limit. Route-level code splitting was already in place (`src/App.tsx` uses `import.meta.glob` + `React.lazy` per page).

## jsx-a11y + dangerouslySetInnerHTML ban

`eslint-plugin-jsx-a11y` added (`eslint.config.js`, `recommended` flat config) plus a `no-restricted-syntax` rule banning the `dangerouslySetInnerHTML` JSX attribute. One pre-existing finding: `src/components/MultiCurrencyPayment.tsx` was dead code (not imported anywhere) with several a11y violations (non-interactive `div` with a click handler, unlabeled form control) — deleted rather than patched. `scripts/verify-local.sh` also greps `frontend/src` for `dangerouslySetInnerHTML` as a backstop gate.
