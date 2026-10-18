//! Focused public API evidence for FR-019's split interface criteria.

use ix_trace_rs::trace;
use quire_contract_model::{
    ContractPackage, DiagnosticCode, ValidationOptions, MAX_SEMANTIC_COLLECTION_ITEMS,
    MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
use serde_json::{json, Value};

#[trace("TC-018", "FR-019-AC-12")]
#[test]
fn tc_018_semantic_and_wire_limits_are_exact_public_u32_constants() {
    let limits: [u32; 4] = [
        MAX_SEMANTIC_NODES,
        MAX_SEMANTIC_DEPTH,
        MAX_SEMANTIC_COLLECTION_ITEMS,
        MAX_WIRE_JSON_DEPTH,
    ];
    assert_eq!(limits, [25_000, 256, 10_000, 576]);
}

#[trace("TC-018", "FR-019-AC-9")]
#[test]
fn tc_018_an_unknown_package_member_refuses_before_validation() {
    let mut package: Value = serde_json::from_str(include_str!(
        "../../corpus/contract-v0.1/inputs/package-constructs.json"
    ))
    .expect("published package fixture is JSON");
    let original = package.to_string();
    assert!(ContractPackage::from_json_str(&original, ValidationOptions::strict()).is_ok());
    package["unexpected"] = json!(1);
    let diagnostics =
        ContractPackage::from_json_str(&package.to_string(), ValidationOptions::strict())
            .expect_err("unknown package member must refuse");
    assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidWireFormat);
}
