# 05 — Frontend (Vite + React + TS on the `@dfinity/static-site` recipe)

The frontend is **read-only for the science itself**: no screen classifies or reviews. The screen list comes from `../DESIGN_BRIEF.md`.

**Functional first, design last (owner, 2026-09-27).** Phase 6 builds every screen **unstyled but complete**: semantic HTML, one small utility stylesheet (spacing, grid, a readable type scale), real data, all states, and accessibility. No mockup fidelity. The look is applied in **Phase 9 (Design & polish)**, near the end: import the mockups (FE-0 moves there), add tokens and components, restyle the screens, then a responsive pass. Playwright selectors use roles and labels, never CSS classes, so restyling can't break the tests.

**Visual source of truth:** the Claude Design project `c2cf5a8d-5b7d-4280-9bab-0b6a76cb4619`, files `Space Compute Mockups.dc.html` (main screens), `Discovery Museum.dc.html` (discovery gallery/feed), and `support.js`. **Task FE-0 imports these** (it needs `/design-login`) and records any mismatch with this spec in `docs/specs/05a-design-reconciliation.md` before Phase 9 starts. This spec defines behaviour and data; the mockups define look and layout.

## 1. Stack & setup
- React 19, TypeScript strict, React Router, TanStack Query (cache + polling), CSS approach per the mockups (default: CSS modules plus design tokens from the mockups).
- `@icp-sdk/core@^6`, `@icp-sdk/auth@^10` (Internet Identity, mainnet `id.ai`), `@icp-sdk/signer@^6` (OISY; popup transport), `@icp-sdk/canisters@^4` (ICP ledger `icrc2_approve`), `@icp-sdk/bindgen` (Vite plugin; bindings from the committed `.did` files).
- The root key and canister IDs come from the `ic_env` cookie (`safeGetCanisterEnv()`). **Never** `fetchRootKey()`.
- Hosting: `_redirects` for SPA routing; `_headers` sets a strict CSP: `img-src 'self' https://data.<domain> data:`; `connect-src` for the IC API and `id.ai`; `frame-ancestors 'none'`.
- II: the derivation origin is pinned to the production domain; `/.well-known/ii-app-metadata` provides the name and logo (see the `internet-identity` skill).

## 2. Routes & data

| # | Route | Screen | Auth | Data (canister.method) | Refresh |
|---|---|---|---|---|---|
| 1b | `/about`, `/terms`, `/privacy`, `/credits` | Static pages: how it works, Terms of Service, privacy (analytics consent, what is public on-chain), JWST/DJA/survey credits, public treasury runway (`treasury.status`) | – | treasury.status | 60 s |
| 1c | `/practice` | Practice set: a download link and instructions for self-evaluating an agent offline | – | static | – |
| 1 | `/` | Landing | – | platform.get_stats, list_discoveries{status: Confirmed, limit 6} | 60 s |
| 2 | `/discoveries` | Feed ("Discovery Museum") | – | platform.list_discoveries: public = resolved only (filters category, Confirmed/Rejected; infinite scroll) | 60 s |
| 3 | `/d/:publicId` | Discovery detail + **citation block** | – (the owner also sees their own under-review items) | platform.get_discovery, get_citation (verified; show a "verified" mark) | 30 s while under review |
| 4 | `/aaa/:id` | AAA public profile | – | platform.get_aaa_public, list_aaa_credits, get_protocol (badge/tier labels are static) | 60 s |
| 5 | `/leaderboard` | Leaderboard | – | platform.get_leaderboard | 60 s |
| 6 | `/spawn` | Spawn flow | II | payments.get_quote_spawn, get_deposit_account, spawn_aaa, get_op (poll) | 3 s while op pending |
| 7 | `/dashboard` | Dashboard | II + AAA | aaa.status (fallback platform.get_aaa_public), platform.list_aaa_activity, list_aaa_credits, payments.get_mandate | 30 s |
| 8 | `/connect` | Connect your agent | II + AAA | aaa.status (operators, last_used_at), aaa.add_operator/remove_operator | 10 s |
| 9 | `/records` | Activity & records | II + AAA | aaa.list_records (fallback platform.list_aaa_activity), aaa.get_record, platform.get_protocol(version) | manual |
| 10 | `/fuel` | Fuel & billing | II + AAA | aaa.status, payments.get_quote_topup, top_up, get_deposit_account, set_mandate, get_mandate, list_ops_for_aaa, get_btc_deposit_address, notify_btc_deposit, get_eth_deposit_info, notify_eth_deposit, get_treasury_status (pack availability); relay `/checkout`, `/portal` | 5 s while op pending; BTC notify every 5 min |

