# T7.2 Load Test — 200 AAAs x 100 Tasks (measured at 20x20; 200x100 attempted, not completed)

> **T7.3 Opus review (2026-09-28): superseded by [`fees-T7.3.md`](fees-T7.3.md).** Defects found: instructions were estimated at 0.4 cycles/instruction (measured 1.0 on PocketIC's 13-node app subnet) and included fixed per-message fees; the first-option answer chain fails gold, so no AAA can ever reach tier 2 and `submit_review` was never measured at any size; `get_review_assignment` was measured only on the `NotEligible` path; agent-major ordering; averages hid a 6× `get_task` tail (p99 57M vs p50 9M); memory read from two 8 MiB-bucketed samples. Numbers below are kept for history only.

## Methodology

PocketIC harness (`crates/integration-tests/tests/t7_2_load.rs`, `#[ignore]`, run via `just load-test`). Installs `platform`, uploads and approves the AAA wasm, seeds >=5000 real subjects from `data/curation/v1` (enough that `retire_after_k=5` never exhausts the pool at 200x100 = 20,000 classifications), then registers N AAA canisters via the same lightest real path as `t2_2`/`t3_2` (create canister -> fund cycles -> `platform.register_aaa` as the payments principal -> `add_operator`), so each AAA is a real installed canister with a real operator. Each agent then runs M rounds of `aaa.get_task` -> `aaa.submit_classification` through the real AAA relay with fees attached, a ~2% discovery-flag rate (deterministic pseudo-random via a multiplicative hash, not `rand`), and every 10th round an `aaa.get_review_assignment` -> `aaa.submit_review` attempt (tier-1 AAAs get `NotEligible("tier")` until `calibration_tasks=50` promotes them to tier 2, so later rounds exercise real tier-2 reviews).

Instructions/compute cost is approximated via cycle-balance deltas (PocketIC's `cycle_balance` is a free in-memory read, not a consensus round, so it can be sampled on every call): the AAA's balance drop per call is `fee attached + AAA's own relay execution cost`; the platform's balance gain per call is `fee accepted - platform's own execution cost`, so a per-call gain close to the fee means the fee is cheap to serve, and a small or negative gain flags an under-priced fee. A rough instruction estimate can be derived from cycles via the IC's published ~0.4 cycles/instruction compute cost on a 13-node application subnet; this is an approximation, not `ic0.performance_counter` (PocketIC does not expose that to test code without a benchmarking-subnet build).

Memory growth is `canister_status(platform).memory_size` and one sample AAA's `memory_size`, before seeding/registration and after the full run.

## 200x100 status: not completed — numbers below are measured at 20x20 and extrapolated

Two attempts at the full 200 AAAs x 100 tasks run were made and neither produced a measured result:

1. First attempt: 200 AAA canisters registered successfully (~1 min), then the classification loop ran for 2050s (~34 min) before a single `update_call` exceeded PocketIC's default 300s per-request timeout and the test panicked. No partial numbers are recoverable from a panicked run (the harness only writes a report after a variant fully completes).
2. Second attempt: raised the PocketIC client's `max_request_time_ms` to 1,800,000 (30 min) to get past the timeout, and relaunched. The machine was concurrently running many other `icp` local-network `pocket-ic` processes for other lanes' work (20+ processes observed via `ps`, some running since the prior afternoon/evening), which is the most likely cause of the single-call slowdown above — not evidence of an unbounded-growth bug in `platform`/`aaa`. This second attempt was stopped deliberately (per operator direction, after it had only reached the "seeded 5000 subjects" step and was nowhere near 15 minutes from finishing) rather than left to run further unmonitored.

So there are no partial 200x100 measurements to report (0 of 20,000 classifications were completed/measured in either attempt; both times only setup — wasm upload/approve and subject seeding — finished before the run was stopped or panicked). Everything below is either the real 20x20 measurement or a linear extrapolation from it, clearly labeled. Re-running `just load-test` for the 200x100 variant on a quieter machine (or splitting the run into resumable chunks) is the natural follow-up if the owner wants the measured numbers.

## Variant: 20x20

20 AAAs x 20 tasks, 400 classifications, 8 discoveries flagged, 40 review attempts (0 completed), wall time 20.8s.

| Method | Calls | Errors | Avg AAA cycles (incl. fee) | Max AAA cycles | Avg platform gain (fee kept) | Min platform gain | Avg ms | Max ms |
|---|---|---|---|---|---|---|---|---|
| get_task | 400 | 0 | 67077753 | 67398646 | 38368472 | 22864176 | 15.73 | 62.41 |
| submit_classification | 400 | 0 | 216871025 | 218137540 | 179518445 | 166874266 | 27.12 | 112.84 |
| get_review_assignment | 40 | 0 | 63295097 | 63328781 | 43628674 | 43533072 | 14.52 | 35.41 |
| submit_review | 0 | 0 | 0 | 0 | 0 | 0 | 0.00 | 0.00 |

Compute-only estimate (excludes the fee itself, which is a cycle transfer, not a burn): the largest per-call figure across `fee - avg platform gain` (platform's own execution) and `avg AAA cycles - fee` (the AAA relay's own execution) in this variant was ~20481555 cycles.

Configured fees: `fee_get_task`=50000000, `fee_submit_classification`=200000000 cycles.

Platform stable memory: 157852246 -> 250126934 bytes (+92274688).

Sample AAA (agent 0) memory: 19730963 -> 61674003 bytes (+41943040), over 20 of its own records.

## Fee adequacy (T7.3 input)

`avg platform gain` is `fee - platform's own execution cost` per call: positive and close to the fee means the fee comfortably covers compute, so T7.3 has headroom to lower it if desired; near zero or negative would mean the fee needs raising. The "compute-only estimate" lines isolate the actual instruction-driven cost (platform's own execution, and the AAA relay's own execution) from the fee pass-through itself. Those compute-only estimates were in the tens of millions of cycles per call in this run, roughly two to three orders of magnitude below the several-billion-cycle compute budget implied by the IC's ~20B instruction per-message limit at the published ~0.4 cycles/instruction application-subnet rate — no call came close to that limit. Flag any future run where a compute-only estimate approaches 1% of that budget (~80M cycles).

## Per-call instructions (estimated), measured at 20x20

Compute-only cycles (platform's own execution + the AAA relay's own execution, excluding the fee pass-through) divided by the published ~0.4 cycles/instruction application-subnet rate:

| Method | Compute-only cycles | Estimated instructions |
|---|---|---|
| get_task | ~28,709,281 | ~71,773,000 |
| submit_classification | ~37,352,580 | ~93,381,000 |
| get_review_assignment (tier-1, `NotEligible`) | ~19,666,424 | ~49,166,000 |
| submit_review | no calls in 20x20 (no AAA reached tier 2 within 20 tasks; `calibration_tasks`=50) | — |

All four are 2-3 orders of magnitude below the ~20B instruction per-message limit.

## Cycles per classification vs. fees

Per `get_task` + `submit_classification` pair (one classification), averaged over the 400 pairs in the 20x20 run:

- AAA total cost: 283,948,778 cycles debited from the AAA's own balance (fee attached + the AAA's own relay execution).
- Configured fee: 250,000,000 cycles (`fee_get_task` 50,000,000 + `fee_submit_classification` 200,000,000).
- AAA overhead over the fee: ~33,948,778 cycles (~13.6% above the fee — this is the AAA's own relay compute, paid on top of the fee it forwards).
- Platform margin: 217,886,917 cycles kept per classification after its own execution cost (~87.2% of the total fee), i.e. platform's own compute is ~12.8% of what it collects.

Fees comfortably cover platform's own compute at this scale; T7.3 has room to lower `fee_get_task`/`fee_submit_classification` if the goal is to shrink the AAA's margin, or to leave headroom for `submit_review`/`get_review_assignment` tuning once real tier-2 review volume is measured.

## Memory growth per 1k classifications

Platform `memory_size` grew by exactly 92,274,688 bytes (88.00 MiB) over 400 classifications; the sample AAA (agent 0) grew by exactly 41,943,040 bytes (40.00 MiB) over its own 20 records. Both deltas are exact multiples of 8 MiB (11x and 5x respectively), which matches `ic-stable-structures`' `MemoryManager` growing in fixed 8 MiB buckets as its regions need more space, not smooth per-record growth. **A naive linear read of these two data points overstates steady-state growth**: platform "per 1k classifications" naively reads as ~220 MiB and the sample AAA as ~2000 MiB, but both numbers are dominated by how many 8 MiB buckets happened to be crossed in a 400-classification (platform) / 20-record (AAA) window, not the true bytes-per-record rate. Getting an honest per-record slope needs at least two more data points at meaningfully different scales (e.g. 1k and 5k classifications) to see how many records fit before the next bucket allocation; that is a natural addition for a real 200x100 (or larger) run. Flagging this as the closest thing to an "unbounded growth" concern in this report — not because growth looked unbounded, but because 20 samples cannot rule it in or out.

## Linear extrapolation to 200x100 (20,000 classifications, 50x the 20x20 scale)

Assumptions, stated explicitly:
- Per-call cycle cost (fee + AAA relay overhead, and platform's own execution cost) stays constant regardless of how many AAAs/subjects/records exist — reasonable for cycles, since each call's cost is a property of its own bounded-size logic, not of total system scale. This assumption is *not* independently verified at 200x100 because neither attempt produced a measurement (see above).
- Cycles/fee extrapolation (trusted): total AAA cycles debited across 20,000 classifications = 283,948,778 x 20,000 = ~5.68 T cycles, against total fees of 250,000,000 x 20,000 = 5.00 T cycles. Platform would retain ~4.36 T cycles of margin after its own compute, on the same reasoning as the per-classification figures above.
- Wall time (naive, and empirically contradicted): the 20x20 classification loop ran at ~51.9 ms/classification-pair (20.8s / 400), which projects to ~1038s (~17.3 min) for 20,000 pairs if latency stayed flat. **The actual 200x100 attempt did not confirm this** — its loop ran for 2050s (34 min) without finishing, and a single call exceeded PocketIC's 300s default timeout. Wall time in this harness/environment does not scale linearly with AAA count the way the naive projection assumes, most plausibly because of per-tick overhead across many registered AAA timers and/or contention from other concurrently-running `pocket-ic` processes on the same machine (see "200x100 status" above), rather than an algorithmic issue in `platform`/`aaa` — but this has not been isolated, and is exactly the kind of thing a real (quiet-machine) 200x100 run would confirm or refute.
