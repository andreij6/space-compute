# 12 — `treasury` canister: owner-funded treasury → continuous cycles (Rust)

Decision (owner, 2026-09-27): **Space Compute will not control or stake an NNS neuron.** Instead, the owner directly funds the treasury canister with ICP. The `treasury` canister acts as an automated cycles keeper and float manager, ensuring canisters are topped up before they risk freezing.

## 1. Why a separate canister (ADR-19)
The treasury canister holds the app's operational reserve. It maintains the smallest possible attack surface:
- One small canister with **no public update methods** except `status` and `health` queries.
- Admin methods only (access-controlled to team principals).
- No user-facing mutation logic.
- `payments` (which handles user traffic) never holds or controls the operational reserve.

## 2. Responsibilities
1. **Holds owner-funded ICP reserve.** The owner deposits ICP directly into the treasury canister's default account (`treasury_canister_id`).
2. **Keeps app canisters topped up (the cycles keeper).** Every 6 h, an automated timer checks each watched canister, ordered by criticality: `platform`, `payments`, `frontend`, and `treasury` itself.
   - Calls `canister_status` on the management canister (`aaaaa-aa`) to read the cycle balance and recent burn rate. Treasury is a co-controller of each watched canister.
   - If `balance < target_days × daily_burn`, treasury transfers ICP to the CMC and calls `notify_top_up` to restore the balance to `target_days` (default: 60 days, minimum top-up: 0.5T cycles).
3. **Feeds the fuel treasury** (only when BTC/ETH fuel packs are enabled, and never while `health().reserve_breached`: ops cycles always take priority). If `payments.TREASURY` ICP is below `fuel_floor × 2`, transfers the shortfall, capped at `fuel_topup_cap_per_week`.
4. **Enforces reserve floor & runway alerts.** It never spends below `reserve_e8s` (default: 3 months of projected cycles burn). If available ICP drops below the reserve, it raises an alert (the admin overview banner plus a GitHub Action poller) and pauses non-ICP intake so ops canisters never starve.

## 3. State & API
- **Memory:** `Config`, `WatchList` (canister id, priority, `target_days`), append-only `Ledger` of every deposit, top-up, and transfer, and `Metrics` (burn EMA per canister).
- **Queries:**
  - `health()` (public): `{ reserve_breached : bool; min_runway_days : nat32; worst_canister : opt principal }`. Polled by `payments` to pause non-ICP intake when runway < 21 days (04 §6.2b).
  - `status()` (public, for transparency): treasury ICP balance, daily burn rate across canisters, cycles runway per canister in days, and projected months of total runway.
  - `history(cursor, limit) -> (vec HistoryItem, opt nat64)`
- **Admin methods (audit-logged):**
  - `admin_set_config(Config)`
  - `admin_watch(canister, priority, target_days)` / `admin_unwatch(canister)`
  - `admin_topup_now(canister)` (forces an immediate top-up check)
  - `admin_withdraw(to, amount)`, requiring two admin approvals within 24 h.

## 4. Operational Funding Runbook
1. **Initial funding:** Transfer ICP from the owner wallet to the treasury canister principal:
   ```bash
   icp ledger transfer --to <treasury-canister-id> --amount <amount_icp> -e production
   ```
2. **Recommended float:** ~$25/month of cycles burn (~37.5 ICP/year at $8/ICP). Depositing 10–20 ICP provides 3–6 months of complete operational runway.
3. **Runway monitoring:** The admin dashboard displays remaining runway days. If runway drops below 21 days, `health()` flags `reserve_breached`, notifying the owner to add funds before any canister approaches freezing.

## 5. Acceptance (PocketIC with ICP Ledger & CMC)
1. Deposit ICP into the treasury canister account.
2. A watched canister with simulated low cycle balance is automatically topped up to `target_days` on the next 6h timer tick.
3. A watched canister with adequate cycles is skipped.
4. When treasury ICP drops to `reserve_e8s`, top-ups cease and `health().reserve_breached` evaluates to `true`.
5. `admin_withdraw` without a second admin confirmation does not execute.
6. The canister survives upgrades with its watchlist, metrics, and ledger history intact, and its timers resume cleanly.

## 6. Architecture Note: Removal of NNS Neuron
An NNS neuron integration was evaluated during the technical spike phase (SP-5, SP-6, and Review R-101) to harvest voting maturity. Following review of controller immutability, 7-day maturity modulation delays, and governance coupling with `proof-of-burn`, the architecture was intentionally simplified:
- No NNS neuron is created, staked, or controlled by Space Compute.
- Testing in PocketIC is dramatically simplified: no NNS governance wasm, mock neurons, or 7-day maturity advancement logic is required.
- All funding is explicit, transparent, and direct from the owner.
