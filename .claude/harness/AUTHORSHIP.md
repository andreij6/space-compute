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
| T8.15, T8.16 (plan) | a901db8, f179153 | owner-review task, 20k dataset task |
| plan/Gantt/handoff | c2e466a, 3c45daf, b6efcf0, a001d80, 4bdc6a2, f4b3eea, a0bd99c | bookkeeping |

## Other agents — lead must review
| Task | Agent | Commit | Review status |
|---|---|---|---|
| T6.1 | gemini-antigravity (no trailer) | 32a300a | **Reviewed 2026-09-27: not done** — mock data, no II auth/ic_env/bindgen/tests; reset to in_progress 30% (see progress.md) |
| T6.1 screenshots | gemini-antigravity | uncommitted (`docs/demos/T6.1/`, `scripts/capture_screenshots.py`) | pending; script writes to the agent's private folder |
| T2.2 | other agent (no Agent trailer) | b416e8b | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.3 | other agent (no Agent trailer) | 862ebfc | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.4 | other agent (no Agent trailer) | 258489f | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.5 | other agent (no Agent trailer) | 8096466 | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.6 | other agent (no Agent trailer) | 907a945 | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.8 | other agent (no Agent trailer) | 1f15e0c | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T2.7 | other agent (no Agent trailer) | ad9b922 | **pending lead review** (marked done in tasks.json; `just verify` green 2026-09-27) |
| T3.2 | other agent | 2ad559e | **pending lead review** — note: ships owner-only test hook `simulate_sys_unknown_once` in the prod interface |
| T3.3 | other agent | a002eff | **pending lead review** — note: `sync_credit_copy` lets the owner write arbitrary credits into its copy (spec: platform-pulled) |
| docs/board, docs/guide | other agent | d767278 | pending |

## How to tell from git
`git log --format='%h %s | %(trailers:key=Co-Authored-By,valueonly)%(trailers:key=Agent,valueonly)'` — lead commits carry `Co-Authored-By: Claude Opus 5.5`; anything else goes in "Other agents" and gets reviewed.
