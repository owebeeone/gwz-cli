# `gwz log` v0 traceability

Date: 2026-08-31

Settle step: S4.1

Normative sources: `GwzLogRequirements.md` and `GwzLogAmbiguityRezo.md`

This is the S4.1 requirements-to-tests sweep required by `GwzLogPlan.md` §2.
Every implemented v0 MUST and SHOULD row has a named checked-in test below.
Names are Rust function names or Python pytest functions; paths are relative to
the named repository. A row may have broader coverage than the representative
acceptance named here. `L-COA-8` and `L-OUT-3` are listed separately because
the adopted requirements explicitly defer them to v2.

## Selection, operands, and history

| Row | Named checked-in acceptance |
| --- | --- |
| L-SEL-1 | gwz-cli `src/tests/g09.rs::operands_and_post_dash_pathspecs_stay_in_distinct_wire_fields`; gwz-py `src/tests/test_cli_log.py::test_log_parser_mirrors_full_s31_surface_and_splits_pathspecs` |
| L-SEL-2 | gwz-core `src/operation/commit_log/tests.rs::l_sel_2_default_selection_includes_root_and_all_active_members` |
| L-SEL-3 | gwz-core `src/operation/commit_log/tests.rs::l_sel_3_tagged_narrows_to_repositories_containing_every_exact_local_tag` |
| L-RNG-1 | gwz-core `src/operation/commit_log/tests.rs::l_rng_1_zero_or_more_revision_operands_use_the_diff_grammar`, `l_rng_1_three_dot_range_uses_symmetric_history`, and `f5_magic_pathspecs_match_native_rev_list_from_root_and_member_subdirectories`; gwz-py `src/tests/test_log_real_workspace.py::test_workspace_and_member_cwd_pathspec_routing_matches_native_git_magic` |
| L-RNG-2 | gwz-core `src/operation/commit_log/tests.rs::l_rng_2_no_operand_histories_start_at_each_repository_head` |
| L-RNG-3 | gwz-core `src/operation/commit_log/tests.rs::l_rng_3_snapshot_resolves_independently_for_each_member` and `l_rng_3_snapshot_to_snapshot_range_resolves_both_endpoints` |
| L-RNG-4 | gwz-core `src/operation/commit_log/tests.rs::l_rng_4_lock_resolves_each_member_to_pin_dot_dot_head_and_degrades_root` and `l_rng_4_missing_and_unborn_lock_rows_degrade_benignly_and_strictly` |
| L-RNG-5 | gwz-core `src/operation/commit_log/tests.rs::l_rng_5_local_read_is_network_free_and_does_not_take_the_mutation_lock` and `f1_path_history_is_offline_and_read_only_for_a_promisor_clone` |
| L-RNG-6 | gwz-core `src/operation/commit_log/tests.rs::l_rng_6_log_internal_dotted_snapshot_ids_work_on_both_range_sides`, `l_rng_6_log_standalone_legacy_dotted_snapshot_ids_remain_accessible`, and `l_rng_6_log_teaches_for_open_legacy_range_endpoints_before_shorter_matches` |

## Coalescing, tolerance, ordering, and filters

