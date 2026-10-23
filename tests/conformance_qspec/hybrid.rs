// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-966 hybrid nodes read from the owning checkout, without a fixture copy.

use super::*;

fn published_hybrid_package() -> Value {
    let package = read_json(&fixtures_dir().join("positive-all-families.json"))
        .expect("QSpec published package");
    let hybrid = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes")
        .iter()
        .filter(|node| node["node_tag"] == "hybrid")
        .count();
    assert_eq!(hybrid, 3, "QSpec #415 Tank model, mode and transition");
    package
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn qspec_hybrid_family_in_published_package_admits() {
    let package = published_hybrid_package();
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Admitted(admitted) = read else {
        panic!("QSpec hybrid package must admit: {read:?}");
    };
    assert_eq!(
        admitted
            .graph()
            .nodes
            .iter()
            .filter(|node| node.node_tag.as_ref() == "hybrid")
            .count(),
        3
    );
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn qspec_hybrid_schema_defects_refuse_before_identity() {
    for (form, change) in [
        ("hybrid_transition", "declaration"),
        ("hybrid_mode", "occurrence_role"),
        ("hybrid_model", "class"),
    ] {
        let mut package = published_hybrid_package();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("published nodes")
            .iter()
            .position(|node| node["node_tag"] == "hybrid" && node["semantic_form"] == form)
            .expect("published hybrid form");
        let node = &mut package["semantic_graph"]["nodes"][position];
        match change {
            "declaration" => node["declaration"] = json!({"qualified_name":["Invalid"]}),
            "occurrence_role" => node["occurrences"][0]["role"] = json!("claim"),
            "class" => {
                node["body"]["members"][0]["value"]["members"][0]["members"][1]["value"]["value"] =
                    json!("continuous")
            }
            _ => unreachable!("closed mutation list"),
        }
        let read = read_package(&package).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{form} {change} must refuse: {read:?}");
        };
        assert_eq!(refusal.code, Code::MalformedWire, "{form} {change}");
        assert!(
            refusal.path.as_ref().is_some_and(|path| path
                .to_string()
                .starts_with(&format!("/semantic_graph/nodes/{position}"))),
            "{form} {change}: {refusal:?}"
        );
    }
}
