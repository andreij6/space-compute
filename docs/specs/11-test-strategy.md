# 11 — Automated test strategy (normative)

**Rule:** nothing ships without automated tests. Each task's commit includes the tests that prove its acceptance criteria. There is **no remote CI**: `just verify` (`scripts/verify-local.sh`) runs every gate below locally, and a task may not be committed to `main` unless it passes. There is no manual-only QA step in the release path; human exploratory testing during beta is extra.

## 1. Layers & tools

| Layer | Tooling | What it covers | Where |
|---|---|---|---|
| Unit (Rust) | `cargo test` (native) | Pure logic: protocol-path validation, evaluate(), reputation/XP/tiers/badges, claim cells, citation text, fee math, subaccount derivation, rate conversions, saga state machine transitions | each crate `src/**/tests.rs` |
| Property (Rust) | `proptest` | Invariants: replay(log) == incremental progress; evaluate() monotone in agree-weight; claim index never yields two open discoveries for one cell+category; saga never double-pays; spent ≤ cap | `crates/*/tests/prop_*.rs` |
| Fuzz (Rust) | `cargo-fuzz` (nightly job, 10 min/target) | Candid decoding of every public method's args; input validators (names, rationales, answers, positions) | `fuzz/` |
| Canister integration | `pocket-ic` 16 with `IcpFeatures` (real ICP ledger, CMC and registry installed by PocketIC — no wasm pinning; `integration_tests::pic::IcpEnv`), XRC mock and ckBTC/ckETH ledgers + mock minters added with T5.8–T5.10 | Every acceptance list in 02, 03, 04, 12; multi-canister sagas; `SYS_UNKNOWN`/timeout injection; stopped-callee failures; time travel for leases, timers, top-up sweeps | `crates/integration-tests/` |
| Upgrade & compatibility | PocketIC + `candid` subtype check (`didc check new.did old.did`) | vN-1 → vN state survives for every canister; the Candid interface stays backward-compatible (breaking changes need an ADR) | `just verify` (compat step) |
| Security regression | PocketIC | One test per threat row in 08 (S1–S25): anonymous caller, non-operator, expired key, unsynced key, tampered AAA wasm, allowance hijack, reentrancy (concurrent calls), honeypot leak, admin-only paths, citation immutability | `tests/security_*.rs` |
| Curation pipeline | `pytest` with fixture mini-mosaics (tiny FITS) | select, gold mapping, render determinism (golden hashes), dossier schema (JSON Schema `sc-dossier/1`), publish manifest | `tools/curation/tests/` |
| Frontend unit/component | `vitest` + Testing Library | Formatting (cycles→days, z→lookback), the payment strategy map, ApiError→message mapping, feature-flag gating (card hidden), consent gate, analytics payload sanitiser (no principals or free text) | `frontend/src/**/*.test.ts(x)` |
| E2E (web) | Playwright against the **local network**, seeded by `tools/seed-local` (50 fixture dossiers served locally) | Full journeys: sign-in (II test identity), spawn via ICP deposit, dashboard, connect agent, records, discovery → citation "verified", fuel top-up + auto top-up, BTC/ETH flows with mock minters, admin console (every screen + the pause switch), non-admin blocked, card tab absent while its flag is off | `frontend/e2e/` |
| Accessibility | `@axe-core/playwright` in every E2E page visit; Lighthouse (local, `just verify-web`) on 3 key pages | No serious or critical axe violations; Lighthouse a11y ≥ 90, performance ≥ 80 | `just verify` |
| Agent E2E (deterministic) | `agent-kit/tests/bot-agent` — a scripted, non-LLM agent (TypeScript, `@icp-sdk/core`) that follows the skill's loop exactly | Keys, tasks, dossier hash checks, submissions, first-claim receipts, reviews, rate-limit handling, operator revocation mid-run | `just verify` on local network |
| Agent E2E (LLM, nightly) | Claude Code headless (`claude -p`) with the operator skill against staging | Skill usability: ≥ 10 valid classifications per run, prompt-injection honeypot non-compliance, no `InvalidInput` loops | `just nightly` (local launchd/cron), results → OKR KR2.2/2.3 |
| Load & cost | PocketIC script: 200 AAAs × 100 tasks + 50 reviewers | Instructions/call, memory growth, fee adequacy, query latency; fails if cost per 1,000 classifications exceeds budget (OKR KR4.4) | weekly + pre-release |
| Ops smoke | `just ops-smoke` against staging/prod (read-only), scheduled locally via launchd/cron | Canister statuses, treasury runway, bucket manifest hash sample, CSP headers | every 6 h |

## 2. Gates (`just verify` must pass before a task is committed)
1. All tests pass: unit, property, integration, security, compat, pytest, vitest, Playwright, the bot agent.
2. **Coverage:** `cargo llvm-cov` ≥ 90% lines on `platform`, `payments` and `treasury`, and ≥ 85% on `aaa` and `sc-types`; vitest ≥ 80% lines on `frontend/src/lib`. Coverage may not drop more than 0.5% per task.
3. **Traceability:** `docs/specs/traceability.md` maps every acceptance item (e.g. `02§11#4`, `04§6.7#10`, `S21`) to named tests. A script fails `just verify` when an acceptance item has no test, or a test ID points at nothing.
4. **A skipped test is a failed test.** PocketIC suites must `panic!` when the wasm or the PocketIC binary is missing; in proof-of-burn they printed a skip message and CI stayed green. `verify-local.sh` also greps test output for `skipped`/`ignored`.
5. `.did` drift check, AAA wasm ≤ 1.5 MiB gz, frontend bundle budgets, and no `dfx`/`fetchRootKey` strings in the repo.
6. Nightly via `just nightly` (non-blocking; failures are written to `.claude/harness/progress.md` for the next session): fuzzing, the LLM agent E2E, load test.

## 3. Test data & environments
- **Fixtures:**
  - 50 synthetic-but-realistic dossiers, 10 of them gold, plus 5 honeypots
  - a tiny FITS set
  - fixed-seed RNG (a test-only `admin_set_seed`, compiled only with feature `test-hooks`)
- **Time control:** PocketIC `advance_time` for leases, starvation, the 30-day claim reopen, the auto top-up interval, and maturity disbursement delays.
- **Environments:** `local` for `just verify`; `staging` for nightly LLM and E2E runs and the pre-release soak. Stripe is **never** needed in tests: the `stripe_credit` endpoint is exercised with a mock relay principal.

## 4. Definition of done (every task)
The task has a **demo** the owner can check in under a minute: a `just demo <task>` test with narrated steps, or a screenshot or image in `docs/demos/<task>/`. See `.claude/harness/README.md`.
The acceptance tests are written and green, the traceability rows are added, coverage gates hold, and the docs/spec have been updated if behaviour changed.
