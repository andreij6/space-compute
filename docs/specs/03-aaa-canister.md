# 03 — `aaa` canister (Rust, one per user)

The AAA is the user's **authenticated relay** to `platform` and their **personal repository** of work. The user owns it and pays its cycles. Target wasm: **≤ 1.5 MiB gzipped** (built with `ic-wasm shrink` and gzip), so the platform can install it in one call.

## 1. Init
```candid
type AaaInit = record { owner : principal; platform_id : principal; payments_id : principal;
                        name : text; avatar_seed : nat64 };
```
- `#[init]` stores the config and starts the timers. `#[post_upgrade]` restarts the timers and keeps the state.

## 2. Roles
| Role | Check | Capabilities |
|---|---|---|
| Owner | `caller == owner` | everything |
| Operator | `caller ∈ operators` | work methods (§4.1) and read methods only |
| Platform | `caller == platform_id` | none in the MVP (reserved) |
| Anyone | — | `get_api_doc`, `status_public` |

`#[inspect_message]` rejects ingress to update methods from callers that are neither owner nor operator. This is only a cycle-saving pre-filter; every method still checks the role itself.

## 3. Stable state
| MemId | Structure | Content |
|---|---|---|
| 0 | `StableCell<Config>` | owner, platform_id, payments_id, name, avatar_seed, agent_label, auto_topup: Option<threshold_cycles>, wasm_version |
| 1 | `StableBTreeMap<Principal, Operator>` | `Operator { label: String (≤32), added_at, last_used_at }` (max 5) |
| 2 | `StableBTreeMap<u64, Record>` | local repository (§5), key = local seq |
| 3 | `StableBTreeMap<nat64 task_id, u64 seq>` | idempotency index |
| 4 | `StableCell<Stats>` | counters, last_activity_at, last_heartbeat_at, credits_cursor |
| 5 | `StableBTreeMap<String public_id, CreditCopy>` | copy of citations this AAA is on |

## 4. API

### 4.1 Work methods (owner or operator). Each forwards to `platform` with the fee attached.
```candid
get_task : () -> (Result_Task);                                   // fee_get_task
submit_classification : (ClassificationSubmission) -> (Result_ClassificationReceipt);
get_review_assignment : () -> (Result_OptReviewAssignment);
submit_review : (ReviewSubmission) -> (Result_ReviewReceipt);
```
Forwarding rules:
- Bind the caller before the await. **Set `submitted_by = msg_caller()`**, overwriting anything in the agent's payload. Reject operators whose key has expired. Then make a bounded-wait call (timeout 60 s) with `with_cycles(fee)`.
- The AAA exposes **no generic call-forwarding method**. It can only make the four platform calls listed here and the two payments calls (`request_auto_topup`, heartbeat-style status).
- **Fees come from a cached `Params` table** refreshed daily from `platform.get_params()`. On `InsufficientFee{required}`, update the cache and retry once.
- **Low-cycles guard:** if `canister_cycle_balance() < freezing_reserve + 50 × max_fee`, refuse with `Internal("low cycles: top up")` and trigger `request_auto_topup` (§6) instead of letting the canister freeze mid-work.
- On `SYS_UNKNOWN` from a submit: retry the same submission once. Idempotency is by `task_id`/`assignment_id` in `platform`, so a retry is safe.
- On success, write a `Record` to the repository and bump the stats (`operator.last_used_at`).
- `agent_label` defaults to `Config.agent_label` when the submission omits it.

### 4.2 Owner-only (`set_profile` forwards to `platform.update_aaa_profile`; a name conflict returns `Conflict`)
```candid
add_operator : (principal, text, opt nat64 /*expires_at_ns*/) -> (Result);   // max 5; rejects the anonymous and owner principals; then calls platform.sync_operators
remove_operator : (principal) -> (Result);           // takes effect locally at once, and platform-side via sync_operators
set_profile : (record { name : opt text; avatar_seed : opt nat64 }) -> (Result);  // also pushes to platform (no fee)
set_agent_label : (opt text) -> (Result);
set_auto_topup : (opt nat) -> (Result);            // Some(threshold_cycles) enables the 6h check; None disables it (R-28). The frontend sets it together with payments.set_mandate
```

### 4.3 Queries
```candid
whoami : () -> (variant { Owner; Operator; None }) query;
status : () -> (Status) query;                    // owner/operator: cycles, freezing estimate, operators, stats, version
status_public : () -> (PublicStatus) query;       // name, version, owner, last_activity
list_records : (record { kind : opt RecordKind; cursor : opt nat64; limit : nat16 }) -> (Page_Record) query;
get_record : (nat64) -> (opt Record) query;
list_credits : (opt text, nat16) -> (Page_CreditCopy) query;
get_api_doc : () -> (text) query;                 // markdown: how an agent uses this AAA (mirrors 06 skill)
```
`Status.days_of_fuel_estimate` = `(balance - freezing_reserve) / avg_daily_burn`, where `avg_daily_burn` is an EMA of the balance deltas sampled by the 6 h timer, excluding top-ups.

## 5. Records (the personal repository)
```
Record { v, seq, at, kind: Classification|Discovery|Review|TopUp|Operator|Profile,
         task_or_assignment_id: Option<u64>, subject: Option<SubjectRef>, answers: Vec<Answer>,
         discovery_public_id: Option<String>, category, vote: Option<Vote>, rationale: Option<String>,
         outcome: Option<Outcome>, xp_awarded: u32, agent_label: Option<String>, fee: u128, by: Principal }
```
- Records are append-only. Outcomes of discoveries and reviews are filled in during the credits sync (§6), as a separate `outcome` update on the same record.
- A storage quota applies: once there are more than 1,000,000 records, the oldest classification records are pruned first (discoveries and reviews are never pruned). The platform keeps the authoritative copy regardless.

## 6. Timers
- **Every 6 h:**
  - sample the cycle balance for the burn EMA
  - if auto top-up is enabled and `balance < threshold`, call `payments.request_auto_topup()` (bounded wait; errors logged in stats)
- **Every 24 h:**
  - `platform.heartbeat({cycles, wasm_version})`
  - refresh the params
  - credits sync: `platform.list_aaa_credits(self, cursor)` → upsert `CreditCopy`, update the outcomes in the matching records, advance the cursor
- Timers are cheap, and a failed timer tick never traps (every error is caught and counted).

## 7. Security notes
- The AAA never holds ICP and has no transfer methods.
- The owner may remove `platform` as a controller. The AAA keeps working; upgrades then become manual.
- An operator can't add operators, change the profile, or read anything owner-private beyond `status` (MVP: status is not sensitive).

## 8. Acceptance criteria (PocketIC)
1. A non-operator ingress to `get_task` is rejected. An operator succeeds, and the fee is deducted from the AAA's balance.
2. The owner adds and removes operators. A removed operator is rejected immediately.
3. A submit retried after a simulated `SYS_UNKNOWN` produces exactly one platform classification and one local record.
4. Records and credits survive an upgrade.
5. Below the threshold, the 6 h timer calls `payments.request_auto_topup` exactly once per interval.
6. The wasm is ≤ 1.5 MiB gzipped (checked in CI).
