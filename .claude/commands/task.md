---
description: Run one plan task end-to-end with the harness protocol (tests first, demo, retro).
argument-hint: <task id, e.g. T2.3>
---
Run task **$ARGUMENTS** following `.claude/harness/README.md` exactly:

1. Look up $ARGUMENTS in `.claude/harness/tasks.json`. Stop if any dep isn't `done`. Compare `tier` with your model: below tier on H → stop and tell the owner which model to switch to; above tier → note it and continue.
2. Write `$ARGUMENTS` to `.claude/harness/current`; set its `status` to `in_progress`.
3. Read only the spec sections in `spec` (grep, then read ±40 lines) and any skill the task needs. Obey the LESSONS in the session brief.
4. Spike tasks (`SP-*`) are closed only by a runnable `sp_<n>_…` test against real canisters where PocketIC can host them; research notes are hypotheses (L-017).
5. Write the acceptance tests first, then the smallest implementation that passes. Follow spec 01 §6 conventions.
6. Run the spec 11 gates that apply. Quote pass/fail counts; a skip is a failure.
7. Build the `demo` named in tasks.json (tests named `<id in snake case>_…` so `just demo $ARGUMENTS` finds them, with narrated steps, or screenshots/images in `docs/demos/$ARGUMENTS/`).
8. If the tier is L/M and you touched money, auth or stable state → get one Opus review pass (subagent, model opus) and fix what it finds.
9. Set `status: "done"`, `pct: 1`; run `just plan`. Commit and push straight to `main` in a single commit (`git commit -m "$ARGUMENTS: <summary>" && git push origin main`). No branching or PRs (saves tokens).
10. Run /retro, then send the owner the demo card (README format). If the owner is away, also send a push notification with its first line.
