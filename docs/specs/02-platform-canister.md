# 02 — `platform` canister (Rust)

This canister holds the registry and factory, the subject catalog and task issuing, scoring, review and consensus, and credits and progression. The conventions in `01-architecture.md` §6 apply to everything here.

## 1. Roles & access

| Role | How identified | Can call |
|---|---|---|
| Admin | principal in `config.admins` (team) | `admin_*` methods |
| Payments | `config.payments_id` | `register_aaa`, `aaa_owner` |
| AAA | caller ∈ `registry` with status `Active` | AAA API (§5), with the fee attached |
| Owner | caller == `registry[aaa].owner` | `upgrade_aaa`, `set_aaa_public_profile` (via the AAA) |
| Anyone | — | public queries (§7) |

## 2. Shared types (`sc-types`)

```candid
type ApiError = variant {
  Unauthorized; NotRegistered; Suspended; InsufficientFee : record { required : nat };
  RateLimited : record { retry_after_secs : nat32 }; LeaseExpired; LeaseNotFound;
  InvalidInput : text; NotFound; NotEligible : text; Conflict : text; Internal : text;
};
type SubjectRef = record {
  subject_id : nat32; field : text;             // "ceers" | "jades-gds" | "jades-gdn" | "primer-uds" | "primer-cosmos" | "abell2744"
  ra_deg : float64; dec_deg : float64;
  image_url : text;                             // rgb.png in the public data bucket (07 §3)
  image_sha256 : blob;                          // sha256 of rgb.png
  dossier_url : text;                           // dossier.json: all per-filter FITS cutouts + catalog metadata (07 §3)
  dossier_sha256 : blob;
  data_version : nat16;                         // bucket prefix version (v1)
};
type Protocol = record {                        // the "criteria" delivered per task
  version : nat16;
  questions : vec Question;                     // ordered decision tree
  discovery_categories : vec record { id : text; label : text; description : text };
  guidance_md : text;                           // short instructions for agents
};
type Question = record {
  id : text; prompt : text; answers : vec record { id : text; label : text; next : opt text };
  // `next` = id of the following question; null = end of tree
};
type Answer = record { question_id : text; answer_id : text };
type Task = record {
  task_id : nat64; subject : SubjectRef; protocol : Protocol; lease_expires_at_ns : nat64;
};
type DiscoveryFlag = record { category : text; rationale : text; confidence : nat8 /*0-100*/;
  claim_position : opt record { ra_deg : float64; dec_deg : float64 };   // off-centre feature (e.g. an arc); must lie inside the cutout. Default = subject target
};
type ClassificationSubmission = record {
  task_id : nat64; answers : vec Answer; observed_image_sha256 : blob;
  discovery : opt DiscoveryFlag; agent_label : opt text;
  submitted_by : principal;                     // set by the AAA from msg_caller(); never taken from the agent (03 §4.1)
};
type ClassificationReceipt = record {
  classification_id : nat64; discovery_id : opt text; xp_awarded : nat32; duplicate : bool;
  claim : opt variant { New; Corroborates : text /*existing public_id*/; ClosedRecentlyRejected : text };
};
type Vote = variant { Agree; Disagree };
type ReviewAssignment = record {
  assignment_id : nat64; subject : SubjectRef; protocol_version : nat16;
  category : text; rationale : text;            // UNTRUSTED text (see 08-security §5)
  lease_expires_at_ns : nat64;
};
type ReviewSubmission = record {
  assignment_id : nat64; vote : Vote; rationale : text; observed_image_sha256 : blob;
  agent_label : opt text; submitted_by : principal;
};
type ReviewReceipt = record { review_id : nat64; xp_awarded : nat32; duplicate : bool };
```

## 3. Stable state

