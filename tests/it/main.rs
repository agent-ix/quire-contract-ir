// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Single integration-test binary (IR-236) for the root crate.
//!
//! The ordinary `tests/*.rs` files are modules here, so this crate builds and
//! links `quire-contract-model` once for the default integration-test suite
//! instead of once per file.
//! The QSL-dependent selected-population test has an explicit conformance lane.
//! `tests/support/` moved to `tests/it/support/` and
//! is now a single shared module (`crate::support::*`) instead of a
//! `#[path = "support/…"]` copy compiled into each of several binaries.

mod support;

mod application_node_identity;

mod admitted_guard_context;
mod admitted_result_context;
mod admitted_scalar_operands;
mod canonical_v1_quire_canonical;
mod canonicalization;
mod checked_package_evidence_insertion;
mod checked_package_v2_abstraction_relation;
mod checked_package_v2_adr002_members;
mod checked_package_v2_artifact_refs;
mod checked_package_v2_canonical_encoding;
mod checked_package_v2_catalog_words;
mod checked_package_v2_catalogue_differential;
mod checked_package_v2_claim_terminal;
mod checked_package_v2_comparison_operands;
mod checked_package_v2_composite_operands;
mod checked_package_v2_dependency_reference;
mod checked_package_v2_dependency_selections;
mod checked_package_v2_derived_reference;
mod checked_package_v2_dimension_equivalence;
mod checked_package_v2_enum_order;
mod checked_package_v2_envelope;
mod checked_package_v2_flat_wire;
mod checked_package_v2_frame_bodies;
mod checked_package_v2_frame_entries;
mod checked_package_v2_identity_digests;
mod checked_package_v2_ieee_admission;
mod checked_package_v2_lowering;
mod checked_package_v2_model_fields;
mod checked_package_v2_model_members;
mod checked_package_v2_model_root;
mod checked_package_v2_node_map;
mod checked_package_v2_nominal_api;
mod checked_package_v2_operand_families;
mod checked_package_v2_operation_arguments;
mod checked_package_v2_owners;
mod checked_package_v2_parameters;
mod checked_package_v2_protocol_body;
mod checked_package_v2_protocol_reference;
mod checked_package_v2_reader;
mod checked_package_v2_recursive_leaves;
mod checked_package_v2_refinement;
mod checked_package_v2_require_bounds;
mod checked_package_v2_schedulability;
mod checked_package_v2_structural_keys;
mod checked_package_v2_temporal;
mod checked_package_v2_term_data_writer;
mod checked_package_v2_typed_terms;
mod checked_package_v2_union;
mod complete_v1_checked_package;
mod complete_v1_contract_package;
#[cfg(feature = "conformance-cli")]
mod conformance;
#[cfg(feature = "conformance-cli")]
mod conformance_mismatch_independence;
mod cycle_free_model;
mod executable_binding;
mod expression;
mod fr014_recheck;
mod fr019_api;
mod identity;
mod integration;
mod kani_outcome_construction;
mod kani_outcome_tc223;
mod kani_shared;
mod nfr003_invalid_input;
mod output_mapping;
mod public_interface;
mod source_obligation;
mod source_obligation_ir_v2;
mod std001_code;
mod string_edge_gate;

mod nominal_reads;
