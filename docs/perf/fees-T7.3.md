# T7.3 — Cycle cost measurement & fee retune

Raw data: `docs/perf/load-test-T7.3.json`. Harness: `crates/integration-tests/tests/t7_2_load.rs`: `t7_3_pricing_probe`, `t7_3_load_app_40x70`, `t7_3_load_multiplier_10x20` (`#[ignore]`, run with `just load-test`). All runs used a clean worktree at `37fe2de` so other lanes' uncommitted edits were not included. Several other lanes' `pocket-ic` servers were running at the same time. That affects wall time but not cycles, which are deterministic: two 40×70 runs gave identical numbers.

## 1. Review of T7.2 (b9378ff): verdict is "directionally right, not usable for pricing"

| # | Defect | Effect | Fix |
|---|---|---|---|
| 1 | Instructions were estimated at 0.4 cycles/instruction. | The measured price on PocketIC's app subnet is **1.0 cycle/instruction**, so the T7.2 instruction counts are about 2.5× too high. They also counted fixed per-message fees (about 6.5M per ingress update, plus the xnet call fee) as instructions. | `t7_3_pricing_probe` calibrates the price against `performance_counter(0)`. |
| 2 | The answer chain always picks the first option, so every gold answer is wrong and reputation stays below 60%. | No AAA can ever reach tier 2, at *any* run size. `submit_review` was never measured, and `get_review_assignment` was measured only on its cheapest path (`NotEligible`). | Agents answer gold subjects correctly, and review calls are made every round after calibration: 108 real reviews, 396 empty assignments and 496 `NotEligible` calls. |
| 3 | The loop was agent-major: each AAA ran all of its tasks before the next AAA started. | This is unrepresentative for the review queue, eligibility and the pool cursor. | Round-robin loop. |
| 4 | Only averages and min/max of raw balance deltas were reported. | This hid the `get_task` tail: p50 is 9.0M and p99 is 57.0M. | The harness records per-call platform cost (fee accepted minus the platform's gain) and AAA relay overhead (AAA drop minus fee), with p50/p95/p99. |
| 5 | Memory growth came from 2 samples at 8 MiB `MemoryManager` bucket granularity. | No conclusion was possible. | The harness samples `memory_metrics.stable_memory_size` every round and fits a regression: **~44.4 KB of stable memory per classification** (heap is flat). |
| 6 | The 13-node vs 34-node question was not addressed. | — | The same workload was run on the fiduciary (34-node) subnet: cost ×2.61–2.62 per call, matching 34/13 = 2.615. |
| 7 | The committed `.md` was hand-edited beyond what the generator produced. | The next `just load-test` run would have overwritten it. | The harness now writes JSON only, and this narrative file is written by hand. |
| 8 | The 20×20 run was not representative. | It stays below calibration (50 tasks), seeds 5,000 subjects for 400 classifications, and golds seen per AAA stay low. | 40×70 (2,800 classifications), with 2,800 subjects. |

The per-call attribution method (cycle-balance deltas around one ingress `update_call`, which completes the inter-canister call and its callback) is sound. It correctly charges the platform for its own `canister_info` provenance call. No timers fire during the loop because PocketIC time barely advances.

## 2. Subnet pricing (measured, `t7_3_pricing_probe`)

| | app (13-node) | fiduciary (34-node) | ratio |
|---|---|---|---|
| cycles / instruction | 1.000 | 2.615 | 2.615 |
| ingress update base (reception + execution) | 6.55M | 17.13M | 2.615 |
| `cost_create_canister` | 500B | 1,307.7B | 2.615 |
| `cost_call` per KiB payload | 1.02M | 2.68M | 2.615 |

PocketIC application subnets are priced as 13-node mainnet subnets. The results below use 13-node prices. For a 34-node subnet, multiply them by 2.615.

## 3. Per-call cost, 40 AAAs × 70 tasks, 13-node (`app13_40x70`)

Platform's own cost is what the fee must cover. AAA overhead is the owner's relay cost on top of the fee. All figures are in cycles.

| call | n | platform avg | p50 | p95 | p99 | max | AAA overhead avg |
|---|---|---|---|---|---|---|---|
| get_task | 2800 | 15.08M | 8.96M | 48.39M | 57.04M | 70.56M | 17.05M |
| submit_classification (no flag) | 2744 | 20.76M | 20.86M | 21.37M | 21.55M | 21.92M | 17.24M |
| submit_classification (flagged) | 56 | 37.80M | 39.31M | 45.90M | 46.97M | 47.29M | 17.58M |
| submit_classification (all) | 2800 | 21.10M | | | ≈47M | | 17.25M |
| get_review_assignment (assigned) | 108 | 8.45M | 8.43M | 8.96M | 9.04M | 9.45M | 14.33M |
| get_review_assignment (none) | 396 | 6.99M | | | 7.19M | | 13.77M |
| get_review_assignment (NotEligible) | 496 | 6.33M | | | 6.36M | | 13.32M |
| submit_review | 108 | 24.54M | 19.21M | 36.01M | 36.29M | 36.70M | 16.61M |

The 34-node check (`fiduciary34_10x20` vs `app13_10x20`) gave these ratios: get_task 2.42× on the average and 2.44× on the max (the samples are small and the gold scan is uneven); submit 2.62×; get_review 2.61×; AAA overhead 2.61×.

## 4. Fee formula and chosen values

`fee ≥ required = max(2 × avg, p99)`, taking platform's own cost on a 13-node subnet and rounding up to a round number. The unit test `t7_3_default_fees_cover_measured_platform_cost_times_safety_factor` (`crates/platform/src/config.rs`) recomputes this from the measured constants. It also asserts `fee ≤ 1.5 × required`, so owners are not overcharged.

| param | old | required | **new** | coverage of avg cost | on 34-node, would need |
|---|---|---|---|---|---|
| `fee_get_task` | 50M | 57.0M (p99) | **60M** | 4.0× | 157M |
| `fee_submit_classification` | 200M | 47.0M (p99) | **50M** | 2.4× | 123M |
| `fee_get_review` | 50M | 16.9M (2×avg) | **20M** | 2.4× (assigned) | 44M |
| `fee_submit_review` | 200M | 49.1M (2×avg) | **50M** | 2.0× | 128M |

The AAA's cached defaults (`crates/aaa/src/params.rs`) now match these values. A fee change is safe for AAAs already deployed: the platform accepts exactly the fee and refunds any excess. A fee raise costs one `InsufficientFee` retry, and the daily tick then refreshes the cached fee.

## 5. Owner cost, and what the platform keeps

These assume a 13-node AAA, 1 T cycles = 1 XDR = $1.35, and **ICP = $8** (the assumption spec 12 uses, 5.93 T cycles/ICP).

| per classification (get_task + submit) | old | new |
|---|---|---|
| fees | 250M | 110M |
| AAA relay overhead | 34.3M | 34.3M |
| **owner total** | **284.3M** | **144.3M** |
| platform's own execution cost | 36.2M | 36.2M |
| platform margin (share of fees) | 213.8M (86%) | 73.8M (67%) |

| per 100 classifications | old | new |
|---|---|---|
| cycles | 28.4B | 14.4B |
| USD | $0.038 | **$0.0195** |
| ICP @ $8 | 0.0048 | **0.0024** |

At the new values, 1,000 classifications cost $0.19, against KR4.4's target of ≤ $2. If the AAA runs on a 34-node subnet, its overhead is ×2.615 and the owner pays 199.7M per classification. A tier-2 review costs the owner 20M + 50M + 31M of overhead = 101M.

## 6. Other checks against the measured burn

- **`spawn_creation_fee_cycles`: 100B → 500B (fixed in payments).** The measured `cost_create_canister` is 500B on 13-node subnets. With 100B, a spawn quoted (1T + 100B) × 1.02 = 1.122T, and the new AAA kept only about 0.62T. The same gap explains T6.6's finding that "0.5T sponsor was too little to install": creation fee 100B + 0.5T, minus the real 500B, left 0.1T. With 500B, the AAA keeps ≥ 1T (`t7_3_spawn_quote_covers_measured_creation_fee_and_initial_cycles`). If the CMC ever places an AAA on a 34-node subnet, creation costs 1.31T. The CMC's default subnets are 13-node.
- **`aaa_initial_cycles` 1T: keep.** It covers about 6,900 classifications at 144M. The AAA's idle storage is 70 MB (mostly 8 MiB first-touch buckets), about 0.26T/yr at the published 127k cycles/GiB/s. The low-cycles guard drops to 10B + 50 × 60M = 13B.
- **Sponsor minimum 1T: keep.** With the corrected creation fee, a sponsored AAA now actually starts with about 1T.
- **Treasury `reserve_e8s` 5 ICP ($40): keep.** Platform execution is fee-funded and net-positive: fees cover 3.0× its per-call cost. Its idle cost is storage: about 300 MB, roughly 1.1T/yr ($1.5/yr), plus about 165M cycles per classification-year (see below).
- **Payments `spawn_quote_buffer_bp` 200 (2%): keep.** It absorbs XDR rate drift within `rate_max_age_secs`. The measured costs are deterministic and are now built into the quote itself.

## 7. Findings for follow-up (platform code, outside T7.3 scope)

1. *(Resolved in T7.12, §8: the 44 KB was 8 MiB bucket first-touch; the marginal cost is ≈1.3 KB.)* **Stable storage is about 44 KB per classification** (regression over 52 per-round samples; heap is flat). At the published 127k cycles/GiB/s, that is about **165M cycles per classification per year** on 13-node. That exceeds the 73.8M fee margin after about 5 months, so KR4.1 cannot hold long-term at this footprint. The content per classification is under 1 KB, so the footprint looks like overhead: candid-encoded `Unbounded` values with type tables across about 10 maps, plus the `SUBJECTS` rewrite on every tally. Shrinking it is cheaper for owners than pricing storage into the fee. With the storage cost included, `fee_submit_classification` would need about 200M, which happens to match the old estimate.
2. *(Resolved in T7.12, §8: gold index, mem 16.)* **The `get_task` gold pick is a linear `SUBJECTS` scan** (`catalog::issue_task`). It is O(golds already seen / gold density), which explains the p95/p99 tail. At tier 5 (≥ 300 gold tasks), a gold `get_task` would scan about 1,900 subjects, roughly 400M+ instructions. The pool-exhausted fallback scans every subject. Every AAA also receives gold subjects in the same order, which makes them easy for colluding agents to share. Fix with a gold index and a random or per-AAA start. Then re-measure and lower `fee_get_task`.
3. `get_review_assignment` charges the full fee to tier-1 AAAs that get `NotEligible`. The AAA could check its tier before paying.

## 8. T7.12 follow-up: storage per classification and the gold index

**Where the 44 KB went.** The per-round samples in `load-test-T7.3.json` (`app13_40x70.platform_stable_by_round`) change only in whole 8 MiB steps: +6 buckets in round 1, +4 in round 2, +1 at 520 and 1,240 classifications, and +7 at 1,600 when reviews start. Each step is a `MemoryManager` first touch of another virtual memory (leases, seen-set, classifications, indexes, reviews...). About 19 × 8 MiB = 152 MiB of fixed cost was spread over 2,800 classifications. The real marginal cost was already small.

Marginal stable bytes per classification, measured by `t7_12_steady_state_stable_bytes_per_classification_at_most_3_kib` (40 AAAs × 40 rounds through `issue_task` + `process_submission`, protocol v1 shape, 64 KiB page resolution):

| mem | map | bytes / classification |
|---|---|---|
| 20 | `CLASSIFICATIONS` | 409 |
| 41 | event log data | 409 |
| 12 | `LEASES` | 204 (now pruned 7 days after expiry) |
| 35 | per-AAA classification index | 81 |
| 47 | per-AAA activity index | 81 |
| 13 | seen-set | 40 |
| 21 | per-subject classification index | 40 |
| 44 | leaderboard | 40 |
| | **total** | **≈1.3 KB** (≈1.1 KB once leases are pruned) |

**Fix.** Fresh installs now use 1 MiB buckets (`memory::BUCKET_PAGES = 16`). A first touch costs 1 MiB instead of 8 MiB, so the fixed cost of the same run is about 19 MiB. Amortized over 2,800 classifications that is ≈7 KB + 1.3 KB ≈ 8.3 KB, and it tends to 1.3 KB as volume grows. An existing canister keeps its 8 MiB layout, because the bucket size is read from its header. Platform idle storage for a fresh install falls from ~300 MB to ~40 MB. Storage per classification-year falls from ~165M to **~4.9M cycles**, well inside the 73.8M fee margin. The record encoding was not changed: `Classification` (≤ 512 B with 7 answers) and `Event` are not worth a v2 migration at this size.

**Gold `get_task`.** Mem 16 is a gold index of active gold subject ids, maintained on every subject write and backfilled once on upgrade. A gold pick starts at a random key and walks forward, wrapping, and skips subjects this AAA has already seen. The cost depends on the golds this AAA has seen, not on pool size. The pool-exhausted fallback uses the same index. The random start also gives each AAA a different gold order.

**Still to measure** (owner rule: integration runs at the end): `t7_12_storage_per_classification_and_flat_get_task_tail` (`just load-test`) asserts second-half stable growth ≤ 10 KB/classification and get_task p99 below T7.3's 57.04M. Then `fee_get_task` can be lowered.
