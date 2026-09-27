# 04 — `payments` canister (Rust)

There are two funding models:
- **ICP payments are pass-through.** ICP goes from the user to the CMC, and cycles go to the user's AAA. Spawn, one-time top-up and auto top-up all work this way (§1–§4).
- **Non-ICP payments are treasury-backed** (§6). A **$5 fuel pack** paid by **card (Stripe)**, **BTC (ckBTC)** or **ETH (ckETH)** is funded from the **central ICP treasury**, which the canister converts to cycles for the AAA.

The canister never holds per-user balances, apart from transient deposit subaccounts that are swept on notify. It has the highest review bar in the codebase: see `08-security.md` §4 and the `canister-security` and `icrc-ledger` skills.

External canisters: ICP ledger `ryjl3-tyaaa-aaaaa-aaaba-cai`, CMC `rkp4c-7iaaa-aaaaa-aaaca-cai`, Exchange Rate Canister (XRC) `uf6dk-hyaaa-aaaaq-qaaaq-cai`, ckBTC ledger `mxzaz-hqaaa-aaaar-qaada-cai` and minter `mqygn-kiaaa-aaaar-qaadq-cai`, ckETH ledger `ss2fx-dyaaa-aaaar-qacoq-cai` and minter (id pinned in config). IDs are read from init args, so the local, staging and PocketIC environments can inject their own.

## 0. Feature flags (ADR-20)
`Config.features = { card : bool /*false at launch*/; btc : bool; eth : bool; sponsored_spawn : bool }`.
- `get_features()` is a public query that the frontend reads on load.
- A disabled method returns `FeatureDisabled`. This covers `stripe_credit` while `card = false`, and the BTC/ETH endpoints when those flags are off.
- `admin_set_features` flips a flag and is audit-logged.
- Launch config: `card = false`, `btc = true`, `eth = true`, `sponsored_spawn = true`. BTC/ETH can also be switched off with one flag if the compliance check (T8.8) says so.

## 0b. Sponsored first spawn (invite codes) — replaces card onboarding while Stripe is off
- Admins mint invite codes in batches: `admin_mint_invites(count, sponsor_cycles, expires_at) -> vec text`. Only `sha256(code)` is stored on-chain, and codes are shown once.
- `spawn_aaa { path = Invite { code } }` burns the code and creates the AAA with **treasury-sponsored cycles** (creation fee + `sponsor_cycles`, default 0.5T ≈ $0.70 of fuel).
  - The ICP comes from the `payments.TREASURY` account, which `treasury` (12) feeds from the owner-funded reserve.
- Limits:
  - one sponsored AAA per owner principal, ever
  - a daily sponsor budget cap
  - codes are single-use and expire
- Sponsored AAAs run normally; their owners top up with ICP/BTC/ETH once the starter fuel is used.

## 1. Funding paths

| Path | User action | Canister action |
|---|---|---|
| **A. Wallet (ICRC-2)** | Approves in OISY (via `@icp-sdk/signer`) an allowance to spender `(payments, spender_subaccount = S(purpose, beneficiary))` | `icrc2_transfer_from(from = payer, spender_subaccount = S(..), to = CMC deposit account, memo)` then `notify_*` |
| **B. Deposit address** | Sends ICP from any wallet or exchange to `(payments, D(purpose, beneficiary))`, shown as account id + QR | `sweep(...)`: moves the balance minus the fee to the CMC deposit account, then `notify_*` |

- `S(purpose, beneficiary) = sha256("sc-spender" || purpose || beneficiary_bytes)`.
- `D(purpose, beneficiary) = sha256("sc-deposit" || purpose || beneficiary_bytes)`.
- `purpose ∈ {spawn, topup, auto}`.

**Why beneficiary-bound subaccounts:** anyone can call `transfer_from` on the payments canister's behalf. Binding each allowance to a spender subaccount derived from its beneficiary means a third party calling our methods with someone else's approval can only ever fund *that* approval's beneficiary. See `REVIEW.md` finding R-03.

