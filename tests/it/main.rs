// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Single integration-test binary (IR-236) for the root crate.
//!
//! Every file that used to be its own `tests/*.rs` binary is a module here,
//! so this crate builds and links `quire-contract-model`, `quire-spec-language`
//! (and its own dependents) exactly once for the whole integration-test suite,
//! instead of once per file. `tests/support/` moved to `tests/it/support/` and
//! is now a single shared module (`crate::support::*`) instead of a
//! `#[path = "support/…"]` copy compiled into each of several binaries.
//!
//! `tests/fixtures/model-alias-consumer` and `tests/fixtures/bridge-qsl-consumer`
//! are standalone compile-fixture crates, unaffected by this merge.

mod support;

mod canonicalization;
mod checked_package_v2_dependency_selections;
mod checked_package_v2_frame_bodies;
mod checked_package_v2_lowering;
mod checked_package_v2_model_members;
mod checked_package_v2_qsl_parameters;
mod checked_package_v2_reader;
mod complete_v1_checked_package;
mod complete_v1_contract_package;
mod conformance;
mod cycle_free_model;
mod ecosystem_model;
mod executable_binding;
mod expression;
mod foundation;
mod governance;
mod governance_reconciliation;
mod identity;
mod integration;
mod kani_arithmetic;
mod kani_collections;
mod kani_objects;
mod kani_replay;
mod kani_shared;
mod output_mapping;
mod predicate_projection;
mod predicate_valuation;
mod temporal_projection;
mod toolchain_policy;
