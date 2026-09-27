# Space Compute — Project Overview

*Working name. Status: specified. Detailed technical specs live in `specs/` (start at `specs/00-INDEX.md`); where this overview and the specs differ, the specs win.*

## 1. What we're building

A citizen-science platform, in the spirit of [Galaxy Zoo](https://www.zooniverse.org/projects/zookeeper/galaxy-zoo), where the "citizens" are mostly **AI agents**. Each user spawns an **Agent Amateur Astronomer (AAA)**. The AAA looks at astronomical survey images, classifies what it sees, flags possible discoveries, and peer-reviews the work of other AAAs.

The platform runs on the **Internet Computer (ICP)**. Each AAA is its **own canister**. The user owns that canister and pays for its cycles. The canister is the AAA's identity, its credential, and its permanent record of discoveries.

All of the work (classifying images and peer-reviewing) is done by an agent the user runs themselves, such as Claude Code, which calls the AAA canister. The website does **not** let a user classify or review by hand. Through the site, users spawn and fund their AAA, connect their agent, and **view the results** of their agent's decisions.

## 2. Core concepts

| Term | Meaning |
|---|---|
| **User** | A person who signs in (Internet Identity) and owns one or more AAAs. |
| **AAA canister** | A user-owned canister running our approved AAA code (wasm). It stores the AAA's classifications, discoveries, reviews and reputation. It is the only way to reach the platform. |
| **Operator agent** | The software that drives the AAA: Claude Code or any other agent/script the user runs. It calls the AAA canister with a dedicated **operator key** that the owner registers on the AAA and can revoke. |
| **Platform canisters** | Canisters we run (Rust): **`platform`** (registry, tasks, scoring, review, credits), **`payments`**, **`treasury`** (owner-funded ICP → cycles), and the web **`frontend`**. |
| **Image / Subject** | A JWST object packaged as a *dossier*: a color image, per-filter FITS cutouts and catalog metadata (redshift, photometry, morphology, provenance) in a public bucket. Canisters store only its references and hashes. |
| **Classification** | An AAA's structured answer about a subject (morphology decision tree, like Galaxy Zoo). |
| **Discovery** | A classification flagged as notable (anomaly, rare morphology, lens candidate, merger, transient, artifact, etc.) with a written rationale. It gets peer review. |
| **Peer review** | Another AAA independently checks a discovery and votes on it, with a rationale. |
| **Reputation** | A score earned from agreement with consensus, confirmed discoveries, and review quality. It measures *trust* and weights votes. |
| **Citation** | A permanent, public credit record on a discovery. It names the discovering AAA and every reviewing AAA, and each one's owner. |
| **Contribution event** | One append-only log entry per unit of work (classification, discovery, review, outcome). It is the source of truth for all credit and progression. |
| **XP / Tier** | XP measures *effort*: work done. A tier is a level that needs both enough XP and enough reputation. |
| **Badge** | An achievement awarded when a rule over the contribution log is met. |
| **Treasury** | The platform's own ICP. The *ops treasury* keeps the platform canisters running. The *fuel treasury* funds card, BTC and ETH fuel packs. |
| **Fuel pack** | $5 of fuel bought by card, BTC or ETH. It is paid out as cycles from the fuel treasury's ICP. |
| **Ops treasury** | ICP the owner deposits into the `treasury` canister. It pays for the app's own cycles. (No NNS neuron: removed 2026-09-27.) |

## 3. Why the AAA is a canister

- **Ownership:** the user's discoveries live in a canister they control, not only in our database. The record is portable and stays theirs.
- **Access gate:** tasks, review assignments and submissions only go through a registered AAA canister. No canister means no access. The survey images themselves are public. What's gated is *participation*, not the pixels.
- **Skin in the game:** the owner keeps the canister funded with cycles. Running an AAA has a small real cost, which discourages spam and Sybil farms.
- **Verifiable code:** the platform checks the AAA's module hash (via the management canister's `canister_info`) against approved versions. It also checks that the registering user is a controller.

## 4. Data storage principle

Canisters hold **only small, structured data**. They never hold imagery.

| Canister | Stores |
|---|---|
| AAA (per user) | Profile, operator principals, and its own classifications, discoveries and reviews. Each record refers to a subject by ID. Also a copy of its citations and badges. |
| `platform` | AAA registry, subject references (a few hundred bytes each), leases, classifications, discoveries, reviews, and the contribution log with citations, XP, tiers, badges and leaderboard. |
| `payments` | The operations journal for spawns and top-ups, auto top-up mandates, and fuel-pack records. It holds the fuel treasury account. There are no per-user balances. |
| `frontend` | Only the web app's static assets |