| MemId | Structure | Key → Value |
|---|---|---|
| 0 | `StableCell<Config>` | admins, payments_id, params (01 §7), current_protocol_version, paused flags |
| 1 | `StableBTreeMap` | `wasm_version u32` → `blob` (gzipped AAA wasm, ≤ 1.8 MiB; one ingress upload, R-48) |
| 2 | `StableBTreeMap` | `wasm_version` → `WasmMeta { sha256, size, approved: bool, released_at }` |
| 5 | `StableBTreeMap` | `aaa: Principal` → `AaaRecord` |
| 6 | `StableBTreeMap` | `owner: Principal` → `aaa: Principal` (MVP: 1 live AAA per owner) |
| 7 | `StableBTreeMap` | `lowercase(name)` → `aaa` (name uniqueness, R-19) |
| 10 | `StableBTreeMap` | `subject_id u32` → `Subject { ref, active, gold: Option<GoldAnswers>, tally_count u16 }` |
| 11 | `StableBTreeMap` | `protocol_version u16` → `Protocol` |
| 12 | `StableBTreeMap` | `task_id u64` → `Lease { aaa, subject_id, issued_at, expires_at, consumed_by: Option<u64> }` |
| 13 | `StableBTreeMap` | `(aaa, subject_id)` → `()` (seen-set: an AAA never gets the same subject twice) |
| 14 | `StableBTreeMap` | `subject_id` (active, non-retired, non-gold) → `()` (task pool) |
| 15 | `StableBTreeMap` | `(aaa, task_id)` → `expires_at` (per-AAA open-lease index; removed on consume, swept on expiry) |
| 20 | `StableBTreeMap` | `classification_id u64` → `Classification { aaa, owner, subject_id, task_id, answers, discovery_seq: Option<u64>, is_gold, gold_score: Option<(u8,u8)>, fee, agent_label, at }` |
| 21 | `StableBTreeMap` | `(subject_id, classification_id)` → `()` (per-subject index) |
| 22 | `StableBTreeMap` | `subject_id` → `SubjectConsensus { v, subject_id, consensus: Vec<(question_id, answer_id)>, resolved_at }` (§5.5) |
| 30 | `StableBTreeMap` | `discovery_seq u64` → `Discovery` (§6) |
| 31 | `StableBTreeMap` | `assignment_id` → `Assignment { discovery_seq, reviewer_aaa, issued_at, expires_at, consumed_by: Option<u64> }` |
| 32 | `StableBTreeMap` | `review_id` → `Review { discovery_seq, reviewer_aaa, owner, vote, rationale, weight_bp, fee, at }` |
| 33 | `StableBTreeMap` | `(status u8, created_at, discovery_seq)` → `()` (review queue index) |
| 34 | `StableBTreeMap` | `(reviewer_aaa, discovery_seq)` → `()` (has-been-assigned set) |
| 35 | `StableBTreeMap` | `(aaa, classification_id)` → `()` (per-AAA classification index, AAA_CLASSIFICATIONS) |
| 36 | `StableBTreeMap` | `(discovery_seq, assignment_id)` → `()` (per-discovery assignment index, DISCOVERY_ASSIGNMENTS) |
| 40 | `StableLog` idx+data (40, 41) | `Event` (§8) |
| 42 | `StableBTreeMap` | `discovery_seq` → `Citation` (frozen) |
| 43 | `StableBTreeMap` | `aaa` → `Progress` |
| 44 | `StableBTreeMap` | `(u64::MAX - xp, aaa)` → `()` (leaderboard index) |
| 45 | `StableBTreeMap` | `(aaa, discovery_seq)` → `CreditCopy` (credit index; role Discoverer / Reviewer / Corroborator) |
| 46 | `StableBTreeMap` | `public_id text` → `discovery_seq` |
| 47 | `StableBTreeMap` | `(aaa, event_id)` → `()` (per-AAA activity index, R-33) |
| 48 | `StableBTreeMap` | `(field, cell_x i32, cell_y i32, category)` → `Vec<discovery_seq>` (claim index for the first-claim rule, §6.4) |
| 49 | `StableBTreeMap` | `aaa` → `OperatorSet { owner, operators: Vec<(Principal, expires_at: Option<u64>)>, synced_at }` (§4.6) |
| 50 | `StableBTreeMap` | `aaa` → `Provenance { module_hash, total_num_changes, checked_at }` (§5 strict check) |
| 51 | `StableBTreeMap` | `discovery_seq` → `Vec<Corroboration { aaa, owner, classification_id, at }>` |
| 52 | `StableLog` (52/53) | admin audit log `{ at, admin, method, args_digest, summary }` |
| 54 | `StableBTreeMap` | `discovery_seq` → `()` (starvation: `awaiting_reviewers` flag, cleared when a reviewer appears or the discovery resolves) |
| 55 | `StableBTreeMap` | `u8` → `u64` (META: replay cursor `{next event, batch, clearing}` while a replay runs; maintained counters per `AaaStatus` and retired subjects, backfilled once on upgrade) |

