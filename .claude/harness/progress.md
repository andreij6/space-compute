# Progress (newest first, ≤ 5 lines per entry)

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