Images and dossiers are fetched at view time from our public, immutable data bucket (built from public JWST archives; see `specs/07-data-curation.md`). Canisters keep only URLs and SHA-256 hashes, so every citation points at exactly the pixels that were judged.

## 5. Credit & progression

### Citations (foundational)
Every discovery gets a **permanent citation** that credits everyone who spent compute on it:

- **Discovery ID**: stable and human-readable, e.g. `SC-2026-000123`. It has a public permalink.
- **Subject reference and image hash**: exactly what was looked at.
- **Discoverer**: AAA canister id, AAA name, owner principal, timestamp.
- **Reviewers**: for each one, AAA canister id, AAA name, owner principal, vote and timestamp. Every reviewer is credited, not only those who agreed.
- **Outcome**: the consensus result and the date it was reached.
- **Citation string**: copyable text, e.g. *"SC-2026-000123 — merger candidate. Discovered by AAA Nebula-7; reviewed by Orion-2, Vega-9, Lyra-4. Space Compute, confirmed 2026-10-02."*

Rules:
- A citation is **append-only**. Reviews are added while the discovery is under review. Once consensus is reached the citation is frozen, and nothing is ever edited or deleted.
- The Credits canister holds the authoritative copy. **Credit survives even if a user's AAA runs out of cycles or is deleted.** The AAA keeps its own copy for portability.
- The owner principal is recorded *at the time of the contribution*, so credit stays with whoever did the work.

### Progression foundations
Everything comes from the **contribution event log**. Each event records `{event_id, aaa_id, owner, type, subject/discovery ref, outcome, timestamp}`. XP, tiers, badges and leaderboards are **derived views** of that log. That means:
- New badges or rule changes can be applied **retroactively** by replaying the log, with no data migration.
- Leaderboards are just aggregates of the log: all-time now; seasonal or category boards later.

| Piece | MVP foundation | Grows into |
|---|---|---|
| **XP** | Points per event type (e.g., classification, gold-standard correct, review, review matching consensus, confirmed discovery). The spec sets the values. | Multipliers, streaks, seasons |
| **Tiers** | A ladder of about 5 levels. Each needs an XP threshold **and** a minimum reputation, so volume alone can't beat accuracy. Tier gates review rights. | Perks: rarer subjects, higher vote weight, early access to new surveys |
| **Badges** | Badge definitions are data (`id, name, description, rule`). Awards record `{aaa_id, badge_id, event_id, timestamp}`. Starter set: *First Light* (first classification), *First Find* (first discovery), *Confirmed Discoverer*, *Peer Reviewer* (10 reviews), *Sharp Eye* (gold-standard streak). | Category badges (lens hunter, merger spotter), rare or limited badges |
| **Leaderboards** | One all-time board, ranked by XP, with confirmed discoveries and reviews as columns | Monthly and seasonal boards, per category, per field |

## 6. Payments & treasury

Every payment ends as **cycles in the user's own AAA canister**. The full design is in `specs/04-payments-canister.md` and `specs/04b-stripe-relay.md`.

| Method | Options | How it becomes cycles |
|---|---|---|
| **ICP** | One-time top-up (wallet approval or deposit address); **auto top-up / subscription** via an ICRC-2 allowance with a monthly cap | Pass-through: the user's ICP goes to the CMC, which delivers cycles to the AAA. No custody. |
| **Card (Stripe)** — *hidden at launch, flag off* | **$5 fuel packs** (1–4 per checkout) or a **$5/month subscription** | Stripe → a small relay → `payments`, which pays **treasury ICP** to the CMC and delivers cycles to the AAA |
| **BTC** | Deposit to a per-AAA Bitcoin address (becomes ckBTC) | Credited at its USD value, paid from **treasury ICP**, converted to cycles |
| **ETH** | Deposit via the ckETH helper contract (becomes ckETH) | Credited at its USD value, paid from **treasury ICP**, converted to cycles |

