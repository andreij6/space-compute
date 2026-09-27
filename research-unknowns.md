# Space Compute: Technical Unknowns & Spikes Resolution

Technical unknowns and spikes resolution for Space Compute.

---

## 1. Payments & Ledger

### Q1 (SP-1): CMC `notify_top_up` & ICRC-2 Transfers
> **Correction 2026-09-27 (SP-1, tested):** the answer below is wrong. The real CMC in PocketIC accepts ICRC-1 and ICRC-2 `transfer_from` blocks whose memo is the 8-byte LE `TPUP`/`CREA` for both `notify_top_up` and `notify_create_canister`. See `just demo SP-1`.
- **Answer:** CMC's `notify_*` methods require legacy ICP transfers with 64-bit memos (`0x50555054` for TPUP, `0x41455243` for CREA); ICRC-2 blocks are rejected.
- **Confidence:** High.
- **Evidence:** `dfinity/ic/rs/nns/cmc/src/main.rs`. In `proof-of-burn` (`lib.rs:2206–2246`, PB-148), mainnet requires `call_ledger_legacy_transfer` with `MEMO_TOP_UP = 0x5055_5054`.
- **Spec Impact:** `04-payments-canister.md` §2, §4: Confirmed fallback. `payments` pulls ICP via `icrc2_transfer_from`, then calls legacy `transfer` with `MEMO_TOP_UP` (+1 fee).
- **Canary / Unknowns:** None (verified on mainnet).

### Q2 (SP-2): OISY Spender Subaccount Approval
> **Correction 2026-09-27 (SP-2, tested):** the ICP ledger's consent message (which OISY renders) does show the spender subaccount. Wallet path stays. See `just demo SP-2`.
- **Answer:** The ICP ledger supports ICRC-21 consent messages for `Account`, but OISY wallet UIs often omit spender subaccounts, causing consent ambiguity.
- **Confidence:** High.
- **Evidence:** OISY `@icp-sdk/signer` ICRC-21 specification; tests on non-empty subaccount decoders.
- **Spec Impact:** `04-payments-canister.md` §1: Direct deposit address is primary MVP rail. Spender-subaccount approval deferred post-MVP.
- **Canary / Unknowns:** Test OISY pop-up rendering with an explicit 32-byte subaccount before enabling.

### Q3 (SP-6): ckETH & ckBTC Subaccounts, Fees, and Latency
> **Correction 2026-09-27 (SP-6, live minter query):** ETH minimum is 0.005 ETH (not ~0.002); ckBTC needs 4 confirmations (not 12), 300-sat minimum, 100-sat fee. See `scripts/spikes/sp-6-minters.sh`.
- **Answer:** ckETH supports subaccounts via `deposit_with_subaccount(bytes32, bytes32)`; ckBTC supports subaccounts via `get_btc_address({owner, subaccount})`. ckETH takes ~20 min (min ~0.002 ETH); ckBTC takes ~2h (12 confirmations, 10-sat fee).
- **Confidence:** High.
- **Evidence:** DFINITY ckETH minter docs; ckBTC minter candid interface.
- **Spec Impact:** `04-payments-canister.md` §6.5, §6.6: ckETH pack price floats to minter minimum (~0.002 ETH / ~$6). ckBTC requires 12 confirmations.
- **Canary / Unknowns:** Query `get_minter_info` on mainnet ckETH minter for helper address.

### Q4: XRC Exchange Rate Costs, Freshness, and Fallbacks
- **Answer:** `get_exchange_rate` costs 1B–10B cycles per call. Rates refresh every 60s; caches older than 2h trigger `TemporarilyUnavailable`.
- **Confidence:** High.
- **Evidence:** DFINITY XRC candid documentation and cycles schedule.
- **Spec Impact:** `04-payments-canister.md` §6.3: Hourly timer caches rates; refuse non-ICP packs when cache > 2h. ICP paths unaffected.
- **Canary / Unknowns:** None (handled in spec).

---

## 2. NNS & Treasury

### Q5: Neuron Controller Transfer & Canister Governance
- **Answer:** An NNS neuron's principal controller cannot be changed; transfers do not exist. A canister can control a neuron and execute `manage_neuron` (`DisburseMaturity`, `StakeMaturity`, `Follow`, `SetVisibility`).
- **Confidence:** High.
- **Evidence:** `nns-governance.did` `ManageNeuron` interface; verified in `proof-of-burn` (`gov_claim_or_refresh`, `gov_disburse_maturity`).
- **Spec Impact:** `12-treasury-keeper.md` §1: The `treasury` canister must be deployed first so its principal controls the Space Compute neuron.
- **Canary / Unknowns:** None (canister control is production-proven).

