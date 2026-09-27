# Space Compute — OKRs

*Living document. Update the "Current" column and status at every milestone (M1–M7 in `specs/10-tasks.md`) and at the end of each phase. Status: 🟢 on track · 🟡 at risk · 🔴 off track · ⚪ not started · ✅ done.*

**Cycle:** MVP build, 2026-09-28 → 2027-02-12 (public launch)
**Last updated:** 2026-09-26 (baseline, planning complete)

---

## O1 — Ship a trustworthy, agent-run citizen-science MVP on ICP

| KR | Target | Current | Status |
|---|---|---|---|
| KR1.1 Public launch on mainnet with all 10 designed screens live | by 2027-02-12 | planning done | ⚪ |
| KR1.2 All spec acceptance checks traced to automated tests and passing in local verification suite (11 §2) | 100% | 0% | ⚪ |
| KR1.5 Line coverage on platform/payments/treasury | ≥ 90% | — | ⚪ |
| KR1.3 External security review: 0 open high/critical findings at launch | 0 | — | ⚪ |
| KR1.4 Critical-path milestones hit within 5 working days of baseline (M1–M7) | 7/7 | 0/7 | ⚪ |

## O2 — Make running an agent astronomer effortless

| KR | Target | Current | Status |
|---|---|---|---|
| KR2.1 A new owner goes from sign-in to a first classification by their agent (Firebase funnel `sign_in` → `agent_first_activity`) | ≤ 15 min (median, beta) | — | ⚪ |
| KR2.2 Agent submissions rejected as `InvalidInput` (skill quality) | < 2% | — | ⚪ |
| KR2.3 Reference agent resists prompt-injection honeypots | ≥ 95% | — | ⚪ |
| KR2.4 Owners who fund (ICP/BTC/ETH/invite) successfully on the first try | ≥ 90% | — | ⚪ |
| KR2.5 Invite-code owners whose agent completes ≥ 50 tasks | ≥ 60% | — | ⚪ |

## O3 — Produce science people can cite

| KR | Target | Current | Status |
|---|---|---|---|
| KR3.1 Classifications completed during beta | ≥ 10,000 | 0 | ⚪ |
| KR3.2 Mean agreement with gold labels (tier ≥ 2 AAAs) | ≥ 85% | — | ⚪ |
| KR3.3 Confirmed discoveries with verified (certified) citations | ≥ 25 by launch | 0 | ⚪ |
| KR3.4 Discoveries resolved within 7 days of flagging | ≥ 80% | — | ⚪ |

## O4 — Sustainable economics

| KR | Target | Current | Status |
|---|---|---|---|
| KR4.1 Platform canister cycles covered by per-call fees + fuel-pack margin (beta month); the gap is the owner-funded float | ≥ 100% | — | ⚪ |
| KR4.2 Fuel treasury never breaches its reserve floor | 0 breaches | — | ⚪ |
| KR4.3 App canisters topped up automatically by the treasury keeper, no manual top-ups | 100% of months | — | ⚪ |
| KR4.5 Non-ICP intake auto-pauses before ops cycles run low (and nothing is stranded) | 0 ops-canister runway breaches while intake is open | — | ⚪ |
| KR4.4 Median owner cost per 1,000 classifications, published | ≤ $2 | — | ⚪ |

---

## Change log
| Date | Change |
|---|---|
| 2026-09-27 | T2.3: Catalog dispatch (subjects, protocol, leases, seen-set, get_task) completed in platform canister with rotating cursor, rate-limiting, and seen-set guarantee that an AAA never sees the same subject twice. |
| 2026-09-27 | T2.2: AAA registry & factory (register/install/verify/upgrade/profile) completed in platform canister, with strict provenance, name collision -2 auto-suffix, and PocketIC acceptance tests passing. |
| 2026-09-27 | T6.1: Frontend app scaffold implemented with 25 responsive mobile/desktop screens, certified static-site recipe in icp.yaml, responsive drawer and bottom bar navigation, and multi-currency fuel flow. |
| 2026-09-27 | No remote CI (owner directive): verification is local via `scripts/verify-local.sh` and Justfile; deterministic local deployment script `scripts/deploy-local.sh` established early; harness expanded with `local-deploy` skill. |
| 2026-09-27 | Continuous data refresh every 15 days (T7.11, 07 §5c) so agents always have new subjects. |
| 2026-09-27 | NNS neuron removed (owner): the treasury is owner-funded ICP, with the cycles keeper and health() unchanged; T5.17/T5.18 shrink, T8.5 = fund the prod float. Launch unchanged. |
| 2026-09-27 | Staffing: Claude Code builds everything in 3 parallel lanes; launch 2027-02-11. |
| 2026-09-27 | Intake auto-pause (KR4.5); functional-first UI with a design phase at the end, so launch moves to 2027-03-03; proof-of-burn neuron as the long-term yield source; model tiers + demos per task; agent harness. |
| 2026-09-27 | Whole-app gap review: Stripe hidden (card KR replaced), treasury keeper KR, test coverage KRs, invite-code KR. |
| 2026-09-26 | JWST data source; submission security; first-claim rule; admin console; Firebase analytics now the measurement source for O2 funnel KRs. |
| 2026-09-26 | Baseline OKRs set after specs, adversarial review and Gantt. Added card/BTC/ETH fuel packs (owner decision), so KR2.4 and KR4.2–4.3 were added. |
