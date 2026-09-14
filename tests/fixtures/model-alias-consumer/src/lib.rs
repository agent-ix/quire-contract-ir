//! Compile fixture proving the historical dependency key remains source-compatible.

/// Returns the current schema version through the historical crate import.
pub const fn current_schema() -> quire_contract_ir::SchemaVersion {
    quire_contract_ir::SchemaVersion::V1_0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tracing: TC-041, FR-028-AC-3.
    #[test]
    fn tc_041_historical_import_resolves_to_the_model_package() {
        assert_eq!(current_schema().major(), 1);
        assert_eq!(current_schema().minor(), 0);
    }
}