- Sign-in routing: after II sign-in, `platform.aaa_by_owner(me)` decides the route. No AAA → `/spawn`; an AAA → `/dashboard`.

## 2b. Admin console (`/admin/*`)
The console is visible only when `msg_caller()` is in the platform or payments admin list. The UI gate is cosmetic; every canister method re-checks. Admins sign in with Internet Identity; production admins should use a hardware-key passkey.

| Route | Screen | Data | Actions |
|---|---|---|---|
| `/admin` | Overview | platform.admin_overview, payments.admin_overview | Pause/unpause tasks, reviews, spawns and non-ICP payments (typed confirmation) |
| `/admin/aaas` | AAAs: search/filter, detail drawer (owner, status, provenance, operators, progress, events) | admin_list_aaas, admin_get_aaa | Suspend/unsuspend, retry install, block card packs |
| `/admin/discoveries` | Discoveries incl. under-review, starving, corroborations; honeypot accuracy per reviewer | admin_list_discoveries, admin_honeypot_stats | Add honeypots (JSON upload). There is deliberately no edit or delete for citations. |
| `/admin/data` | Subjects & protocols: counts by field/gold/active; protocol versions; data-bucket manifest status | admin_list_subjects, admin_list_protocols | Activate/deactivate subjects, add a protocol, set the current protocol |
| `/admin/payments` | Treasury & payments: balances, caps, rates, failed/stuck ops, Stripe reconciliation status | payments.admin_overview, admin_list_ops | Resume op, block/unblock card, set params |
| `/admin/releases` | AAA wasm versions (hash, size, approved, adoption %) | admin_list_wasm | Upload wasm, approve (typed confirmation + shows the sha256) |
| `/admin/settings` | Params (platform + payments) with diff preview; **feature flags** (card/BTC/ETH/sponsored spawn); admins list | admin_get_params, get_features, admin_list_admins | Edit params, toggle flags (typed confirmation), add/remove admin (the last admin is protected) |
| `/admin/invites` | Invite-code batches: minted, used, expired, sponsor budget used | payments.admin_list_invites | Mint a batch (codes shown once, CSV download), revoke unused |
| `/admin/treasury` | Treasury ICP balance, deposit history, per-canister cycles runway, reserve floor, `health()` state, projected months of runway, and the non-ICP intake state (04 §6.2b) | treasury.status, health, history; payments.get_features | Top up now, watch list, force intake open/paused, withdraw (two-admin approval) |
| `/admin/moderation` | AAA names flagged by the blocklist or reports | admin_list_aaas{flagged} | Force-rename an AAA (`admin_rename_aaa`, audit-logged; citations keep the name used at the time) |
| `/admin/audit` | Merged audit log from both canisters | admin_audit_log ×2 | — |

