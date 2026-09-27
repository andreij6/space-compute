# Learnings from proof-of-burn (Caldera / Cycles of Influence)

Source: `../proof-of-burn`, read 2026-09-27. It is a live ICP app (Rust backend, React, NNS neurons, `icp` CLI, Firebase) and our closest precedent. Each lesson names where it is now enforced in this plan.

## 1. Engineering
| Lesson from proof-of-burn | Where we enforce it |
|---|---|
| One 27k-line `lib.rs` needed a hand-maintained "section map" skill just to navigate | 01 §6: files of at most ~800 lines, one module per concern |
| Hand-kept `.did` drifted, and bindings broke silently | 01 §6: `export_candid!` + CI diff |
| PocketIC tests **skip** (and stay green) when the wasm or the binary is missing | 11 §2 gate 4: a skip is a failure |
| New fields on stored structs need `serde(default)`; never renumber a `MemoryId` | 01 §6 (we use Candid + `Option` fields + `v: u8`) |
| Canister IDs permuted after a local wipe; baked-in ledger IDs broke every call | 01 §6: IDs come from the environment, never from code |
| A feature flag sat silently OFF on prod and stopped the neuron harvest | 01 §6: flags never gate money timers; post-deploy smoke asserts expected flag values (T8.1) |
| Local ledger bug PB-148: CMC `notify_top_up` fails locally | Money flows are tested in PocketIC with real NNS wasms, not the local replica (11) |
| An ICPSwap call blew the 5B instruction limit and retried every sweep | Paginate every external call and back off on failure; the load test (T7.2) measures instructions |
| Snapshot creation failed on memory-grow cycles; keep at most 2 snapshots | T8.1 runbook: rotate to 2 and top up before taking a snapshot |
| An idempotent one-shot `deploy-local.sh` saved hours | T3.8 `tools/seed-local` is one idempotent command |
| "Always pass `--identity`"; "never touch mainnet unless asked" | Harness guard hook (`.claude/harness/hooks/guard.sh`) |

## 2. Product & ops
| Lesson | Where we enforce it |
|---|---|
| Feature sprawl: 74 endpoints and 8 feature areas were later deleted | MVP scope list (OVERVIEW §10); new ideas go on the roadmap, not into the build |
| Copy about money went stale in many places (UI, llms files, docs) | Render numbers from config queries; the agent skill reads `get_api_doc`; there is one copy sweep per money change (harness checklist) |
| Money modals that "froze" confused users; plain-English errors fixed support load | 05: staged transaction modal; `ApiError` → plain-English map (vitest) |
| GA4 lumped every view under one title; admin traffic polluted the stats | 05 §4b: per-route titles, `is_admin` user property, hashed user id |
| A per-deploy log (commit, snapshot id, flags, smoke results) made incidents quick to trace | T8.1: `docs/ops/deploy-log.md`, one entry per deploy |
| Two controllers minimum on value-holding canisters; a 90-day freezing threshold | 01 §5b (we use 180 days on treasury) |
| Canary with tiny amounts on mainnet, because PocketIC NNS mocks ≠ mainnet | T8.5/T8.6 checklist: a 0.1 ICP canary harvest and top-up |

## 3. Using proof-of-burn's neuron to power Space Compute (superseded 2026-09-27: the owner removed the neuron; the treasury is owner-funded, spec 12. Kept for reference.)
- Proof-of-burn's backend already harvests maturity (`DisburseMaturity`, hourly check, ≥ 1.05 ICP threshold, about 7 days to mint). That flow is proven in production.
- **Neuron controllers can't be changed** (hotkeys only), so we redirect the **yield**, not the neuron. Design: spec 12 §6.
- **Only app-owned stake is in scope.** The pooled term neurons and the Perm/Early-Adopter neuron hold users' ICP. They are wound down under proof-of-burn's own terms, never repurposed.
- Owner action before T8.5: confirm which neuron proof-of-burn fully owns, and who controls it. The mainnet leader neuron `17802688826615984104` is pinned in its code; if a wallet rather than the canister controls it, the harvest redirect has to be done from that wallet.
- The sunset sequence is its own project (later): freeze new intake on proof-of-burn → settle user positions → strip the backend to a neuron vault → add `treasury` as a co-controller.
