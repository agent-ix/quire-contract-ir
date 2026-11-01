// SPDX-License-Identifier: AGPL-3.0-or-later
//! The owning STD-616 vectors exercise the canonical public target union.
use ix_trace_rs::trace;
use quire_contract_model::CheckedFairnessTarget;
use serde_json::Value;

#[test]
#[trace("TC-048", "FR-038-AC-98", "FR-349-AC-9")]
fn owner_vectors_have_one_exact_typed_fairness_target() {
    let root = std::env::var_os("QUIRE_SPECIFICATION_DIR").expect("QSpec checkout required");
    let path = std::path::PathBuf::from(root)
        .join("proposals/checked-package-v2/temporal-fairness-target-vectors.json");
    let vectors: Value =
        serde_json::from_slice(&std::fs::read(path).expect("owning STD-616 vectors")).unwrap();
    for group in ["positive", "reader_cases"] {
        for vector in vectors[group].as_array().unwrap() {
            let wire = &vector["member"]["target"];
            let target: CheckedFairnessTarget = serde_json::from_value(wire.clone())
                .unwrap_or_else(|e| panic!("{}: {e}", vector["id"]));
            assert_eq!(
                serde_json::to_value(target).unwrap(),
                *wire,
                "{}",
                vector["id"]
            );
        }
    }
    for vector in vectors["malformed"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<CheckedFairnessTarget>(vector["member"]["target"].clone())
                .is_err(),
            "{}",
            vector["id"]
        );
    }
}
