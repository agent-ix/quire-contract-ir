//! Compile fixture proving the bridge and real QSL owner APIs compose acyclically.

use quire_contract_ir::SchemaVersion;
use quire_spec_language::protocol_artifact::{checked_predicate, temporal_subject};

/// Returns items imported from both sides of the intended bridge boundary.
pub fn selected_contracts() -> (SchemaVersion, &'static str, &'static str) {
    (
        SchemaVersion::V1_0,
        std::any::type_name::<checked_predicate::ValidatedCheckedPredicate>(),
        std::any::type_name::<temporal_subject::ValidatedTemporalSubject>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tracing: TC-041, FR-028-AC-3, FR-028-AC-5.
    #[test]
    fn tc_041_real_owner_modules_and_bridge_are_importable_together() {
        let (version, _, _) = selected_contracts();
        assert_eq!((version.major(), version.minor()), (1, 0));
    }
}
