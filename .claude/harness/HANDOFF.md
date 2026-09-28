# Handoff prompt (paste into a new coding agent)

You are continuing Space Compute, an agent-run citizen-science astronomy app on the Internet Computer. Repo: /Users/andrejones/Desktop/workspace/projects/space-compute. Work autonomously; do not ask the owner questions, go with your recommendations.

## Read first (in this order, only what you need)
1. `CLAUDE.md` / `AGENTS.md` (hard rules), `.claude/harness/README.md` (protocol: `/task <id>` then `/retro`).
2. `.claude/harness/AUTHORSHIP.md` (who did what; add your row with every task), `.claude/harness/progress.md` (newest first) and `.claude/harness/LESSONS.md` (L-001…L-025; obey them).
3. `.claude/harness/tasks.json` (status of every task), `docs/specs/10-tasks.md` (plan table), `docs/specs/00-INDEX.md`.
4. For each task: only the spec sections named in its `spec` field (grep, then read ±40 lines).

## State (2026-09-27)
- Done: all of P0 and P1 foundations (T1.1–T1.8, SP-1…SP-7), T2.1 platform skeleton, T3.1 AAA skeleton, T5.1 payments skeleton. `just verify` is fully green (50 Rust tests, coverage gates, pytest, frontend, traceability).
- In progress: T6.1 frontend scaffold (30%, built by another agent on mock data; see the 2026-09-27 review entry in progress.md for what is missing).
- Data: 5,000 JWST dossiers (v1) in `target/bucket` (9.3 GB, all hashes verified), manifest in `data/curation/v1/`. The selection is inclusive by owner decision: stars, no-redshift and edge objects are kept.
- Templates to copy for new canister modules: `crates/platform/src/{memory,config,audit,api,timers,lib}.rs` and the test `crates/integration-tests/tests/t2_1_platform_skeleton.rs`. Canister glue lives in `lib.rs`/`api.rs`/`timers.rs` (excluded from coverage and covered by PocketIC); logic goes in plain modules with unit tests (L-022).

## Next tasks (ready now; respect deps in tasks.json)
T2.2 registry & factory (H), T2.3 catalog/get_task (M), T2.5 event log (M), then T2.4, T2.6, T2.8, T2.7; T5.2 CMC/quotes (M), T5.17 treasury (H). Use the task's model tier: H → Opus, M → Sonnet, L → Haiku. Parallel lanes: at most 3, each touching different crates; stage explicit paths only (L-020).

## Owner decisions you must respect
- Rust canisters only; `icp` CLI only, never `dfx`. Always pass `--identity sc-deployer|sc-admin|sc-user|anonymous` and `-e local`. Never use the default identity `prod-deployer` (it's a password-protected mainnet key). No mainnet/staging commands unless the owner asks.
- **No Stripe/card code at all until the whole app is ready** (deferred items D1–D5 in 10-tasks.md).
- No NNS neuron; the owner funds the treasury directly (spec 12).
- Canisters store image references (URL + sha256) only, never images.
- No code comments. Functional first; no styling before Phase 9.
- v1 dataset is 5,000 subjects; T8.16 scales it to 20,000 before beta.
- Until R2 exists (owner task T8.14), agents need the dossiers over HTTP: T3.8's seed script should serve `target/bucket` on localhost and seed subject URLs pointing there.
- The owner reviews the discovery/peer-review process (T8.15, due 2026-10-01) before T4.x review code is built. Check OKR.md / progress.md for the outcome before starting T4.1.

## Per task, every time
1. `just task-status <id> in_progress`; write the acceptance tests first (names `t<id>_…`, e.g. `t2_2_…`, so `just demo <id>` finds them).
2. Implement the minimum that passes. Run `scripts/check-candid.sh --write` after API changes.
3. While building (owner rule 2026-09-28): `just verify-unit` must be green — fmt, clippy, candid, unit tests only. Write integration/PocketIC/e2e tests but do NOT run them; the full `just verify` runs once when all coding is complete. Add traceability rows in `docs/specs/traceability.md`.
4. Any task implemented by a non-Opus model (Sonnet/Haiku/external), and every H-tier task or anything touching money, auth or stable state: get one Opus review pass and fix what it finds (owner rule 2026-09-28).
5. Add a progress.md entry (≤5 lines, newest first) and a LESSONS entry if something went wrong. Then `just task-status <id> done`, `just plan`.
6. **Commit and push straight to main after every task**: `git add <explicit paths>` then `git commit -m "<id>: <summary>" -m "Agent: <your tool>/<model>"` (plus your AUTHORSHIP.md row in "Other agents") then `git push origin main`. One commit per task; no branches or PRs. The lead session reviews your tasks against their acceptance lines before they count as done.
7. Keep `docs/OKR.md` current when scope or parameters change.

## Gantt (owner's Google Sheet)
Sheet: https://docs.google.com/spreadsheets/d/1kPN4smVMnWE_0jPmLXbRXyevcpzHlqMGdaU3wKS6qXI/edit?gid=1738822749. After tasks complete (at least when about 8% of the usage window remains): run `just gantt-paste`, then in Claude in Chrome, wait for the sheet to load, press Cmd+J (focuses the Name box; clicking it before load sends the paste into the Name box) → `A2:K200` → Enter → Delete → Cmd+J → `A2` → Enter → Cmd+V. Jump to A110 to confirm no leftover rows. Never create a new sheet or rename the owner's sheet (L-001).

## Owner-only items (don't block on them; remind the owner)
T8.14 R2 bucket + token then `just publish-data`; T8.15 process review; T8.12 Firebase; T8.8 crypto compliance; T8.13 domain.
