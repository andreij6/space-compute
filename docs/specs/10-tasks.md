# 10 — Task plan (source for the Gantt chart)

Generated from the plan model; baseline start 2026-09-28. **Claude Code writes all the code** in 3 parallel lanes (parallel sessions committing straight to `main`; `Lane` column). Rows with lane `you` are the owner's: sign-off, beta, treasury funding, accounts, external audit. Days are agent-days, including tests, demo and review. US holidays excluded.
Each task's detailed specification is the linked spec section. A task is done when its acceptance check passes in `just verify` (no remote CI) or is recorded in the named doc.

## P0 — Planning

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T0.1 | Overview & design brief | - | you | 3 | — | 2026-09-21 | 2026-09-23 | OVERVIEW, DESIGN_BRIEF | Approved docs | note: 3-line summary in chat |
| T0.2 | Detailed specs + adversarial review | - | you | 3 | T0.1 | 2026-09-24 | 2026-09-25 | specs/01-09, REVIEW | All findings resolved or spiked | note: 3-line summary in chat |
| T0.3 | Gantt, review deck, OKRs | - | you | 1 | T0.2 | 2026-09-26 | 2026-09-26 | OKR.md | Artifacts published | note: 3-line summary in chat |
| T0.4 | Owner review & sign-off of plan | - | you | 3 | T0.3 | 2026-09-28 | 2026-09-30 | — | Go / no-go recorded in OKR.md | note: 3-line summary in chat |

## P1 — Foundations & spikes

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T1.1 | Repo scaffold: cargo workspace, icp.yaml, toolchain, .gitignore | L | 1 | 1 | T0.4 | 2026-10-01 | 2026-10-01 | 01 §3 | `icp build` succeeds for empty canisters | test: just demo T1.1  (narrated PocketIC/pytest run) |
| T1.2 | sc-types shared crate (types, ApiError, limits) | M | 1 | 2 | T1.1 | 2026-10-02 | 2026-10-05 | 02 §2, 08 §3 | Types compile; candid round-trip tests | test: just demo T1.2  (narrated PocketIC/pytest run) |
| T1.3 | Local verification & deploy harness (verify-local.sh, deploy-local.sh, did drift, wasm size) | L | 2 | 2 | T1.1 | 2026-10-02 | 2026-10-05 | 09 §2 | Local verification & deploy scripts pass | test: just demo T1.3  (narrated PocketIC/pytest run) |
| T1.4 | PocketIC harness with ICP ledger + CMC wasms | M | 3 | 2 | T1.1 | 2026-10-02 | 2026-10-05 | 09 §1 | Harness test mints ICP, calls CMC | test: just demo T1.4  (narrated PocketIC/pytest run) |
| SP-7 | Spike: Galaxy Zoo JWST (CEERS) labels, licence, mapping | M | 1 | 1 | T0.4 | 2026-10-06 | 2026-10-06 | REVIEW SP-7 | Gold source decided | note: decision + 1 proof command |
| T1.5 | JWST subject selection + gold (DJA v7 catalogs, 6 fields) | M | 1 | 5 | SP-7 | 2026-10-07 | 2026-10-13 | 07 §1,§4,§5.1-2 | 20k selected; ≥2k gold | image: docs/demos/T1.5/contact-sheet.png |
| T1.7 | Render dossiers (cutouts, RGB, dossier.json) + R2 bucket + manifest | M | 1 | 5 | T1.5 | 2026-10-14 | 2026-10-20 | 07 §3,§5.3-5 | Hash verify; 40 QA spot-checks | image: docs/demos/T1.7/contact-sheet.png |
| SP-1 | Spike: CMC + ICRC-2 memo behaviour | M | 2 | 1 | T1.4 | 2026-10-06 | 2026-10-06 | REVIEW SP-1 | Decision recorded in 04 §2 | note: decision + 1 proof command |
| SP-3 | Spike: frozen canister query behaviour | L | 3 | 1 | T1.4 | 2026-10-06 | 2026-10-06 | REVIEW SP-3 | Decision recorded | note: decision + 1 proof command |
| SP-4 | Spike: canister_info cost/latency | L | 2 | 1 | T1.4 | 2026-10-07 | 2026-10-07 | REVIEW SP-4 | Decision recorded | note: decision + 1 proof command |
| SP-6 | Spike: ckETH helper subaccount deposit + minimum | M | 3 | 1 | T1.4 | 2026-10-07 | 2026-10-07 | REVIEW SP-6 | Decision recorded in 04 §6.6 | note: decision + 1 proof command |
| SP-2 | Spike: OISY approve w/ spender subaccount | M | 2 | 1 | T0.4 | 2026-10-08 | 2026-10-08 | REVIEW SP-2 | Decision recorded | note: decision + 1 proof command |
| T1.8 | Test infra: coverage gate, proptest, fuzz, bot-agent harness, traceability, fixture dossiers (Playwright+axe → T6.10) | M | 3 | 3 | T1.3, T1.4 | 2026-10-08 | 2026-10-12 | 11 | Gates run in just verify on empty suites | test: just demo T1.8  (narrated PocketIC/pytest run) |

