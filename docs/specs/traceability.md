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
| 5k selected; ≥ 750 gold (committed v1 data; 20k before beta) | T1.5 | test_t1_5_committed_v1_selection_meets_acceptance |
| Streamed cutouts equal direct slices (incl. edges) | T1.7 | test_t1_7_streamed_cutouts_equal_direct_slices_including_edges |
| Cutout WCS points at the target | T1.7 | test_t1_7_cutout_wcs_points_at_the_target |
| RICE FITS round-trip keeps the signal | T1.7 | test_t1_7_rice_fits_round_trip_keeps_signal |
| RGB channels share one grid when filters differ in pixel scale | T1.7 | test_t1_7_rgb_common_grid_aligns_mixed_pixel_scales |
| All dossiers rendered + hashed; manifest verifies; 40 QA spot-checks | T1.7 | test_t1_7_committed_manifest_and_qa_meet_acceptance |
| Upgrade keeps config (params, admins, pause flags, audit log); RNG reseeded | T2.1 | t2_1_upgrade_keeps_config_admins_audit_log_and_reseeds_rng |
| Params bounded; admin set rules (no anonymous, last admin kept) | T2.1 | t2_1_admin_set_rules_and_stored_config_round_trip |
| Role matrix enforced (owner, operator, expired operator, platform, anonymous, stranger) | T3.1 | t3_1_aaa_role_matrix_owner_operator_platform_anon_stranger |
| Spawned AAA verified; upgrade works; name unique with -2 suffix | T2.2 | t2_2_spawned_aaa_verified_and_upgrade_works |
| Journal persisted pre-await (a trap after the await rolls back only the post-await transition; the pre-await entry survives) | T5.1 | t5_1_payments_skeleton_config_journal_guard_admin |
| Quotes within 2% of what the real CMC mints; XDR cache primed on install and refreshable on demand; per-purpose deposit accounts derive distinct subaccounts | T5.2 | t5_2_quotes_are_within_2_percent_of_what_the_cmc_actually_mints |
| Catalog dispatch: subjects, protocol, leases, seen-set (never same subject twice) | T2.3 | t2_3_never_same_subject_twice_and_pool_dispatch |
| Retire at K=5; idempotent submit | T2.4 | t2_4_retire_at_k5_and_idempotent_submit |
| Events append; paged activity | T2.5 | t2_5_events_append_and_paged_activity |
| Queries paged ≤100 | T2.6 | t2_6_public_queries_paged_limit_100 |
| Foreign/expired/unsynced submitter rejected | T2.8 | t2_8_foreign_expired_unsynced_submitter_rejected |
| Gold/answers admin-only; get_task submitter check; calibration gold ends; verify internal; admin suspension sticks; payments-only register | T2.9 | t2_9_gold_hidden_submitter_checked_calibration_ends_verify_internal |
| Retry skips install when approved module present; unrecorded total_num_changes suspends + audit; duplicate returns original receipt | T2.9 | t2_9_retry_skips_existing_module_and_unrecorded_change_suspends |
| Controllers ⊇ {owner, platform}; real total_num_changes recorded | T2.9 | t2_9_register_requires_owner_and_platform_and_records_real_changes |
| verify never lifts admin suspension; num_changes mismatch suspends | T2.9 | t2_9_verify_never_lifts_admin_suspension_and_checks_num_changes |
| Heartbeat limited even at 0 cycles; self-reported version ignored | T2.9 | t2_9_heartbeat_rate_limited_even_with_zero_cycles_and_ignores_version |
| Profile update: Active only, 1/hour | T2.9 | t2_9_profile_update_requires_active_and_is_hourly |
| Per-AAA open-lease index (mem 15) | T2.9 | t2_9_open_lease_index_tracks_consumption_and_sweeps_expiry |
| Task pool cursor range iteration wraps | T2.9 | t2_9_pool_cursor_rotates_and_wraps |
| Submission limits (08 §3) at the boundary | T2.9 | t2_9_submission_limits_enforced_at_boundary |
| Leaderboard cursor (inverted_xp, aaa) with carried rank | T2.9 | t2_9_leaderboard_cursor_pages_through_ties_with_continuous_rank |
| Re-adding subjects keeps progress; published protocols immutable | T2.9 | t2_9_readding_subjects_keeps_progress_and_protocols_are_immutable |
| Per-AAA CallerGuard for awaiting mutations | T2.9 | t2_9_guard_blocks_same_aaa_until_dropped |
| Registered AAA task receipt, fees, and duplicate idempotency | T2.7 | t2_7_registered_aaa_task_receipt_and_idempotent_duplicate |
| 5th classification retires subject; seen-set never reissues | T2.7 | t2_7_fifth_classification_retires_and_seen_set_never_reissues |
| One record per task under SYS_UNKNOWN | T3.2 | t3_2_one_record_per_task_under_sys_unknown |
| Records survive upgrade | T3.3 | t3_3_records_and_credits_survive_canister_upgrade |
| get_api_doc documents every public method; wasm ≤ 1.5 MiB gz (verify gate) | T3.5 | t3_5_api_doc_lists_every_public_method_in_the_candid |
| Keeper tops up low canisters via ledger→CMC, skips healthy ones; reserve floor stops top-ups and flags health() | T5.17 | t5_17_keeper_tops_up_low_canisters_and_health_flags_the_reserve |
| Reserve floor never crossed; transfer retry reuses created_at; burn EMA ignores top-ups | T5.17 | t5_17_reserve_floor_is_never_crossed |
| Sensitive config changes (admins, lower reserve) need a second admin | T5.17 | t5_17_sensitive_config_changes_need_a_second_admin |
| 12 §5 #1-6: deposit, top-up, skip, reserve, two-admin withdraw, upgrade + timers resume | T5.18 | t5_18_withdraw_needs_two_admins_and_state_survives_upgrade |
