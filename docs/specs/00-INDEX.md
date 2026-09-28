# Space Compute — Specification Index

Start here. These specs are written for the implementing agents (Claude Code). Founder intent & decision scorecards are in `../board/`, product context is in `../OVERVIEW.md`, screens are in `../DESIGN_BRIEF.md`, and goals are in `../OKR.md`.

| # | Spec | Covers |
|---|---|---|
| — | [Board Charter & Scorecard](../board/00-CHARTER.md) | founder intent, solo-dev leverage, ICP economic engine, spectator wonder, 5-gate scorecard |
| 01 | [Architecture & decisions](01-architecture.md) | ADRs, system diagram, repo layout, conventions, parameters |
| 02 | [`platform` canister](02-platform-canister.md) | registry/factory, tasks & gold, scoring, discoveries, review & consensus, credits (citations, XP, tiers, badges, leaderboard) |
| 03 | [`aaa` canister](03-aaa-canister.md) | the user-owned agent canister: relay, operators, repository, timers |
| 04 | [`payments` canister](04-payments-canister.md) | ICP pass-through (spawn, top-up, auto top-up); card/BTC/ETH fuel packs deferred (D1-D12) |
| 04b | [Stripe relay](04b-stripe-relay.md) | the only off-chain component |
| 05 | [Frontend](05-frontend.md) | routes, data sources, behaviours, payment component |
| 05a | [Design reconciliation](05a-design-reconciliation.md) | design sources, tokens, components, pending mockup comparison |
| 06 | [Agent toolkit](06-agent-toolkit.md) | the operator skill for Claude Code |
| 07 | [Data curation](07-data-curation.md) | subjects, gold labels, protocol v1, honeypots |
| 08 | [Security & abuse](08-security.md) | threat model, input limits |
| 09 | [Testing & ops](09-testing-ops.md) | test pyramid, local verify & deploy, monitoring, treasury runbooks |
| 11 | [Automated test strategy](11-test-strategy.md) | layers, local gates (`just verify`), coverage, traceability, bot agent |
| 12 | [`treasury` canister](12-treasury-keeper.md) | owner-funded treasury → automatic cycles top-ups |
| 10 | [Task plan](10-tasks.md) | every task with owner, dependencies, dates, spec link, acceptance |
| — | [Adversarial review log](REVIEW.md) | findings R-01…R-78, spikes SP-1…SP-6 |
| — | [Feature Test Inventory](../TESTING.md) | living test inventory with readiness, automated, and manual QA checkboxes |

## Rules for implementing agents
1. **Rust only** for canisters, using the **`icp` CLI** (never `dfx`). Read the matching skill in `.claude/skills/` before writing code: `icp-cli`, `stable-memory`, `multi-canister`, `canister-security`, `icrc-ledger`, `cycles-management`, `ckbtc`, `certified-variables`, `internet-identity`, `wallet-integration`, `static-site`, `rust-skills`.
2. One task = one commit, straight to `main` (`<task-id>: <summary>`), only after `just verify` passes. No branches or PRs.
3. The specs are normative. If a spec is wrong or ambiguous, update the spec in the same commit and note it in `REVIEW.md`.
4. Never store secrets in canister state. Never add an admin path that mutates citations.
5. Run tasks with the harness (`/task <id>` then `/retro`, see `../../.claude/harness/README.md`). Status lives in `.claude/harness/tasks.json`; `python tools/plan/plan.py` regenerates 10-tasks.md and the Gantt workbook. Update `../OKR.md` at each milestone.
6. Use the model tier the task names (H Opus 5.5 · M Sonnet 5 · L Haiku 4.5). Every task ships a demo the owner can check in under a minute.
7. Read `../LEARNINGS-proof-of-burn.md` once. Its rules are already folded into 01 §6, 05 and 11.

## Tracking
- **Gantt & progress:** Google Sheet https://docs.google.com/spreadsheets/d/1kPN4smVMnWE_0jPmLXbRXyevcpzHlqMGdaU3wKS6qXI/edit (Gantt / Progress / Milestones tabs). Refresh: `just plan`, then `just gantt-paste` and paste into Gantt!A2 (done via Claude in Chrome — the Drive connector cannot edit cells). Local copy: `docs/space-compute-gantt-v3.xlsx`.
- **Baseline:** start 2026-09-28, public launch 2027-02-02 (owner decisions 2026-09-27: no Stripe until the app is ready, ICP-only payments for MVP moved launch earlier). Claude Code writes all the code in 3 parallel lanes (parallel sessions committing straight to `main`); the owner signs off, reviews demo cards, runs the beta, sets up accounts and funds the treasury.