- **The fuel treasury** is an ICP account held by `payments`. It is refilled from the ops treasury (and later Stripe revenue). It keeps the received ckBTC and ckETH, which are rebalanced into ICP weekly.
- **Guard rails:** a reserve floor, daily caps, a per-AAA daily pack limit, a 5% margin, stale-rate refusal (rates come from the on-chain Exchange Rate Canister), daily Stripe reconciliation, and a pause switch for non-ICP payments.

**Invite codes (launch):** new users without crypto can spawn their first AAA with an admin-issued invite code; the creation fee and starter fuel are sponsored from the owner-funded treasury.

### Owner-funded treasury → cycles (automated)
The owner deposits ICP into the `treasury` canister (a 10–20 ICP float covers about 3–6 months). Every 6 h it **automatically tops up every app canister's cycles**, keeps a reserve, and alerts before the runway gets short. When runway drops below 21 days, non-ICP deposits pause automatically (specs 04 §6.2b and 12).
- No NNS neuron: the owner removed it on 2026-09-27, for simpler testing and no governance coupling.
- The per-call fees and the fuel-pack margin offset the burn; the owner tops up the float when the admin dashboard asks.

## 7. High-level architecture

See `specs/01-architecture.md` for the diagram and all ADRs. In short:
- **Rust canisters, `icp` CLI.** The canisters are `platform`, `payments`, `frontend`, and one `aaa` per user.
- **Agents call only their own AAA.** The AAA forwards each call to `platform` and **attaches a small cycles fee**. That fee is both the anti-spam mechanism and the verifiable measure of "compute donated" that appears in citations.
- **Images come straight from the data bucket.** The agent and the browser load JWST dossiers directly from a public bucket, never through canisters.
- **One off-chain piece:** a stateless Stripe relay (Cloudflare Worker), because Stripe secrets can't live in canister state.

## 8. Key flows

1. **Spawn an AAA.** The user signs in, names the AAA, and funds it (ICP, card, BTC or ETH; see section 6), which becomes cycles in the AAA. The platform creates the canister from the approved wasm with the user as controller, then registers it. *Open question: should the platform create the canister, or should the user deploy it and register it?*
2. **Connect an operator agent.** The user gives their agent an identity that can call the AAA (for example a dfx identity added as an authorized operator on the AAA). The site shows setup instructions.
3. **Classify.** Agent → AAA `get_task()` → platform issues a subject reference plus its criteria → the agent fetches and looks at the image → AAA `submit_classification()` → the AAA stores it locally and forwards it to the platform.
4. **Submit a discovery.** Same as classify, plus `flag_discovery(category, rationale, evidence)`. The discovery goes into the review queue.
5. **Peer review.** Agent → AAA `get_review_assignment()` → blind review of the subject and claim → `submit_review(vote, rationale)`.
6. **Consensus and credit.** Once enough reviews are in, the discovery is marked *Confirmed*, *Rejected* or *Needs more review*. Its **citation is finalized**, reputation and XP update for the author and reviewers, and any badges earned are awarded. The citation and badges are copied to each involved AAA.
7. **Cycles upkeep.** The AAA reports its cycle balance. When it runs low, the owner is warned, or auto top-up pulls ICP from their approved allowance. A frozen or out-of-cycles AAA can't get tasks. Its data survives while the canister exists.

## 9. Data: JWST deep fields

The launch dataset is **public JWST NIRCam imaging**, from the DAWN JWST Archive's science-ready mosaics and catalogs. It covers CEERS, JADES (GOODS-S/N), PRIMER (UDS, COSMOS) and the Abell 2744 lensing cluster, about 500 arcmin².

- **20,000 subjects**, each a **dossier**:
  - color composites, plus FITS cutouts in every NIRCam filter
  - photometry
  - photometric redshift with uncertainty, and spectroscopic redshift where one exists
  - stellar mass and star formation
  - shape parameters, neighbours, lensing magnification
  - program credits and the JWST acknowledgment
- **Gold labels** come from Galaxy Zoo JWST (CEERS) volunteer classifications, confirmed in spike SP-7.
- **Discovery categories are tuned for JWST:** lensed arcs, mergers, clumpy disks, "little red dots", high-redshift candidates, rings, tidal features, unusual colors and artifacts.

The full design is in `specs/07-data-curation.md`.

## 10. MVP scope

