// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec's producer-assigned node ordinal and uniqueness controls.

use super::*;

/// Add the authored siblings around the original derived Integer node. The
/// source-map rows follow each new identity, so the reader decides the graph
/// identity rule rather than failing on missing source correspondence.
fn case_package(base: &Value, case: &Value) -> (Value, Vec<usize>) {
    let mut package = base.clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes");
    let original = nodes
        .iter()
        .position(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("published derived Integer node");
    let template = nodes[original].clone();
    assert_eq!(template["node_id"]["ordinal"], 0);
    assert_eq!(template["semantic_type"], template["node_id"]);
    let source = package["source_map"]
        .as_array()
        .expect("published source map")
        .iter()
        .find(|row| row["node_id"] == template["node_id"])
        .expect("Integer source correspondence")
        .clone();
    let siblings = case["siblings"].as_array().expect("authored siblings");
    assert!(!siblings.is_empty(), "{} has no siblings", case["id"]);
    let before = match case["placement"].as_str().expect("placement") {
        "before" => true,
        "after" => false,
        other => panic!("unknown placement {other}"),
    };
    let mut positions = Vec::with_capacity(siblings.len());
    for (index, sibling) in siblings.iter().enumerate() {
        let ordinal = sibling["ordinal"].as_u64().expect("producer ordinal");
        let mut node = template.clone();
        node["node_id"]["ordinal"] = json!(ordinal);
        node["semantic_type"] = node["node_id"].clone();
        let position = if before {
            original + index
        } else {
            package["semantic_graph"]["nodes"]
                .as_array()
                .expect("graph nodes")
                .len()
        };
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes")
            .insert(position, node.clone());
        let mut correspondence = source.clone();
        correspondence["node_id"] = node["node_id"].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(correspondence);
        positions.push(position);
    }
    (package, positions)
}

/// Trace: TC-233, FR-322-AC-272, FR-322-AC-273.
#[trace("TC-233", "FR-322-AC-272", "FR-322-AC-273")]
#[test]
fn qspec_node_ordinal_uniqueness_cases_read_exactly() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("node-ordinal-vectors.json"))
        .expect("authoritative QSpec node ordinal vectors");
    let base = read_json(&fixtures.join("positive-all-families.json"))
        .expect("authoritative QSpec base package");
    let cases = vectors["uniqueness_cases"]
        .as_array()
        .expect("uniqueness cases");
    assert_eq!(cases.len(), 4, "NO-05..NO-08 must be present");
    for (number, case) in cases.iter().enumerate() {
        let id = format!("NO-{:02}", number + 5);
        assert_eq!(case["id"], id, "authored case order");
        let expected = if number < 2 {
            "admitted"
        } else {
            "invalid_semantic_graph"
        };
        assert_eq!(case["expected"], expected, "{id}: authored outcome");
        let (package, positions) = case_package(&base, case);
        let result = read_package(&package).expect("production read result");
        match case["expected"].as_str().expect("expected result") {
            "admitted" => {
                let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                    panic!("{id}: expected admission, got {result:?}");
                };
                for position in positions {
                    assert_eq!(
                        serde_json::to_value(&admitted.graph().nodes[position].node_id)
                            .expect("retained node id"),
                        package["semantic_graph"]["nodes"][position]["node_id"],
                        "{id}: producer ordinal retained"
                    );
                }
            }
            "invalid_semantic_graph" => {
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id}: expected graph refusal, got {result:?}");
                };
                assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{id}");
                let at = usize::try_from(case["at"].as_u64().expect("sibling index"))
                    .expect("sibling index fits usize");
                let position = positions[at];
                let locus = case["locus"]
                    .as_str()
                    .expect("authored locus")
                    .replace("{i}", &position.to_string());
                assert_eq!(
                    refusal.loci[0].path.as_ref().map(ToString::to_string),
                    Some(locus),
                    "{id}: exact second-node locus"
                );
            }
            other => panic!("{id}: unknown expected result {other}"),
        }
    }
}