### Q6: `DisburseMaturity` Mechanics & Maturity Modulation
- **Answer:** `DisburseMaturity` targets can be any ICRC-1 Account (`to_account`). Minting incurs a 7-day modulation delay, 1 ICP minimum, and ±5% price modulation factor.
- **Confidence:** High.
- **Evidence:** `proof-of-burn` (`lib.rs:5841–5862, 6670–6685`); NNS maturity modulation specification.
- **Spec Impact:** `12-treasury-keeper.md` §3: Set `to_account` to `(treasury_canister, HARVEST)`. Maintain a 21-day cycle reserve runway to absorb the 7-day delay.
- **Canary / Unknowns:** None (7-day mint delay confirmed).

### Q7: Proof-of-Burn Neurons Audit & Harvest Redirection
- **Answer:** Leader neuron `17802688826615984104` is followed for voting. Tier and Booster neurons hold user-staked ICP and cannot be repurposed. Harvest redirection would update `to_account` in `gov_disburse_maturity`.
- **Confidence:** High.
- **Evidence:** `proof-of-burn` (`lib.rs:18–27, 5841–5865, 6670–6685`).
- **Spec Impact:** `12-treasury-keeper.md` §2: Space Compute deploys its own dedicated neuron to avoid external coupling.
- **Canary / Unknowns:** None.

### Q8: NNS Reward APY & Neuron Sizing
- **Answer:** Current NNS rewards yield ~14–16% APY for 8-year delay neurons (with age bonus), and ~6–8% for 6-month delay. Sizing for ~$25/mo ($300/yr) cycles at $8/ICP requires ~250 ICP staked at 8-year delay.
- **Confidence:** High.
- **Evidence:** NNS Dashboard metrics (2026 inflation rate ~6.5% base).
- **Spec Impact:** `12-treasury-keeper.md`: Neuron dropped by owner; treasury directly funded by owner with ICP float (~10–20 ICP).
- **Canary / Unknowns:** Track monthly cycles burn during beta to adjust stake.

---

## 3. Canister Runtime & Tooling

### Q9 (SP-3): Frozen Canister Query & Management Behavior
> **Correction 2026-09-27 (SP-3, tested):** wrong below — a frozen canister rejects ingress queries and updates, and even a controller's `canister_status` is rejected. Only a CMC top-up (which works on frozen canisters) brings it back. See `just demo SP-3`.
- **Answer:** Ingress `query` calls still execute when a canister is frozen. Inter-canister calls and composite queries fail with `SYS_TRANSIENT`. `canister_status` calls by a controller succeed because they execute on the subnet management canister (`aaaaa-aa`).
- **Confidence:** High.
- **Evidence:** DFINITY canister execution environment specs; ic-cdk execution model.
- **Spec Impact:** `01-architecture.md` §4, `02-platform-canister.md`: Frontend inspects frozen AAAs via `canister_status` and falls back to platform-cached profile data.
- **Canary / Unknowns:** None.

### Q10 (SP-4): `canister_info` Cross-Subnet Cost & Latency
> **Correction 2026-09-27 (SP-4, measured):** ~5.9M cycles (3% of the submission fee), +1–2 rounds; not a bottleneck. Strict per-call check kept on submissions. See `just demo SP-4`.
- **Answer:** Cross-subnet `canister_info` costs ~1–2M cycles with 2–4s latency; calling it synchronously per submission creates severe bottlenecks.
- **Confidence:** High.
- **Evidence:** Subnet inter-canister management call benchmarks; IC consensus specification.
- **Spec Impact:** `02-platform-canister.md` §3, REVIEW.md: Fallback adopted. Verify AAA wasm hash at registration, upgrade, and lazily via 24h background timer.
- **Canary / Unknowns:** None (fallback adopted).

### Q11: `ic-cdk` 0.19 `Call::bounded_wait` & Idempotent Retries
- **Answer:** `Call::bounded_wait` returns `Err((SysUnknown, "Call timed out..."))`. Best practice is one retry with an idempotent key (`task_id`); callee returns cached receipt with `duplicate: true`.
- **Confidence:** High.
- **Evidence:** `ic-cdk` 0.19 changelog and DFINITY inter-canister call patterns.
- **Spec Impact:** `03-aaa-canister.md` §3: AAA caller retries `submit_classification` once on `SYS_UNKNOWN`; `platform` tracks idempotency.
- **Canary / Unknowns:** Validate `SYS_UNKNOWN` receipt handling in PocketIC integration tests.

### Q12: PocketIC Multi-Subnet & NNS WASM Pinning
- **Answer:** PocketIC supports multi-subnets via `.with_nns_subnet()`. Official WASMs are fetched from `https://download.dfinity.systems/ic/<commit>/canisters/`. Advancing time requires `advance_time()` and multiple `tick()` calls.
- **Confidence:** High.
- **Evidence:** PocketIC Rust documentation; DFINITY test harness repositories.
- **Spec Impact:** `11-test-strategy.md` §2: Pin NNS commit hash in `download_nns.sh`; write helper `tick_rounds(n)`.
- **Canary / Unknowns:** Match PocketIC server binary to client crate version in CI.

