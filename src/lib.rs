//! Versioned, implementation-language-independent semantic contract model.

mod binding;
mod canonical;
mod conformance;
mod coverage;
mod expression;
mod identity;
mod limits;
mod wire;

pub use binding::*;
pub use canonical::*;
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