## 3. Screen behaviour notes (what the mockups can't show)
- **Images:** loaded directly from `subject.image_url` (JWST color composite), with zoom and pan (CSS transform, no library needed; a pinch/zoom lib only if the mockups require it). A **"Data" panel** fetched from `dossier_url` shows the field and program, the filters used for the colors, RA/Dec, z_phot ± error (or z_spec), stellar mass, magnification when lensed, the per-filter FITS download links, and the JWST acknowledgment. If the image fails to load, show the "Image unavailable from survey" state.
- **Citation block:**
  - Discoverer first, then reviewers with vote chips and tier insignia; `public_id`; outcome and date; a copy-citation button (the `text` field); a permalink.
  - "Verified" appears only when certificate validation passes.
  - Under-review discoveries are visible only to their owner (spec 02 §7). The owner sees "Citation in progress: n of m reviews" with no reviewer identities or votes until resolution. The landing page shows only the aggregate "N discoveries under review".
- **Agent vs manual:** everything is agent work. The records view shows `agent_label` ("self-reported") when present.
- **Fuel gauge:** based on `days_of_fuel_estimate`, with states Healthy (> 14 d), Low (3–14 d), Critical (< 3 d), Paused (frozen: the `aaa.status` call fails with a frozen reject, so fall back to platform data and show the "paused" banner with a top-up CTA).
- **Payment component (shared by spawn and fuel):** amount → method tab [Wallet (OISY) | Deposit address] → pay → op status timeline (Pending → Pulled → Notified → Done / Failed / Refunded).
  - Wallet: `@icp-sdk/signer` requests `icrc2_approve` to spender `{owner: payments, subaccount: S(purpose, beneficiary)}` (the subaccount comes from a `payments` query helper, never computed client-side), then calls `spawn_aaa`/`top_up` with `payer` = the wallet account.
  - Deposit: shows the account id + QR (a local QR lib such as `qrcode` is allowed), then an "I've sent it" button that calls `spawn_aaa{Deposit}`/`top_up{Deposit}`, with a balance poll as a hint.
  - **Feature flags:** the payment component renders only methods where `payments.get_features()` is true. At launch the **card tab is absent** (not greyed out), and spawn offers ICP, BTC, ETH, or an **invite code** (sponsored spawn, 04 §0b).
  - **Card ($5 packs), deferred:** choose 1–4 packs or the monthly subscription → `POST relay/checkout` → redirect to Stripe Checkout → return to `/fuel?paid=1` → poll `list_ops_for_aaa` until a `FuelPack{Card}` op is `Done`. "Manage subscription" → `relay/portal`.
  - **BTC:** show the address from `get_btc_deposit_address` + QR + "≈ $X at current rate"; poll `notify_btc_deposit`; show the confirmation progress.
  - **ETH:** "Pay with wallet" uses EIP-1193 (`window.ethereum`) to send the ckETH helper-contract deposit with `principal_bytes32` + `subaccount_bytes32` (the calldata is encoded by a ~30-line helper; no web3 library). There is also a fallback showing the manual instructions. Poll `notify_eth_deposit`; show "minting (~20 min)".
  - If `get_treasury_status` shows packs as unavailable (floor or cap reached, or stale rates), the card/BTC/ETH tabs are disabled with the message "Card & crypto fuel temporarily unavailable — ICP still works".
  - The component is a strategy map `{icpWallet, icpDeposit, card, btc, eth}`.
- **Auto top-up:** set the monthly cap and per-top-up amount → OISY approve (cap × 12, 1-year expiry) → `set_mandate`. Show "used this month", "allowance remaining", and the `needs_attention` state.
- **Connect your agent:** step-by-step instructions (copy buttons):
  1. `icp identity new sc-operator-<date>`
  2. `icp identity principal --identity <name>`
  3. paste the principal here → `add_operator`
  4. install the skill (`agent-kit`)
  5. run the first-contact command
  
  Connected = an operator exists with `last_used_at < 24 h`. Also list the operators, with revoke.
- **Empty and loading states** exactly as the brief lists. Every update call shows pending state for ≥ 300 ms and handles rejects with a human message mapped from `ApiError`.
- **Accessibility:** keyboard navigable, visible focus, alt text from category and subject, WCAG AA contrast.