## 2. CMC interaction
- **Top-up:** transfer to `(CMC, subaccount = principal_to_subaccount(aaa))` with memo `TPUP` (`0x50555054`), then `notify_top_up { block_index, canister_id = aaa }`.
- **Create:** transfer to `(CMC, subaccount = principal_to_subaccount(payments))` with memo `CREA` (`0x41455243`), then `notify_create_canister { block_index, controller = payments, settings = { controllers = [owner, platform], freezing_threshold = 60 days } , subnet_selection = null }`.
- **Spike SP-1 — resolved 2026-09-27 (PocketIC, real CMC): yes.** The CMC accepts an ICRC-2 `icrc2_transfer_from` (and a plain `icrc1_transfer`) whose ICRC-1 memo is the 8-byte little-endian u64 `TPUP`/`CREA`, for both `notify_top_up` and `notify_create_canister`. So `payments` pulls straight from the payer into the CMC deposit account in one ledger call; no intermediate account, no extra fee, no `CmcDepositMode` switch. A transfer without the memo is refunded (minus fee) to the sender. Proof: `just demo SP-1` (`crates/integration-tests/tests/sp_1_cmc_memo.rs`).
- `notify_*` is idempotent per block index. On `Processing` or `SYS_UNKNOWN`, retry. On `Refunded { block_index }`, mark the op `Refunded` and record the refund block; the CMC refunds to the sender.

## 3. Operations journal (saga)
```
Op { v, id: u64, kind: Spawn{owner,name,avatar_seed} | TopUp{aaa} | AutoTopUp{aaa}
          | FuelPack{aaa, source: Card{stripe_ref} | Btc{sats} | Eth{wei}, usd_cents, packs},
     path: Wallet{payer: Account} | Deposit | Treasury, amount_e8s, created_at, created_by: Principal,
     state: Pending | Pulled{block} | Credited{usd_cents} | TreasuryPaid{block} | Notified{canister_or_cycles} | Registered | Done
          | Failed{reason} | Refunded{block}, updated_at, attempts: u8 }
```
- Mem 0 is config. Mem 1 is `ops`. Mem 2 is `owner → aaa` (from spawns). Mem 3 is `aaa → Mandate { payer, topup_e8s, cap_30d_e8s, enabled, needs_attention, last_auto_at }`. Mem 4 is `(aaa, op_id)` → `(at, e8s)` (auto top-up history for the **rolling 30-day** cap, R-50). The in-flight lock set lives in the heap, not stable memory.
- **The state is persisted before every await.** Every step can be resumed: `resume(op_id)` (anyone) and a 5-minute timer both advance ops that are not `Done`/`Failed`/`Refunded`.
- **Per-beneficiary `CallerGuard`**: only one in-flight op per `(purpose, beneficiary)`.

## 4. API
```candid
// quotes
get_quote_spawn : () -> (Quote) query;      // e8s needed = creation fee + aaa_initial_cycles, via the CACHED CMC xdr rate (+2% buffer, + ledger fees). Queries can't call the CMC; an hourly timer refreshes the rate via get_icp_xdr_conversion_rate (R-54)
get_quote_topup : (nat /*cycles*/) -> (Quote) query;
get_deposit_account : (Purpose, principal /*beneficiary*/) -> (text /*account id hex*/, Account) query;

// spawn (caller = owner principal from II; not anonymous; must not already own an AAA)
spawn_aaa : (record { name : text; avatar_seed : nat64; path : PayPath }) -> (Result_OpId);
// PayPath = variant { Wallet : record { payer : Account; amount_e8s : nat64 }; Deposit; Invite : record { code : text } }
// Deposit path: reads balance of D(spawn, caller); requires >= quote.

// one-time top-up (anyone may gift fuel to any registered AAA)
top_up : (record { aaa : principal; path : PayPath }) -> (Result_OpId);    // min 0.1 ICP

// auto top-up mandate (caller = aaa owner)
set_mandate : (record { aaa : principal; payer : Account; topup_e8s : nat64;
                        cap_30d_e8s : nat64; enabled : bool }) -> (Result);   // UI calls it "monthly limit"
get_mandate : (principal) -> (opt MandateView) query;   // includes spent in the last 30 days, allowance remaining (cached)
request_auto_topup : () -> (Result_OpId);               // caller = the AAA canister itself

// status
get_op : (nat64) -> (opt OpView) query;
list_ops_for_aaa : (principal, opt nat64, nat16) -> (Page_OpView) query;
list_ops_for_owner : (principal, opt nat64, nat16) -> (Page_OpView) query;
resume : (nat64) -> (Result);

// admin (all mutations are written to an on-chain audit log)
admin_set_params / admin_pause / admin_add_admin / admin_remove_admin
admin_overview : () -> (PaymentsOverview) query;   // treasury balances (ICP, ckBTC, ckETH), 24h spend vs caps, reserve headroom, rates + age, pause flags, failed/stuck ops, stripe_daily_usd used, own cycles
admin_list_ops : (record { state : opt OpState; kind : opt OpKind; since : opt nat64 }, opt nat64, nat16) -> (Page_OpView) query;
admin_audit_log : (opt nat64, nat16) -> (Page_AuditEntry) query;
admin_block_card / admin_unblock_card : (principal) -> (Result);
admin_set_stripe_relay : (principal) -> (Result);
admin_treasury_withdraw : (record { ledger : Ledger; to : Account; amount : nat }) -> (Result);
```
- **`spawn_aaa` sequence:**
  0. Pre-checks (before any funds move): the caller is not anonymous; the per-owner guard is free; `platform.aaa_by_owner(caller)` is none or `Deleted`; `platform.check_name(name) == Ok`.
