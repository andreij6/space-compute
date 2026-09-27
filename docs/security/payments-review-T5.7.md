# Payments internal security review (T5.7)

- Scope: `crates/payments`, ICP-only MVP. Covers the spawn saga (Deposit, Wallet, Invite), one-time top-up, auto top-up mandates, the TREASURY subaccount and `admin_treasury_withdraw`, feature flags, invites, admin APIs, and the journal and resume path. Card, ckBTC and ckETH (D5–D12) are deferred and out of scope.
- Gate: `08-security.md` §4. That means the `canister-security` and `icrc-ledger` skill checklists, plus acceptance tests 04 §5 #3, #4 and #7.
- Reviewer: internal (Claude, T5.7), 2026-09-27. An external review before production is still recommended (T7.5).
- Result: **checklist signed.** 4 high and 3 medium defects were fixed, each with a failing-first test. The ⚠️ items below are accepted risks with owners.

Line references are to `crates/payments/src/*` as of the T5.7 commit.

## 1. `canister-security` checklist

| # | Item | Status | Evidence |
|---|---|---|---|
| CS-1 | Anonymous caller is rejected on every state-changing user method | ✅ | `top_up` api.rs:703, `set_mandate` :736, `request_auto_topup` :843, `spawn_aaa` :1014, `require_admin` :35 (anonymous is never an admin: config.rs `add_admin`). `resume` is anonymous-callable **by design** (04 §3): it only advances an existing journal entry, and the entry fixes the beneficiary. |
| CS-2 | Admin gating on every `admin_*` method (updates and queries) | ✅ | `require_admin()` is the first line of every `admin_*` method (api.rs:57–300, :1103). Test: `t5_15_admin_treasury_withdraw_moves_icp_and_is_admin_gated`, `t5_1_*` |
| CS-3 | Owner check: `set_mandate` only by the AAA owner confirmed by `platform.aaa_owner` | ✅ | api.rs:740–744. Tests: `t5_5_*` |
| CS-4 | AAA-caller check: `request_auto_topup` only by the AAA itself | ✅ | caller = `msg_caller()` is the mandate key (api.rs:842–848). A mandate exists only if the owner set it through CS-3 |
| CS-5 | `msg_caller()` is bound before any await | ✅ | Every entry point reads the caller on its first line, before its first await (lines above) |
| CS-6 | Per-beneficiary `CallerGuard` held across await+mutate, released on trap/err (Drop) | ✅ | guard.rs:23,39. Guard keys: `spawn/owner` :1026, `topup/aaa` :717, `auto/aaa` :847. `resume` takes the same key as the op kind (:1074–1085). Test: `t5_6_*` (04 §5 #7) |
| CS-7 | The journal is persisted before every await (saga), and every step can be resumed | ✅ | The op is created before the first ledger await. The pull amount is pinned with `journal::fix_pull` (journal.rs:206) **before** the transfer await (api.rs:636, :927). Tests: `t5_7_fix_pull_pins_the_first_amount_and_refuses_after_pending`, `t5_3_*` (04 §5 #4) |
| CS-8 | Resume timer eventually reaches every non-terminal op | ✅ (fixed H-4) | `journal::resumable_from` with a heap watermark (journal.rs:221, timers.rs:23). Test: `t5_7_resumable_from_reaches_ops_beyond_the_first_page_and_advances_the_watermark` |
| CS-9 | Kill switch actually stops new money movement | ✅ (fixed H-3) | `ensure_not_paused` (api.rs:570) runs before any await in `spawn_aaa`/`top_up`/`request_auto_topup`. In-flight sagas are still allowed to finish. Test: `t5_7_pause_flags_block_new_spawn_topup_and_auto_topup` |
| CS-10 | Bounded-wait calls only | ✅ | All calls use `Call::bounded_wait` (api.rs, rate.rs) |
| CS-11 | Integer overflow / truncation | ✅ | e8s sums use `saturating_add` (journal.rs `stats_since`, mandate.rs `spent_last_30d`, invites.rs `next_daily_budget`/`sponsor_spawn`). Withdraw amount > u64 is rejected, not saturated (api.rs:1104). `cycles_to_e8s` checks overflow (quote.rs:26) |
| CS-12 | Unbounded iteration | ⚠️ | Resume sweep bounded to 50 ops per tick (fixed). Still O(n) over all ops: `admin_overview` (`stats_since`, `failed_count`, `stuck_count`) and the `admin_list_ops` filter scan (admin-only, instruction-limited). The per-AAA auto-top-up history is scanned in full by `spent_last_30d` and never pruned (≈4/day/AAA). Listed as L-2 |
| CS-13 | Stable-memory ids match 04 §3 | ✅ | memory.rs:8–20: 0 config, 1 ops, 2 owner→aaa, 3 mandate, 4 history (as spec). 5 XDR rate, 6 reserved (Stripe, D5), 7 BTC cache (reserved), 8–10 invites, 60/61 audit. `Op.pull_e8s` is `opt`, so old records decode (test `t5_7_ops_stored_before_pull_e8s_existed_still_decode`) |
| CS-14 | Upgrade safety | ✅ | Stable structures only. `post_upgrade` only bootstraps admins when the list is empty (lib.rs:47). The heap guard set and the watermark reset safely |
| CS-15 | Admin actions audit-logged | ✅ | `audit()` on every admin mutation. `admin_treasury_withdraw` now logs the intent **before** the await and the outcome after it (api.rs:1110–1132) |

## 2. `icrc-ledger` checklist

| # | Item | Status | Evidence |
|---|---|---|---|
| IL-1 | Every transfer sets `created_at_time` | ✅ | `icrc1_transfer_once`/`icrc2_transfer_from_once` always set it (api.rs:370–490) |
| IL-2 | `created_at_time` is fixed per intent, so retries dedupe | ✅ (fixed C-1) | Every saga pull uses `op.created_at` (api.rs:611, :615, :775, :890–898). Withdraw accepts a caller-supplied `created_at_time` (api.rs:1109). Test: `t5_7_treasury_withdraw_retry_with_the_same_created_at_time_pays_once` |
| IL-3 | Done/Failed/Unknown classification. Timeouts are **not** failures | ✅ (fixed C-1/H-2) | `ledger::classify_transfer[_from]` (ledger.rs:86,104) and `settle` (api.rs:494). A call error, `TemporarilyUnavailable` or `CreatedInFuture` is Unknown: the op stays resumable with the same args. `Duplicate` is Done. Definitive rejects are Failed. Tests: `t5_7_classify_transfer_never_treats_ambiguous_outcomes_as_failed`, `t5_7_classify_transfer_from_flags_allowance_and_funds_as_funding_rejections`, `t5_7_a_rejected_wallet_pull_fails_the_op_and_is_never_retried` |
| IL-4 | Fee handling / `BadFee` retried once with the same `created_at_time` | ✅ / ⚠️ | api.rs:416, :473. The fee is admin-configured (`icp_ledger_fee_e8s`) and not cached from `icrc1_fee`. Listed as L-1 |
| IL-5 | Allowance scoped to `S(purpose, beneficiary)` | ✅ | deposit.rs:26–40. Spawn uses `S(spawn, caller)`, top-up `S(topup, aaa)`, auto `S(auto, aaa)`. Test: `t5_6_*` (04 §5 #3) |
| IL-6 | Deposit sweeps drain `D(purpose, beneficiary)` to that beneficiary only | ✅ | api.rs:577 (`topup_pull_amount`), :867 (`spawn_pull_amount`). Tests: `t5_3_*`, `t5_4_*` |
| IL-7 | CMC memos and notify idempotency | ✅ | cmc.rs memo tests. Notify is retried on the same block. `Refunded` → `Refunded{block}` |

## 3. Money-path specifics

| # | Item | Status | Evidence |
|---|---|---|---|
| MP-1 | Mandate 30-day cap and interval enforced **before** any funds move, including in-flight ops | ✅ (fixed M-1) | `check_eligible` runs, then `mandate::reserve` (records the spend and `last_auto_at`) is persisted before the pull (api.rs:862, mandate.rs:112). The reservation is released on a definitive reject or a CMC refund. Test: `t5_7_reserve_counts_an_in_flight_auto_topup_against_interval_and_cap` |
| MP-2 | Invites: burned before pull, single-use, one sponsored AAA per owner | ✅ (fixed M-2/M-3) | `invites::sponsor_spawn` (invites.rs:176) validates the code, rate and daily budget read-only, then burns and marks. A definitive treasury reject un-marks the owner (api.rs:907). Test: `t5_7_sponsor_spawn_does_not_burn_the_code_when_budget_or_rate_refuses`, `t5_16_*` |
| MP-3 | `admin_treasury_withdraw` gating/logging/dedupe | ✅ (fixed M-4) | admin-only, intent+outcome audit, fixed `created_at_time`, non-zero u64 amount |
| MP-4 | 04 §5 #3 allowance hijack | ✅ | `t5_6_*` third-party test |
| MP-5 | 04 §5 #4 kill platform mid-spawn → resume, no double charge | ✅ | `t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure`, `t5_6_*` |
| MP-6 | 04 §5 #7 concurrent top-ups can't over-pull an allowance | ✅ | `t5_6_*` guard test |

## 4. Defects

| Id | Sev | Defect | Fix | Test |
|---|---|---|---|---|
| C-1 | Critical | Every pull (`icrc1_transfer`, `icrc2_transfer_from`) and `admin_treasury_withdraw` used a fresh `time()` as `created_at_time`, and for Deposit/Wallet top-up recomputed the amount on each attempt. A timed-out (Unknown) call was treated as an error, and `resume` or the timer re-sent a **different** transaction. The Wallet and Invite paths could pay twice; the Deposit path could get stuck with its funds moved (L-028). | Fixed intent: `created_at_time = op.created_at`, amount pinned via `fix_pull` before the await; Unknown keeps the op resumable | `t5_7_fix_pull_*`, `t5_7_classify_*`, `t5_7_treasury_withdraw_retry_*` |
| H-2 | High | A definitive ledger reject (e.g. `InsufficientAllowance`) left the op `Pending`. The 5-minute timer kept retrying it forever, so a later approval (e.g. for a retry spawn) was pulled by the **dead** op too: a double charge or double spawn. | Rejects → `Failed` (auto: needs_attention + release) | `t5_7_a_rejected_wallet_pull_fails_the_op_and_is_never_retried` |
| H-3 | High | `admin_pause` flags were stored but never enforced. The S1 kill switch was a no-op. | `ensure_not_paused` on the spawn/top_up/auto entry points | `t5_7_pause_flags_block_new_spawn_topup_and_auto_topup` |
| H-4 | High | The resume sweep only looked at ops 0–49 (`list_by_created(None, 50)`), so stuck ops after the first 50 were never resumed. | `resumable_from` + watermark | `t5_7_resumable_from_*` |
| M-1 | Medium | The auto top-up cap and interval counted only `Done` ops. An in-flight or stuck op (Unknown pull) let the AAA request another pull. | Reserve the spend and interval before the pull | `t5_7_reserve_*` |
| M-2 | Medium | The invite code was burned before the daily-budget/rate check. A refusal consumed the code and marked the owner "sponsored" forever. | Read-only checks first, burn last | `t5_7_sponsor_spawn_*` |
| M-3 | Medium | A treasury reject on an Invite spawn left the owner permanently marked sponsored. | `fail_spawn` un-marks the owner | covered by `t5_7_sponsor_spawn_*` (`unmark_sponsored`) |
| M-4 | Medium | Withdraw: no intent log before the await, no dedupe on an admin retry, and an amount > u64 silently saturated. | Intent/outcome audit, `created_at_time` arg, strict u64 | `t5_7_treasury_withdraw_retry_*` |

The fix commit is the T5.7 commit (`T5.7: internal payments security review + fixes`). All fixes are in that one commit.

## 5. Open items (not fixed, accepted)

- **L-1 (low):** the ledger fee comes from `Params.icp_ledger_fee_e8s` (admin-set) and is not refreshed from `icrc1_fee` or updated on `BadFee`. Each call still self-corrects once. Fix when the fee changes on mainnet or with D-tasks.
- **L-2 (low):** the O(n) scans in `admin_overview`/`admin_list_ops` and the unpruned `AUTO_TOPUP_HISTORY` (CS-12). Add an index and a prune-on-write when ops exceed ~100k.
- **L-3 (low):** `admin_journal_demo` (a T5.1 demo) is still exposed. It is admin-only, but it creates synthetic ops. Remove it before staging (T7.x).
- **L-4 (low):** a `Pending` pull that stays Unknown for more than 24 h gets `TooOld` and is marked `Failed`, although the first attempt may have landed. Admins must reconcile these by hand using `failed_ops` in `admin_overview` and the ledger. The 5-minute resume makes this very unlikely.
- **L-5 (low):** `Params.validate` casts `u128` cycles to `u64` with `as`, so admin-set values above `u64::MAX` bypass the range check. This is admin-only.
- **R-1 (risk):** a single admin can add admins, and a single admin can withdraw the whole treasury. There is no 2-of-n rule in payments; the treasury keeper (12) has one. This is mitigated by S11 (hardware-wallet admins) and the audit log. Revisit before production (T7.5).
- **R-2 (risk):** a Deposit spawn/top-up called before the funds arrive now ends `Failed` (no funds moved). The user retries after depositing. This is the intended behaviour.
- **R-3 (risk):** mandates are not revoked when an AAA is deleted or changes owner. The payer's allowance is still bounded by `S(auto, aaa)`.

## 6. Sign-off

All items in the 08 §4 gate are ✅ or accepted ⚠️ with the owners above. Acceptance tests 04 §5 #3, #4 and #7 pass (`t5_6_*`, `t5_3_*`). **Checklist signed: internal review T5.7, 2026-09-27.**
