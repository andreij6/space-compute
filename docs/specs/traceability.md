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
| One record per task under retry | T3.2 | t3_2_one_record_per_task_under_retry |
| Records survive upgrade | T3.3 | t3_3_records_and_credits_survive_canister_upgrade |
| get_api_doc documents every public method; wasm ≤ 1.5 MiB gz (verify gate) | T3.5 | t3_5_api_doc_lists_every_public_method_in_the_candid |
| Burn EMA excludes top-ups and converges to the sampled daily rate | T3.4 | t3_4_steady_burn_converges_toward_the_daily_rate |
| 6h timer samples burn EMA and triggers payments.request_auto_topup below threshold | T3.4 | t3_4_burn_ema_heartbeat_credits_and_auto_topup_timers |
| 24h timer refreshes cached fees, heartbeats platform, and pulls credits without trapping | T3.4 | t3_4_burn_ema_heartbeat_credits_and_auto_topup_timers |
| Idempotency keys are independent for tasks and reviews sharing a numeric id | T3.4 | t3_4_idempotency_is_independent_for_tasks_and_reviews |
| Credit copies are matched by public_id, never by subject_id-as-task-id | T3.4 | t3_4_upsert_credit_updates_the_correct_record_by_public_id_not_subject_id |
| sync_credit_copy and simulate_sys_unknown_once removed from the production interface | T3.4 | t3_4_burn_ema_heartbeat_credits_and_auto_topup_timers |
| 03 §8.1: non-operator ingress to get_task rejected; operator succeeds and the fee is deducted | T3.6 | t3_2_one_record_per_task_under_retry |
| 03 §8.2: owner adds/removes operators; a removed operator is rejected immediately | T3.6 | t3_2_one_record_per_task_under_retry |
| 03 §8.3: a submit retried after a real SYS_UNKNOWN reject (platform stopped, not a debug backdoor) produces exactly one platform classification and one local record | T3.6 | t3_6_submit_retried_after_sys_unknown_produces_one_classification_and_one_record |
| 03 §8.4: records survive an upgrade (credits half depends on platform.list_aaa_credits being wired to real consensus, tracked separately) | T3.6 | t3_3_records_and_credits_survive_canister_upgrade |
| 03 §8.5: below threshold, the 6h timer calls payments.request_auto_topup exactly once per elapsed interval | T3.6 | t3_6_below_threshold_the_6h_timer_calls_request_auto_topup_exactly_once_per_interval |
| 03 §8.6: aaa wasm is <= 1.5 MiB after a real ic-wasm shrink + gzip | T3.6 | t3_6_aaa_wasm_shrunk_and_gzipped_is_at_most_1_5_mib |
| One command seeds local net (protocol v1, 500 subjects across 6 fields, AAA wasm); re-seeding is idempotent | T3.8 | t3_8_seed_args_load_into_platform_and_reseeding_is_idempotent |
| Existing wasm version is immutable; identical re-upload is a no-op | T3.8 | t3_8_existing_wasm_version_is_immutable_and_identical_reupload_is_a_no_op |
| Keeper tops up low canisters via ledger→CMC, skips healthy ones; reserve floor stops top-ups and flags health() | T5.17 | t5_17_keeper_tops_up_low_canisters_and_health_flags_the_reserve |
| Reserve floor never crossed; transfer retry reuses created_at; burn EMA ignores top-ups | T5.17 | t5_17_reserve_floor_is_never_crossed |
| Sensitive config changes (admins, lower reserve) need a second admin | T5.17 | t5_17_sensitive_config_changes_need_a_second_admin |
| 12 §5 #1-6: deposit, top-up, skip, reserve, two-admin withdraw, upgrade + timers resume | T5.18 | t5_18_withdraw_needs_two_admins_and_state_survives_upgrade |
| 02 §11 #9: a new AAA reaches tier 2 in ≤ 60 honest all-correct tasks | T4.3 | t4_3_new_aaa_reaches_tier2_within_60_honest_tasks |
| 02 §11 #9 (PocketIC, full get_task/submit_classification flow) | T4.3 | t4_3_new_aaa_reaches_tier2_in_60_tasks_and_replay_matches_incremental |
| 02 §8.2: admin_replay_progression rebuilds Progress + leaderboard from event 0 in batches, driven by a timer to completion | T4.3 | t4_3_replay_from_event_0_reproduces_identical_progress |
| 02 §11 #8: replay from event 0 reproduces an identical Progress (PocketIC, get_aaa_public + get_leaderboard before/after) | T4.3 | t4_3_new_aaa_reaches_tier2_in_60_tasks_and_replay_matches_incremental |
| 02 §8: list_aaa_credits(aaa, cursor) paginated by discovery_seq (mem 45 credit index), matching the AAA's daily pull | T4.3 | t4_3_list_aaa_credits_pages_by_discovery_seq_per_aaa |
| 02 §6: `public_id = SC-{year}-{seq:06}`, monotonic per discovery | T4.1 | t4_1_create_assigns_monotonic_seq_and_well_formed_public_id |
| 01 §7 / 02 §5.2: flag rate enforced — `max_flag_rate_bp` (≤10% of an AAA's last 100 classifications) rejects the flag, not the submission | T4.1 | t4_1_discovery_flag_creates_record_and_enforces_rolling_rate_limit |
| 02 §5.2: mismatched-image and unknown-category flags are dropped without failing the classification | T4.1 | t4_1_mismatched_image_and_unknown_category_silently_drop_the_flag |
| 02 §11: PocketIC — a flag creates a Discovery with a well-formed public_id, and the 101st flag in a rolling 100-classification window is dropped while submit_classification still succeeds | T4.1 | t4_1_discovery_flag_public_id_and_rolling_rate_limit |
| 02 §6.4: claim cell = floor(ra·cos(dec)·3600/claim_cell_arcsec), floor(dec·3600/claim_cell_arcsec), incl. negative dec and cell boundaries | T4.9 | t4_9_claim_cell_math_matches_spec_including_negative_dec_and_boundaries |
| 02 §6.4 / §3 mem 48+51: open claim in the 3×3 neighbouring cells (same field + category) → Corroboration, repeat flagger → no-op | T4.9 | t4_9_resolve_corroborates_open_claim_in_neighbouring_cells_and_noops_repeat_flaggers |
| 02 §6.4: Rejected claim within claim_reopen_days → ClosedRecentlyRejected; after the window a New claim opens; Confirmed still corroborates | T4.9 | t4_9_rejected_claim_blocks_within_reopen_window_then_reopens |
| 02 §6.4: submit_classification — same cell+category → 1 Discovery + corroborations; other category → separate Discovery; recently rejected → no new Discovery | T4.9 | t4_9_same_cell_flags_collapse_into_one_discovery_with_corroborations |
| 02 §11: PocketIC — concurrent flags → one discovery (3 AAAs, same position+category: 1 New + 2 Corroborates; different category opens its own) | T4.9 | t4_9_concurrent_same_cell_flags_resolve_to_one_discovery |
| 04 §5 #2: Deposit spawn reaches Done, platform shows the AAA registered, and the deposit account is swept to zero | T5.3 | t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure |
| 04 §5 #4: platform can't register mid-saga → op stays Notified with no double charge; resume completes it; a duplicate resume on a Done op is a no-op | T5.3 | t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure |
| 04 §4: top_up(aaa, path) — anyone may gift fuel to any registered AAA, no owner check; min 0.1 ICP enforced before any funds move | T5.4 | t5_4_a_stranger_gifts_fuel_to_a_registered_aaa_via_deposit_and_below_minimum_is_rejected |
| 04 §2: Top-up transfer to (CMC, subaccount = principal_to_subaccount(aaa)) with memo TPUP, then notify_top_up; Deposit path sweeps D(topup, aaa) to zero and the AAA's real cycle balance increases | T5.4 | t5_4_a_stranger_gifts_fuel_to_a_registered_aaa_via_deposit_and_below_minimum_is_rejected |
| 04 §5 #5: auto top-up respects the interval and the monthly cap, and flags a revoked allowance | T5.5 | t5_5_request_auto_topup_succeeds_within_cap_and_interval_then_second_call_rejected_by_interval |
| 04 §5 #5: exceeding cap_30d_e8s is rejected without moving funds | T5.5 | t5_5_request_auto_topup_rejects_when_it_would_exceed_the_rolling_30d_cap |
| 04 §4/§57: rolling 30-day spend window sums only entries within the window, per aaa | T5.5 | t5_5_rolling_window_sums_only_entries_within_30_days |
| 04 §4: request_auto_topup eligibility — disabled mandate, min interval, rolling cap | T5.5 | t5_5_check_eligible_rejects_disabled_mandate |
| 04 §4: InsufficientAllowance/InsufficientFunds fails the op and marks the mandate needs_attention | T5.5 | t5_5_mark_needs_attention_round_trips |
