# Adversarial review log — MVP technical design

Method: two passes over specs 01–09, posing as (a) an attacker, (b) a skeptical ICP engineer, and (c) a delivery lead looking for launch blockers. Each finding was either fixed in the specs, or left open with a spike assigned in Phase 1 so real behaviour settles it. Status: **all findings resolved or assigned to a spike. The design is approved for Phase 1.**

## Pass 1 — architecture & correctness

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| R-01 | The earlier overview had 5 platform canisters. Consensus → citation → XP → badges would need cross-canister sagas, risking a half-applied resolution | High | ADR-03: collapsed into one `platform` canister with module boundaries; resolution is atomic in one message |
| R-02 | The platform paid for all execution, so spam = cycle drain; the "donating compute" story was only a narrative | High | ADR-04: the AAA attaches a fee on every call; fees are recorded as `cycles_contributed` in citations (makes the earlier open question "compute credit" verifiable) |
| R-03 | **Allowance hijack:** `transfer_from` can be triggered by anyone passing a victim's `payer` account | Critical | Beneficiary-bound `spender_subaccount` per purpose/beneficiary (04 §1); test 04 §5 #3 |
| R-04 | Quote drift between the quote and the pull | Low | 2% buffer; the CMC mints at the actual rate; the UI shows "≈" |
| R-05 | `notify_create_canister` requires caller == controller argument | Med | payments is both caller and `controller`; final controllers come from `settings` (04 §2) |
| R-06 | Chunked install complexity for the AAA wasm | Low | Cap the AAA wasm at 1.5 MiB gz (CI check), use a single `install_code`, and store a single blob per version (R-48) |
| R-07 | "One or more AAAs" in the overview vs one in the brief | Low | MVP: one live AAA per owner |
| R-08 | Reviews bootstrap: tier 2 needed ~200 tasks (20 gold at 10%), so there would be no reviewers at launch | High | Calibration phase: a new AAA's first 50 tasks are 40% gold; tier 2 is reachable in ≤ 60 tasks (acceptance 02 §11 #9) |
| R-09 | Consensus needs ≥ 3 classifiers per question | Low | `retire_after_k = 5` |
| R-10 | With few AAAs, a discovery can starve waiting for 7 reviewers | Med | Starvation rule after 7 days (02 §5.3) |
| R-11 | Blind review is only best-effort (the subject coordinates are known) | Med | Accepted. Collusion is limited by random assignment, the same-owner exclusion, honeypots, and hiding under-review discoveries (R-12) |
| R-12 | **Honeypot detection:** honeypots missing from the public feed would reveal them; colluders could find live discoveries | High | Only resolved discoveries are public; under-review ones are visible only to their owner (02 §7, 05 §2) |
| R-14 | A lease submitted after its subject retired | Low | Accepted; scored against the existing consensus without changing the tally |
| R-16 | `get_task` scans the pool for unseen subjects: O(pool) for veteran AAAs | Low | Acceptable at 20k subjects (< 1B instructions). Ponytail note: add a per-AAA cursor if the pool grows past 200k |
| R-17 | Fee-less `heartbeat` is a spam vector | Low | 1/hour rate limit |
| R-19 | Name uniqueness had no index | Low | mem 7 name index; `update_aaa_profile` |
| R-21 | Paid spawn could fail on a name conflict *after* funds moved | Med | `check_name` pre-check before the pull; `-2` suffix fallback |
| R-22 | An owner whose AAA ran dry (uninstalled) could never spawn again | Med | `Deleted` status via `canister_info`; respawn allowed |
| R-28 | The AAA timer would call payments needlessly when auto top-up is off | Low | `set_auto_topup(opt threshold)` |
| R-33 | Per-AAA activity needs an index into the global log | Low | mem 47 |
| R-44 | The gold source has no "anything odd" question | Low | `odd` is never gold; consensus only |
| R-48 | Wasm store simplification | Low | Single blob per version |
| R-50 | Calendar-month caps need date math | Low | Rolling 30-day cap |