| Row | Named checked-in acceptance |
| --- | --- |
| L-COA-1 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_1_real_trailer_siblings_group_across_repositories`, `l_coa_1_only_one_canonical_lowercase_uuid_v7_is_authoritative`, and `l_coa_1_wrong_variant_uuid_v7_looking_claim_is_marker_invalid` |
| L-COA-2 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_2_must_merge_same_message_forall_fan_out`, `l_coa_2_must_not_merge_same_message_with_different_author`, `l_coa_2_must_not_merge_same_author_name_with_different_email`, `l_coa_2_must_not_merge_outside_committer_window`, `l_coa_2_must_not_merge_outside_author_window`, `l_coa_2_must_not_merge_distinct_markers_with_identical_messages`, `l_coa_2_must_not_merge_marked_commit_with_matching_unmarked_commit`, `l_coa_2_must_not_merge_same_repository_twins`, and `l_coa_2_must_not_merge_rebase_restamps_with_old_author_dates` |
| L-COA-3 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_3_no_coalesce_yields_singleton_groups` |
| L-COA-4 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_4_and_6_latest_timestamp_and_all_provenance_values` |
| L-COA-5 | gwz-core `src/operation/commit_log/tests.rs::l_coa_5_l_env_7_partial_filtering_keeps_survivors_and_empty_is_ok` |
| L-COA-6 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_4_and_6_latest_timestamp_and_all_provenance_values`; gwz-cli `src/tests/g11.rs::l_coa_6_all_machine_provenance_tokens_and_marker_invalid_encoding_are_exact` |
| L-COA-7 | gwz-core `src/operation/commit_log/merge_tests.rs::l_coa_7_window_boundary_is_inclusive_and_one_second_beyond_splits` and `l_coa_7_frontier_eligibility_is_exact_and_bounded` |
| L-COA-9 | gwz-core `src/operation/commit_log/coalesce_tests.rs::l_coa_9_mangled_separator_claim_is_marker_invalid` and `l_coa_9_identical_invalid_markers_never_heuristic_coalesce` |
| L-TOL-1 | gwz-core `src/operation/commit_log/tests.rs::l_tol_1_unreadable_member_degrades_without_stopping_other_histories` |
| L-TOL-2 | gwz-core `src/operation/commit_log/tests.rs::l_tol_2_mixed_resolvable_and_unresolvable_members_degrade_independently` and `l_tol_2_default_degradation_is_benign_and_strict_escalates_aggregate` |
| L-TOL-3 | gwz-core `src/operation/commit_log/tests.rs::l_tol_3_unborn_repository_contributes_no_entries_and_a_degradation` |
| L-TOL-4 | gwz-core `src/operation/commit_log/tests.rs::l_tol_4_detached_member_logs_normally_from_the_detached_commit` |
| L-TOL-5 | gwz-core `src/operation/commit_log/tests.rs::l_tol_5_shallow_member_contributes_every_locally_available_commit` |
| L-TOL-6 | gwz-core `src/operation/commit_log/tests.rs::l_tol_6_conf_integrity_mismatch_does_not_gate_history_reads` |
| L-ORD-1 | gwz-core `src/operation/commit_log/tests.rs::l_ord_1_cursor_matches_git_log_default_order` |
| L-ORD-2 | gwz-core `src/operation/commit_log/merge_tests.rs::l_ord_2_equal_time_group_tie_uses_least_sibling_member_then_hash` |
| L-DEP-1 | gwz-core `src/operation/commit_log/merge_tests.rs::l_dep_1_default_is_global_50_and_explicit_windows_lift_only_the_default`; gwz-py `src/tests/test_log_real_workspace.py::test_default_explicit_zero_and_filter_lift_depths_have_exact_hashes` |
| L-FIL-1 | gwz-core `src/operation/commit_log/tests.rs::l_fil_1_and_l_env_5_regexes_filter_raw_message_and_combined_author`, `l_fil_1_first_parent_and_no_merges_match_native_git_premerge`, and `l_fil_1_ancestry_filters_match_native_merge_range_with_and_without_paths` |

## Output, protocol, clients, and lifecycle

| Row | Named checked-in acceptance |
| --- | --- |
| L-OUT-1 | gwz-cli `src/tests/g10.rs::compact_rendering_uses_committer_offset_complete_subject_and_member_sets` and `full_rendering_has_complete_member_table_git_identity_date_and_body` |
| L-OUT-2 | gwz-cli `src/tests/g10.rs::compact_rendering_uses_committer_offset_complete_subject_and_member_sets` and `real_runner_renders_compact_and_full_records_and_releases_each_spool` |
| L-OUT-4 | gwz-cli `src/tests/g10.rs::degradation_summary_is_stderr_safe_and_names_member_reason_and_operand` |
| L-OUT-5 | gwz-cli `src/tests/g10.rs::color_policy_uses_only_the_flag_and_stdout_tty_state` and `full_flag_and_command_help_describe_the_human_modes_and_no_pager` |
| L-JSN-1 | gwz-cli `src/tests/g11.rs::l_jsn_1_json_is_one_ordered_schema_document_with_uniform_members_and_exact_fields` |
| L-JSN-2 | gwz-cli `src/tests/g11.rs::l_jsn_2_degradation_reasons_are_stable_and_optional_context_is_preserved` |
| L-PRO-1 | gwz-core `tests/protocol.rs::log_protocol_wire_values_are_additive` and `log_addition_preserves_the_complete_pre_log_wire_projection`; gwz-py `src/tests/test_log_protocol.py::test_log_addition_preserves_every_pre_existing_wire_shape_and_slot` |
| L-INT-1 | gwz-core `src/operation/commit_log/tests.rs::l_int_1_dispatch_spools_every_record_with_cursor_eof_and_release`, `l_int_1_post_registration_spool_failure_removes_the_output_authority`, and `l_int_1_strict_degradation_sets_final_aggregate_and_streams_once` |
| L-PY-1 | gwz-py `src/tests/test_cli_log.py::test_log_parser_mirrors_full_s31_surface_and_splits_pathspecs` and `test_log_handler_lowers_every_active_cli_field_at_the_real_seam` |
| L-PY-2 | gwz-py `src/tests/test_client_log.py::test_log_lowers_exact_request_and_preserves_absent_tri_states`, `test_log_output_yields_complete_records_then_releases_at_eof`, and `test_log_output_releases_when_consumer_closes_early` |
| L-PY-3 | gwz-py `src/tests/test_cli_log_render.py::test_human_rendering_matches_captured_rust_compact_and_full_bytes` and `test_actual_python_cli_machine_bytes_match_rust_and_release`; real oracle `src/tests/test_log_real_workspace.py::test_compact_full_json_and_jsonl_match_for_real_history` |
| L-EXIT-1 | gwz-core `src/operation/commit_log/tests.rs::l_exit_1_aggregate_tables_every_degradation_kind_and_truth_class`; gwz-cli `src/tests/g09.rs::log_aggregate_exit_mapping_uses_the_shared_response_seam` and `actual_runner_consumes_partial_and_failed_core_aggregates`; gwz-py `src/tests/test_native_log.py::test_native_cli_log_maps_partial_and_strict_failed_to_exit_one` |
| L-PRF-1 | gwz-core `src/operation/commit_log/merge_tests.rs::l_prf_1_streaming_has_a_window_bounded_high_water_and_stops_at_cap` |
| L-PRF-2 | gwz-core `src/operation/commit_log/merge_tests.rs::f6_jobs_values_overlap_to_the_ceiling_and_preserve_complete_events` and `src/operation/commit_log/tests.rs::f4_jobs_one_bounds_real_path_reader_lifetimes` |

## Executable-environment rows

| Row | Named checked-in acceptance |
| --- | --- |
| L-ENV-1 | gwz-core `src/operation/commit_log/merge_tests.rs::l_env_1_orders_absolute_i64_instants_and_preserves_offsets` and `src/operation/commit_log/tests.rs::l_env_1_l_env_12_projection_preserves_seconds_and_marks_lossy_bytes` |
| L-ENV-2 | gwz-core `src/operation/commit_log/merge_tests.rs::l_coa_7_window_boundary_is_inclusive_and_one_second_beyond_splits` and `l_env_2_non_monotone_frontier_escape_repeats_marker_provenance` |
| L-ENV-3 | gwz-core `src/operation/commit_log/merge_tests.rs::l_env_3_cap_force_closes_an_open_group_with_seen_siblings_only` and `f5_satisfied_cap_never_pulls_or_reports_the_successor` |
| L-ENV-4 | gwz-core `src/operation/commit_log/merge_tests.rs::l_coa_7_frontier_eligibility_is_exact_and_bounded` (flat non-monotone tail high-water) and `f6_jobs_values_overlap_to_the_ceiling_and_preserve_complete_events` (byte-complete jobs equality) |
| L-ENV-5 | gwz-core `src/operation/commit_log/tests.rs::l_env_5_6_filters_distinct_raw_author_and_committer_surfaces` and `l_env_5_invalid_regex_refuses_before_workspace_access` |
| L-ENV-6 | gwz-core `src/operation/commit_log/tests.rs::l_env_6_time_grammar_is_exact_local_and_inclusive` and `l_env_6_epoch_extremes_and_dst_gap_overlap_fail_closed` |
| L-ENV-7 | gwz-core `src/operation/commit_log/tests.rs::l_env_7_filtering_precedes_cap_and_order_for_every_jobs_value` |
| L-ENV-8 | gwz-cli `src/tests/g09.rs::cap_lowering_distinguishes_omitted_n_zero_and_no_limit`, `cap_conflict_and_negative_value_are_clap_rejections`, and `all_filter_and_behavior_flags_lower_without_client_semantics` |
| L-ENV-9 | gwz-cli `src/tests/g09.rs::broken_pipe_is_immediate_success_and_releases_unread_log` and `src/tests/g11.rs::l_env_9_machine_broken_pipe_stops_before_any_hidden_spool_read`; gwz-py `src/tests/test_cli_log_render.py::test_machine_record_epipe_stops_before_later_records_and_releases` |
| L-ENV-10 | gwz-cli `src/tests/g10.rs::human_fields_are_lossy_and_c0_sanitized_without_width_truncation`; gwz-py `src/tests/test_cli_log_render.py::test_human_sanitization_member_boundaries_color_and_degradation_match_rust` |
| L-ENV-11 | gwz-cli `src/tests/g10.rs::compact_rendering_uses_committer_offset_complete_subject_and_member_sets` and `zero_entry_run_has_empty_stdout_success_and_benign_degradation_on_stderr` |
| L-ENV-12 | gwz-cli `src/tests/g11.rs::l_env_12_times_use_exact_i64_seconds_and_each_commit_offset`; gwz-py `src/tests/test_cli_log_render.py::test_machine_record_bytes_match_captured_rust_oracles_at_lossy_edge` |
| L-ENV-13 | gwz-cli `src/tests/g11.rs::l_env_13_jsonl_has_exact_header_single_lines_and_stops_at_explicit_eof` and `l_env_13_empty_outputs_and_one_record_bytes_are_canonical` |
| L-ENV-14 | gwz-py `src/tests/test_cli_log_render.py::test_actual_python_cli_machine_bytes_match_rust_and_release` and `src/tests/test_log_real_workspace.py::test_cross_client_oracle_rejects_collapsed_invocations_and_mismatched_bytes` |

## Explicitly deferred rows

| Row | Disposition |
| --- | --- |
| L-COA-8 | v2 deferred by the 2026-08-30 S1.1-B terminal disposition; v0 deliberately retains safe splitting and promises no retry identity. S1.2 records this disposition and the shipped trailer/commit-log authority in gwz-core `dev-docs/GwzCommitMarker.md` at exact docs-only commit `eb3a37c3d657b28c9fb3c85054056aa9192ee353`. |
| L-OUT-3 | v2 deferred grouped rendering; not part of the v0 implementation or acceptance gate. |

No requirement row assigns a release, version bump, dependency-pin change, or
protocol-version bump to S4.1. `GwzLogPlan.md` §1.6 explicitly leaves shipping
to a later operator decision, so the settle preserves all three repositories'
versions and pins.
