use ix_trace_rs::trace;
use quire_contract_model::SchemaVersion;

/// Trace: TC-015
#[trace("TC-015")]
#[test]
fn tc_015_public_api_exposes_the_v1_wire_schema() {
    assert_eq!(SchemaVersion::V1_1.major(), 1);
    assert_eq!(SchemaVersion::V1_1.minor(), 1);
}
