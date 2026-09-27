# 09 — Testing, CI/CD & operations

## 1. Testing pyramid
*Superseded in detail by `11-test-strategy.md`, which is normative. The table below is the summary.*
| Layer | Tooling | Scope |
|---|---|---|
| Unit | `cargo test` (native target) | pure logic: protocol validation, evaluate(), reputation, tiers, badges, XP, citation text, subaccount derivation |
| Property | `proptest` | evaluate() monotonicity; replay(log) == incremental Progress; protocol paths |
| Integration | `pocket-ic` crate, `crates/integration-tests` | every acceptance list in 02–04; multi-canister flows with the real ICP ledger + CMC wasms (downloaded, sha-pinned in `tests/wasm.lock`) |
| Upgrade | PocketIC | install vN-1 → populate → upgrade to vN → assert state (per canister) |
| Frontend | `vitest` for utils; Playwright smoke against the local network | route rendering, spawn via Deposit, citation verification |
| Agent e2e | scripted Claude Code run against staging | 06 §5 |
| Load | PocketIC script: 200 AAAs × 100 tasks | instructions per call, memory growth, fee adequacy |

## 2. Verification & Deploy Harness (Local First — No Remote CI)
- Local verification (`just verify` / `scripts/verify-local.sh`): `cargo fmt --check`, `clippy -D warnings`, unit + PocketIC integration tests, build wasms, `candid-extractor` drift check against committed `.did`, AAA wasm size ≤ 1.5 MiB gz, frontend typecheck/lint/test/build.
- Deterministic deploy (`just deploy-local` / `scripts/deploy-local.sh`): one-shot idempotent script that starts network, installs ledgers once (Init mode, never upgraded), deploys core canisters, registers AAA wasm template, wires inter-canister calls, and seeds test data.
- Production/staging deploy: manual scripted deploy using `icp deploy -e <env>` with explicit identity checks and guardrails (`guard.sh`).

## 3. Operations
- **Cycles monitoring:** a scheduled `just ops-smoke` (local launchd/cron) checks `icp canister status` on platform/payments/frontend every 6 h and alerts (email + admin banner) when the balance < 30 days of burn.
- **Cycles for the app are automatic:** the `treasury` canister (12) holds an owner-funded ICP balance, checks cycles on platform, payments, frontend, and itself every 6 h, and tops them up via CMC `notify_top_up`. Humans only watch `/admin/treasury` and respond to runway alerts.
- **Two treasuries:**
  - **Ops treasury** is the `treasury` canister account, directly funded with ICP by the owner. It continuously monitors and tops up the platform canisters.
  - **Fuel treasury** is the `payments` canister's `TREASURY` subaccount. It backs card, BTC and ETH fuel packs, and is refilled from the ops treasury and from Stripe revenue.
- **Treasury funding (owner direct deposit):**
  - owner transfers ICP directly to the `treasury` canister account (`icp ledger transfer --to <treasury-id> --amount <amount_icp> -e production`)
  - maintain a recommended float of 10–20 ICP (~3–6 months of operations)
  - log deposits in `docs/ops/treasury-log.md`
- **Treasury ops (`docs/ops/treasury.md`):**
  - weekly: convert accumulated ckBTC/ckETH to ICP (manual DEX swap or OTC) and top up the treasury ICP from Stripe revenue
  - keep the balance ≥ 2× `treasury_reserve_floor`
  - alert when the balance is below 1.5× the floor
- **Stripe reconciliation:** a daily local job runs `tools/reconcile-stripe` (Stripe paid list vs `payments.list_ops` of kind `FuelPack{Card}`). A mismatch → alert + pause.
- **Relay deploy:** `wrangler deploy` from the `relay/` directory, with secrets set via `wrangler secret put`. Staging uses Stripe test mode only.
- **Backups:** the platform exposes `admin_export_events(from, limit)`; a weekly job archives the log to object storage (it's public data anyway).
- **Incident switches:** `admin_pause` (tasks/reviews/spawns) and `payments.admin_pause`.
- **Release:** semver per canister; changelog; the AAA wasm version is bumped in `platform` only after staging soak ≥ 48 h.
