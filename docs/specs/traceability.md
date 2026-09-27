# Traceability: acceptance items → proofs

`scripts/traceability.py` (run by `just verify`) fails when a done task has no row, when a test named here does not exist, or when a proof script is missing. Proofs are test function names (`t…`/`sp_…`) or a script path.

| Item | Task | Proof |
|---|---|---|
| `icp build` succeeds for every environment canister | T1.1 | t1_1_icp_build_produces_every_canister_wasm |
| Every canister commits its generated `.did` | T1.1 | t1_1_every_canister_commits_its_candid_interface |
| Toolchain pinned with the wasm target | T1.1 | t1_1_toolchain_is_pinned_with_wasm_target |
| sc-types round-trip through candid | T1.2 | t1_2_every_shared_type_survives_a_candid_round_trip |
| ApiError variants match 02 §2 | T1.2 | t1_2_api_error_has_every_spec_variant |
| 08 §3 input limits | T1.2 | t1_2_input_limits_match_the_security_spec |
| Candid shapes match 02 §2 | T1.2 | t1_2_candid_shapes_match_the_platform_spec |
| `.did` drift gate catches interface changes | T1.3 | t1_3_candid_drift_check_passes_then_catches_a_changed_interface |
| `deploy-local` is idempotent | T1.3 | t1_3_deploy_local_is_idempotent_and_every_canister_answers |
| PocketIC harness mints ICP and tops up via the CMC | T1.4 | t1_4_harness_mints_icp_and_tops_up_a_canister_through_the_cmc |
| CMC memo behaviour (SP-1) | SP-1 | sp_1_cmc_notify_top_up_accepts_which_transfer_kinds |
| OISY/ledger consent message (SP-2) | SP-2 | sp_2_ledger_consent_message_shows_the_spender_subaccount |
| Frozen canister behaviour (SP-3) | SP-3 | sp_3_frozen_aaa_rejects_queries_until_topped_up_via_the_cmc |
| canister_info cost/latency (SP-4) | SP-4 | sp_4_canister_info_cost_and_latency_same_vs_cross_subnet |
| ckETH/ckBTC minter facts (SP-6) | SP-6 | scripts/spikes/sp-6-minters.sh |
| Gold source decision (SP-7) | SP-7 | scripts/spikes/sp-7-gz-candels.py |
| Property tests for validators and round-trips | T1.8 | t1_8_prop_name_validation_never_panics_and_is_idempotent |
| Error messages and validator branches covered | T1.8 | t1_8_every_api_error_has_a_readable_message |
| 50 fixture dossiers (10 gold, 5 honeypots) | T1.8 | test_t1_8_fixture_set_has_50_subjects_10_gold_5_honeypots |
| Fixture dossiers match sc-dossier/1 and their hashes | T1.8 | test_t1_8_every_dossier_matches_schema_and_hashes |
| Gold/honeypot status never in a dossier | T1.8 | test_t1_8_gold_and_honeypot_status_never_leak_into_dossiers |
| Deterministic bot agent reaches local canisters | T1.8 | t1_8_bot_agent_talks_to_every_local_canister |
| Selection criteria (07 §5.1) | T1.5 | test_t1_5_criteria_follow_spec_07 |
| Field quotas and z-stratification, deterministic | T1.5 | test_t1_5_selection_is_deterministic_and_ids_unique |
| Gold mapping, thresholds, 0.3″ match, z < 2 guard | T1.5 | test_t1_5_gold_mapping_thresholds_and_z_guard |
| 20k selected; ≥ 2k gold (committed v1 data) | T1.5 | test_t1_5_committed_v1_selection_meets_acceptance |

## Waivers (printed on every `just verify` until removed)

| Task | Reason |
|---|---|
| T6.1 | Marked done by a parallel session with no automated tests (`lint`/`test` scripts are `echo` placeholders). Vitest + Playwright/axe land with T6.10; remove this waiver when they do. |
