---
description: Post-task retrospective that improves the harness itself.
---
Retro for the task in `.claude/harness/current` (or this session if none):

1. Read `.claude/harness/metrics/<this session>.json`. Compare its tokens with other metrics files of the same tier; flag anything over 2× the median.
2. List every friction point in this session: failed tool calls, retries, rework, wrong assumptions, slow or token-heavy steps, owner corrections. Be concrete: what happened and what it cost.
3. For each one, update `.claude/harness/LESSONS.md`: add `trigger → rule`, or increment `hits` on an existing lesson. Escalate a lesson that has 2+ hits one rung up the ladder (a skill checklist line or a CLAUDE.md rule → then a hook, script or test), and actually make that change now.
4. Prune LESSONS to ≤ 40 active lines. A promoted lesson becomes a one-line pointer.
5. Prepend ≤ 5 lines to `.claude/harness/progress.md`: what was done, the demo, next ready task, owner asks, and "harness: <what changed>".
6. Delete `.claude/harness/current`. Commit and push straight to `main`: `git commit -m "retro: <task>" && git push origin main`.
Keep the retro itself cheap: no re-reading of code, just the transcript facts and these files.