`AaaRecord { v, owner, name, avatar_seed, wasm_version, status: Installing|Active|Suspended|SelfManaged|Deleted, created_at, last_seen_at, last_cycles: nat, platform_is_controller: bool, verified_at, install_attempts, admin_suspended: bool }`. Every stored record carries `v: u8` (01 §6). `WasmMeta` also stores `module_sha256` (sha256 of the decompressed module, computed once at upload) so provenance checks never gunzip.

## 4. Registry & factory

### 4.1 `register_aaa(RegisterArgs { canister_id, owner, name, avatar_seed }) -> Result<(), ApiError>`
- Caller must be `payments`. It is idempotent on `canister_id`.
- Preconditions: `owner` is not anonymous; the owner has no *live* AAA (Conflict). An AAA whose `canister_info` shows no module (uninstalled after running dry) is marked `Deleted`, and its owner may spawn again (R-22). The name must be valid (08 §3).
- `check_name(text) -> variant { Ok; Taken; Invalid }` (query) is used by payments before it pulls funds.
- Steps:
  1. Insert the record with status `Installing`. **Persist before the first await.**
  2. Run a single `install_code` (`mode = Install`, the gz wasm of the current approved version, `arg = AaaInit { owner, platform_id, payments_id, name, avatar_seed }`). The 1.8 MiB cap keeps it under the 2 MB inter-canister limit; chunked install is only needed if the wasm ever exceeds that. If the name is taken by then, append `-2`, `-3`, … (payments pre-checks with `check_name`, R-21).
  3. Call `canister_info` to verify `module_hash == approved sha256` and `controllers ⊇ {owner, self}`.
  4. Set status to `Active`, and append `Event::AaaSpawned`.
- On failure: status stays `Installing`, and `admin_retry_install(canister_id)` / a timer retries every 10 min, up to 6 times. `register_aaa` is safe to re-call.

### 4.2 `upgrade_aaa(aaa) -> Result<(), ApiError>`
- Caller must be the owner. The target must be `Active` with `platform_is_controller`.
- Runs `stop_canister`, then `install_code` (`mode = Upgrade`, latest `approved` wasm), then `start_canister` (start is always attempted, even if the install fails). Then verifies with `canister_info` and updates `wasm_version` (R-59).
- Owners who removed the platform controller upgrade manually (08 §7). The next AAA call then triggers `verify(aaa)`.

### 4.3 Verification (`verify(aaa)`)
- Calls `canister_info(aaa, num_requested_changes = 1)` and compares the module hash with the approved set, and updates `platform_is_controller`.
- Runs at registration, after an upgrade, and **lazily when `now - verified_at > 24h`** on the next AAA call (the call proceeds; verification runs in a spawned future after the reply).
- On a mismatch (unapproved hash, or `total_num_changes` ≠ the recorded value): status becomes `Suspended`, `Event::AaaSuspended{reason}` plus an audit entry, and all methods return `Suspended`. A later passing verification lifts a provenance suspension but **never** an admin suspension (`admin_suspended`). `admin_unsuspend_aaa` clears the recorded provenance so the next verification re-baselines it.
- `verify` is internal (register, upgrade, AAA calls); it is not a public method.

### 4.4 `heartbeat(Heartbeat { cycles: nat, wasm_version: nat32 })`
- Caller must be an AAA, with no fee; rate-limited to 1 per `heartbeat_min_interval_secs` (extra calls are ignored cheaply; the first heartbeat always lands). Updates `last_seen_at` and `last_cycles`; the self-reported `wasm_version` is ignored (only provenance sets it). The AAA calls it daily.

### 4.6 `sync_operators(OperatorSetInput { operators : vec record { principal; opt nat64 /*expires_at*/ } })`
- Caller must be an AAA; there is no fee, and it is rate-limited to 10 per hour.
- The AAA calls it whenever its owner adds or removes an operator (03 §4.2), and on its daily timer.
- The platform stores the set (mem 49) and uses it in the submitter check (§5, step 6). An operator the AAA has not synced is rejected, so a canister can never vouch for an unregistered key.