## P2 — Platform core

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T2.1 | platform skeleton: config/admin, memory map, timers, RNG | M | 2 | 2 | T1.2 | 2026-10-09 | 2026-10-12 | 02 §3, §9-10 | Upgrade keeps config | test: just demo T2.1  (narrated PocketIC/pytest run) |
| T2.2 | Registry & factory: register/install/verify/upgrade/profile | H | 2 | 4 | T2.1, SP-4 | 2026-10-13 | 2026-10-16 | 02 §4 | Spawned AAA verified; upgrade works | test: just demo T2.2  (narrated PocketIC/pytest run) |
| T2.3 | Catalog: subjects, protocol, leases, seen-set, get_task (gold+calibration) | M | 3 | 4 | T2.1 | 2026-10-13 | 2026-10-16 | 02 §5.1 | Never same subject twice | test: just demo T2.3  (narrated PocketIC/pytest run) |
| T2.4 | Scoring: validation, gold, tallies, retirement, consensus | H | 2 | 4 | T2.3 | 2026-10-19 | 2026-10-22 | 02 §5.2, §5.5 | Retire at K=5; idempotent submit | test: just demo T2.4  (narrated PocketIC/pytest run) |
| T2.5 | Event log + per-AAA index + activity query | M | 3 | 2 | T2.1 | 2026-10-19 | 2026-10-20 | 02 §8.1 | Events append; paged activity | test: just demo T2.5  (narrated PocketIC/pytest run) |
| T2.6 | Public queries: stats, protocol, aaa_by_owner, aaa_public | L | 1 | 1 | T2.2 | 2026-10-21 | 2026-10-21 | 02 §7 | Queries paged ≤100 | test: just demo T2.6  (narrated PocketIC/pytest run) |
| T2.8 | Submitter security: strict provenance, operator sync, submitted_by | H | 3 | 3 | T2.2 | 2026-10-21 | 2026-10-23 | 02 §4.6, §5; 08 S21-23 | Foreign/expired/unsynced submitter rejected | test: just demo T2.8  (narrated PocketIC/pytest run) |
| T2.7 | Platform PocketIC tests (02 §11 #1-3) | M | 1 | 2 | T2.4, T1.4 | 2026-10-23 | 2026-10-26 | 02 §11 | Tests green | test: just demo T2.7  (narrated PocketIC/pytest run) |

## P3 — AAA canister & agent kit

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T3.1 | AAA skeleton: roles, operators, config, inspect_message | M | 2 | 2 | T1.2 | 2026-10-23 | 2026-10-26 | 03 §1-3 | Role matrix enforced | test: just demo T3.1  (narrated PocketIC/pytest run) |
| T3.2 | AAA forwarding w/ fees, retries, idempotency, low-cycles guard | H | 1 | 3 | T3.1, T2.4 | 2026-10-27 | 2026-10-29 | 03 §4.1 | One record per task under SYS_UNKNOWN | test: just demo T3.2  (narrated PocketIC/pytest run) |
| T3.3 | AAA repository records + queries + credits copy | L | 1 | 2 | T3.2 | 2026-10-30 | 2026-11-02 | 03 §5 | Records survive upgrade | test: just demo T3.3  (narrated PocketIC/pytest run) |
| T3.4 | AAA timers: burn EMA, heartbeat, credits sync, auto top-up trigger | M | 1 | 2 | T3.3 | 2026-11-03 | 2026-11-04 | 03 §6 | Timer tests pass | test: just demo T3.4  (narrated PocketIC/pytest run) |
| T3.5 | get_api_doc + wasm size budget ≤1.5 MiB gz | L | 2 | 1 | T3.2 | 2026-10-30 | 2026-10-30 | 03 §4.3 | Size check in just verify | test: just demo T3.5  (narrated PocketIC/pytest run) |
| T3.6 | AAA PocketIC tests (03 §8) | M | 1 | 2 | T3.4 | 2026-11-05 | 2026-11-06 | 03 §8 | Tests green | test: just demo T3.6  (narrated PocketIC/pytest run) |
| T3.8 | Local seed script + dev loop (tools/seed-local) | L | 3 | 1 | T2.2, T3.2, T1.7, T1.8 | 2026-10-30 | 2026-10-30 | 05 §5 | One command seeds local net | test: just demo T3.8  (narrated PocketIC/pytest run) |
| T3.7 | Operator skill + JWST protocol primer + analyze.py | H | 2 | 4 | T3.5, T1.7 | 2026-11-02 | 2026-11-05 | 06 | 10 local tasks by Claude Code | test: just demo T3.7  (narrated PocketIC/pytest run) |
| T3.9 | Headless runner recipe + practice.py self-eval | L | 2 | 1 | T3.7 | 2026-11-06 | 2026-11-06 | 06 §2b-2c | Runner stops at fuel guard | test: just demo T3.9  (narrated PocketIC/pytest run) |

## P4 — Review, consensus & credits

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T4.3 | Progression: XP, reputation, tiers, badges, leaderboard, replay | M | 3 | 4 | T2.5, T2.7 | 2026-11-02 | 2026-11-05 | 02 §8.2 | Replay == incremental | test: just demo T4.3  (narrated PocketIC/pytest run) |
| T4.1 | Discoveries: flagging, rate limit, public IDs | M | 3 | 2 | T2.4 | 2026-11-06 | 2026-11-09 | 02 §6 | Flag rate enforced | test: just demo T4.1  (narrated PocketIC/pytest run) |
| T4.9 | First-claim rule: sky claim index + corroborations | H | 1 | 2 | T4.1 | 2026-11-10 | 2026-11-11 | 02 §6.4 | Concurrent flags → one discovery | test: just demo T4.9  (narrated PocketIC/pytest run) |
| T4.2 | Review assignment: eligibility, blind, leases, honeypots | H | 2 | 3 | T4.1, T4.3 | 2026-11-10 | 2026-11-12 | 02 §5.3, §6.3 | Same-owner never assigned | test: just demo T4.2  (narrated PocketIC/pytest run) |
| T4.4 | evaluate(), atomic resolution, starvation rule | H | 1 | 3 | T4.2 | 2026-11-13 | 2026-11-17 | 02 §6.2 | Atomic resolution test | test: just demo T4.4  (narrated PocketIC/pytest run) |
| T4.5 | Citations + certification tree + get_citation | H | 1 | 3 | T4.4 | 2026-11-18 | 2026-11-20 | 02 §8.3 | Witness verifies after upgrade | test: just demo T4.5  (narrated PocketIC/pytest run) |
| T4.6 | Visibility rules & public discovery queries | M | 1 | 2 | T4.5 | 2026-11-23 | 2026-11-24 | 02 §7 | Under-review hidden from non-owners | test: just demo T4.6  (narrated PocketIC/pytest run) |
| T4.7 | Honeypot seeding (120 specs) | L | 2 | 1 | T4.2, T1.7 | 2026-11-13 | 2026-11-13 | 07 §3.6 | Honeypots uploaded locally | image: docs/demos/T4.7/contact-sheet.png |
| T4.10 | Admin read APIs + audit log (platform) | L | 1 | 2 | T4.6 | 2026-11-25 | 2026-11-30 | 02 §9 | Admin queries paged; audit on every mutation | test: just demo T4.10  (narrated PocketIC/pytest run) |
| T4.11 | Name blocklist, admin_rename_aaa, house AAAs | L | 1 | 1 | T4.10 | 2026-12-01 | 2026-12-01 | 02 §9 | Renamed AAA keeps historical citation name | test: just demo T4.11  (narrated PocketIC/pytest run) |
| T4.8 | Review/credits tests + proptests (02 §11 #4-9) | M | 2 | 3 | T4.6, T4.9 | 2026-11-25 | 2026-12-01 | 02 §11, 09 §1 | Tests green | test: just demo T4.8  (narrated PocketIC/pytest run) |

## P5 — Payments

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T5.1 | payments skeleton: config, journal, guards, admin | M | 3 | 2 | T1.2, SP-1 | 2026-11-10 | 2026-11-11 | 04 §3 | Journal persisted pre-await | test: just demo T5.1  (narrated PocketIC/pytest run) |
| T5.2 | CMC integration, XDR cache, quotes, deposit accounts | M | 3 | 2 | T5.1 | 2026-11-12 | 2026-11-13 | 04 §2, §4 | Quotes within 2% | test: just demo T5.2  (narrated PocketIC/pytest run) |
| T5.3 | Spawn saga (wallet + deposit) + resume timer | H | 3 | 4 | T5.2, T2.2 | 2026-11-16 | 2026-11-19 | 04 §4 | 04 §5 #1,#2,#4 | test: just demo T5.3  (narrated PocketIC/pytest run) |
| T5.4 | One-time top-up (wallet + deposit sweep) | M | 3 | 2 | T5.2 | 2026-11-20 | 2026-11-23 | 04 §4 | Gift top-up works | test: just demo T5.4  (narrated PocketIC/pytest run) |
| T5.5 | Auto top-up mandates, rolling 30-day cap | M | 3 | 3 | T5.4, T3.4 | 2026-11-24 | 2026-11-30 | 04 §4 | 04 §5 #5 | test: just demo T5.5  (narrated PocketIC/pytest run) |
| T5.6 | Payments PocketIC tests — ICP paths (04 §5) | M | 3 | 3 | T5.3, T5.5 | 2026-12-01 | 2026-12-03 | 04 §5 | All 7 green | test: just demo T5.6  (narrated PocketIC/pytest run) |
| T5.8 | XRC rates, fuel treasury guard rails + auto intake pause | H | 1 | 4 | T5.2 | 2026-12-02 | 2026-12-07 | 04 §6.1-6.3, §6.2b | 04 §6.7 #11-13, #15-17 | test: just demo T5.8  (narrated PocketIC/pytest run) |
| T5.12 | stripe_credit endpoint (flag-gated, mock-relay tests) | L | 1 | 2 | T5.8 | 2026-12-08 | 2026-12-09 | 04 §6.4, 04b §3 | Idempotent on stripe_ref | test: just demo T5.12  (narrated PocketIC/pytest run) |
| T5.9 | ckBTC fuel-pack deposits (address, update_balance, sweep) | H | 2 | 3 | T5.8 | 2026-12-08 | 2026-12-10 | 04 §6.5 | 04 §6.7 #10 | test: just demo T5.9  (narrated PocketIC/pytest run) |
| T5.10 | ckETH fuel-pack deposits | M | 3 | 2 | T5.8, SP-6 | 2026-12-08 | 2026-12-09 | 04 §6.6 | Mock mint → top-up | test: just demo T5.10  (narrated PocketIC/pytest run) |
| T5.13 | Non-ICP payment tests (04 §6.7) | M | 1 | 2 | T5.9, T5.10, T5.12 | 2026-12-11 | 2026-12-14 | 04 §6.7 | All green | test: just demo T5.13  (narrated PocketIC/pytest run) |
| T5.15 | Payments admin APIs + audit log | L | 1 | 1 | T5.13 | 2026-12-15 | 2026-12-15 | 04 §4 admin | Overview matches ledger balances | test: just demo T5.15  (narrated PocketIC/pytest run) |
| T5.16 | Feature flags + invite codes (sponsored spawn) | M | 3 | 2 | T5.3 | 2026-12-10 | 2026-12-11 | 04 §0-0b | card=false → FeatureDisabled; invite single-use | test: just demo T5.16  (narrated PocketIC/pytest run) |
| T5.17 | treasury canister: owner-funded reserve, cycles keeper, health() | H | 2 | 3 | T2.1, T1.4 | 2026-12-11 | 2026-12-15 | 12 §1-4 | Keeper tops up; health() drives intake pause | test: just demo T5.17  (narrated PocketIC/pytest run) |
| T5.18 | treasury PocketIC tests (ICP ledger + CMC, no NNS) | M | 1 | 1 | T5.17 | 2026-12-16 | 2026-12-16 | 12 §5 | 12 §5 #1-6 green | test: just demo T5.18  (narrated PocketIC/pytest run) |
| T5.7 | Internal payments security review | H | 3 | 2 | T5.6, T5.13, T5.16 | 2026-12-15 | 2026-12-16 | 08 §4 | Checklist signed | test: just demo T5.7  (narrated PocketIC/pytest run) |

## P6 — Frontend

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T6.1 | App scaffold: routing, bindgen, II auth, ic_env, CSP | M | 2 | 3 | T1.1 | 2026-12-16 | 2026-12-18 | 05 §1 | Sign-in works locally | shots: docs/demos/T6.1/*.png (Playwright) |
| T6.3 | Landing, Discovery Museum feed, Discovery detail + citation verify | M | 1 | 5 | T6.1 | 2026-12-21 | 2026-12-29 | 05 §2-3 | Verified mark fails closed | shots: docs/demos/T6.3/*.png (Playwright) |
| T6.4 | AAA profile + Leaderboard | L | 1 | 3 | T6.3 | 2026-12-30 | 2027-01-05 | 05 §2 | Tier/badges render | shots: docs/demos/T6.4/*.png (Playwright) |
| T6.5 | Payment component (wallet + deposit) | M | 2 | 4 | T6.1, SP-2, T5.4 | 2026-12-21 | 2026-12-28 | 05 §3 | Deposit path e2e local | shots: docs/demos/T6.5/*.png (Playwright) |
| T6.6 | Spawn flow | L | 2 | 2 | T6.5, T5.3 | 2026-12-29 | 2026-12-30 | 05 §2 | Spawn e2e local | shots: docs/demos/T6.6/*.png (Playwright) |
| T6.7 | Owner dashboard | M | 3 | 4 | T6.1, T3.3, T4.6 | 2026-12-21 | 2026-12-28 | 05 §2-3 | Frozen fallback works | shots: docs/demos/T6.7/*.png (Playwright) |
| T6.8 | Connect your agent + Activity & records | L | 3 | 3 | T6.7 | 2026-12-29 | 2027-01-04 | 05 §3 | Operator add/revoke | shots: docs/demos/T6.8/*.png (Playwright) |
| T6.9 | Fuel & billing + auto top-up | L | 2 | 3 | T6.5, T5.5 | 2027-01-04 | 2027-01-06 | 05 §3 | Mandate states render | shots: docs/demos/T6.9/*.png (Playwright) |
| T6.11 | BTC & ETH methods + feature-flag gating (card hidden) | M | 3 | 4 | T6.5, T5.9, T5.10 | 2027-01-05 | 2027-01-08 | 05 §3 | Test-mode card + mock BTC/ETH e2e | shots: docs/demos/T6.11/*.png (Playwright) |
| T6.12 | Admin console (8 screens) | M | 1 | 5 | T6.1, T4.10, T5.15 | 2027-01-06 | 2027-01-12 | 05 §2b | Non-admin gets 404/Unauthorized | shots: docs/demos/T6.12/*.png (Playwright) |
| T6.13 | Firebase analytics + consent + event taxonomy + perf traces | L | 2 | 2 | T6.1 | 2027-01-07 | 2027-01-08 | 05 §4b | No requests before consent | shots: docs/demos/T6.13/*.png (Playwright) |
| T6.14 | About/legal/practice pages, invite spawn UI, admin invites/treasury/moderation | L | 1 | 3 | T6.12, T5.16, T5.18 | 2027-01-13 | 2027-01-15 | 05 §2, §2b | E2E: invite spawn; non-admin blocked | shots: docs/demos/T6.14/*.png (Playwright) |
| T6.10 | States, accessibility, Playwright smoke | M | 1 | 3 | T6.4, T6.6, T6.8, T6.9, T6.11, T6.12, T6.13, T6.14 | 2027-01-18 | 2027-01-20 | 05 §5 | Lighthouse a11y ≥90 | shots: docs/demos/T6.10/*.png (Playwright) |

## P7 — Hardening

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T7.1 | Upgrade tests for all canisters | M | 2 | 2 | T4.8, T5.6 | 2027-01-11 | 2027-01-12 | 09 §1 | vN-1→vN state intact | test: just demo T7.1  (narrated PocketIC/pytest run) |
| T7.2 | Load test: 200 AAAs × 100 tasks | M | 2 | 2 | T7.1 | 2027-01-13 | 2027-01-14 | 09 §1 | Report committed | test: just demo T7.2  (narrated PocketIC/pytest run) |
| T7.3 | Cycle cost measurement & fee retune | M | 2 | 2 | T7.2 | 2027-01-15 | 2027-01-18 | 01 §7, SP-5 | Params updated | test: just demo T7.3  (narrated PocketIC/pytest run) |
| T7.4 | Reproducible AAA build + manual upgrade docs | M | 3 | 2 | T3.6, T5.7 | 2027-01-11 | 2027-01-12 | 08 §7 | Hash reproducible on 2 machines | test: just demo T7.4  (narrated PocketIC/pytest run) |
| T7.7 | Prompt-injection test with reference agent | H | 3 | 1 | T3.7, T4.7 | 2027-01-13 | 2027-01-13 | 06 §5 | ≥95% non-compliance | test: just demo T7.7  (narrated PocketIC/pytest run) |
| T7.9 | Traceability audit + coverage gates to thresholds | L | 3 | 2 | T7.1 | 2027-01-14 | 2027-01-15 | 11 §2 | Every acceptance item mapped; coverage ≥ gates | test: just demo T7.9  (narrated PocketIC/pytest run) |
| T7.10 | Practice set + open data release tooling | L | 3 | 2 | T1.7, T4.8 | 2027-01-18 | 2027-01-19 | 07 §5b | Release v0 reproducible | image: docs/demos/T7.10/contact-sheet.png |
| T7.5 | External security review (payments + platform) | - | you | 5 | T5.7, T4.8 | 2026-12-17 | 2026-12-23 | 08 §4 | Report received | note: 3-line summary in chat |
| T7.6 | Fix security findings | H | 2 | 3 | T7.5 | 2027-01-19 | 2027-01-21 | 08 | All high/critical closed | test: just demo T7.6  (narrated PocketIC/pytest run) |

## P8 — Beta & launch

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T8.1 | Staging deploy, deploy workflow, snapshots, cycles monitoring | M | 3 | 2 | T7.1 | 2027-01-20 | 2027-01-21 | 09 §2-3 | Staging live | test: just demo T8.1  (narrated PocketIC/pytest run) |
| T8.2 | Upload subjects/protocol/honeypots to staging | L | 1 | 1 | T8.1, T7.6 | 2027-01-22 | 2027-01-22 | 07 | Counts verified | test: just demo T8.2  (narrated PocketIC/pytest run) |
| T8.5 | Fund prod treasury with ICP float (10–20 ICP) + confirm runway | - | you | 1 | T8.6 | 2027-02-10 | 2027-02-10 | 12 §4 | status() shows ≥ 90 days runway | note: 3-line summary in chat |
| T8.3 | Closed beta (10-20 owners) | - | you | 10 | T8.2, T6.10 | 2027-01-25 | 2027-02-05 | OKR KR | Beta KRs measured | note: 3-line summary in chat |
| T8.4 | Beta fixes & tuning | M | 1 | 10 | T8.2, T6.10 | 2027-01-25 | 2027-02-05 | — | No open P0/P1 bugs | test: just demo T8.4  (narrated PocketIC/pytest run) |
| T8.6 | Production deploy, II metadata, treasury watch list | M | 1 | 2 | T8.3, T8.4, T8.13 | 2027-02-08 | 2027-02-09 | 09 §2 | Prod live | test: just demo T8.6  (narrated PocketIC/pytest run) |
| T8.7 | Public launch | - | you | 1 | T8.6, T8.5, T9.5 | 2027-02-11 | 2027-02-11 | OKR | Launch announced | note: 3-line summary in chat |
| T8.13 | Custom domain + II alternative origins | - | you | 1 | T8.1 | 2027-01-22 | 2027-01-22 | 05 §1 | Domain serves frontend; II principal stable | note: 3-line summary in chat |
| T8.12 | Firebase projects (staging/prod) + GA4 funnels & dashboards | - | you | 1 | T0.4 | 2026-10-01 | 2026-10-01 | 05 §4b | Funnel dashboard live | note: 3-line summary in chat |
| T8.8 | Compliance check for card & crypto fuel packs (recommended) | - | you | 5 | T0.4 | 2026-10-01 | 2026-10-07 | 08 S20 | Written go/no-go | note: 3-line summary in chat |
| T8.9 | Enable BTC/ETH on production (card stays off) | - | you | 1 | T8.6, T8.8 | 2027-02-10 | 2027-02-10 | 04 §6.2 | admin_pause_non_icp off | note: 3-line summary in chat |

## P9 — Design & polish (after function)

| ID | Task | Model | Lane | Days | Depends on | Start | End | Spec | Acceptance | Demo |
|---|---|---|---|---|---|---|---|---|---|---|
| T9.1 | Import mockups (Claude Design), reconcile, design tokens + shared components | M | 2 | 4 | T6.10 | 2027-01-22 | 2027-01-27 | 05 intro | 05a-design-reconciliation.md + component page | shots: docs/demos/T9.1/*.png (Playwright) |
| T9.2 | Style public pages: landing, museum, discovery, profile, leaderboard | M | 2 | 4 | T9.1 | 2027-01-28 | 2027-02-02 | 05 §2-3, mockups | Visual snapshots approved by owner | shots: docs/demos/T9.2/*.png (Playwright) |
| T9.3 | Style owner flows: spawn, dashboard, connect agent, fuel, payments | M | 3 | 4 | T9.1 | 2027-01-28 | 2027-02-02 | 05 §2-3, mockups | Visual snapshots approved by owner | shots: docs/demos/T9.3/*.png (Playwright) |
| T9.4 | Style admin console + about/legal/practice pages | L | 2 | 2 | T9.1 | 2027-02-03 | 2027-02-04 | 05 §2b | Visual snapshots | shots: docs/demos/T9.4/*.png (Playwright) |
| T9.5 | Responsive/mobile pass, visual-regression baseline, a11y + Lighthouse re-run | L | 2 | 2 | T9.2, T9.3, T9.4 | 2027-02-05 | 2027-02-08 | 05 §4-5 | Lighthouse a11y ≥90, perf ≥80; baseline committed | shots: docs/demos/T9.5/*.png (Playwright) |

## Deferred (not scheduled — owner decision 2026-09-27: Stripe hidden at launch)

| ID | Task | Spec |
|---|---|---|
| D1 | Stripe relay Worker: checkout, webhook, portal | 04b |
| D2 | Stripe account, products, Radar | 04b §2 |
| D3 | Card tab + subscription management UI | 05 §3 |
| D4 | Stripe reconciliation job + enable `features.card` | 04b §3, 09 §3 |

## Milestones

| Milestone | Gate task | Date |
|---|---|---|
| M1 Foundations + JWST data ready | T1.7 | 2026-10-20 |
| M2 Local alpha: agent classifies end-to-end | T3.8 | 2026-10-30 |
| M3 Review & credits complete | T4.8 | 2026-12-01 |
| M4 Payments complete | T5.7 | 2026-12-16 |
| M5 Frontend feature-complete (unstyled) | T6.10 | 2027-01-20 |
| M5b Design applied | T9.5 | 2027-02-08 |
| M6 Staging loaded & beta-ready backend | T8.2 | 2027-01-22 |
| M7 Public launch | T8.7 | 2027-02-11 |
