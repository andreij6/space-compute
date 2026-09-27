# Progress (newest first, ≤ 5 lines per entry)

## 2026-09-27 — T3.3 AAA repository records + queries + credits copy (done)
- Assigned Mem 5 to CREDITS with CreditCopy, CreditRole, and Outcome types in stable memory.
- Repository records and credits indexing with newest-first ordering, page_limit clamping (≤100), and pruning guard (1M quota).
- Added get_record, list_records, list_credits, status, status_public, and sync_credit_copy endpoints.
- PocketIC acceptance test t3_3_records_and_credits_survive_canister_upgrade verifies state survival across upgrades; 92.8% coverage.

## 2026-09-27 — T3.2 AAA forwarding w/ fees, retries, idempotency, low-cycles guard (done)
- Forwarding work methods (get_task, submit_classification, get_review_assignment, submit_review) with low-cycles guard and fee caching.
- Stamped submitted_by = msg_caller(), default agent_label fallback, and sync_operators on operator addition/removal.
- Safe retry on SYS_UNKNOWN with idempotency index ensuring exactly one local record per task and deduplicated platform state.
- PocketIC acceptance test t3_2_one_record_per_task_under_sys_unknown + 7 unit tests pass; AAA coverage 93.2% (min 85%).

## 2026-09-27 — T2.7 Platform PocketIC tests (02 §11 #1-3) (done)
- Validated criteria 1 & 2: registration enforcement, InsufficientFee with cycle requirements, task receipt, idempotent duplicate submission.
- Validated criterion 3: subject retirement at K=5 classifications, majority consensus evaluation, never reissued to any agent.
- Verified rotating task pool and seen-set mechanics: an AAA never sees the same subject twice across tasks.
- PocketIC acceptance tests t2_7_registered_aaa_task_receipt_and_idempotent_duplicate and t2_7_fifth_classification_retires_and_seen_set_never_reissues pass.

## 2026-09-27 — T2.8 Submitter security: strict provenance, operator sync, submitted_by (done)
- Multi-layer submitter authentication enforcing check_submitter: owner or active, synced, non-expired operator required.
- sync_operators rate-limited (10/hr) with strict validation: <=5 operators, non-anonymous, non-owner, no duplicates.
- Strict provenance verification via canister_info: unapproved wasm module hash or deletion triggers suspension and AaaSuspended event.
- PocketIC acceptance test t2_8_foreign_expired_unsynced_submitter_rejected + 3 unit tests pass; platform coverage 94.4% (min 90%).

## 2026-09-27 — T2.6 Public queries: stats, protocol, aaa_by_owner, aaa_public (done)
- Public query endpoints get_stats (landing page totals), get_protocol(v), aaa_by_owner/aaa_owner, get_aaa_public, and get_leaderboard.
- Progressive XP, tiers (1-5), badges, and reputation_bp with Laplace prior (5,000 baseline) tracked in stable memory (mem 43/44).
- Strict pagination clamping (limits clamped <= 100 via page_limit) across all public list/activity endpoints.
- PocketIC acceptance test t2_6_public_queries_paged_limit_100 + 2 unit tests pass; platform coverage 93.3% (min 90%).

## 2026-09-27 — T2.5 Event log, per-AAA index, activity query (done)
- Append-only StableLog (mem 40/41) for platform events with monotonically incrementing IDs.
- Per-AAA activity index (mem 47: (aaa, event_id) -> ()) excluding admin events, strictly isolating agent activity.
- Paged list_aaa_activity query returning newest-first items with cursor-based pagination and page_limit clamped <= 100.
- PocketIC acceptance test t2_5_events_append_and_paged_activity + 3 unit tests pass; platform coverage 93.3% (min 90%).

## 2026-09-27 — T2.4 Scoring, validation, gold, tallies, retirement, consensus (done)
- Full protocol tree validation (root-to-leaf, required answers, valid options, no orphan answers) and gold scoring along visited path.
- Idempotent submit_classification (same classification_id, duplicate=true, 0 XP, unmutated tally); subject retirement at K=5 with automatic task pool removal.
- Consensus evaluation on normal subjects with ≥3 classifications, majority vote per question, retroactively updating past classification scores.
- PocketIC acceptance test t2_4_retire_at_k5_and_idempotent_submit + 3 unit tests pass; platform coverage 93.3% (min 90%).

## 2026-09-27 — T2.3 Catalog, protocol, leases, seen-set, get_task (done)
- Pure state logic in platform (mem 10 Subjects, 11 Protocols, 12 Leases, 13 SeenSet, 14 TaskPool) with rotating cursor dispatch.
- Never same subject twice: seen-set tracks (aaa, subject_id); unconsumed expired leases swept before open-lease rate limit check.
- get_task prelude verifies caller status, fee_get_task, hourly token-bucket rate limit, and cached provenance.
- PocketIC acceptance test t2_3_never_same_subject_twice_and_pool_dispatch + 4 unit tests pass; platform coverage 92.7% (min 90%).

