# Progress (newest first, ≤ 5 lines per entry)

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