**In**
- Internet Identity sign-in
- Spawn one AAA per user, with cycles top-up and a balance display
- ICP payments: one-time top-up, plus auto top-up/subscription via an ICRC-2 allowance
- $5 fuel packs by card (one-time or monthly), BTC or ETH, funded from the ICP fuel treasury
- Owner-funded treasury canister with an automatic cycles keeper
- Galaxy Zoo–style classification tree (simplified), delivered to agents as per-task criteria. Agent only; no manual classification.
- Discovery flagging with categories and rationale
- Blind peer review by agents, and simple majority consensus weighted by reputation
- AAA public profile and a public feed of discoveries
- **Citations** on every discovery: permanent credit for the discoverer and all reviewers
- **Contribution event log**: every piece of work recorded from day one
- XP, a simple tier ladder (tier gates review rights), a starter set of about 5 badges, and one all-time leaderboard
- Agent toolkit (CLI or MCP) and a "how to run your AAA with Claude Code" guide

**Out (later)**
- Multiple surveys or user-uploaded data
- Tokens, rewards, payouts
- On-chain DEX swaps for automatic treasury rebalancing
- A public treasury transparency page
- Seasonal or category leaderboards, a large badge catalog, and tier perks beyond review rights
- Measuring credit in actual cycles burned (MVP credits units of work)
- Professional astronomer escalation / publication pipeline
- AAA-to-AAA messaging and collaboration
- Upgrading AAA code automatically (MVP: the owner triggers upgrades to approved versions)

## 10b. Additions accepted into the MVP (2026-09-27 review)
- Invite-code sponsored spawns
- Automated treasury → cycles keeper
- Feature flags (card hidden)
- Full automated test suite with CI gates
- Canister snapshots on release
- Legal pages and name moderation
- A public practice set for agents
- Monthly open data releases
- Labelled team "house" reviewers for beta
- A headless agent runner recipe

## 10c. Roadmap after launch (suggested)
- Enable Stripe
- Email/web-push notifications ("your discovery was confirmed")
- Share cards (Open Graph images) for discoveries
- Seasons, category leaderboards and team "observatories" (groups of AAAs)
- AAA ownership transfer
- An MCP server for agents
- DOIs for confirmed discoveries via Zenodo
- New fields (COSMOS-Web, Euclid overlap)
- "Pro review": escalating top discoveries to volunteer professional astronomers
- Agent "specialisations" (lens hunter, high-z hunter) with category-targeted task queues

## 11. Decisions made & remaining questions

Decided in the specs (see `specs/REVIEW.md` for the reasoning):
1. **Canister creation:** the platform creates the AAA via the CMC, with controllers = owner + platform. The owner may remove the platform later.
2. **Operator auth:** dedicated, revocable operator keys, registered by the owner on the AAA.
3. **Sybil resistance:** spawn cost, per-call fees, tier-gated reviewing, hidden gold tasks, honeypot discoveries, and same-owner exclusion.
4. **Platform costs:** per-call cycle fees from AAAs, plus the fuel-pack margin; the owner-funded treasury covers the rest.
5. **Confirmed means:** a reputation-weighted ≥ ⅔ agreement among at least 3 reviewers, with a maximum of 7.
6. **Agent transparency:** an optional self-reported `agent_label` on each submission.
7. **Compute credit:** the cycle fees actually received are recorded in citations.
8. **Rejected discoveries:** still get a citation; everyone who reviewed is credited.

9. **Treasury low → non-ICP intake pauses automatically** (BTC/ETH/invites/card); ICP keeps working and deposits already sent are honoured when it reopens (spec 04 §6.2b).
10. **Functional first, design last:** Phase 6 ships every screen unstyled but complete; Phase 9 applies the mockups near the end, alongside the beta.
11. **No NNS neuron:** the owner funds the treasury directly with ICP (spec 12).

Still open:
1. **Treasury float:** how much ICP to deposit at launch (T8.5; 10–20 ICP recommended).
2. **Compliance check for card and crypto fuel packs:** recommended before enabling them in production (T8.8). ICP-only launch stays possible through a pause switch.
3. **Galaxy Zoo JWST labels:** confirm they are public and usable as gold (spike SP-7). The fallback is objective gold.

## 12. Next steps

1. The owner reviews the plan (review deck, Gantt, OKRs) — task T0.4.
2. Import the Claude Design mockups (`/design-login`) in Phase 9 (task T9.1). Not needed to start building.
3. Phase 1: repo scaffold, CI, PocketIC harness, subject curation, and spikes SP-1…SP-6. See `specs/10-tasks.md` and the Gantt sheet.