## 4b. Analytics — Firebase (GA4) event tracking
- **Setup:** Firebase project `space-compute-prod` (and a separate `-staging`). Use the modular Web SDK (`firebase/app`, `firebase/analytics`, `firebase/performance`), **lazy-loaded after consent**, outside the initial bundle.
- **Consent:** a banner offers "Allow analytics" or "Essential only". Consent Mode v2 defaults are denied; the choice is stored in `localStorage`. Admin routes are never tracked.
- **Identity:** `setUserId(hmac_sha256(principal, SITE_SALT))`, where `SITE_SALT` is a build-time constant. Principals, AAA names, rationales and any free text are **never** sent.
- **Per-route `document.title`**, otherwise GA4 lumps every view under one title (a proof-of-burn lesson).
- **User properties:** `is_admin` (so admin traffic can be filtered out), `has_aaa`, `aaa_tier`, `auto_topup_enabled`, `payment_methods_used` (set of enums).
- **Event taxonomy** (snake_case; parameters are enums or numbers only):

| Event | Params |
|---|---|
| `page_view` (automatic, SPA router hook) | `route` (template, e.g. `/d/:id`) |
| `sign_in` / `sign_out` | `method: ii` |
| `spawn_started` / `spawn_completed` / `spawn_failed` | `method: icp_wallet \| icp_deposit \| card \| btc \| eth`, `error_code` |
| `payment_started` / `payment_completed` / `payment_failed` | `method`, `kind: topup \| pack \| subscription \| auto_topup`, `usd_bucket` |
| `auto_topup_set` / `auto_topup_revoked` | `method` |
| `agent_connect_viewed` / `operator_added` / `operator_removed` | — |
| `agent_first_activity` | `minutes_since_spawn` (fired once when the dashboard first sees agent activity) |
| `discovery_viewed` / `citation_copied` / `citation_verified` | `category`, `status`, `verified: bool` |
| `leaderboard_viewed` / `profile_viewed` | — |
| `fuel_low_banner_shown` / `fuel_paused_banner_shown` | — |
| `error_shown` | `error_code` (ApiError variant), `route` |

- **Performance Monitoring:** web vitals, plus custom traces for `ic_update_call` (method name as an attribute) and `dossier_fetch`.
- **Dashboards (owner):**
  - GA4 funnels: sign-in → spawn → agent connected → first activity → first top-up
  - payment method mix and failure rates
  - retention (7/30-day owners returning)
  - on-chain science metrics stay in `/admin` (source of truth), not Firebase
- **CSP additions:** `script-src https://www.googletagmanager.com`; `connect-src https://*.google-analytics.com https://*.analytics.google.com https://firebaseinstallations.googleapis.com https://firebaselogging-pa.googleapis.com https://firebaseremoteconfig.googleapis.com`.
- **Budget:** analytics chunk ≤ 60 KB gz, loaded after first paint.

## 4. Non-functional
- Bundle ≤ 350 KB gz initial; routes code-split.
- Third-party scripts: only Firebase Analytics and Performance (§5), consent-gated and lazy-loaded.
- Untrusted text (names, rationales) is rendered as text only. Never `dangerouslySetInnerHTML`.

## 5. Acceptance
0. Admin: a non-admin principal gets 404 on `/admin` and `Unauthorized` from every admin method. Every admin mutation appears in the audit view.
0b. Analytics: no Firebase network requests happen before consent; the event payloads in the debug view contain no principals or free text.
1. All 10 routes render against a local deployment seeded by `tools/seed-local` (task T1.6), with loading, empty and error states.
2. Signed-out visitors can browse 1–5; owner routes redirect to sign-in.
3. Spawn works end-to-end locally via the Deposit path (the wallet path is exercised in staging with OISY).
4. The citation "verified" mark fails closed (a tampered witness shows unverified).
5. Lighthouse accessibility ≥ 90 on Landing, Discovery detail and Dashboard.
