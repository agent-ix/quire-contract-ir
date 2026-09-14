//! Compile fixture proving the bridge and real QSL owner APIs compose acyclically.

use quire_contract_ir::SchemaVersion;
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};

/// Returns identities imported from both sides of the intended bridge boundary.
pub fn selected_contracts() -> (SchemaVersion, &'static str, &'static str) {
    (
        SchemaVersion::V1_0,
        checked_predicate::SCHEMA_SHA256,
        temporal_subject::SCHEMA_SHA256,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tracing: TC-041, FR-028-AC-3, FR-028-AC-5.
    #[test]
    fn tc_041_real_owner_modules_and_bridge_are_importable_together() {
        let (version, predicate_schema, temporal_schema) = selected_contracts();
        assert_eq!((version.major(), version.minor()), (1, 0));
        assert_eq!(
            predicate_schema,
            "459b72a948ddc17be824412b04929fb5795ea033b0bf2aa3f60cf378bc42a531"
        );
        assert_eq!(
            temporal_schema,
            "e72fce683648b18dd7e0f64b438f7dd5d63c48c015b4bcbce9a9f9aef3c096e9"
        );
    }
}
