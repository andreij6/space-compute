# 01 — Architecture & Decisions

Status: **Approved for MVP** (post adversarial review, see `REVIEW.md`).
Audience: implementing agents. Product context: `../OVERVIEW.md`. Screens: `../DESIGN_BRIEF.md`.

## 1. Architecture Decision Records (ADRs)

| # | Decision | Why | Rejected alternative |
|---|---|---|---|
| ADR-01 | **Rust** for all canisters (`ic-cdk 0.20`, `ic-stable-structures 0.7`, `candid 0.10`; bumped from 0.19 at T1.1). No Motoko. | Owner requirement; mature stable-structures; PocketIC testing. | Motoko |
| ADR-02 | **`icp` CLI** (`icp.yaml`, recipes `@dfinity/rust@v3.4.0` (icp-cli ≥ 1.6), `@dfinity/static-site@v0.4.0`). Never `dfx`. | `dfx` is legacy (see `.claude/skills/icp-cli`). | dfx |
| ADR-03 | **Three platform canisters**: `platform`, `payments`, `frontend`. Plus one `aaa` canister per user. | The science flow needs registry, tasks, review, consensus and credits to change *atomically*: a consensus result, a frozen citation, XP, badges and a leaderboard update all land in one message. MVP data is far below 1 GB. `payments` is kept separate because it moves funds: it has a smaller attack surface, its own upgrade cadence, and a stricter review bar. | Five platform canisters (the earlier overview); it would add cross-canister sagas for no MVP benefit. The split path is kept via module boundaries (§4). |
| ADR-04 | The **AAA is an authenticated relay + personal repository**. Agents call their AAA; the AAA calls `platform` with **attached cycles** (a per-call fee). | Satisfies "canister required for access". Makes the user's compute donation *real and measurable* (fees received are recorded in citations). Protects `platform` from cycle drain. | Agents call `platform` directly: it breaks the access model, and the platform would pay for all execution. |
| ADR-05 | **ICP payments are pass-through** (wallet ICRC-2 or deposit subaccount → CMC → the AAA's cycles). **Card, BTC and ETH buy $5 fuel packs funded from a central ICP treasury** (owner decision, 2026-09-26): the payment is verified, the treasury's ICP goes to the CMC, and cycles go to the AAA. There are no per-user balances in either case. | One conversion path (treasury ICP → cycles) for every non-ICP rail; no DEX dependency in the request path. | Prepaid fuel-credit balances; on-chain DEX swap per payment |
| ADR-06 | AAA controllers = **[owner, platform]**. The owner may remove `platform`, and the AAA then becomes "self-managed" (manual upgrades). | The owner is sovereign; the platform can offer one-click upgrades of approved wasm. | Platform-only controller; owner-deployed only |
| ADR-07 | **Operator auth = dedicated operator principals** registered on the AAA by the owner. The alternative flow, `icp identity link web --app <domain>`, is supported but not the default. | Long-running agents need non-expiring, revocable, least-privilege keys. II delegations expire and carry full owner power. | Owner principal for agents |
| ADR-08 | **Images are never stored in canisters.** A subject is a reference (field, RA/Dec, `rgb.png` URL + SHA-256, `dossier.json` URL + SHA-256). | Storage and cost. | Asset-canister mirror |
| ADR-18 | **Launch data = public JWST NIRCam deep fields via the DAWN JWST Archive (DJA v7)**. The data is pre-rendered once into immutable, content-addressed **subject dossiers** (color images, per-filter FITS cutouts, photometry, photo-z/spec-z, morphology, lensing, provenance) in a public R2 bucket (07). | Agents need science-grade data plus context to make real discoveries; hashes must be stable for citations; we don't overload volunteer cutout services. | Hot-linking live cutout services; MAST pipeline reprocessing |
| ADR-09 | **Gold standard = Galaxy Zoo JWST (CEERS) volunteer consensus** (high-confidence answers only; spike SP-7). Fallback: objective gold (stars/artifacts, spectroscopic high-z). | Credible ground truth on the same instrument and fields. | Hand-labelled set |
| ADR-10 | **Append-only event log** (`ic_stable_structures::Log`) is the source of truth; XP, tiers, badges, leaderboard and credits are derived views that can be replayed. | Retroactive badge and rule changes without migrations. | Mutable counters only |
| ADR-11 | **Agent toolkit = a Claude Code skill + the `icp` CLI.** No custom MCP server in the MVP. The AAA also exposes `get_api_doc()`. | Least code. Claude Code can already run `icp canister call`, `curl`, and view images. | Custom MCP server (post-MVP) |
| ADR-12 | Frontend: **Vite + React 19 + TypeScript**, TanStack Query, React Router; `@icp-sdk/core@^6`, `@icp-sdk/auth@^10`, `@icp-sdk/signer@^6` (OISY), `@icp-sdk/canisters@^4`, `@icp-sdk/bindgen`. | Current IC SDK line; mainstream stack. | Svelte, Next.js (no SSR on IC) |
| ADR-13 | Integration tests use **PocketIC** (`pocket-ic` crate) with real ICP ledger + CMC wasm. | Deterministic multi-canister tests, including payments. | Local network only |
| ADR-14 | **Reviewing is gated by tier (≥ 2).** Honeypot discoveries measure reviewer quality. | Resists sybils and rubber-stamping. | Open review |
| ADR-16 | **Stripe via a stateless Cloudflare Worker relay** (`04b`). It is the only off-chain component, because Stripe secrets can't live in canister state. | Canister memory is readable by node providers. | HTTPS outcalls holding a Stripe key in the canister |
| ADR-17 | **ckBTC / ckETH minters** receive BTC and ETH. Rates come from the **Exchange Rate Canister**. | Native chain-key, no bridges; a decentralized oracle. | Third-party processors |
| ADR-19 | **A fourth platform canister, `treasury`** (12), holds an owner-funded ICP reserve, monitors cycle burn across all app canisters, and automatically tops them up via CMC `notify_top_up`. | Isolates the operational reserve with zero user-facing attack surface; automates cycle replenishment. | Manual runbook; `payments` controlling the reserve |
| ADR-20 | **Feature flags** in `payments` config (`card`, `btc`, `eth`, `sponsored_spawn`), exposed by the `get_features()` query. The frontend hides disabled methods, and the canister rejects them with `FeatureDisabled`. **Launch: `card = false`** — Stripe is hidden and no Stripe account is needed. | Ship without Stripe; enable later without a redeploy. | Deleting the card code |
| ADR-21 | **Tests are part of every task** (11): unit, property, fuzz, PocketIC (with the NNS subnet), security regression, Playwright E2E, deterministic bot agent; coverage and traceability gates in CI. | "Fully tested with automated tests" (owner). | Manual QA |
| ADR-15 | Citations are **frozen at consensus** and served with **certified query responses** (`ic-certified-map`). | Permanent, verifiable credit. | Plain queries |

## 2. System diagram

```
Owner (browser, Internet Identity) ─────────────┐
   │ read-only science views, spawn, fuel, ops  │ ICRC-2 approve (OISY via @icp-sdk/signer)
   ▼                                            ▼
┌──────────┐   queries/updates    ┌─────────────────────────┐      ┌─────────────┐
│ frontend │ ───────────────────▶ │ platform (Rust)         │◀────▶│ payments    │
│ (static- │                      │  registry+factory       │      │ (Rust)      │
│  site)   │ ───────────────────▶ │  catalog/tasks          │      │ spawn, top- │
└──────────┘                      │  scoring                │      │ ups, auto   │
                                  │  review+consensus       │      │ top-up      │
Operator agent (Claude Code)      │  credits/progression    │      └──────┬──────┘
   │ icp canister call            └───────────▲─────────────┘             │ transfer_from /
   │ (operator principal)                     │ bounded-wait calls         │ notify_*
   ▼                                          │ + attached cycles fee      ▼
┌──────────────────────┐ ─────────────────────┘                  ┌───────────────┐
│ aaa (Rust, per user) │ ◀── cycles (CMC notify_top_up) ──────────│ ICP ledger,   │
│ owner+operators      │                                          │ CMC (NNS)     │
│ local records        │ ── request_auto_topup ─────▶ payments    └───────────────┘
└──────────────────────┘
Images + dossiers: agent/browser fetch directly from the public data bucket (never via canisters)
```

## 3. Repository layout (monorepo)

```
space-compute/
  icp.yaml                     # canisters: platform, payments, frontend (aaa built, not deployed by env)
  Cargo.toml                   # workspace
  rust-toolchain.toml          # pin stable toolchain + wasm32-unknown-unknown
  crates/
    sc-types/                  # shared Candid types (ONE source of truth, used by all canisters + tests)
    platform/                  # canister; modules: registry, catalog, scoring, review, credits, admin
    payments/                  # canister
    aaa/                       # canister (wasm uploaded into platform for the factory)
    integration-tests/         # PocketIC tests
  frontend/                    # Vite app; bindings generated from crates/*/*.did
  relay/                       # Cloudflare Worker: Stripe checkout + webhook -> payments.stripe_credit (04b)
  agent-kit/
    skills/space-compute-astronomer/SKILL.md   # the operator skill users install
  tools/curation/              # offline JWST dossier builder (Python: astropy, astroquery)
  docs/                        # overview, brief, specs, OKR
  .claude/skills/              # dev skills (ICP + Rust)
```

- Each canister commits its `.did` (generated by `candid-extractor`, checked by `just verify` for drift).
- `.icp/data/` is committed; `.icp/cache/` is gitignored.

## 4. Module boundaries inside `platform`

Modules talk only through Rust function APIs in `platform/src/<module>/mod.rs`. Each module owns its own `MemoryId` ranges. This keeps a later split into separate canisters mechanical.

| Module | Owns | Memory IDs |
|---|---|---|
| `config`/`admin` | params, admins, wasm store | 0–4 |
| `registry` | AAA records, owner index, verification | 5–9 |
| `catalog` | subjects, gold, leases, seen-set, protocol versions | 10–19 |
| `scoring` | classifications, subject tallies, reputation inputs | 20–29 |
| `review` | discoveries, assignments, reviews, honeypots | 30–39 |
| `credits` | event log, citations, progression, leaderboard, credit index, certification tree | 40–59 |

## 4b. Releases
Before every upgrade: `icp canister snapshot create` (canister snapshots) for each canister being upgraded, then upgrade, then smoke-test. Roll back with `snapshot load` if the smoke test fails. Snapshots are kept for the last 3 releases.

## 5. Environments

| Env | Network | Purpose | Controllers |
|---|---|---|---|
| `local` | managed local network (`icp network start -d`) | dev, e2e | dev identity |
| `staging` | `ic` | beta, separate canister IDs, test ICP amounts | team + backup |
| `production` | `ic` | launch | team hardware-wallet principal + backup principal |

The II derivation origin is pinned to the production domain, so principals stay stable if the frontend URL changes.

## 5b. Canister settings (all environments)

These must be set on every deployed canister (`icp canister settings update`), not left at their defaults:

| Canister | Freezing threshold | Controllers | Other |
|---|---|---|---|
| `platform` | 90 days (`7776000`) | team + backup principal + `treasury` | `wasm_memory_limit` 3 GiB |
| `payments` | 90 days | team + backup principal + `treasury` | — |
| `frontend` | 30 days | team + backup principal + `treasury` | — |
| `treasury` | 180 days | team hardware wallet + backup principal | smallest API; no user traffic |
| `aaa` (set at spawn by payments/CMC) | 60 days | owner + platform | — |

A missing backup controller or a low threshold is a launch blocker: when a canister's cycles reach zero, its code and data are uninstalled (see `.claude/skills/cycles-management`).

## 6. Cross-cutting conventions (all Rust canisters)

- State lives in `ic-stable-structures` (`StableBTreeMap`, `StableCell`, `StableLog`) behind one `MemoryManager`. **No heap state that must survive upgrades.** No `pre_upgrade` serialization. Every canister defines `#[init]` and `#[post_upgrade]`.
- Values are encoded with Candid, `Bound::Unbounded`, and a schema version field `v: u8` on every stored record (additive changes only; new fields are `Option`).
- Every update method: reject the anonymous principal, run an explicit role check (never rely on `inspect_message`, which is a cycle-saving pre-filter only), and bind `msg_caller()` before any `.await`.
- All inter-canister calls are **bounded-wait** (`Call::bounded_wait`), have idempotency keys, and handle `SYS_UNKNOWN` by querying the outcome (see each spec).
- Per-key `CallerGuard` on any method that awaits and mutates, so concurrent calls can't interleave (reentrancy).
- Errors are a shared `ApiError` variant (`sc-types`), never traps for expected failures.
- Input limits are enforced at the boundary (see `08-security.md` §3).
- Time is `ic_cdk::api::time()` (ns). IDs are monotonic `u64`, except for public discovery IDs (`SC-YYYY-NNNNNN`).
- **Small modules, not one big file.** No source file over ~800 lines. Proof-of-burn's 27k-line `lib.rs` needed a hand-kept section map before an agent could navigate it.
- **Candid is generated, not hand-kept:** `ic_cdk::export_candid!()`, and `just verify` diffs the output against the committed `.did`.
- **Never reuse or renumber a `MemoryId`**, and never bake canister IDs into code or init args. Resolve IDs from the environment (`icp.yaml` / `ic_env`). In proof-of-burn, IDs permuted after a local wipe and every call broke.
- Feature flags **never gate money timers** (harvest, top-ups, sweeps). Proof-of-burn once shipped with a flag silently off on prod, which stopped its harvest.
- Randomness: `raw_rand` seeds a `ChaCha20Rng` stored in heap (reseeded hourly by timer and after upgrade). It is used only for task selection, gold/honeypot injection, and reviewer assignment. It is never security-critical.

## 7. Key parameters (tunable via `platform.set_params`, admin)

| Param | MVP default | Notes |
|---|---|---|
| `fee_get_task` | 50_000_000 cycles | attached by AAA |
| `fee_submit_classification` | 200_000_000 | recorded as compute contributed |
| `fee_get_review` | 50_000_000 | |
| `fee_submit_review` | 200_000_000 | |
| `retire_after_k` | 5 classifications/subject | gold never retires |
| `gold_rate_bp` | 1000 (10%) | share of tasks that are gold |
| `calibration_tasks` / `calibration_gold_rate_bp` | 50 / 4000 | a new AAA's first 50 tasks are 40% gold, so tier 2 is reachable in about 50 tasks (R-08) |
| `review_starvation_days` | 7 | resolve with ≥3 reviews by weighted majority if no eligible reviewer appears (R-10) |
| `heartbeat_min_interval_secs` | 3600 | fee-less calls are rate-limited (R-17) |
| `honeypot_rate_bp` | 1000 (10%) | share of review assignments |
| `lease_task_secs` | 1800 | |
| `lease_review_secs` | 86400 | |
| `max_open_leases_per_aaa` | 3 | |
| `max_tasks_per_aaa_per_hour` | 120 | |
| `reviews_min` / `reviews_max` | 3 / 7 | |
| `max_flag_rate_bp` | 1000 | ≤10% of an AAA's last 100 classifications may be flagged |
| `aaa_initial_cycles` | 1_000_000_000_000 | on top of creation fee |
| `data_refresh_interval_days` | 15 | curation job cadence (07 §5c), off-chain scheduler |
| `auto_topup_min_interval_secs` | 21600 | |
| `fuel_pack_usd_cents` / `margin_bp` | 500 / 500 | $5 pack, 5% margin (payments) |
| `treasury_reserve_floor_e8s` | 50 ICP | fuel packs refused below it |
| `treasury_daily_cap_e8s` / `stripe_daily_usd_cap` | 100 ICP / $1,000 | limit on the damage from a compromised relay |
| `per_aaa_daily_packs` | 4 | |
| `rate_max_age_secs` | 7200 | stale XRC rates → refuse packs |
| `claim_cell_arcsec` / `claim_reopen_days` | 1.5 / 30 | first-claim rule (02 §6.4) |

The fees above are estimates. Task T7.3 measures real costs and retunes them.
