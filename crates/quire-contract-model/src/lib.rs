//! Versioned, implementation-language-independent semantic contract model.
//!
//! This package is the cycle-free substrate shared by Quire owner crates and
//! the `quire-contract-ir` compatibility bridge. It intentionally has no
//! dependency on language, observation, protocol, temporal-logic, or bridge
//! packages.
//!
//! The fault-injection surface is test-only: a default build does not export
//! it (FR-019-AC-4).
//!
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail\nuse quire_contract_model::MappingAllocationPoint;\n```"
)]
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail\nlet _ = quire_contract_model::MappingExecutionControl::fail_allocation_at;\n```"
)]

mod binding;
mod canonical;
mod checked_package;
mod conformance;
mod coverage;
mod expression;
mod identity;
mod limits;
mod output_mapping;
mod wire;

pub use binding::*;
pub use canonical::*;
pub use checked_package::*;
pub use conformance::{
    expected_inventory, hex_digest, run_manifest, ConformanceOperation, FixtureResult,
    FixtureStatus, RunnerError, RunnerErrorCode, ToolIdentity, ValidationOptions,
    CONFORMANCE_BOUNDARIES, CONFORMANCE_PROTOCOL, CONFORMANCE_SCHEMA_ID,
    MAX_CONFORMANCE_FILE_BYTES, MAX_CONFORMANCE_FIXTURES, MAX_CONFORMANCE_TOTAL_BYTES,
    PACKAGE_SCHEMA_ID, PUBLIC_CONSTRUCT_TAGS,
};
pub use coverage::*;
pub use expression::*;
pub use identity::*;
pub use limits::{
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
pub use output_mapping::*;