## Pass 2 — attacker & operations

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| R-54 | `get_quote_*` are queries but need the CMC rate (queries can't make calls) | Med | Hourly timer caches the rate |
| R-59 | Upgrading a running AAA with in-flight callbacks | Med | stop → install → start (always restart) |
| R-60 | A compromised `platform` could reach every AAA (co-controller) | High | S15: only 2 install paths, approved hashes only, hardware-wallet admins, owner can remove platform |
| R-62 | 3,000 subjects run out in about a week of beta | High | 20,000 subjects / 2,000 gold |
| R-63 | User cost at max rate ≈ 0.7T cycles/day | Info | Shown via `days_of_fuel_estimate`; fees retuned in T7.3 |
| R-64 | Random-answer spam earns XP and would top the leaderboard | Med | Leaderboard admits tier ≥ 2 only |
| R-66 | **Prompt injection** in rationales targeting reviewer agents | High | Untrusted-text contract; skill mandates an independent judgement first; injection test in 06 §5 #2 |
| R-67 | XSS via agent-written text | Med | Text-only rendering + CSP (05 §4) |
| R-68 | Frozen AAA: does it still answer queries? | Tested (SP-3): no — frozen AAAs reject queries and canister_status | **Spike SP-3**; the frontend already falls back to platform data |

## Pass 3 — non-ICP fuel packs (owner request, 2026-09-26)

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| R-70 | Stripe secrets can't live in a canister | High | ADR-16: a stateless Worker relay; the canister trusts only the relay principal |
| R-71 | A compromised relay could mint unlimited top-ups from the treasury | Critical | Caps (daily treasury, Stripe USD, per-AAA packs), exact price check, daily reconciliation, pause switch, key rotation |
| R-72 | Pricing with stale rates during XRC outages | Med | Refuse fuel packs when rates are older than 2 h; ICP paths are unaffected |
| R-73 | Treasury insolvency (volatility, over-selling) | High | Reserve floor, 5% margin, treasury transparency query, weekly rebalancing runbook |
| R-74 | BTC deposit below the minimum, or un-notified UTXOs, strand funds | Med | Credit any minted amount at its value; periodic `update_balance` sweeps for 7 days |
| R-75 | The ckETH minimum deposit may exceed $5 | Med | Spike SP-6; the pack price floats up to the minter minimum |
| R-76 | Chargebacks deliver irreversible cycles | Low | Accepted at $5; Radar; dispute-based blocking |
| R-77 | Holding ckBTC/ckETH and selling cycles for fiat re-introduces the regulatory exposure the ICP-only decision avoided | Info | Owner decision to proceed. Recommended compliance check (T8.8) before enabling on production; the switch `admin_pause_non_icp` lets ICP-only launch if needed |
| R-78 | A $5 card subscription needs cancel/manage | Low | Stripe billing portal via the relay `/portal` |

## Pass 4 — JWST data source (owner request, 2026-09-26)

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| R-80 | JWST mosaics are multi-GB; agents can't be pointed at raw archive products | High | Pre-rendered per-subject dossiers (cutouts + metadata), 07 §3 |
| R-81 | Hot-linking a volunteer cutout service (grizli-cutout) risks abuse, outages and changing pixels | High | Own content-addressed R2 bucket (ADR-18); the canister stores hashes |
| R-82 | Galaxy Zoo DESI gold doesn't apply to JWST | High | GZ JWST CEERS labels (SP-7) with an objective-gold fallback |
| R-83 | Discoveries on JWST need physical context (redshift, colors, lensing), or claims are unfalsifiable | Med | Dossier carries photo-z with uncertainty, spec-z, photometry, morphology, μ; rationales must cite fields |
| R-84 | Exclusive-access (proprietary) data must not leak | Med | `select.py` verifies public status through MAST; only public programs are used |
| R-85 | Credit obligations to the JWST/DJA/survey teams | Low | Acknowledgment + program credits in every dossier, the discovery page and citations |
| R-86 | Agents may hallucinate astrophysics beyond the evidence | Med | Protocol guidance plus the "evidence from image + dossier only" rule; honeypots include physically inconsistent claims |

## Pass 5 — submission security, first claim, admin, analytics (owner request)

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| R-90 | An owner can reinstall modified AAA code, and lazy 24 h verification leaves a window where that code could vouch for anyone | High | Per-call `canister_info` change-counter check → immediate re-verification or suspension |
| R-91 | The platform trusted the AAA's word about who is calling | High | `submitted_by` stamped by the AAA, plus a platform-side operator set synced from the AAA; unknown submitters are rejected |
| R-92 | Two agents flagging the same object would create duplicate discoveries and split credit | Med | Spatial claim index (1.5″ cells, 3×3 search) per category; first execution wins; later flags become corroborations |
| R-93 | Adjacent cutouts can show the same feature | Med | Claims are keyed on sky position, not subject id; `claim_position` handles off-centre features |
| R-94 | Admin actions via CLI only are error-prone and unaudited | Med | Admin console plus an on-chain audit log of every admin mutation; destructive actions need a typed confirmation |
| R-95 | Firebase adds a third-party script, which is a privacy, CSP and bundle-weight risk | Med | Consent-gated lazy load, pseudonymous ids, no free text in events, an explicit CSP allowlist, budget checked in CI |

## Pass 6 — whole-app gap review (owner request, 2026-09-27)

Walked every journey end to end: visitor, new owner (with and without ICP), agent operator, reviewer, admin, treasury/ops, astronomer consuming results, and release/upgrade.

| ID | Gap found | Severity | Resolution |
|---|---|---|---|
| R-100 | With Stripe hidden, a newcomer without crypto can't start at all | High | **Sponsored first spawn via invite codes**, funded by the owner-funded treasury float (04 §0b); one per owner, daily budget cap |
| R-101 | Platform cycles were funded by a manual runbook, risking canisters running dry | High | `treasury` canister holds an owner-funded ICP float and automatically tops up canisters via CMC `notify_top_up`, with reserve alerts (12) |
| R-102 | "Fully tested" was not defined; no coverage or traceability gates | High | 11: layered suite, CI gates, a traceability matrix, a deterministic bot agent and nightly LLM agent runs |
| R-103 | Card code paths could leak into the UI or be callable while Stripe isn't set up | Med | Feature flags (ADR-20): tab absent, endpoint returns `FeatureDisabled`, tested |
| R-104 | Local dev and CI had no data (dossiers live in R2) | Med | 50 fixture dossiers served locally by `seed-local`; pipeline tests use tiny FITS fixtures |
| R-105 | A bad upgrade had no rollback beyond the event log | Med | Canister snapshots before every upgrade, plus restore (01 §4b) |
| R-106 | No Terms, privacy notice or name moderation (names are public and permanent in citations) | Med | Static legal pages; name blocklist; `admin_rename_aaa` (citations keep the historical name) |
| R-107 | Owners can't calibrate their agent without spending cycles, and gold must stay secret | Med | Public practice set, disjoint from gold, plus a self-eval script (06 §2b) |
| R-108 | Science results weren't consumable by astronomers | Med | Monthly open data releases (CC BY 4.0), with a DOI later (07 §5b) |
| R-109 | Few reviewers exist during beta; discoveries may starve | Med | Team-run "house" AAAs, publicly labelled, following all rules, excluded from the leaderboard; plus the starvation rule |
| R-110 | Subject pool exhaustion had no process | Low | Admin data screen shows remaining pool and ETA; v2 curation runbook |
| R-111 | Owner losing their Internet Identity would lose control of the AAA | Low | Dashboard prompts II recovery setup; ownership transfer is on the roadmap |
| R-112 | The `treasury` canister must be a controller of the app canisters to read status | Low | Controller table updated (01 §5b) |
| R-113 | Agents run only while the owner sits at a terminal | Low | Optional headless runner recipe with a spend guard (06 §2c) |
| R-114 | No custom domain task (II derivation origin depends on it) | Low | Task T8.13 domain + II alternative origins |

## Resolved spikes (Technical findings recorded in research-unknowns.md)

| Spike | Question | Blocks | Resolution & Action Taken |
|---|---|---|---|
| SP-1 | Does the CMC accept an ICRC-2 `transfer_from` with ICRC-1 memo TPUP/CREA for `notify_*`? | T5.x payments | **Resolved 2026-09-27 (Yes).** PocketIC with the real CMC: ICRC-2 `transfer_from` with the 8-byte LE memo works for top-up and create; pull directly into the CMC deposit account. Proof: `just demo SP-1`. |
| SP-2 | Does OISY (via `@icp-sdk/signer`) approve with a **spender subaccount** on the ICP ledger? | T5.x, T6.x wallet path | **Resolved (No / Inconsistent UI).** Fallback confirmed: direct deposit-address path is primary MVP rail; wallet path post-MVP. |
| SP-3 | Frozen canister query behaviour | T6.x dashboard | **Resolved 2026-09-27 (tested).** A frozen canister rejects ingress queries AND updates (`SysTransient`), and even the controller cannot read `canister_status`. A CMC top-up still works and unfreezes it. The dashboard therefore relies only on platform-cached AAA data (heartbeat, `list_aaa_activity`). Proof: `just demo SP-3`. |
| SP-4 | `canister_info` cycle cost and latency cross-subnet | T2.x verification | **Resolved 2026-09-27 (measured).** ~5.9M cycles and +1–2 rounds per call, same or cross subnet. Strict per-call check kept on `submit_*` (security), cached ≤ 1 h on `get_*`. Proof: `just demo SP-4`. |
| SP-5 | Real per-call cycles for the platform methods (instructions) | Fee params | Scheduled for Phase 2 benchmarking; initial cycle fees configured per `01 §4`. Retune in T7.3. |
| SP-6 | ckETH helper contract: subaccount deposits and minimum deposit | T5.10 | **Resolved.** Subaccounts supported via helper contract. Pack price floats to minter minimum (~0.002 ETH / ~$6). ckBTC requires 12 confirmations. |
| SP-7 | Galaxy Zoo JWST (CEERS) classifications: public release, licence, and question mapping to protocol v1 | T1.5 gold | **Resolved.** Smethurst et al. (2025) released ~7,000 classifications under CC BY 4.0 mapping 1:1. Filter ~2,000 gold subjects (votes ≥ 20, agreement ≥ 0.8). |

## Accepted risks
- Blindness is best-effort (R-11).
- Reputation is gameable by a very patient adversary who answers gold correctly and lies elsewhere. Consensus scoring and honeypots reduce this. Revisit after beta data.
- Data availability: the dossiers live in our own R2 bucket, so upstream services (DJA, MAST) being down doesn't stop work; only a re-curation would.
