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
    doc = "```compile_fail,E0432\nuse quire_contract_model::MappingAllocationPoint;\n```"
)]
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail,E0599\nlet _ = quire_contract_model::MappingExecutionControl::fail_allocation_at;\n```"
)]
//! The artifact references have closed member sets (FR-019-AC-6, FR-038
//! "Artifact references"): a definition reference is `{authority, identity}`, a
//! source reference `{authority, identity, digest_domain, digest}` and a locator
//! `{authority, identity, domain}`. Each builds with exactly its members,
//!
//! ```
//! use quire_contract_model::{CheckedArtifactLocator, CheckedArtifactRef, CheckedSourceRef};
//! let _ = CheckedArtifactRef { authority: "a".into(), identity: "i".into() };
//! let _ = CheckedSourceRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     digest_domain: "d".into(),
//!     digest: "x".into(),
//! };
//! let _ = CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     domain: "d".into(),
//! };
//! ```
//!
//! and fails to build with a further member, such as `revision`. Stable rustc
//! does not check the error code a `compile_fail` doctest names, and a type
//! change to an added `revision` field would fail these probes for another
//! reason, so the real oracles for a member added to a type are the positive
//! doctest above, whose literal must name every member, and
//! `tc_058_the_artifact_reference_member_sets_are_exact` in the root crate's
//! tests; these probes pin that an absent member is refused:
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedArtifactRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedSourceRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     digest_domain: "d".into(),
//!     digest: "x".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     domain: "d".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! and no revision type exists:
//!
//! ```compile_fail,E0432
//! use quire_contract_model::CheckedRevision;
//! ```

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
    expected_inventory, run_corpus, ConformanceOperation, FixtureResult, FixtureStatus,
    RunnerError, RunnerErrorCode, ToolIdentity, ValidationOptions, CONFORMANCE_BOUNDARIES,
    CONFORMANCE_PROTOCOL, CONFORMANCE_SCHEMA_ID, MAX_CONFORMANCE_FILE_BYTES,
    MAX_CONFORMANCE_FIXTURES, MAX_CONFORMANCE_TOTAL_BYTES, PACKAGE_SCHEMA_ID,
    PUBLIC_CONSTRUCT_TAGS,
};
pub use coverage::*;
pub use expression::*;
pub use identity::*;
pub use limits::{
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
pub use output_mapping::*;