1. The op goes `Pending`.
  2. Pull or sweep → `Pulled{block}`.
  3. `notify_create_canister` → `Notified{canister_id}`, and record `owner → aaa`.
  4. `platform.register_aaa{canister_id, owner, name, avatar_seed}` → `Registered`.
  5. → `Done`.
- **`request_auto_topup`:**
  - The caller must be an AAA known to payments (it spawned it) or confirmed by `platform.aaa_owner`, which is cached.
  - The mandate must be enabled; `now - last_auto ≥ auto_topup_min_interval_secs`; and `spent(last 30 days) + topup ≤ cap_30d`.
  - Then `transfer_from(payer, S(auto, aaa))` → CMC → `notify_top_up`.
  - On `InsufficientAllowance` or `InsufficientFunds`: the op is `Failed`, and the mandate is marked `needs_attention` (the UI shows "allowance revoked/insufficient").
- **Frontend helper for a wallet approval:** approve `amount = topup_e8s * 12` (or the monthly cap × 12), with `expires_at = now + 365 d`, to spender `(payments, S(auto, aaa))`. The canister enforces the monthly cap on top of that.
- The ledger fee is fetched from `icrc1_fee` and cached; on `BadFee{expected}`, update the cache and retry once.
- Every transfer sets `created_at_time`, for ledger dedup.

## 5. Acceptance criteria — ICP paths (PocketIC, with real ledger + CMC wasms)
1. Wallet spawn: approve → `spawn_aaa` → the AAA exists with controllers [owner, platform], is registered, and has cycles ≈ the initial amount.
2. Deposit spawn: transfer to the deposit account → `spawn_aaa{Deposit}` → the same result; the deposit account is empty afterwards (except dust < fee).
3. A third party calling `top_up{aaa: X, Wallet{payer: victim}}` can only use allowances the victim granted to `S(topup, X)`. An allowance granted for Y is never usable for X.
4. Kill `platform` mid-spawn (stopped canister) → the op stays `Notified`; after restart, `resume` completes it. No double charge (ledger dedup plus the journal).
5. Auto top-up respects the interval and the monthly cap, and flags a revoked allowance.
6. A CMC refund path (simulated invalid canister) ends in `Refunded`.
7. Concurrent `top_up` calls for the same AAA can't both pull from the same allowance past its amount (guard).

## 6. Non-ICP fuel packs (treasury-backed)

### 6.1 Product
- A **fuel pack = US$5**. The AAA receives `cycles = ICP(usd 5 × (1 − margin)) → CMC` at the moment of top-up. Default `margin_bp = 500` (5%) covers card fees, the ckBTC/ckETH minter fees, and volatility.
- Card: 1–4 packs per checkout, or a **monthly card subscription** of 1 pack/month (Stripe Billing).
- BTC/ETH: the user deposits any amount ≥ the pack price (or ≥ the minter minimum, if higher). The credit is the **full USD value** of the deposit at the rate when minted, converted in units of cents rather than whole packs.

