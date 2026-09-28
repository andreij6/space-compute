# Authorship ledger

Which agent did which task, so the reviewer knows what it wrote itself and what it has to verify (L-021: another agent's "done" is a hypothesis until checked against its acceptance line).

**Reviewer / lead:** the Claude Code session that planned the project and built the foundations ("lead"). The lead reviews every task in the "Other agents" table before it counts as done.

## Rules for every agent
- Commit message ends with a trailer naming the agent: `Agent: <tool>/<model>` (e.g. `Agent: claude-code/opus-5.5`, `Agent: gemini-antigravity`). Stage explicit paths only.
- Add one row to the right table below in the same commit as the task.
- Never edit another agent's rows; the lead moves rows to "Reviewed" after checking.

## Lead (Claude Code, Opus 5.5; sub-agents noted) — self-built, no external review needed
| Task | Commit | Notes |
|---|---|---|
| T0.1–T0.4 | 84774e7 (+ planning) | specs, reviews, Gantt, OKRs, sign-off |
| T1.1 | dffcdf0 | repo scaffold |
| T1.2 | 8b0161d | sc-types |
| T1.3 | f3df24e | verify/deploy harness |
| T1.4 | 5fd6b38 | PocketIC harness |
| T1.5 | e9fe11d, d99c6b8 | selection + gold; inclusive 5k re-selection |
| T1.7 | 11c1c30 | 5,000 dossiers, common-grid RGB |
| T1.8 | 69cdd16 | test infra |
| SP-1…SP-4, SP-6, SP-7, SP-2 | 72e2e7b, ccbb9dd, 6b77598, 23bc953, 7451eed, 692043a | spikes (note: SP commits accidentally include the other agent's frontend files, L-020) |
| T2.1 | 48c589a | platform skeleton (Opus review applied) |
| T3.1 | fdf33c1 | AAA skeleton — written by a Sonnet sub-agent of the lead, spot-checked by the lead |
| T5.1 | 08d322b, aef2b5b | payments skeleton — Sonnet sub-agent; Stripe removed by the lead |
| T3.5 | (see commit) | started by the other agent (stopped mid-task); lead fixed the doc (wrong `icp identity get-principal`, 7 undocumented methods) and added a Candid-drift test |
| T5.17, T5.18 | (see commits) | treasury keeper; Opus-reviewed, 8 defects fixed |
| T2.9 | 10fca44 | Opus sub-agent; fixes for the other agent's P2 defects |
| T3.4 | e6d521d | Sonnet sub-agent; AAA timers + fixes for the other agent's T3.2/T3.3 defects |
| T5.2 | d277b51 | Sonnet sub-agent |
| T3.8 | (see commit) | lead; seed + platform fixes (admin bootstrap, immutable wasm versions) |
| T4.3 | 14308f7 | Sonnet sub-agent |
| T3.6 | 5d786c1 | Sonnet sub-agent |
| T5.3 | e490359 | Sonnet sub-agent |
| T4.1 | df56991 | Sonnet sub-agent |
| T5.4 | 890db97 | Sonnet sub-agent |
| T4.9 | d8c5086 | Opus sub-agent |
| T5.5 | fcbd1f9 | Sonnet sub-agent; also applied btc/eth=false defaults |
| T4.2 | 8571ac0 | Opus sub-agent |
| T5.6 | d6cdeff | Sonnet sub-agent |
| T4.4 | c614027 | Opus sub-agent |
| T5.16 | 1ead550 | Sonnet sub-agent |
| T4.5 | 7f930dc | Opus sub-agent |
| T3.7 | 5cf3c75, 2fe096e | Sonnet sub-agents |
| T5.15 | 25c786a | Sonnet sub-agent |
| T3.9 | 9f5e7fa | Sonnet sub-agent |
| T5.7 | 4883aa4 | Opus sub-agent; review found 1 critical (double pay on retry), 4 high, 4 medium — all fixed |
| T4.7 | 8d862a1 | Sonnet sub-agent |
| T6.1 | 75837e0 | Opus sub-agent (replaced external agent's mock scaffold) |
| T4.6 | 3447982 | Sonnet sub-agent |
| T4.8 | 81b0d0a | Sonnet sub-agent (3 proptests; §11 #4–9 mapped) |
| T6.3 | 825baee | Sonnet sub-agent |
| T6.5 | 2de302c | Sonnet sub-agent |
| T6.4 | 26652d0 | Sonnet sub-agent |
| T7.10 | 794e000 | Sonnet sub-agent |
| T6.7 | 45548c5 | Sonnet sub-agent |
| T7.4 | 4612992 | Sonnet sub-agent |
| T4.10 | fb1b52a | Sonnet sub-agent |
| T4.11 | 300a3b3 | Sonnet sub-agent |
| T6.8 | cb38fa2 | Sonnet sub-agent |
| T6.9 | dc3fc22 | Sonnet sub-agent |
| T7.1 | e3f7ee2 | Sonnet sub-agent; found+fixed upgrade trap (AaaRecord.is_house non-optional) |
| T6.6 | 6fbcbae | Sonnet sub-agent |
| T6.12 | f73dfe6 | Sonnet sub-agent |
| T7.9 | 57d0848 | Sonnet sub-agent; audit gate 61 mapped / 8 waived / 0 unmapped |
| T7.7 | 8d5eb76 | Opus sub-agent; 3/20 → hardened skill → 0/20 compliance (haiku reference agents) |
| gap-fill (list_ops_for_aaa/owner, DiscoveryCard image) | 521e37d | Sonnet sub-agent |
| invite sponsor minimum | 50cfe18 | lead |
| T6.14 | f02d6aa | Sonnet sub-agent |
| T8.1 (tooling prep; staging deploy itself awaits owner) | 689653e | Sonnet sub-agent |
| T7.2 | b9378ff | Sonnet sub-agent; 20×20 measured, 200×100 not completed (PocketIC timeouts under load) — Opus review pending |
| T8.15, T8.16 (plan) | a901db8, f179153 | owner-review task, 20k dataset task |
| plan/Gantt/handoff | c2e466a, 3c45daf, b6efcf0, a001d80, 4bdc6a2, f4b3eea, a0bd99c | bookkeeping |

## Other agents — lead must review
| Task | Agent | Commit | Review status |
|---|---|---|---|
| T6.1 | gemini-antigravity (no trailer) | 32a300a | **Reviewed 2026-09-27: not done; superseded by lead's 75837e0** — mock data, no II auth/ic_env/bindgen/tests; reset to in_progress 30% (see progress.md) |
| T6.1 screenshots | gemini-antigravity | uncommitted (`docs/demos/T6.1/`, `scripts/capture_screenshots.py`) | pending; script writes to the agent's private folder |
| T2.2 | other agent (no Agent trailer) | b416e8b | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (verify callable by anyone; provenance not compared; payments_id=caller; install not idempotent)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.3 | other agent (no Agent trailer) | 862ebfc | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (**gold answers exposed by public queries**; full lease/pool scans)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.4 | other agent (no Agent trailer) | 258489f | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (consensus not reputation-weighted; duplicate receipt loses xp)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.5 | other agent (no Agent trailer) | 8096466 | **Reviewed 2026-09-27 (Opus): DONE** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.6 | other agent (no Agent trailer) | 907a945 | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (leaderboard cursor ties/rank)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.8 | other agent (no Agent trailer) | 1f15e0c | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (strict provenance missing; get_task lacks submitter check)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T2.7 | other agent (no Agent trailer) | ad9b922 | **Reviewed 2026-09-27 (Opus): DONE** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T3.2 | other agent | 2ad559e | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (test hook in prod; task/review idempotency key collision)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| T3.3 | other agent | a002eff | **Reviewed 2026-09-27 (Opus): DONE-WITH-FIXES (sync_credit_copy forgeable; list_records unbounded; credit→record key wrong)** → fixes in T2.9 (platform) / T3.4 (AAA) |
| docs/board, docs/guide | other agent | d767278 | pending |

| frontend WIP (external agent, uncommitted, 27 files: copy tweaks, PracticePage deleted) | external | saved to `.claude/harness/external/frontend-wip-2026-09-27.patch`, working tree reset so T6.x lanes start clean | not applied; re-apply with `git apply` if wanted |

## How to tell from git
`git log --format='%h %s | %(trailers:key=Co-Authored-By,valueonly)%(trailers:key=Agent,valueonly)'` — lead commits carry `Co-Authored-By: Claude Opus 5.5`; anything else goes in "Other agents" and gets reviewed.

## Opus reviews of non-Opus work (owner rule 2026-09-28)
| Batch | Tasks | Verdict | Fix lane |
|---|---|---|---|
| AAA + agent-kit | T3.1, T3.4, T3.6, T3.7, T3.9 | 14 defects (1 high: runner can't parse real status; retry double-fee; skill ran add_operator; set_profile missing) | Opus fix lane in progress |
| Platform | T4.1, T4.3, T4.6, T4.8, T4.10, T4.11, gap-fill | 8 defects (2 high: event queries leak hidden discoverers/honeypots/gold; replay double-counts) | Opus fix lane in progress |
| Tooling | T4.7, T7.1, T7.4, T7.9, T7.10, T8.1 prep | 6 defects (2 critical: release ships gold; honeypot text predicts vote) + T7.1 vN→vN for 3 canisters | Opus fix lane in progress |
| Frontend | T6.3–T6.9, T6.12, T6.14, gap-fill | T6.3 FAILS (citation verify never succeeds: JS vs Rust candid bytes; CSP blocks images) + 13 more (fuel panel remount → double pay risk, blind treasury approve, mock footer) | platform part in platform fix lane; frontend Opus fix lane queued after T6.10 lands |
