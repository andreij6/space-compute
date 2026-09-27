# Harness — how every task runs (and how the harness improves itself)

The files below carry state between sessions. They are based on Anthropic's "effective harnesses for long-running agents" and OpenAI's AGENTS.md practice:
- a machine-readable task list (JSON resists accidental edits better than Markdown)
- a short progress log
- git history
- lessons that get promoted into deterministic checks

| File | Purpose | Who edits |
|---|---|---|
| `tasks.json` | Every task: tier/model, deps, spec, acceptance, **demo**, status | Agent edits only `status` + `pct`; `python tools/plan/plan.py` regenerates everything else |
| `progress.md` | Newest-first handoff notes, 5 lines per task | Agent, at retro |
| `LESSONS.md` | Mistakes and wasted time/tokens turned into rules | Agent, at retro |
| `metrics/` | Tokens, tool errors and duration per session (Stop hook) | Hook only |
| `hooks/` | `session_brief.py` (context at start), `guard.sh` (blocks dangerous commands), `stop_metrics.py` | Agent, when a lesson is promoted |
| `current` | Id of the task in progress | `/task`, `/retro` |

## Git Workflow (Straight to Main — Zero Branching)
To conserve tokens and eliminate branch-switching and PR review overhead:
- **No branches, no PRs:** Push each finished task straight to `main` in a single clean commit.
- **Commit format:** `git commit -m "<task-id>: <summary>" && git push origin main`.
- If another session committed first, pull with rebase: `git pull --rebase origin main`.
- **Owner-only tasks** (`lane: null`, model `human`) are the owner's: sign-off, beta, treasury funding, accounts, external audit. Put them on the owner's list at the start of the task they block (L-006).

## The loop: `/task <id>` → `/retro`
1. **Pick & check.** Deps done? Is the session model right for the tier?
   - H = Opus 5.5 high effort · M = Sonnet 5 · L = Haiku 4.5.
   - Running *above* tier: say so and continue. Running *below* tier on an H task: stop and ask the owner to switch.
2. **Load little.** Read only the task's spec sections, plus skills named there. The session brief has already injected the lessons.
3. **Tests first.** Write the acceptance tests (they fail), then write the smallest implementation that passes.
4. **Gates.** Run the local verification gates via `just verify` (`scripts/verify-local.sh`) and quote the pass/fail counts. A skip is a failure (no remote CI).
5. **Deploy & Demo** (the owner looks at this, not the diff):
   - `deploy:` run `just deploy-local` (`scripts/deploy-local.sh`) for deterministic setup
   - `test:` `just demo <id>` runs every test whose name starts with the id in snake case (T4.9 → `t4_9_…`), with narrated steps (`println!("✓ spawned AAA …")`)
   - `shots:` Playwright screenshots in `docs/demos/<id>/`
   - `image:` a contact sheet in `docs/demos/<id>/`
6. **Review.** An L/M task that touched money, auth or canister state → one Opus review pass before it counts as done.
7. **Close.** Set `status: "done"`, `pct: 1` in `tasks.json`, then `just plan` (regenerates 10-tasks.md + Gantt). Commit and push straight to `main` in a single commit (`git commit -m "T<id>: <summary>" && git push origin main`). No branching or PRs.
8. **`/retro`** (always, even when everything went well).
9. **Tell the owner** with a demo card (below). Don't send a report.

## Demo card (the only thing the owner has to read)
```
✅ T4.9 First-claim rule — done (sonnet-5, 1h40m, 610k tokens)
Demo:  just demo T4.9   → two agents flag the same galaxy; the first wins, the second becomes a corroborator
Proof: 14 tests green · coverage 93% · docs/demos/T4.9/output.txt
Needs you: nothing
Harness: +1 lesson (L-012), guard now blocks `icp … --mode reinstall` on non-local
```
At each milestone, republish the **progress artifact**: Gantt %, the demo cards and the screenshots, on one link. When the owner is away, use a push notification with the card's first line.

## How the harness improves itself (`/retro`)
Each retro looks at the session's `metrics/` file plus what actually happened: failed tool calls, retries, rework, wrong assumptions, anything slow or token-heavy.
- **Scriptable & Reusable tasks** → Write a deterministic script in `scripts/` and author an agent skill in `.claude/skills/<name>/SKILL.md` so the entire harness continuously gets stronger.
- **New problem** → add a lesson: `trigger → rule`.
- **Repeat problem** → increment `hits`, then escalate it one rung up the ladder:
  1. a lesson (advice, read every session)
  2. a checklist line in a skill, or a rule in `CLAUDE.md`
  3. a script, hook or test that makes the mistake impossible (e.g. `guard.sh`, local verification check)
- **Prune.** Keep `LESSONS.md` at 40 active lines or fewer. A lesson that has been promoted is deleted and replaced with a pointer.
- **Log harness changes** in `progress.md` under the task ("harness: …"), so the owner can see the harness evolve.
- **Token budget signals.** Flag a session that used more than 2× the median tokens of other tasks in the same tier, then find the cause. Common causes: reading whole files, repeating searches, large pastes, retry loops.
