# Space Compute

Agent-run citizen-science astronomy on the Internet Computer. Start with `docs/specs/00-INDEX.md`.

- Canisters are **Rust** (never Motoko), built and deployed with the **`icp` CLI** (never `dfx`).
- **No remote CI:** All testing and verification are local-first via `just verify` (`scripts/verify-local.sh`).
- **Deterministic Deployments:** Use `just deploy-local` (`scripts/deploy-local.sh`) for repeatable local deployments. Never deploy ad-hoc.
- **Harness & Skills Growth:** Whenever a scriptable/reusable workflow is identified, add a script in `scripts/` and a corresponding skill in `.claude/skills/`.
- The specs in `docs/specs/` are normative. Tasks and status live in `.claude/harness/tasks.json`.
- Work through the harness: `/task <id>` then `/retro` (`.claude/harness/README.md`). Use the task's model tier; ship a demo, not a report; the retro improves the harness after every task.
- Functional first: no styling work before Phase 9.
- **Maintain OKF:** Maintain the Operational Knowledge Framework (`okf/`, runbooks, ops specs) and keep `docs/OKR.md` current at every milestone and whenever scope or parameters change.
- **No Code Comments:** Do not write code comments. Keep code clean, idiomatic, and self-documenting.
- **Straight to Main (Zero Branching):** Push each task straight to `main` in a single commit (`git commit -m "<task-id>: <summary>" && git push origin main`). No branching or pull requests to save tokens and avoid merge overhead.
- Current phase: **plan signed off by the owner (T0.4, 2026-09-27).** Coding starts at T1.1 when the owner says go.