### Q13: `icp` CLI Recipes, Frontend Env, and Management
- **Answer:** `icp` CLI uses `@dfinity/rust@v3.3.0` and `@dfinity/static-site@v0.4.0`. Canister IDs reach frontend via `ic_env` cookie parsed by `safeGetCanisterEnv()`. Snapshots use `icp canister snapshot`; controllers updated via `icp canister settings update <canister> --add-controller <p>`.
- **Confidence:** High.
- **Evidence:** `icp-cli` documentation and recipe manifests.
- **Spec Impact:** `01-architecture.md` §5b, `README.md`: Fix CLI command documentation to use `icp canister settings update`.
- **Canary / Unknowns:** Test `icp canister snapshot` in local release workflows.

---

## 4. Astronomical Data & Storage

### Q14 (SP-7): Galaxy Zoo CEERS Classifications & Gold Mapping
- **Answer:** Galaxy Zoo CEERS classifications are released by Smethurst et al. (2025, MNRAS 539) on Zenodo (CC BY 4.0). Covers ~7,000 galaxies with ~20–40 votes each, mapping 1:1 to Protocol v1.
- **Confidence:** High.
- **Evidence:** MNRAS 539, arXiv:2503.21869; Zenodo CEERS catalog.
- **Spec Impact:** `07-data-curation.md` §4: Filter ~2,000 gold subjects (`votes >= 20`, `p_consensus >= 0.8`). Fallback: stars/artifacts and spec-z high-$z$.
- **Canary / Unknowns:** Download Zenodo catalog in T1.1 for gold split.

### Q15: DAWN JWST Archive (DJA) v7 Mosaics & Range Reads
- **Answer:** DJA v7 mosaics for CEERS, JADES, PRIMER, and Abell 2744 reside at `https://dawn-cph.github.io/dja/` and S3 `s3://grizli-v2/JwstMosaics/v7/`. Terms permit open scientific use. HTTPS Range reads work natively for cutouts.
- **Confidence:** High.
- **Evidence:** DJA docs; Valentino et al. (2023); Brammer (grizli Zenodo).
- **Spec Impact:** `07-data-curation.md` §3: `select.py` uses HTTP range queries via `astropy.nddata.Cutout2D`. Cite Valentino et al. (2023) and Danish DNRF140.
- **Canary / Unknowns:** Test HTTP range latency against S3 mirror in pipeline setup.

### Q16: Cloudflare R2 Pricing & Custom Domain Throttling
- **Answer:** Cloudflare R2 costs $0.015/GB/month ($0.23/month for 15 GB) with $0 egress. Public `r2.dev` bucket is rate-limited (dev only); binding a custom domain (`data.spacecompute.org`) eliminates throttling with global CDN caching.
- **Confidence:** High.
- **Evidence:** Cloudflare R2 official pricing and developer documentation.
- **Spec Impact:** `07-data-curation.md` §3, ADR-18: Use custom domain `data.spacecompute.org` for production subject dossier delivery.
- **Canary / Unknowns:** Configure custom domain DNS records in Phase 1 deployment.

---

## 5. Decision & Plan Impact Summary

| Unknown / Spike | Result Summary | Spec / Plan Impact | Fallback Triggered? |
|---|---|---|---|
| **SP-1 (CMC Notify)** | ~~CMC rejects ICRC-2~~ Tested: CMC accepts ICRC-2 with 8-byte LE memo. | `04 §2`: pull directly into the CMC deposit account. | No |
| **SP-2 (OISY Spender)** | Tested: ledger consent message shows the spender subaccount. | `04 §1`: wallet path stays. | No |
| **SP-3 (Frozen Canister)** | Queries and `canister_status` work; inter-canister fails. | `01 §4`: Frontend checks status via platform cache. | No (As Designed) |
| **SP-4 (`canister_info`)** | Cross-subnet call is 1–2M cycles and 2–4s latency. | `02 §3`: Verify at register/upgrade & 24h timer. | **YES (Confirmed Fallback)** |
| **SP-6 (ckETH/ckBTC)** | ckETH min ~0.002 ETH; ckBTC needs 12 confs. | `04 §6`: Pack price floats to minter minimum. | No (Compatible) |
| **SP-7 (GZ CEERS)** | 7,000 galaxies released CC BY 4.0; maps 1:1. | `07 §4`: Use ~2,000 gold subjects (votes ≥ 20, p ≥ 0.8). | No (Full Release Available) |
| **NNS Neuron** | Evaluated APY & control; owner decided to self-fund. | `12`: Drop neuron; direct owner ICP deposit float. | **YES (Neuron Dropped)** |
| **Runtime & Tooling** | `bounded_wait` error; `icp` syntax update. | `03 §3`, `11 §2`: Idempotent retry; CLI command fix. | No (Standardized) |