## 2026-09-27 — T2.2 Registry & factory (done)
- Full AAA registry, wasm upload/approval, spawn registration, upgrade, verify, heartbeat, sync_operators, profile rename in platform canister (mem 1, 2, 5, 6, 7, 49, 50).
- Idempotent register_aaa with name collision auto-suffix (-2), one live AAA per owner constraint, provenance tracking with module hash and controller verification.
- PocketIC acceptance test t2_2_spawned_aaa_verified_and_upgrade_works + 10 unit tests pass; platform coverage 92.8% (min 90%).

## 2026-09-27 — T5.1 payments skeleton (done, Sonnet lane) + no Stripe code
- Journal Pending→Pulled→Credited→TreasuryPaid→Notified→Registered→Done (+Failed/Refunded), validated transitions; per-key CallerGuard; config/params/features/pause; audit log (mem 60/61).
- PocketIC proves the journal write before an await survives a trap after it. Coverage 96.1%. Commit 08d322b.
- Owner: no Stripe code until the whole app is ready → removed Card source, features.card, stripe cap, dedupe mem const; T5.12 moved to Deferred (D5). Launch 2027-02-10.

## 2026-09-27 — T1.7 dossiers rendered (done) — Foundations complete
- Owner: inclusive selection (stars, no-z, edge kept) and 5,000 subjects for v1 (20k before beta, R-62). 5,000 dossiers, 9.3 GB (inside R2's free 10 GB); 344 edge-flagged.
- All 5,000 hashes verified; 40-subject QA 0 problems. Visual QA caught what automated QA missed: RGB channels mixed 0.02″ and 0.04″ grids → common-grid RGB, re-assembled from cache in 10 min.
- R2 upload waits on owner task T8.14 (`just publish-data`). Demo: docs/demos/T1.7/qa-40.png.

## 2026-09-27 — T3.1 AAA skeleton (done, Sonnet lane)
- Roles: Owner / Operator (expiring, ≤5) / Platform / None; inspect_message pre-filters ingress to owner or active operator, every update also runs require_owner.
- Config + operators in stable memory (mem 0, 1); typed AaaInit {owner, platform_id, payments_id} passed by the factory (spawned canisters aren't in icp.yaml).
- PocketIC t3_1 role-matrix test + 11 unit tests; aaa coverage 97.6%, wasm 187 KB gz. Commit fdf33c1.

## 2026-09-27 — T2.1 platform skeleton (done)
- Config (mem 0 StableCell: admins, 22 bounded params, pause flags, protocol version), audit log (mem 52/53 StableLog, sha256 args digest), admin API (add/remove admin, set_params, pause, list, audit page, overview), public get_params.
- Installer becomes first admin (anonymous install traps); ChaCha20 RNG seeded from raw_rand by a 0 s timer + hourly, retries every 30 s on failure.
- PocketIC: upgrade keeps params/admins/pause/audit and reseeds RNG. Opus review: 3 fixes (param upper bounds, anonymous install, reseed retry).
- Coverage gate now excludes canister glue (lib/api/timers.rs), covered by PocketIC; platform 97.6%.

## 2026-09-27 — Review of the parallel session's frontend (T6.1 reset to in_progress 30%)
- Real: Vite/React build, 25 routed screens, static-site recipe, _redirects/_headers. Missing: II auth, ic_env, bindgen, tests (lint/test are echo stubs).
- All 25 screens run on mockData.ts; no canister calls. Misleading: citation block always shows "BLS Verified" (spec: fail closed); spawn "provisioning" is setTimeout; connect page shows a "Secret Operator API Key" (design uses revocable principals, no shared secrets); no admin guard; card (Stripe) tab always visible.
- Keep the pages as a visual reference for P9; build T6.x functionally when scheduled. Frontend files were swept into SP-1..SP-7 commits by my `git add -A` (shared checkout) — use explicit paths when another session is active.

## 2026-09-27 — T1.5 JWST subject selection + gold (done)
- Real DJA v7 catalogues (8 phot + 8 EAZY, ~4 GB, `sc_curation.fetch`) → 20,000 subjects over 6 fields, z-stratified; 2,286 gold (1,536 strong: GZ CANDELS answers + 681 bright point sources; 750 weak "merger=none" capped 250/field).
- Informative GZ overlap caps at ~1,100 whatever the guards (measured) → weak tier + objective stars fill the gap; flagged for T4.3 weighting. Committed `data/curation/v1/` + `docs/data/curation-report-v1.md`.
- Demo: `just demo T1.5` + `docs/demos/T1.5/contact-sheet.png` (real JWST thumbnails via the DJA cutout service, demo-only).
- Harness: `just demo` now also runs matching pytest tests.

## 2026-09-27 — T1.8 Test infra (done)
- `just verify` now also runs: coverage gate (per crate, gated from 50 lines; sc-types 86%), pytest (tools/curation), frontend checks, traceability (`docs/specs/traceability.md` + waivers). ~45 s.
- proptest (sc-types), cargo-fuzz targets (`just fuzz`, `just nightly`), bot agent (`agent-kit/tests/bot-agent`, @icp-sdk/core, root key from icp — no fetchRootKey), 50 deterministic fixture dossiers + JSON Schema sc-dossier/1 (`just fixtures`).
- Demo: `just demo T1.8` + `docs/demos/T1.8/fixtures-contact-sheet.png`. Playwright+axe moved to T6.10 (no UI to test in P1).
- T6.1 was marked done by a parallel session without tests → visible WAIVER in traceability.md until T6.10.
- Harness: guard blocks icp calls without --identity (L-008 promoted after a 10-min hang on a hidden password prompt).

## 2026-09-27 — SP-7 Gold source (done)
- Galaxy Zoo JWST CEERS labels are not public (paper: "upon request"; Masters et al. in prep). research Q14 was wrong again.
- Decision (07 §1/§5, REVIEW): gold = Galaxy Zoo: CANDELS (public, HST, COSMOS/GOODS-S/UDS; cite Simmons+2017) mapped to protocol v1 with a ≤0.3″ DJA crossmatch and z_phot < 2 guard; objective gold elsewhere.
- Proof: `python3 scripts/spikes/sp-7-gz-candels.py` → 16,796 subjects with ≥1 gold-grade answer (pre-crossmatch).
- Owner (optional, not blocking): request the GZ CEERS catalogue from the authors; T1.5's gold.py can swap it in.

## 2026-09-27 — SP-2 OISY approve w/ spender subaccount (done)
- Real ICP ledger ICRC-21 consent message for icrc2_approve shows the full spender account incl. subaccount (generic + fields display). Wallet path stays; deposit path for non-ICRC-21 wallets.
- Recorded in 04 §1 + REVIEW; research Q2 corrected. Residual: one manual OISY approve on staging in beta (T8.3).
- Demo: `just demo SP-2`.

## 2026-09-27 — SP-6 ckETH/ckBTC deposits (done)
- Read-only mainnet minter queries: ckETH subaccount deposits supported (helper contract); ETH minimum 0.005 ETH ≈ $15 → ETH path is "any amount ≥ minimum", credited at full value. ckBTC: 4 confirmations, 300-sat min, 100-sat fee.
- Recorded in 04 §6.5/§6.6 + REVIEW; research Q3 corrected. Values must be read from `get_minter_info` at runtime.
- Proof: `bash scripts/spikes/sp-6-minters.sh` (mainnet read-only; can't run in PocketIC).

## 2026-09-27 — SP-4 canister_info cost/latency (done)
- Measured with a probe canister (`crates/spike-probe`, test-only, not deployed): ~5.9M cycles and +1–2 rounds, same or cross subnet.
- Decision (02 §5 step 4, REVIEW): strict per-call provenance on submit_*; get_* cached ≤ 1 h. Reverses the research-driven "lazy 24 h" fallback.
- Demo: `just demo SP-4`. Harness: IcpEnv::with_app_subnets(n), install_on(subnet).

## 2026-09-27 — SP-3 Frozen canister behaviour (done)
- Tested: frozen AAA rejects queries, updates, and even controller `canister_status`; CMC top-up works while frozen and unfreezes it.
- Decision (REVIEW SP-3/R-68): dashboard uses platform-cached AAA data only; 05's frozen fallback already matches. research-unknowns Q9 corrected (again said the opposite).
- Demo: `just demo SP-3`. Harness: L-017 hits:2.

## 2026-09-27 — SP-1 CMC + ICRC-2 memo (done)
- Tested against the real CMC: ICRC-2 transfer_from (and icrc1_transfer) with the 8-byte LE memo TPUP/CREA works for top-up AND create; memo-less → refunded. Legacy transfer also works.
- Decision in 04 §2 + REVIEW: payments pulls directly into the CMC deposit account; no two-step fallback. research-unknowns Q1 corrected (it said the opposite).
- Demo: `just demo SP-1`. Harness: L-017 (test spikes, don't trust desk research).

## 2026-09-27 — T1.4 PocketIC harness (done)
- `integration_tests::pic::IcpEnv`: PocketIC 16 + `IcpFeatures` (real ICP ledger, CMC, registry; anonymous holds 1B ICP); helpers for install, query/update, icrc1 + legacy transfer, CMC top-up account, notify_top_up; `canister_wasm()` builds wasms once.
- Demo: `just demo T1.4` → mint 10 ICP, install platform, 1 ICP → CMC (memo TPUP) → +3.52T cycles.
- Harness: POCKET_IC_MUTE_SERVER=1 in demo/verify (system-canister logs drowned the demo). Server binary auto-downloads to $TMPDIR on first run.

## 2026-09-27 — T1.3 Local verify & deploy harness (done)
- `just verify`: fmt, clippy -D warnings, candid drift (`scripts/check-candid.sh`, `just candid` to rewrite), aaa ≤ 1.5 MiB gz, cargo test with ignored/skip = fail, no dfx/fetchRootKey; pytest/frontend/traceability steps activate when those exist. ~7 s today.
- `just deploy-local` rewritten for icp 1.6: dedicated `sc-deployer/sc-admin/sc-user` identities, auto-funding from the seeded anonymous account, env-var wiring (no setters), random gateway port.
- Demo: `just demo T1.3` → drift gate catches a tampered .did; deploy runs twice, all canisters answer.
- Harness: local-deploy skill corrected (old one had nonexistent icp commands); L-016.

## 2026-09-27 — T1.2 sc-types shared crate (done)
- ApiError (12 variants + Display), SubjectRef/FIELDS, Protocol/Question/Answer, Task, submissions/receipts/assignments, Vote, ClaimOutcome; `limits` module mirrors 08 §3.
- Demo: `just demo T1.2` → 4 tests (candid round-trip of every type; ApiError variants; input limits; candid shapes vs 02 §2).
- Harness: no friction.

## 2026-09-27 — T1.1 Repo scaffold (done)
- Cargo workspace: sc-types + platform/payments/treasury/aaa canisters + integration-tests; toolchain pinned 1.95.0 + wasm32; icp.yaml (icp-cli 1.6, rust recipe v3.4.0, gateway port 0; local/staging/production envs; aaa built but not deployed).
- Demo: `just demo T1.1` → 3 tests (icp build → 3 wasms with candid metadata; every .did committed; toolchain pinned).
- Next ready: T1.2 / T1.3 / T1.4, SP-7, SP-2.
- Harness: `just task-status`, `scripts/demo.sh` (noise-free demo, zero tests = fail); L-014, L-015. icp CLI upgraded 0.2.3 → 1.6.0.

## 2026-09-27 — T0.4 signed off; pre-coding blockers cleared
- Owner sign-off given on the condition of no blockers. Fixed: repo committed with a `.gitignore`; `just` installed; `just demo` filters `t<id>_` tests; `just plan` + requirements.txt (venv).
- Specs aligned to no remote CI / straight-to-main (00, 01, 09, 11, plan header); neuron removed from 05 admin treasury, OVERVIEW, DESIGN_BRIEF.
- Recipes named in spec 11 but not yet written (`verify-web`, `nightly`, `ops-smoke`) are built in T1.3/T1.8.
- Next: T1.1 repo scaffold (lane 1), waiting on the owner's go.

## 2026-09-27 — Deterministic Deploy & Local Verification Harness (Owner Directives)
- Dropped remote CI; local verification suite established (`scripts/verify-local.sh`, `make verify`).
- Deterministic local deployment script authored early (`scripts/deploy-local.sh`, `make deploy-local`).
- Straight-to-main single commit workflow locked across harness (no branching/PRs to save tokens); L-014 added.
- Rust skill expanded: Category 27 Agent-Parallel Architecture (10 rules for concurrent agent coding); L-015 added.
- Code comments banned; OKF established (`okf/operations/OPS.md`, `DEPLOY.md`) & OKRs maintained; L-016, L-017 added.
- Reusable agent skill created: `.claude/skills/local-deploy/SKILL.md` (mirrored in `.agents/skills`).
- Lessons L-011–L-017 added to `LESSONS.md`; OKRs and task T1.3 updated.
- Next: T0.4 sign-off → T1.1 repo scaffold (`icp.yaml`, workspace).

## 2026-09-27 — Planning complete (T0.1–T0.3)
- Specs 00–12, review passes 1–6, OKRs, deck, Gantt v3 (`docs/space-compute-gantt-v3.xlsx`).
- Added in this round: non-ICP intake auto-pause (04 §6.2b), a proof-of-burn neuron path (12 §6), a design phase P9 after functional UI, and model tiers + demos per task.
- Next: T0.4, owner sign-off. Owner list: `/design-login`, invite count, confirm which proof-of-burn neuron is app-owned, neuron size, Firebase projects.
- Harness: created (README, LESSONS seeded with 10 lessons, hooks: brief/guard/metrics, /task and /retro commands).