### 6.2 Treasury
- **Treasury account = `(payments, subaccount = TREASURY)`** on the ICP ledger. It is funded by the `treasury` canister's owner-funded reserve (12) and by direct team deposits.
- The same canister holds the received ckBTC and ckETH in `(payments, TREASURY)` on those ledgers.
- **Guard rails:**
  - `treasury_reserve_floor_e8s`: fuel packs are refused ("temporarily unavailable") when the ICP balance would drop below it
  - `treasury_daily_cap_e8s`: rolling 24 h spend
  - `per_aaa_daily_packs` (default 4)
  - `admin_pause_non_icp`
- Rates are refreshed every 30 min from the **XRC**: ICP/USD, BTC/USD, ETH/USD. Each call attaches 1B cycles. Rates are cached with a timestamp. **If the cache is older than 2 h, fuel packs are refused** rather than priced stale.
- Admin methods:
  - `admin_treasury_withdraw(to, amount, ledger)`: team-only, logged, used for rebalancing (e.g. moving ckBTC to a DEX for conversion; MVP rebalancing is manual)
  - `get_treasury_status()`: balances, 24 h spend, rates, reserve headroom (public query, for transparency)

### 6.2b Automatic intake pause when the treasury is low (owner, 2026-09-27)
If the app can't pay for cycles, it must stop taking money it can't honour. `payments` keeps an **intake state**: `Open | Paused { reason, since }`.
- **Pause triggers** are checked on a 10-min timer and before every new non-ICP intake:
  - `TREASURY` ICP < `treasury_reserve_floor_e8s`
  - or `treasury.health()` (spec 12 §3) reports `reserve_breached` or `min_runway_days < intake_min_runway_days` (default 21)
  - or the XRC rate cache is stale (existing rule)
- **While paused**, every *new* non-ICP intake returns `TemporarilyUnavailable { reason = TreasuryLow }`:
  - new BTC deposit addresses and ETH deposit info
  - BTC/ETH spawns and invite-code (sponsored) spawns
  - `stripe_credit`, if card is ever enabled
