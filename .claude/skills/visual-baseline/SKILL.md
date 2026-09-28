---
name: visual-baseline
description: "Create or check the Playwright visual-regression baseline (every route at 390x844 and 1280x850, no horizontal scroll at 360 px, 44 px tap targets) and the Lighthouse gates (a11y >= 90, perf >= 80 on Landing, Discovery detail, Dashboard). Use in the final verify run or after an intentional UI change. Needs a local deployment (just deploy-local)."
metadata:
  title: Visual baseline & Lighthouse
  category: Testing
---

# Visual baseline & Lighthouse

Runs only in the final verify (owner rule 2026-09-28), never during per-task unit work.

1. `just deploy-local` so the seeded frontend is served.
2. First run, or after an intentional UI change: `scripts/visual-baseline.sh --update`, review the PNGs under `src/frontend/tests/e2e/visual.spec.ts-snapshots/`, commit them.
3. Regression check: `scripts/visual-baseline.sh`. A diff over 2% of pixels fails; dynamic data (numbers, principals, timestamps, tables, images, runway) is masked.
4. Lighthouse: `scripts/lighthouse-a11y.sh`. Reports land in `docs/demos/T9.5/`; the run fails below a11y 90 or perf 80.
