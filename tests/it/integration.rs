use quire_contract_ir::SchemaVersion;

/// Tracing: TC-015
/// TC-015.
/// FR-011-AC-1.
#[test]
fn tc_015_public_api_exposes_the_v1_wire_schema() {
    assert_eq!(SchemaVersion::V1_0.major(), 1);
    assert_eq!(SchemaVersion::V1_0.minor(), 0);
}