- **ICP paths stay open.** They are pass-through (the user's ICP → CMC) and never touch the treasury.
- **Nothing gets stranded.** A BTC/ETH deposit that lands on an address issued *before* the pause is still minted and swept into `TREASURY`. It becomes a `PendingCredit` owed to that AAA, and pending credits are paid FIFO before any new intake once the pause lifts. The UI marks old addresses "paused — don't send".
- **Resume with hysteresis:** reopen only after two consecutive checks with ICP ≥ 1.5 × floor and a healthy treasury. This stops it flapping.
- `admin_set_intake(variant { Auto; ForceOpen; ForcePause })`, audit-logged. `get_features()` also returns `intake : { paused : bool; reason : opt text; since : opt nat64 }`. The frontend hides the BTC/ETH/invite options and shows a one-line banner ("Card and crypto top-ups are paused while we refuel. ICP still works."). Admin overview gets a red banner; analytics event `intake_paused`.

### 6.3 Paying out a credited pack (common tail, `fund_from_treasury(op)`)
1. `Credited{usd_cents}`: compute `e8s = usd_cents × (1 − margin) / icp_usd_rate`, and check the reserve floor and the caps.
2. `icrc1_transfer` from `TREASURY` → the CMC deposit account for the AAA (memo TPUP) → `TreasuryPaid{block}`.
3. `notify_top_up(block, aaa)` → `Notified{cycles}` → `Done`.

The steps are resumable like every op (§3). A frozen AAA can be topped up; the CMC deposits cycles regardless.

### 6.4 Card (Stripe) via the relay — see `04b-stripe-relay.md`
```candid
// caller = relay principal (config.stripe_relay); idempotent on stripe_ref (checkout session id / invoice id)
stripe_credit : (record { stripe_ref : text; aaa : principal; packs : nat8; usd_cents : nat32;
                          kind : variant { OneTime; SubscriptionRenewal } }) -> (Result_OpId);
// mem 6: stripe_ref -> op_id (dedupe). Validates aaa is registered, usd_cents == packs × 500, packs ∈ 1..4.
```

### 6.5 BTC (ckBTC)
```candid
get_btc_deposit_address : (principal /*aaa*/) -> (Result_Text);   // update: minter.get_btc_address{owner=payments, subaccount=D(btc, aaa)}; cached in mem 7
notify_btc_deposit : (principal /*aaa*/) -> (Result_OpIds);          // anyone; minter.update_balance{owner=payments, subaccount=D(btc,aaa)}
```
- For each `Minted { minted_amount, block_index }` UTXO status:
  - create a `FuelPack{Btc}` op with `usd_cents = sats × btc_usd`
  - sweep the ckBTC from `D(btc, aaa)` into `TREASURY`
  - then run `fund_from_treasury`
- `Checked`/pending statuses are returned to the UI as "waiting for N confirmations".
- A deposit below the minimum is still minted and swept to the treasury, then credited at its value (no minimum is enforced after minting, so no funds are stranded).
- Frontend polling calls `notify_btc_deposit` every 5 min while the fuel page is open. A payments timer also calls it every 30 min for addresses issued in the last 7 days.

### 6.6 ETH (ckETH)
```candid
get_eth_deposit_info : (principal /*aaa*/) -> (EthDepositInfo) query;
// { helper_contract : text; principal_bytes32 : text; subaccount_bytes32 : text /* D(eth, aaa) */; min_wei : nat; }
notify_eth_deposit : (principal /*aaa*/) -> (Result_OpIds);   // reads ckETH balance of (payments, D(eth, aaa)); new balance → op
```
- The user deposits through the **ckETH helper contract** with principal = payments and subaccount = `D(eth, aaa)`. The frontend builds the transaction for any EIP-1193 wallet, such as MetaMask. Minting takes about 20 min.
- Balance > 0 → create a `FuelPack{Eth}` op (`usd_cents = wei × eth_usd`), sweep into `TREASURY`, then `fund_from_treasury`.
- **Spike SP-6:** confirm that the helper contract supports subaccount deposits, and find the minimum deposit. If the minimum is above $5, the ETH pack price becomes that minimum, and the UI shows it.

### 6.6b Spawning with card, BTC or ETH
New users without ICP can spawn with a fuel pack:
- The beneficiary is the **owner principal**, not an AAA. Keys used: Stripe `metadata.owner` + `metadata.name`, and deposit subaccounts `D(btc-spawn, owner)` / `D(eth-spawn, owner)`.
- The credited value must be ≥ the spawn quote, which is roughly creation + initial fuel (about $2). The treasury transfers ICP to the CMC for `notify_create_canister`, and the rest follows the normal spawn saga (§4). Any value left over is delivered as a top-up to the new AAA.
- `stripe_credit.kind` gains `Spawn { owner; name; avatar_seed }`. Pre-checks (name free, owner has no live AAA) run in the relay's `/checkout` **and** again in the canister. If a pre-check fails after payment, the value becomes a pending credit for that owner, usable by the next successful spawn. This is the only case of a held balance; it is bounded to 30 days and then refunded via Stripe by an admin.

### 6.7 Acceptance (PocketIC with ckBTC/ckETH ledgers + mocked minters + XRC mock)
8. A `stripe_credit` replay with the same `stripe_ref` produces one op and one top-up.
9. A `stripe_credit` from a non-relay caller → `Unauthorized`.
10. A mock BTC mint of $12 worth → one op of 1200 cents → the AAA receives cycles ≈ $11.40 of ICP; the treasury ckBTC goes up by the deposit.
11. With the treasury at its floor → fuel packs return `TemporarilyUnavailable`; ICP paths are unaffected.
12. A rate cache older than 2 h → fuel packs are refused.
13. The daily cap and `per_aaa_daily_packs` are enforced.
14. A card-funded spawn (test mode) → the AAA exists, with the leftover value delivered as cycles.
15. Treasury ICP drops below the floor → within one check `get_features().intake.paused = true`, a new BTC address request and an invite spawn both return `TemporarilyUnavailable{TreasuryLow}`, and an ICP top-up still succeeds.
16. A ckBTC deposit minted on a pre-pause address while paused → swept to `TREASURY` and recorded as a `PendingCredit`, with no cycles sent. After the refill and two healthy checks → intake reopens and the pending credit is paid first.
17. `treasury.health()` reporting `min_runway_days` < 21 pauses intake even when `TREASURY` is above its floor. `ForceOpen` overrides, and the override is audit-logged.