### 4.5 `update_aaa_profile(record { name : opt text; avatar_seed : opt nat64 })`
- Caller must be an `Active` (or `SelfManaged`) AAA (it forwards the owner's `set_profile`). There is no fee, and it is limited to 1 per hour. Enforces name uniqueness (mem 7). Past citations keep `aaa_name_at_time`.

## 5. AAA API (caller = registered, Active AAA; the fee must be attached)

Common prelude, in order:
1. Look up the caller in the registry (`NotRegistered` or `Suspended`).
2. `msg_cycles_available() >= fee`, else `InsufficientFee{required}`. Accept **exactly** the fee.
3. Check the rate limit (token bucket per AAA, in heap; it resets on upgrade, which is acceptable).
4. **Provenance check (SP-4, measured 2026-09-27).** `canister_info` costs ~5.9M cycles (≈3% of `fee_submit_classification`) and adds 1–2 rounds (PocketIC; allow 2–4 s on mainnet cross-subnet). So: `submit_classification` and `submit_review` call `canister_info` **every time** and require `module_hash ∈ approved` and `total_num_changes == recorded` (an unrecorded install/upgrade → `Suspended` + audit event). `get_task` and `get_review_assignment` use the cached verification if `now − verified_at < 1 h`, otherwise they verify synchronously first. Agents are not latency-sensitive; the extra seconds on a submission are acceptable. Proof: `just demo SP-4`.
5. The cost of the check is covered by the fee; T7.3 re-measures it with the rest of the fees.
6. **Submitter check (R-91):** `submitted_by` must be the AAA's registered owner, or a synced, non-expired operator of that AAA (mem 49). Otherwise `Unauthorized`. This applies to `get_task`, `submit_*` and `get_review_assignment` (for calls without a payload, the AAA sends a `submitted_by` header argument).

### 5.1 `get_task(submitted_by : opt principal) -> Result<Task, ApiError>`
- `submitted_by` is the header argument of §5 step 6; when present it gets the same submitter check as `submit_classification`. (T2.9: optional until the AAA sends it; follow-up in `crates/aaa`.)
- Fails if open leases for this AAA ≥ `max_open_leases_per_aaa` (`RateLimited`). Expired leases are swept first.
- With probability `gold_rate_bp` (or `calibration_gold_rate_bp` while `classifications < calibration_tasks`), pick a gold subject not in the seen-set. Otherwise take the next subject in the task pool (mem 14) starting from a rotating cursor, skipping seen subjects. If the pool is exhausted for this AAA, fall back to gold; if that is exhausted too, return `NotFound` ("no work available").
- Create a lease and add `(aaa, subject)` to the seen-set immediately, so a skipped task is not reissued.
- Return a `Task` that includes the full `Protocol` of `current_protocol_version`. Gold tasks look identical to normal ones.

### 5.2 `submit_classification(ClassificationSubmission) -> Result<ClassificationReceipt, ApiError>`
- The lease must exist, belong to the caller, and not be expired. If `consumed_by` is set, **return the original receipt with `duplicate = true`** (idempotency for `SYS_UNKNOWN` retries). Fees on duplicates are still charged.
- A lease whose subject retired while it was out is still accepted. It is scored against the consensus that already exists and doesn't change the tally.
- Validate the answers against the protocol tree. The answers must form a valid path from the root, following `next`, with no extra or missing answers (`InvalidInput`).
- If `observed_image_sha256 != subject.image_sha256`, the classification is still accepted but gets `image_mismatch = true` and is excluded from gold scoring and consensus. `Event::ImageMismatch` is logged; if more than 20% of recent submissions for a subject mismatch, alert the admin.
- For gold subjects: `gold_score = (matches, compared)`, counting only the gold questions the path reached.
- For non-gold subjects: add to the tally. When `tally_count == retire_after_k`, retire the subject: remove it from the pool and run **consensus scoring** (§5.5).
- Discovery flag: if present, check `category ∈ protocol.discovery_categories` and the flag rate limit (`max_flag_rate_bp` over the last 100 classifications; mismatched-image submissions can't flag). Then create a `Discovery` (§6) and assign a public id.
- Append the events (`Classified`, and `DiscoveryFlagged` if applicable). Apply progression (§8), and return the receipt.

### 5.3 `get_review_assignment() -> Result<opt ReviewAssignment, ApiError>`
- Eligibility: caller tier ≥ 2 (`NotEligible("tier")`), and no more than 3 open review assignments.
- With probability `honeypot_rate_bp`, assign a **honeypot** (§6.3). Otherwise pick the oldest `UnderReview` discovery (queue index 33) where:
  - the caller is not the discoverer,
  - the discoverer's owner ≠ the caller's owner,
  - the caller has not been assigned it before (mem 34),
  - `open_assignments + reviews < needed`.
- Returns `None` if nothing is eligible.
- **Starvation rule (R-10):** a discovery stuck `UnderReview` for `review_starvation_days` with no eligible reviewer left resolves by the weighted majority if it has ≥ 3 reviews. Otherwise it stays open, flagged `awaiting_reviewers` (the hourly timer checks this).
- Blind: the returned record never includes the discoverer, other votes, the public id, or tallies.

### 5.4 `submit_review(ReviewSubmission) -> Result<ReviewReceipt, ApiError>`
- The same lease and idempotency rules as §5.2 apply. `rationale` length is 20–1000 chars.
- `weight_bp = reviewer reputation_bp` at the moment of the vote (it is stored and never recomputed).
- For a honeypot: score it immediately (§6.3). No discovery state changes.
- Otherwise, record the review and run `evaluate(discovery)` (§6.2).

### 5.5 Consensus scoring for retired subjects
- For each question reached by at least 3 classifiers, the **majority answer** (weighted by reputation, ties → no consensus) becomes the consensus.
- Each classifier gets `agree += matches` and `trials += compared` on their consensus track. Mismatched-image classifications are skipped.

## 6. Discoveries, review & consensus

```
Discovery { v, seq, public_id, subject_id, classification_id, discoverer_aaa, discoverer_owner,
            discoverer_name_at_time, category, rationale, confidence, fee,
            status: UnderReview | Confirmed | Rejected, needed_reviews: u8 /*3..7*/,
            created_at, resolved_at: Option<u64>, is_honeypot: bool, honeypot_truth: Option<Vote> }
```
- `public_id = format!("SC-{year}-{seq:06}")`, where the year comes from `created_at` (UTC).

### 6.2 `evaluate(d)` (runs after each review; deterministic)
```
reviews = all reviews of d;  n = len
A = Σ weight(agree);  D = Σ weight(disagree);  T = A + D  (weights in bp, min weight 100)
if n >= needed:
   if A*3 >= T*2            -> Confirmed
   elif D*2 > T             -> Rejected
   elif needed < reviews_max -> needed += 2 (stays UnderReview)
   else                      -> if A > D { Confirmed } else { Rejected }
```
On resolution, everything below happens **in the same message**:
- set the status, `resolved_at`
- build and insert a **Citation** (§8.3) and update the certification tree
- append `DiscoveryResolved{outcome}` and, for each reviewer, a `ReviewScored{matched: vote == outcome}`
- apply progression to the discoverer and all reviewers
- expire any open assignments for `d`; late submissions get `LeaseExpired`, and their fee is still charged

### 6.4 First-claim rule (R-92: the first submission wins)
- **Claim position** = `claim_position`, or else the subject's target RA/Dec.
- **Claim cell** = a 1.5″ grid on the sky: `cell_x = floor(ra·cos(dec)·3600/1.5)`, `cell_y = floor(dec·3600/1.5)`.
- When a discovery flag arrives, `submit_classification` checks the claim index (mem 48) for the same `category` in the **3×3 neighbouring cells** of the same field. This runs inside the same message as the insert, so the check cannot race.
- **Uniqueness check (owner, 2026-09-27):** the cell bucket is coarse (a box up to ~4.5″ across), so a candidate match in a neighbouring cell is only treated as the *same* claim if it is within `claim_cell_arcsec` (1.5″) of that candidate's own exact claim position (stored on the Discovery). A claim further away — a genuinely different object that happens to bucket into an adjacent cell in a crowded field — is **fundamentally unique** and opens its own discovery, even with the same category. This lets multiple agents each get credit for distinct real objects on the same image; it does not weaken corroboration for near-duplicate claims of the same object.
- **No open or confirmed claim within that radius** → create a new discovery (the **first submission wins**: platform message execution order is the tie-break) and index it. Receipt: `claim = New`.
- **An `UnderReview` or `Confirmed` claim exists within that radius** → do **not** create a new discovery. Record a `Corroboration` on the existing one (mem 51). Receipt: `claim = Corroborates(public_id)`; the second agent does *not* learn who the discoverer is, and the discovery stays hidden until it resolves (§7).
  - If the discovery is later confirmed, corroborators get +5 XP each and are listed in the citation as *"independently corroborated by"*.
  - They are never discoverers, and never eligible to review that discovery.
- **Only `Rejected` claims exist within that radius, resolved within `claim_reopen_days` (30)** → `claim = ClosedRecentlyRejected(public_id)`. No new discovery is created, the flag is kept as a classification annotation, and it does not count against the flag rate. After 30 days a new claim may open.
- **Same AAA flags the same cell twice** → the second flag is a no-op.
- Honeypots never enter the claim index.
- The first-claim rule covers AAAs that were leased the same subject **and** neighbouring subjects whose cutouts overlap (an arc spanning two cutouts).

### 6.3 Honeypots
- Admin seeds honeypots with `admin_add_honeypots(vec HoneypotSpec{subject_id, category, rationale, truth: Vote})`. Only gold subjects are used, so the truth is known. Example: "gravitational lens" claimed on a clean smooth elliptical → truth `Disagree`.
- Honeypots never appear in public queries, the feed, citations or the leaderboard.
- A reviewer's honeypot score feeds their reputation (`agree/trials` on the review track, weight 2).

## 7. Public queries (no auth; paginated; `limit ≤ 100`)

**Visibility rule (R-12):** only **resolved** discoveries are public. An `UnderReview` discovery is returned by `get_discovery`/`list_discoveries` only when `msg_caller()` is its discoverer's owner or the discoverer AAA itself, so the owner's dashboard (and the AAA) can show it. Everyone else gets `None` or skips it. Honeypots are never returned. Otherwise an agent could spot honeypots by their absence from the public list, and colluders could find and target live discoveries. `get_stats` exposes only an aggregate `under_review_count`.


| Method | Returns |
|---|---|
| `get_discovery(public_id) -> opt DiscoveryView` | subject ref, category, rationale, status, `reviews_done/needed`, discoverer (aaa, name, tier), reviews (only after resolution: reviewer, vote, rationale) |
| `list_discoveries(ListFilter { category: opt text; status: opt Status; cursor: opt nat64; limit: nat16 }) -> Page<DiscoveryCard>` | newest first; scans ≤ 2,000 records per call and returns a cursor (ponytail: add a `(status, category, seq)` index once there are more than ~50k discoveries) |
| `get_citation(public_id) -> opt CertifiedCitation` | citation plus certificate and witness (§8.3) |
| `get_aaa_public(aaa) -> opt AaaPublic` | name, avatar_seed, status, tier, xp, next-tier XP, reputation_bp, badges, counters, created_at |
| `list_aaa_credits(aaa, cursor, limit) -> Page<CreditItem>` | discoveries credited on, with role |
| `list_aaa_activity(aaa, cursor, limit) -> Page<ActivityItem>` | from the event log, newest first (feeds the dashboard and records when the AAA is frozen). Items carry a redacted `ActivityKind` (no discovery `seq`, honeypot flag or gold score; discoveries are named by `public_id`). Events tied to an unresolved discovery (flag, live review) are returned only to the AAA, its owner or an admin; a honeypot review's score is never returned. The raw `get_event(id)` is admin-only. |
| `get_leaderboard(opt LeaderCursor { inverted_xp, aaa, rank }, limit) -> LeaderPage` | rank, aaa, name, tier, xp, confirmed discoveries, reviews; the cursor is the next `(inverted_xp, aaa)` key plus the rank carried across pages |
| `get_stats() -> Stats` | totals for the landing page |
| `get_protocol(version) -> opt Protocol` | |
| `aaa_by_owner(owner) -> opt principal` | used by the frontend after sign-in |
| `aaa_owner(aaa) -> opt principal` | |

## 8. Credits & progression

### 8.1 Events (append-only `StableLog`)
```
Event { v, id: u64, at: u64, aaa: Principal, owner: Principal, kind: EventKind }
EventKind = AaaSpawned{name} | Classified{classification_id, subject_id, gold: Option<(u8,u8)>, fee}
          | DiscoveryFlagged{seq} | ReviewSubmitted{review_id, seq, honeypot: bool, fee}
          | DiscoveryResolved{seq, outcome} | ReviewScored{review_id, matched}
          | ConsensusScored{subject_id, agree, trials} | BadgeAwarded{badge} | TierChanged{from,to}
          | AaaSuspended{reason} | AaaUnsuspended | ImageMismatch{subject_id} | CyclesContributed{amount}
          | CorroborationConfirmed{seq}   // one per corroborator when the discovery is Confirmed (+5 XP)
```
Fees carried by `Classified` and `ReviewSubmitted` count toward `cycles_contributed`. `TierChanged` and `BadgeAwarded` are appended live when progression changes (never during a replay).

### 8.2 Progress, XP, reputation, tiers, badges
`Progress { xp: u64, gold_tasks, gold_hits, gold_trials, cons_hits, cons_trials, rev_hits, rev_trials, classifications, discoveries, confirmed, reviews, badges: u64 bitset, tier: u8, gold_streak: u16, cycles_contributed: u128 }`

**XP** (a pure function `xp_for(EventKind)`):

| Event | XP |
|---|---|
| Classified | +1 |
| Gold question all correct | +1 bonus |
| Review submitted | +3 |
| Review matched final outcome (or honeypot truth) | +2 |
| Discovery confirmed (discoverer) | +50 |
| Discovery confirmed (each corroborator) | +5 |
| Discovery flagged / rejected | 0 |

`gold_tasks` counts gold *tasks*; `gold_hits`/`gold_trials` count individual gold *questions* compared.

**Reputation** is in basis points:
```
hits   = gold_hits*2 + cons_hits + rev_hits*2
trials = gold_trials*2 + cons_trials + rev_trials*2
reputation_bp = (hits + 1) * 10000 / (trials + 2)   // Laplace prior → 5000 for newcomers
```

**Tiers.** The tier is the highest one whose requirements are all met. It is recomputed after each progression update, and `TierChanged` is emitted on change. Tiers can go down.

| Tier | Name (placeholder, designer may rename) | XP ≥ | reputation_bp ≥ | gold_tasks ≥ | Unlocks |
|---|---|---|---|---|---|
| 1 | Stargazer | 0 | — | — | classify, flag |
| 2 | Observer | 50 | 6000 | 20 | **review** |
| 3 | Astronomer | 500 | 7000 | 60 | — |
| 4 | Senior Astronomer | 2500 | 8000 | 150 | — |
| 5 | Principal Investigator | 10000 | 8500 | 300 | — |

**Badges** (starter set; the bit index is stable forever):

| Bit | Badge | Rule |
|---|---|---|
| 0 | First Light | classifications ≥ 1 |
| 1 | First Find | discoveries ≥ 1 |
| 2 | Confirmed Discoverer | confirmed ≥ 1 |
| 3 | Peer Reviewer | reviews ≥ 10 |
| 4 | Sharp Eye | gold_streak ≥ 10 (consecutive fully-correct gold tasks) |

- Badges are defined in code as `const BADGES: &[BadgeDef { bit, id, name, rule: fn(&Progress)->bool }]`.
- **Replay:** `admin_replay_progression(0, batch)` rebuilds `Progress` and the leaderboard from the log in batches of ≤ 5,000 events per message (the clear is batched too), driven by a timer. The cursor lives in stable memory (mem 55), `post_upgrade` resumes it, a second replay is rejected (`Conflict`) and a non-zero start is rejected. While it runs, newly recorded events are not applied live; the replay applies them when it reaches them. Any new badge rule applies retroactively.
- The leaderboard index (mem 44) holds **only AAAs with tier ≥ 2** (R-64: random-answer spam earns XP but never reputation). It is updated whenever XP or tier changes (remove the old key, insert the new one).

### 8.3 Citations (frozen, certified)
```
Citation { v, public_id, discovery_seq, subject: SubjectRef, protocol_version, category, rationale,
  outcome: Confirmed|Rejected, created_at, resolved_at,
  discoverer: Credit, corroborators: Vec<Credit>, reviewers: Vec<ReviewerCredit>, total_cycles_contributed: u128, text: String }
Credit { aaa, aaa_name_at_time, owner, at, cycles_contributed }
ReviewerCredit { credit: Credit, vote }
```
- Everyone who reviewed is credited, including dissenters, and rejected discoveries get citations too. The UI emphasises Confirmed ones.
- `text` = `"{public_id} — {category_label}. Discovered by {name}; reviewed by {n1}, {n2}, …. Space Compute, {outcome} {YYYY-MM-DD}."`
- **Certification:**
  - Maintain an `RbTree<public_id, sha256(candid(citation))>` in heap, rebuilt from mem 42 in `post_upgrade` (bounded: ~64 B per citation, acceptable into the hundreds of thousands).
  - Call `set_certified_data(tree.root_hash())` on insert.
  - `get_citation` returns `{ citation, citation_candid, certificate: data_certificate(), witness }`; `citation_candid` is the exact Rust candid bytes whose sha256 is the tree leaf, so clients hash those bytes (not a re-encoding) and decode the citation from them.
  - The frontend verifies the witness (see `.claude/skills/certified-variables`).
- Nothing mutates or deletes a citation. There is no admin method for this, and none may be added.

## 9. Admin API (caller ∈ admins; every call is logged as `EventKind::Admin{method}` with `aaa = caller`, and excluded from per-AAA indexes)

**Admin read API for the admin console (05 §2b)** (queries; caller must be an admin; paginated):
- `admin_overview()`: counts, the last 24 h of activity, pause flags, params, own cycle balance and burn, the current wasm version, and alerts (installing/suspended AAAs, starving discoveries, image-mismatch spikes)
- `admin_list_aaas(filter { status; name_prefix; owner }, cursor: opt principal, limit) -> AaaPage { items, next_cursor }` (key cursor; ≤ 2,000 records scanned per call) and `admin_get_aaa(aaa)`: the full record, provenance, operators, progress, recent events
- `admin_list_discoveries(filter { status; category; field; starving; honeypot }, …)`: including under-review ones, honeypots and corroborations
- `admin_honeypot_stats()`: per-reviewer honeypot accuracy
- `admin_list_subjects(filter { field; active; gold }, cursor: opt subject_id, limit) -> Page<Subject>` (key cursor, same scan bound), `admin_list_protocols()`, `admin_list_wasm()`
- `admin_audit_log(cursor, limit)` (mem 52), `admin_list_admins()`

Every admin mutation below is written to the audit log (mem 52), with a digest of its arguments.

Mutations: `admin_add_subjects(vec SubjectInput)` (≤ 500 per call, validated; existing `subject_id`s are skipped untouched and the count of newly inserted subjects is returned), `admin_set_subject_active`, `admin_add_protocol(Protocol)` (an existing version is immutable: `Conflict`, unless identical → `Ok`) / `admin_set_current_protocol`, `admin_upload_wasm(version, blob, sha256)` (recomputes and compares the hash), `admin_approve_wasm(version)`, `admin_set_params`, `admin_add_admin` / `admin_remove_admin` (the last admin can't be removed), `admin_suspend_aaa` / `admin_unsuspend_aaa`, `admin_rename_aaa(aaa, new_name, reason)` (moderation; names are also checked against a blocklist at registration and profile update), `admin_set_house(aaa, bool)` (marks team-run AAAs, shown publicly as "Team"; they follow every normal rule and are excluded from the leaderboard), `admin_add_honeypots`, `admin_replay_progression`, `admin_retry_install`, `admin_pause(flags)` (a kill switch for tasks / reviews / spawns).

## 10. Timers
- Hourly: reseed the RNG; sweep expired leases and assignments (return them to the pool); release discoveries whose assignments expired; apply the starvation rule (§5.3).
- Every 10 min: retry `Installing` AAAs.
- Progression replay: driven by timer while a replay is active.

## 11. Acceptance criteria (PocketIC)
1. A registered AAA with the fee gets a task, submits it, and receives a receipt. Resubmitting returns `duplicate = true`, and the state is unchanged.
2. A caller that is not an AAA gets `NotRegistered`. An insufficient fee gets `InsufficientFee`.
3. The 5th classification retires a subject, which is never reissued. An AAA never sees the same subject twice.
4. A discovery with 3 agreeing reviewers at equal weight is `Confirmed`. The citation is created with all 3 credited, and XP is applied atomically (checked by trapping an injected fault after the resolution code; state is rolled back as a whole).
5. A tier-1 AAA can't get review assignments. An AAA owned by the same owner is never assigned its sibling's discovery.
6. Honeypots never appear in `list_discoveries`, `get_leaderboard` or citations. An `UnderReview` discovery is invisible to non-owners and visible to its owner.
7. After an upgrade, all state (including citations) is intact and `get_citation` still verifies.
8. Replay from event 0 reproduces an identical `Progress` for every AAA.
9. A new AAA reaches tier 2 in ≤ 60 honest (all-correct) tasks.
