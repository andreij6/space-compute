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
| A fresh machine completes 10 local classifications via the operator skill, then (now that T4.2 exists) a second tier-2 AAA under a different owner gets a review assignment and submits a review | T3.7 | docs/demos/T3.7/session.md |
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
| 02 §6.4: a neighbouring-cell candidate beyond claim_cell_arcsec of its exact position is fundamentally unique → opens its own Discovery instead of corroborating | T4.9 | t4_9_distinct_objects_in_the_same_bucket_are_fundamentally_unique |
| 02 §11 #5 / §5.3: same-owner exclusion — a tier-2 AAA is never assigned a discovery from another AAA of the same owner (two AAAs sharing one owner) | T4.2 | t4_2_same_owner_sibling_aaa_is_never_assigned |
| 02 §11 #5 / §5.3: tier < 2 → NotEligible("tier") (unit + PocketIC) | T4.2 | t4_2_tier1_caller_is_not_eligible |
| 02 §11 #5 / §5.3: tier < 2 → NotEligible("tier") (unit + PocketIC) | T4.2 | t4_2_review_assignment_tier_gate_blind_record_and_three_agrees_confirm |
| 02 §5.3: oldest UnderReview first; not discoverer, not corroborator, not previously assigned (mem 34), open+reviews < needed, ≤ 3 open per reviewer, expired assignments free the slot; blind record | T4.2 | t4_2_assignment_is_blind_and_respects_eligibility |
| 02 §5.3: oldest UnderReview first; not discoverer, not corroborator, not previously assigned (mem 34), open+reviews < needed, ≤ 3 open per reviewer, expired assignments free the slot; blind record | T4.2 | t4_2_slots_open_limit_and_expiry |
| 02 §5.3: PocketIC — the wire ReviewAssignment carries no discoverer, public_id, votes or tallies | T4.2 | t4_2_review_assignment_tier_gate_blind_record_and_three_agrees_confirm |
| 02 §5.4: submit_review lease/ownership/expiry checks, rationale 20–1000, idempotent duplicate receipt, weight_bp captured at vote time | T4.2 | t4_2_submit_validates_lease_and_is_idempotent |
| 02 §6.2: evaluate() — Pending, Confirmed (A·3 ≥ 2T), Rejected (2D > T), needed += 2, final A > D tie-break, min weight 100 | T4.2 | t4_2_evaluate_decide_covers_every_branch |
| 02 §6.2: evaluate() — Pending, Confirmed (A·3 ≥ 2T), Rejected (2D > T), needed += 2, final A > D tie-break, min weight 100 | T4.2 | t4_2_split_three_way_extends_needed_reviews |
| 02 §6.2: evaluate() — Pending, Confirmed (A·3 ≥ 2T), Rejected (2D > T), needed += 2, final A > D tie-break, min weight 100 | T4.2 | t4_2_two_disagreeing_reviewers_reject |
| 02 §11 #4 / §6.2: 3 agreeing reviewers → Confirmed; citation with all reviewers, events, progression, credits; open assignments expire (LeaseExpired) | T4.2 | t4_2_three_agreeing_reviewers_confirm_with_citation_credits_and_expiry |
| 02 §11 #4 / §6.2: 3 agreeing reviewers → Confirmed; citation with all reviewers, events, progression, credits; open assignments expire (LeaseExpired) | T4.2 | t4_2_review_assignment_tier_gate_blind_record_and_three_agrees_confirm |
| 02 §6.3: admin_add_honeypots (gold subjects only); honeypot_rate_bp picks a honeypot; review scored immediately vs truth, discovery state untouched | T4.2 | t4_2_honeypots_are_assigned_by_rate_and_scored_immediately |
| 02 §11 #4: 3 equal-weight agreeing reviewers → Confirmed, all 3 credited, XP applied; a fault trapped after the resolution code rolls back review, citation, credits and XP as a whole (fault-injection feature wasm, never in release) | T4.4 | t4_4_injected_fault_after_resolution_rolls_back_the_whole_message |
| 02 §6.2: submit → evaluate → resolve (citation, events, progression, credits, assignment expiry) and the starvation sweep are plain synchronous fns — one message, no await | T4.4 | t4_4_resolution_path_is_synchronous_within_one_message |
| 02 §5.3 R-10: starvation eligibility — no tier-2 AAA outside discoverer, same owner, previously assigned, corroborators, and no open assignment | T4.4 | t4_4_starvation_eligibility_check |
| 02 §5.3 R-10: starved ≥ 3 reviews → weighted majority (ties Rejected); < 3 → flagged awaiting_reviewers (mem 54), cleared when a reviewer appears or it resolves | T4.4 | t4_4_starved_discovery_resolves_by_weighted_majority_or_awaits_reviewers |
| 02 §5.3 R-10: weighted majority ties resolve Rejected | T4.4 | t4_4_weighted_majority_breaks_ties_to_rejected |
| 02 §10: hourly timer applies the starvation rule — 3/5 reviews, no eligible reviewer, resolves Confirmed only after review_starvation_days (PocketIC) | T4.4 | t4_4_starved_discovery_resolves_by_weighted_majority_on_hourly_timer |
| 02 §8.3: citation `text` format string | T4.5 | t4_5_text_matches_spec_format |
| 02 §8.3: Credit/ReviewerCredit populated for discoverer and every reviewer (dissenters included), cycles_contributed and total, subject copied at resolution | T4.5 | t4_5_citation_credits_every_reviewer_with_cycles_and_subject |
| 02 §8.3: citations frozen (no overwrite); RbTree<public_id, sha256(candid)> rebuilt from mem 42 reproduces the root | T4.5 | t4_5_insert_is_frozen_and_rebuild_reproduces_root |
| 02 §8.3: witness + certificate verification rejects tampered citation, stale certified_data, wrong canister, junk CBOR | T4.5 | t4_5_verify_accepts_witness_and_rejects_tampering |
| 02 §7: get_citation unknown public_id → None | T4.5 | t4_5_certified_unknown_public_id_is_none |
| 02 §11 #7: after an upgrade citations are intact and get_citation still verifies (PocketIC) | T4.5 | t4_5_witness_verifies_after_upgrade |
| 07 §5: 120 deterministic honeypots built from gold_v1.json (mix of true/false claims, categories restricted to protocol v1 discovery_categories) | T4.7 | tools/curation/tests/test_t4_7_honeypots.py |
| 02 §6.3: admin_add_honeypots(vec HoneypotSpec) loads gold-only honeypots; a non-gold subject is rejected; honeypots absent from list_discoveries and get_stats().under_review_count (PocketIC) | T4.7 | t4_7_honeypots_load_into_platform |
| 07 §5 acceptance: honeypots uploaded locally via seed-local (idempotent re-run, admin_audit_log guard) | T4.7 | tools/seed-local/seed.sh |
| 04 §5 #2: Deposit spawn reaches Done, platform shows the AAA registered, and the deposit account is swept to zero | T5.3 | t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure |
| 04 §5 #4: platform can't register mid-saga → op stays Notified with no double charge; resume completes it; a duplicate resume on a Done op is a no-op | T5.3 | t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure |
| 04 §4: top_up(aaa, path) — anyone may gift fuel to any registered AAA, no owner check; min 0.1 ICP enforced before any funds move | T5.4 | t5_4_a_stranger_gifts_fuel_to_a_registered_aaa_via_deposit_and_below_minimum_is_rejected |
| 04 §2: Top-up transfer to (CMC, subaccount = principal_to_subaccount(aaa)) with memo TPUP, then notify_top_up; Deposit path sweeps D(topup, aaa) to zero and the AAA's real cycle balance increases | T5.4 | t5_4_a_stranger_gifts_fuel_to_a_registered_aaa_via_deposit_and_below_minimum_is_rejected |
| 04 §5 #5: auto top-up respects the interval and the monthly cap, and flags a revoked allowance | T5.5 | t5_5_request_auto_topup_succeeds_within_cap_and_interval_then_second_call_rejected_by_interval |
| 04 §5 #5: exceeding cap_30d_e8s is rejected without moving funds | T5.5 | t5_5_request_auto_topup_rejects_when_it_would_exceed_the_rolling_30d_cap |
| 04 §4/§57: rolling 30-day spend window sums only entries within the window, per aaa | T5.5 | t5_5_rolling_window_sums_only_entries_within_30_days |
| 04 §4: request_auto_topup eligibility — disabled mandate, min interval, rolling cap | T5.5 | t5_5_check_eligible_rejects_disabled_mandate |
| 04 §4: InsufficientAllowance/InsufficientFunds fails the op and marks the mandate needs_attention | T5.5 | t5_5_mark_needs_attention_round_trips |
| 04 §5 #1: Wallet spawn — approve → spawn_aaa → AAA exists with controllers [owner, platform], is registered, and has cycles ≈ the initial amount | T5.6 | t5_6_wallet_path_spawn_reaches_done_with_controllers_and_cycles |
| 04 §5 #2: Deposit spawn — transfer to the deposit account → spawn_aaa{Deposit} → the same result; the deposit account is empty afterwards | T5.6 | t5_3_spawn_saga_deposit_path_reaches_done_and_resumes_after_platform_failure |
| 04 §5 #3: a third party calling top_up{aaa: X, Wallet{payer: victim}} can only use allowances the victim granted to S(topup, X); an allowance granted for Y is never usable for X | T5.6 | t5_6_topup_wallet_allowance_is_scoped_to_the_correct_aaa |
| 04 §5 #4: kill platform mid-spawn (stopped canister) → the op stays Notified; after restart, resume completes it; no double charge (ledger dedup plus the journal) | T5.6 | t5_6_spawn_survives_platform_stop_and_resumes_without_double_charge |
| 04 §5 #5: auto top-up respects the interval and the monthly cap, and flags a revoked allowance | T5.6 | t5_6_auto_topup_flags_a_revoked_allowance_without_moving_funds |
| 04 §5 #6: a CMC refund path (simulated invalid canister) ends in Refunded | T5.6 | t5_6_cmc_refunds_a_topup_notify_for_a_canister_that_does_not_exist |
| 04 §5 #7: concurrent top_up calls for the same AAA can't both pull from the same allowance past its amount (guard) | T5.6 | t5_6_concurrent_topups_for_the_same_aaa_cannot_both_pull_the_allowance |
| 04 §0: get_features() is public; card=false, btc=false, eth=false, sponsored_spawn=true at launch | T5.16 | t5_16_get_features_reports_card_btc_eth_off_by_default |
| 04 §0: admin_set_features(Features) is admin-only and audit-logged; a disabled feature returns FeatureDisabled | T5.16 | t5_16_admin_set_features_updates_and_is_audit_logged |
| 04 §0b: admin_mint_invites(count, sponsor_cycles, expires_at) stores only sha256(code); plaintext codes are returned once | T5.16 | t5_16_mint_returns_plaintext_codes_stores_only_hash |
| 04 §0b: spawn_aaa{Invite{code}} burns the code and creates the AAA with treasury-sponsored cycles from payments.TREASURY; the code is single-use | T5.16 | t5_16_invite_path_spawn_reaches_done_with_treasury_cycles_and_burns_the_code |
| 04 §0b: one sponsored AAA per owner principal, ever | T5.16 | t5_16_one_sponsored_aaa_per_owner_ever |
| 04 §0b: a daily sponsor budget cap rejects further sponsored spawns once exhausted | T5.16 | t5_16_daily_sponsor_budget_cap_rejects_once_exceeded |
| 04 §0b: unknown or expired invite codes are rejected | T5.16 | t5_16_expired_and_unknown_invite_codes_are_rejected |
| 04 §4: admin_overview reports the payments canister's main and TREASURY ICP balances, matching a direct icrc1_balance_of query after real ledger ops | T5.15 | t5_15_admin_overview_icp_balance_matches_a_real_icrc1_balance_query |
| 04 §4: admin_overview aggregates 24h op counts/spend by kind and failed/stuck op counts | T5.15 | t5_15_stats_since_buckets_count_and_spend_by_kind_within_window |
| 04 §4: admin_overview counts failed ops and ops stuck (non-terminal, idle past the threshold) | T5.15 | t5_15_failed_and_stuck_counts |
| 04 §4: admin_list_ops filters by kind/state (variant-only match) and since, and pages with a next_cursor | T5.15 | t5_15_list_filtered_by_kind_state_and_since_pages_correctly |
| 04 §4: admin_list_ops filters/paginates end-to-end over PocketIC and is admin-gated | T5.15 | t5_15_admin_list_ops_filters_by_kind_and_paginates |
| 04 §4: admin_audit_log pages with a next_cursor (Page_AuditEntry) | T5.15 | t5_1_audit_log_appends_digests_and_pages |
| 04 §6.2: admin_treasury_withdraw is team-only, logged, and moves real ICP from the TREASURY subaccount via icrc1_transfer | T5.15 | t5_15_admin_treasury_withdraw_moves_icp_and_is_admin_gated |
| 04 §4 (status): list_ops_for_aaa/list_ops_for_owner page newest-first (limit <=100), isolated per aaa/owner, mirroring get_op's view type | T5.19 | t5_19_list_ops_for_aaa_and_owner_page_and_isolate_between_two_aaas |
| 06 §2c: headless runner stops without invoking claude when days_of_fuel_estimate is below the threshold | T3.9 | test_t3_9_run_sh_stops_at_fuel_guard |
| 06 §2c: headless runner invokes claude -p when fuel is above the threshold | T3.9 | test_t3_9_run_sh_proceeds_above_threshold |
| 06 §2b: practice.py scores an agent's answers against the practice key per question | T3.9 | test_t3_9_practice_per_question_accuracy |
| 02 §7: list_discoveries newest-first with category/status filters and cursor pagination (limit ≤100) | T4.6 | t4_6_list_filters_by_category_and_status_newest_first_with_cursor |
| 02 §7/§11 #6: UnderReview discovery hidden from non-owners, visible to its owner and honeypots hidden from everyone | T4.6 | t4_6_is_visible_hides_under_review_from_non_owners_and_honeypots_from_everyone |
| 02 §7: DiscoveryCard (list_discoveries) carries the subject image (image_url/image_sha256) for visible cards | T6.19 | t6_19_discovery_card_carries_the_subject_image_when_given_one |
| 02 §7: list_discoveries hides an UnderReview discovery from a stranger and shows it to its owner | T4.6 | t4_6_list_hides_under_review_and_honeypots_from_non_owners_shows_owner_their_own |
| 02 §7: get_stats under_review_count/confirmed_discoveries count real discoveries only, excluding honeypots | T4.6 | t4_6_stats_counts_real_discoveries_and_excludes_honeypots |
| 02 §11 #6: an UnderReview discovery is invisible via get_discovery/list_discoveries to a stranger but visible to the owner and discoverer AAA; it becomes fully public on resolution | T4.6 | t4_6_under_review_hidden_from_strangers_visible_to_owner_and_discoverer |
| 02 §11 #6: honeypots never appear in list_discoveries or get_leaderboard | T4.6 | t4_6_honeypots_never_appear_in_discoveries_leaderboard_or_citations |
| 08 §4 / S7: a timed-out or ambiguous ledger call is Unknown (retried with the same created_at_time), Duplicate is Done, definitive rejects are Failed | T5.7 | t5_7_classify_transfer_never_treats_ambiguous_outcomes_as_failed |
| 08 §4: InsufficientAllowance/InsufficientFunds are funding rejections (mandate needs_attention) | T5.7 | t5_7_classify_transfer_from_flags_allowance_and_funds_as_funding_rejections |
| 04 §3: the pull amount is pinned in the journal before the transfer await and never changes on resume | T5.7 | t5_7_fix_pull_pins_the_first_amount_and_refuses_after_pending |
| 04 §3: the resume timer reaches non-terminal ops beyond the first page | T5.7 | t5_7_resumable_from_reaches_ops_beyond_the_first_page_and_advances_the_watermark |
| 04 §3: ops stored before pull_e8s existed still decode after upgrade | T5.7 | t5_7_ops_stored_before_pull_e8s_existed_still_decode |
| 04 §4: the auto top-up 30-day cap and interval count in-flight ops before any funds move | T5.7 | t5_7_reserve_counts_an_in_flight_auto_topup_against_interval_and_cap |
| 04 §0b: an invite is burned only after the budget and rate checks pass; single use; one per owner | T5.7 | t5_7_sponsor_spawn_does_not_burn_the_code_when_budget_or_rate_refuses |
| 08 §4: a definitively rejected wallet pull ends the op Failed and a later allowance is never pulled | T5.7 | t5_7_a_rejected_wallet_pull_fails_the_op_and_is_never_retried |
| 08 §2 S1: admin_pause blocks new spawn, top_up and request_auto_topup | T5.7 | t5_7_pause_flags_block_new_spawn_topup_and_auto_topup |
| 04 §4: admin_treasury_withdraw retried with the same created_at_time pays once | T5.7 | t5_7_treasury_withdraw_retry_with_the_same_created_at_time_pays_once |
| 08 §4: payments review checklist signed | T5.7 | docs/security/payments-review-T5.7.md |
| 05 §2 sign-in routing: after II sign-in, platform.aaa_by_owner(me) routes no AAA → /spawn, an AAA → /dashboard; owner routes guard to /signin; AAA routes send AAA-less owners to /spawn; admin gate cosmetic (vitest, mocked actor) | T6.1 | src/frontend/src/auth.test.ts |
| 05 §1: canister ids + root key come from the ic_env cookie; local network uses the local II (id.ai.localhost) with the ic_env root key, production uses id.ai with the pinned derivation origin (vitest) | T6.1 | src/frontend/src/ic.test.ts |
| 05 §1/§4: no runtime root-key fetch, no raw HTML rendering, strict CSP (frame-ancestors 'none', IC API + id.ai connect-src) (vitest) | T6.1 | src/frontend/src/security.test.ts |
| T6.1 acceptance "Sign-in works locally": a new user signs in with Internet Identity on the local network and lands on /spawn; signed-out /dashboard → /signin; landing reads get_stats (Playwright, `npm run e2e` after `just deploy-local`) | T6.1 | src/frontend/tests/e2e/signin.spec.ts |
| T6.1 demo screenshot: signed-in owner routed to /spawn | T6.1 | docs/demos/T6.1/signin.png |
| 05 §2 route 1: `/` reads platform.get_stats and list_discoveries{status Confirmed, limit 6} for a recent-discoveries section (vitest is not used for canister calls; proven live in the e2e screenshot below) | T6.3 | docs/demos/T6.3/01_landing.png |
| 05 §2 route 2: `/discoveries` Discovery Museum feed shows resolved-only discoveries with category and Confirmed/Rejected status filters and cursor-based infinite scroll, 60s refresh (Playwright) | T6.3 | src/frontend/tests/e2e/discovery.spec.ts |
| 05 §2 route 3 / §3: `/d/:publicId` reads get_discovery, shows "Citation in progress: n of m reviews" while under review, and an unknown/hidden public id renders the not-found empty state (Playwright) | T6.3 | src/frontend/tests/e2e/discovery.spec.ts |
| 05 §3: the subject image is fetched and sha256-verified against `image_sha256` before display; a hash mismatch blocks rendering and shows "Image unavailable from survey" (vitest) | T6.3 | src/frontend/src/imageHash.test.ts |
| 05 §5 #4 / 02 §8.3: citation certificate verification (BLS certificate + witness against the ic_env root key, mirroring `crates/platform/src/citations.rs` verify) returns verified for a valid certificate and fails closed (unverified) for a tampered citation, a tampered witness, the wrong root key, or a missing certificate (vitest) | T6.3 | src/frontend/src/citation.test.ts |
| T6.3 demo screenshots: landing with live stats, Discovery Museum feed with filters, discovery-not-found empty state | T6.3 | docs/demos/T6.3/02_museum.png |
| 02 §11 #4: 3 agreeing reviewers → Confirmed, citation with all 3 credited, XP applied atomically (rollback on injected fault) | T4.8 | t4_2_review_assignment_tier_gate_blind_record_and_three_agrees_confirm |
| 02 §11 #4: rollback proof | T4.8 | t4_4_injected_fault_after_resolution_rolls_back_the_whole_message |
| 02 §11 #5: tier-1 AAA gets NotEligible; same-owner sibling never assigned its own discovery | T4.8 | t4_2_same_owner_sibling_aaa_is_never_assigned |
| 02 §11 #6: honeypots never in list_discoveries/get_leaderboard/citations; UnderReview hidden from non-owners, visible to owner | T4.8 | t4_6_honeypots_never_appear_in_discoveries_leaderboard_or_citations |
| 02 §11 #6: UnderReview visibility | T4.8 | t4_6_under_review_hidden_from_strangers_visible_to_owner_and_discoverer |
| 02 §11 #7: after an upgrade all state (incl. citations) is intact and get_citation still verifies | T4.8 | t4_5_witness_verifies_after_upgrade |
| 02 §11 #8: replay from event 0 reproduces an identical Progress for every AAA | T4.8 | t4_3_replay_from_event_0_reproduces_identical_progress |
| 02 §11 #9: a new AAA reaches tier 2 in ≤ 60 honest tasks | T4.8 | t4_3_new_aaa_reaches_tier2_in_60_tasks_and_replay_matches_incremental |
| 09 §1: proptest — decide() (evaluate) is monotone: an extra Agree vote never flips a Confirmed decision to Rejected, nor an extra Disagree flip Rejected to Confirmed | T4.8 | t4_8_prop_decide_never_flips_once_resolved |
| 09 §1: proptest — replay(log) from event 0 reproduces identical incremental Progress for every AAA, over random event sequences | T4.8 | t4_8_prop_replay_from_zero_reproduces_incremental_progress_for_every_aaa |
| 09 §1: proptest — every answer path walked through a random v1-shaped protocol tree validates via validate_answers | T4.8 | t4_8_prop_every_walked_protocol_path_validates |
| 04 §1: the wallet-path spender subaccount S(purpose, beneficiary) = sha256("sc-spender" ‖ tag ‖ beneficiary), computed client-side with WebCrypto since no canister query exposes it, matches the Rust reference vector and differs by purpose/beneficiary (vitest) | T6.5 | src/frontend/src/lib/spenderSubaccount.test.ts |
| 05 §3: quote e8s → ICP display formatting, ApiError → human message for every variant, and the op polling state machine (poll every 3s while Pending/Pulled/Notified/Registered, stop on Done, stop and surface Failed/Refunded as terminal) (vitest, mocked actors) | T6.5 | src/frontend/src/lib/paymentOps.test.ts |
| T6.5 acceptance "Deposit path e2e local": sign in, open the /spawn payment panel, read the Deposit-tab account id from `get_deposit_account`, fund it from a local ledger transfer (`icp token transfer` as `sc-user`), confirm, and poll `get_op` to Done, landing on /dashboard (Playwright, `npm run e2e` after `just deploy-local`) | T6.5 | src/frontend/tests/e2e/deposit.spec.ts |
| T6.5 demo screenshot: deposit path resolved to Done on /spawn | T6.5 | docs/demos/T6.5/deposit-done.png |
| 04 §4 pre-check / sc-types name limits: local name validation (length, charset, trim) with human messages, `check_name` Ok/Taken/Invalid mapped to human messages, a deterministic seed → identicon grid preview (same seed twice, differs across seeds, horizontally symmetric), the spawn step machine (name→payment→progress→done, guarded transitions, RESUME bypass), resume-to-progress/done from a fetched op incl. a failed op, and spawn-op-id localStorage round-trip incl. a corrupted value (vitest) | T6.6 | src/frontend/src/lib/spawnFlow.test.ts |
| T6.6 acceptance "Spawn e2e local" (Deposit path): name step gated by live `check_name`, avatar seed picker, Continue → PaymentPanel Deposit tab, fund the deposit account (`icp token transfer` as `sc-user`), confirm, poll `get_op` to Done (Playwright, `npm run e2e` after `just deploy-local`) | T6.6 | src/frontend/tests/e2e/deposit.spec.ts |
| T6.6 acceptance "Spawn e2e local" (sponsored invite path, 04 §0b): mint an invite (`admin_mint_invites` as `sc-deployer`), fund the payments TREASURY account (`get_treasury_account` + `icp token transfer` as `sc-user`, T5.16), redeem the code from the spawn wizard's Invite tab, poll to Done (Playwright, `npm run e2e` after `just deploy-local`) | T6.6 | src/frontend/tests/e2e/spawn-invite.spec.ts |
| T6.6 demo screenshots: spawn step 1 (name + avatar seed picker), sponsored invite path resolved to Done | T6.6 | docs/demos/T6.6/spawn-invite-done.png |
| 02 §8.2: tier label decodes to every spec tier name, falling back to a generic label for an unknown tier (vitest) | T6.4 | src/frontend/src/progression.test.ts |
| 02 §8.2: badge bitmask decodes every spec badge bit individually, in combination, empty, and fully set (vitest) | T6.4 | src/frontend/src/progression.test.ts |
| 05 §2 route 5: `/leaderboard` cursor paging (inverted_xp, aaa, rank) accumulates rows across pages without duplicates (vitest) | T6.4 | src/frontend/src/paging.test.ts |
| 05 §2 route 4: `/aaa/:id` credited-discoveries cursor paging accumulates without duplicates across pages (vitest) | T6.4 | src/frontend/src/paging.test.ts |
| T6.4 acceptance "Tier/badges render": `/leaderboard` renders (empty state acceptable pre-Observer-tier) and an unknown AAA id on `/aaa/:id` renders the not-found empty state (Playwright, `npm run e2e` after `just deploy-local`) | T6.4 | src/frontend/tests/e2e/aaa-leaderboard.spec.ts |
| T6.4 demo screenshots: leaderboard empty state, AAA profile not-found empty state | T6.4 | docs/demos/T6.4/02_profile_not_found.png |
| 05 §2 route 7 / §3 fuel gauge (SP-3): `/dashboard` reads aaa.status, and when it rejects as frozen/out-of-cycles falls back to platform.get_aaa_public instead of erroring, surfacing a top-up CTA; ApiError Err results map to human messages; auto top-up mandate none/needs_attention/ok states (vitest, mocked actors) | T6.7 | src/frontend/src/lib/dashboard.test.ts |
| T6.7 acceptance "Frozen fallback works": a freshly spawned AAA's `/dashboard` renders live fuel days/cycles/tier, recent activity, credits and mandate state (Playwright, `npm run e2e` after `just deploy-local`) | T6.7 | src/frontend/tests/e2e/dashboard.spec.ts |
| T6.7 demo screenshot: /dashboard with live fuel, activity and credits for a freshly spawned AAA | T6.7 | docs/demos/T6.7/dashboard-live.png |
| 05 §2 row 8 / §3 connect: principal input validation (rejects empty, malformed and anonymous principals), add/remove operator flows mapping ApiError to human messages, a >=300ms minimum pending state on every update call, "Connected" computed as an operator with `last_used_at` under 24h, and the exact setup commands (`icp identity new`, `icp identity principal`, the first-contact `whoami` command with `-e local`/`-n ic`) (vitest, mocked actors) | T6.8 | src/frontend/src/lib/connect.test.ts |
| 05 §2 row 9 / §3: records list paging with `aaa.list_records` falling back to `platform.list_aaa_activity` on a rejected or Err call, and record detail answers rendered against `platform.get_protocol` question/answer labels (vitest, mocked actors) | T6.8 | src/frontend/src/lib/records.test.ts |
| T6.8 acceptance "Operator add/revoke": sign in and spawn an AAA, add a freshly created `icp identity` principal as an operator on `/connect`, see it listed, revoke it, see it gone; `/records` renders (empty state acceptable) (Playwright, `npm run e2e` after `just deploy-local`) | T6.8 | src/frontend/tests/e2e/connect-records.spec.ts |
| T6.8 demo screenshots: /connect with an operator added, /records empty state | T6.8 | docs/demos/T6.8/connect-operator-added.png |
| 06 §2b / 07 §5b: practice_v1 is 200 GZ-CANDELS-labelled subjects, deterministic and disjoint from both selection_v1 and gold_v1 | T7.10 | test_t7_10_choose_is_deterministic_and_disjoint_from_selection_and_gold |
| 06 §2b / 07 §5b: the committed practice_v1 set has exactly 200 unique subjects, none overlapping the task pool or gold | T7.10 | test_t7_10_committed_practice_set_never_overlaps_selection_or_gold |
| 06 §2b: practice_v1 answers are valid protocol v1 questions/answers and `agent-kit/practice.py` scores the key against itself at 100% | T7.10 | test_t7_10_agent_kit_practice_scores_the_committed_answer_key_perfectly |
| 07 §5b: the v0 open-data release (discoveries + citations, manifest/selection/gold metadata) builds byte-for-byte identically on a second run | T7.10 | test_t7_10_release_build_is_reproducible_byte_for_byte |
| 07 §5b: the release's CHECKSUMS.json matches the sha256 of every file it ships | T7.10 | test_t7_10_release_checksums_file_matches_returned_hashes |
| 02 §9: admin_overview/admin_list_aaas/admin_list_discoveries are admin-gated, paged (limit ≤100, cursor), and admin_overview reflects live suspensions | T4.10 | t4_10_admin_read_apis_are_paged_admin_only_and_reflect_state |
| 02 §9: every admin_* update method in platform.did writes an AuditEntry (audit on every mutation) | T4.10 | t4_10_every_admin_mutation_writes_an_audit_entry |
| 02 §9: admin_honeypot_stats per-reviewer accuracy and admin_list_discoveries surface honeypots | T4.10 | t4_10_admin_read_apis_are_paged_admin_only_and_reflect_state |
| 02 §9: admin_rename_aaa renames the live AaaRecord/AaaPublic but a confirmed citation keeps discoverer_name_at_time frozen | T4.11 | t4_11_renamed_aaa_keeps_historical_citation_name |
| 02 §9: name blocklist (case-fold + basic leetspeak) rejects registration and update_aaa_profile, but admin_rename_aaa can override it for moderation | T4.11 | t4_11_name_blocklist_blocks_registration_and_profile_update_but_not_admin_rename |
| 02 §9: admin_set_house marks a team AAA "house"; it keeps progressing but is excluded from get_leaderboard | T4.11 | t4_11_house_aaas_are_labeled_and_excluded_from_the_leaderboard |
| 08 §7 acceptance "Hash reproducible on 2 machines": the AAA wasm builds to the identical module_sha256/gz_sha256 from two independent absolute directories with different env (not run by `just verify` — a release build is too slow for the fast gate; run `just aaa-reproducible-check` before publishing a version) | T7.4 | scripts/aaa-reproducible-check.sh |
| 08 §7 / 03 §7: manual upgrade path for a self-managed AAA — verify module_sha256 against the platform's approved WasmMeta before `icp canister install <aaa> --mode upgrade` | T7.4 | docs/ops/aaa-manual-upgrade.md |
| 05 §2 row 10 / 04 §4: `/fuel` reads `aaa.status` with the T6.7 frozen fallback via `platform.get_aaa_public`, sizes a one-time top-up (`PaymentPanel`, purpose=topup) from `payments.get_quote_topup(cycles)` via the cached XDR rate, and gates the amount at the 0.1 ICP minimum (vitest, mocked actors) | T6.9 | src/frontend/src/lib/paymentOps.test.ts |
| 05 §3: every auto top-up mandate UI state (none / enabled / disabled / needs_attention "allowance revoked or insufficient" / cap_reached) is computed from `payments.get_mandate` and rendered with distinct copy (vitest, mocked actor data) | T6.9 | src/frontend/src/lib/dashboard.test.ts |
| 04 §4: the frontend wallet-approval helper approves max(topup_e8s, monthly cap) × 12 to spender `(payments, S(auto, aaa))`, expires in 365 days (vitest) | T6.9 | src/frontend/src/lib/dashboard.test.ts |
| T6.9 acceptance "Mandate states render": sign in, spawn an AAA, `/fuel` shows the live fuel gauge and the none-mandate state, the auto top-up setup form, and a Deposit one-time top-up reaches Done; the mandate wallet-approve step is exercised with OISY in staging, not headless e2e (05 §5 #3) (Playwright, `npm run e2e` after `just deploy-local`) | T6.9 | src/frontend/tests/e2e/fuel.spec.ts |
| T6.9 demo screenshots: `/fuel` with no mandate configured, a completed one-time top-up, and the auto top-up setup form | T6.9 | docs/demos/T6.9/02_topup_done.png |
| 06 §5 #2: planted prompt-injection rationales (5 payload styles: override, fake admin notice, JSON instruction, operator change/ICP transfer, hidden after benign text) on T4.7 false-claim honeypots; headless reference agents (`claude -p --model haiku`) following the skill do not comply in ≥95% of 20 trials (results in docs/demos/T7.7/results.md) | T7.7 | tools/redteam/prompt_injection.py |
| 09 §1 Upgrade: platform installed at a pinned vN-1 baseline commit (`scripts/upgrade-baseline.sh`), populated through the real API (admins, params, subjects, protocol, 4 registered AAAs, classifications, a Confirmed discovery with 3 reviews), upgraded to the current vN wasm — every read is byte-for-byte intact, the certified citation still verifies, and the hourly reseed timer resumes. Also the failing-first proof for a real upgrade bug found and fixed: `AaaRecord.is_house` (added non-optional in T4.11) trapped on decode from stable memory for any AAA registered before that field existed; fixed by making it `opt bool` (`crates/platform/src/registry.rs`, `progression.rs`, `platform.did`) | T7.1 | t7_1_platform_v_n_minus_1_state_survives_upgrade_to_v_n |
| 09 §1 Upgrade: payments installed at the pinned vN-1 baseline, populated (config, 3 settled journal ops via Deposit/Invite/mandate paths, a real stalled spawn saga), upgraded to vN — config/mandate/ops are byte-for-byte intact, a spent invite and an armed mandate guard both still refuse, and the resume-sweep timer resumes and finishes the stalled saga without a manual `resume()` | T7.1 | t7_1_payments_v_n_minus_1_state_survives_upgrade_to_v_n |
| 09 §1 Upgrade: treasury installed at the pinned vN-1 baseline, populated (watch list, funded history, an executed withdraw, a pending config-change proposal), upgraded to vN — history/watch-list/admins are byte-for-byte intact, the executed withdraw cannot replay, the pending proposal still approves, and the 6h keeper timer resumes | T7.1 | t7_1_treasury_v_n_minus_1_state_survives_upgrade_to_v_n |
| 09 §1 Upgrade: aaa installed at the pinned vN-1 baseline (installed by platform's own registration flow, the real spawn path), populated (2 operators, 2 records incl. one flagged discovery, an auto-topup config), upgraded to vN — records/operators/roles are byte-for-byte intact, and both the 6h burn timer and the 24h heartbeat timer resume | T7.1 | t7_1_aaa_v_n_minus_1_state_survives_upgrade_to_v_n |
| 09 §1 Load: PocketIC harness registers real AAA canisters (platform install + admin-approved wasm + `register_aaa` via the payments principal, the same lightest real path as T2.2/T3.2) against >=5000 real seeded subjects, then drives `get_task`/`submit_classification` (fees attached) through the real AAA relay with a ~2% discovery-flag rate and periodic `get_review_assignment`/`submit_review` once tier-2 is reached; measures cycles burned per call (AAA cost and platform's own execution cost, via cycle-balance deltas), stable-memory growth and wall time at 200x100 and a faster repeatable 20x20 (`just load-test`, `#[ignore]`, not part of `just verify`) — results in `docs/perf/load-test-T7.2.md` / `.json` | T7.2 | t7_2_load_200x100 |
| 02 §11 #1: a registered AAA with the fee gets a task, submits it, receives a receipt; resubmitting returns duplicate = true, state unchanged | T2.7 | t2_7_registered_aaa_task_receipt_and_idempotent_duplicate |
| 02 §11 #2: a non-AAA caller gets NotRegistered; an insufficient fee gets InsufficientFee | T2.3 | t2_3_never_same_subject_twice_and_pool_dispatch |
| 02 §11 #3: the 5th classification retires a subject, never reissued; an AAA never sees the same subject twice | T2.7 | t2_7_fifth_classification_retires_and_seen_set_never_reissues |
| 05 §5 #1: all 10 routes render against a local deployment seeded by tools/seed-local, with loading, empty and error states | T6.3 | src/frontend/tests/e2e/discovery.spec.ts |
| 05 §5 #1: all 10 routes render against a local deployment seeded by tools/seed-local, with loading, empty and error states | T6.7 | src/frontend/tests/e2e/dashboard.spec.ts |
| 05 §5 #1: all 10 routes render against a local deployment seeded by tools/seed-local, with loading, empty and error states | T6.8 | src/frontend/tests/e2e/connect-records.spec.ts |
| 05 §5 #1: all 10 routes render against a local deployment seeded by tools/seed-local, with loading, empty and error states | T6.9 | src/frontend/tests/e2e/fuel.spec.ts |
| 05 §5 #2: signed-out visitors can browse routes 1-5; owner routes redirect to sign-in | T6.1 | src/frontend/src/auth.test.ts |
| 05 §5 #0 / §2b: `/admin/*` gate — a non-admin (or signed-out) principal gets the 404 component and an admin reaches the console; `admin_list_admins` across platform+payments decides it (vitest, mocked actors) | T6.12 | src/frontend/src/lib/admin.test.ts |
| 05 §5 #0: a fresh II identity gets 404 on `/admin`; after `admin_add_admin`, the console renders, a pause/unpause mutation requires typed confirmation, and the mutation appears in the merged audit view (Playwright, local deploy) | T6.12 | src/frontend/tests/e2e/admin.spec.ts |
| 05 §2b: the 11 `/admin/*` screens (overview, aaas, discoveries, data, payments, releases, settings, invites, treasury, moderation, audit) read live canister data via mocked-actor unit tests covering params diff/round-trip, Unauthorized→human-message rendering, audit-log merge, the last-admin protection, and the sponsor_cycles >=1T guard (vitest) | T6.12 | src/frontend/src/lib/admin.test.ts |
| 05 §2 row 1b: `/about`, `/terms`, `/privacy`, `/credits` render how-it-works, ToS, privacy (public on-chain data, no analytics — Firebase removed), and JWST/DJA/survey credits; `/about` polls live `treasury.status` every 60 s (Playwright, local deploy) | T6.14 | src/frontend/tests/e2e/legal-practice-pages.spec.ts |
| 05 §2 row 1c: `/practice` links the committed practice_v1 set (data/curation/v1/practice_*, target/bucket/practice_v1) and gives offline self-eval instructions for `agent-kit/practice.py` (Playwright, local deploy) | T6.14 | src/frontend/tests/e2e/legal-practice-pages.spec.ts |
| 05 §2b: `/admin/aaas` gains a `set house`/`unset house` action (`admin_set_house`) alongside the existing suspend/unsuspend and rename-with-reason moderation actions | T6.14 | src/frontend/src/pages/AdminAAAsPage.tsx |
| 05 §5 #0: acceptance "E2E: invite spawn; non-admin blocked" — sponsored-invite spawn (reuse of T6.6's flow) plus a non-admin denied 404 on every `/admin/*` route and `Unauthorized` from a direct, UI-bypassing actor call to an admin mutation on both platform and payments (Playwright, local deploy) | T6.14 | src/frontend/tests/e2e/admin.spec.ts |
| 05 §5 #5: Lighthouse accessibility >= 90 on Landing (98), Discovery detail (98) and Dashboard (98); reports in docs/demos/T6.10 | T6.10 | scripts/lighthouse-a11y.sh |
| 05 §3 accessibility / 05 §4: keyboard-navigable, visible focus (`:focus-visible`), WCAG AA contrast, no `dangerouslySetInnerHTML` (eslint `no-restricted-syntax` + verify-local grep gate) — 13-route axe smoke (signed-out, signed-in owner, admin) asserts zero console errors and zero serious/critical axe violations per route | T6.10 | src/frontend/tests/e2e/a11y-smoke.spec.ts |
| 05 §4: initial JS bundle <= 350 KiB gz (191,831 B measured), routes code-split via `React.lazy` | T6.10 | scripts/check-bundle-size.sh |
| 05 §2b: `/admin/invites` lists minted/used/expired invites (code_hash hex, sponsor_cycles, minted_at, expires_at, used, used_by — never the plaintext code), paginated, admin-only | T6.10 | crates/integration-tests/tests/t5_16_feature_flags_invites.rs |
| 07 §7 #1: 5,000 subjects rendered, hashed and uploaded; the manifest verifies (random 1% re-download hashes match) | T1.7 | test_t1_7_committed_manifest_and_qa_meet_acceptance |
| 07 §7 #2: >= 2,000 gold subjects (or the documented fallback); 120 honeypots | T1.5 | test_t1_5_committed_v1_selection_meets_acceptance |
| 07 §7 #3: a human spot-check of 40 random dossiers finds no wrong target, wrong WCS or missing band | T1.7 | test_t1_7_committed_manifest_and_qa_meet_acceptance |
| 07 §7 #4: the report is committed to docs/data/curation-report-v1.md | T1.7 | docs/data/curation-report-v1.md |
| 08 §2 S2: Sybil farms (many AAAs to self-confirm) — same-owner AAA never assigned its sibling's discovery | T4.2 | t4_2_same_owner_sibling_aaa_is_never_assigned |
| 08 §2 S3: random or lazy classifications — hidden gold scored immediately | T4.2 | t4_2_honeypots_are_assigned_by_rate_and_scored_immediately |
| 08 §2 S4: rubber-stamp or always-disagree reviewers — honeypots carry both truths | T4.7 | tools/curation/tests/test_t4_7_honeypots.py |
| 08 §2 S5: prompt injection via rationales — red-team honeypot trials | T7.7 | tools/redteam/prompt_injection.py |
| 08 §2 S6: allowance hijack — a third party's top_up can only pull allowances the victim granted to that AAA's spender subaccount | T5.6 | t5_6_topup_wallet_allowance_is_scoped_to_the_correct_aaa |
| 08 §2 S8: modified AAA wasm — provenance re-verification suspends on a num_changes mismatch | T2.9 | t2_9_verify_never_lifts_admin_suspension_and_checks_num_changes |
| 08 §2 S9: XSS via names or rationales — no raw HTML rendering, strict CSP | T6.1 | src/frontend/src/security.test.ts |
| 08 §2 S10: unbounded storage growth — input limits enforced at the boundary | T1.2 | t1_2_input_limits_match_the_security_spec |
| 08 §2 S11: admin key compromise — every admin mutation writes an audit entry | T4.10 | t4_10_every_admin_mutation_writes_an_audit_entry |
| 08 §2 S13: frozen AAA losing data — frozen AAA rejects queries until topped up | SP-3 | sp_3_frozen_aaa_rejects_queries_until_topped_up_via_the_cmc |
| 08 §2 S15: platform is a co-controller of every AAA — controllers restricted to {owner, platform}, real total_num_changes recorded | T2.9 | t2_9_register_requires_owner_and_platform_and_records_real_changes |
| 08 §2 S16: XP farming onto the leaderboard — the leaderboard admits tier >= 2 only | T7.9 | t7_9_s16_leaderboard_excludes_aaa_below_tier2_then_admits_it_at_tier2 |
| 08 §2 S21: someone other than the owner or the owner's agent submitting through an AAA — foreign/expired/unsynced submitter rejected | T2.8 | t2_8_foreign_expired_unsynced_submitter_rejected |
| 08 §2 S22: tampered AAA code vouching for arbitrary callers — num_changes mismatch triggers re-verification | T2.9 | t2_9_verify_never_lifts_admin_suspension_and_checks_num_changes |
| 08 §2 S23: stolen operator key — expired/unsynced operator rejected immediately | T2.8 | t2_8_foreign_expired_unsynced_submitter_rejected |
| 08 §2 S24: claim sniping — concurrent same-cell flags resolve to one discovery, corroborators never become discoverers | T4.9 | t4_9_concurrent_same_cell_flags_resolve_to_one_discovery |

## Waivers

| Item | Reason |
|---|---|
| 06§5#1 | deferred: staging-only acceptance run (no staging environment in local-first `just verify`; the local analog is proven by T3.7's 10 local classifications) |
| 08§2#S12 | deferred: architectural property (bounded-wait calls only, per 01 §6 and the canister-security skill checklist), not independently runtime-testable in PocketIC |
| 08§2#S14 | deferred: architectural fact — no secrets are stored in canister state, nothing to assert against |
| 08§2#S17 | deferred: D6-D12 (non-ICP fuel packs / Stripe relay not implemented) |
| 08§2#S18 | deferred: D6-D12 (XRC-based non-ICP pricing not implemented) |
| 08§2#S19 | deferred: D6-D12 (card/Stripe not implemented) |
| 08§2#S20 | deferred: T7.5 / T8.8 (regulatory compliance review, recommended before production) |
