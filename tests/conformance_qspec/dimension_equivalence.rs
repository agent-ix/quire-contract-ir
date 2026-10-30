// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec XP-01..XP-10 dimension-equivalence vectors through the strict reader.

use super::{fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("dimension-equivalence-vectors.json"))
        .expect("QSpec dimension equivalence vectors")
}

fn id_for(row: &Value) -> Value {
    json!({
        "package": row["package"],
        "path": row["path"],
        "ordinal": row["ordinal"],
    })
}

fn source(identity: &str) -> Value {
    json!({"authority":"agent-ix","identity":identity})
}

fn dimension_node(name: &str, row: &Value, all: &Value) -> Value {
    let id = id_for(row);
    let form = row["form"].as_str().expect("published form");
    let mut terms = row["terms"]
        .as_array()
        .map(|terms| {
            terms
                .iter()
                .map(|term| {
                    let named = term["node"].as_str().expect("term name");
                    json!({
                        "dimension_node_id": id_for(&all[named]),
                        "exponent": term["exponent"],
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    terms.sort_by(|left, right| {
        let left: CheckedNodeId =
            serde_json::from_value(left["dimension_node_id"].clone()).expect("vector node id");
        let right: CheckedNodeId =
            serde_json::from_value(right["dimension_node_id"].clone()).expect("vector node id");
        left.cmp(&right)
    });
    let dependencies = if form == "unit" {
        vec![id_for(&all["rpc_time"])]
    } else {
        terms
            .iter()
            .map(|term| term["dimension_node_id"].clone())
            .collect()
    };
    let body = if form == "unit" {
        json!({
            "term":"unit",
            "dimension_node_id": id_for(&all["rpc_time"]),
            "target_unit_node_id": null,
            "scale":{"numerator":"1","denominator":"1"},
            "offset":{"numerator":"0","denominator":"1"},
        })
    } else {
        json!({"term":"dimension","terms":terms})
    };
    let semantic_type = if form == "unit" {
        id_for(&all["rpc_time"])
    } else {
        id.clone()
    };
    json!({
        "node_id": id,
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type",
        "semantic_form": form,
        "semantic_type": semantic_type,
        "dependencies": dependencies,
        "occurrences": [{"role":"declaration","ordinal":0}],
        "declaration": {"qualified_name": row["path"]},
        "owner": {"kind":"source","authority":"agent-ix","identity":name},
        "body": body,
    })
}

fn equivalence_node(package: &str, case: &Value, index: usize, all: &Value) -> Value {
    let pair = case["pair"].as_array().expect("published pair");
    let first = id_for(&all[pair[0].as_str().expect("first name")]);
    let second = id_for(&all[pair[1].as_str().expect("second name")]);
    let ordinal = case["ordinal"].as_u64().unwrap_or(0);
    let id = json!({
        "package": package,
        "path": ["scalar_type", "dimension_equivalence", first, second],
        "ordinal": ordinal,
    });
    json!({
        "node_id": id,
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type",
        "semantic_form": "dimension_equivalence",
        "semantic_type": id,
        "dependencies": [first, second],
        "occurrences": [{"role":"anchor","ordinal":index}],
        "owner": {"kind":"source","authority":"agent-ix","identity":package},
        "body": {
            "term":"dimension_equivalence",
            "dimension_node_ids":[first,second],
            "root_scale":case["root_scale"],
        },
    })
}

fn package(name: &str, nodes: &[Value], imports: &[&str]) -> Value {
    let mut wire = read_json(&fixtures_dir().join("positive-nominal-identities.json"))
        .expect("published nominal envelope");
    wire["name"] = json!(name);
    wire["semantic_graph"]["nodes"] = json!(nodes);
    wire["lock"]["definition_selections"] = json!([]);
    wire["lock"]["dependency_selections"] = json!(imports
        .iter()
        .map(|identity| json!({"identity":identity}))
        .collect::<Vec<_>>());
    let mut sources = BTreeMap::new();
    sources.insert(name.to_owned(), source(name));
    for node in nodes {
        if let Some(identity) = node["owner"]["identity"].as_str() {
            sources.insert(identity.to_owned(), source(identity));
        }
    }
    wire["lock"]["sources"] = json!(sources.into_values().collect::<Vec<_>>());
    wire["source_map"] = json!(nodes
        .iter()
        .enumerate()
        .map(|(position, node)| {
            let owner = node["owner"]["identity"].as_str().expect("source owner");
            json!({
                "node_id":node["node_id"],
                "role":node["occurrences"][0]["role"],
                "ordinal":node["occurrences"][0]["ordinal"],
                "regions":[{"source":source(owner),"start":position,"end":position+1}],
            })
        })
        .collect::<Vec<_>>());
    wire
}

fn read(wire: &Value, imports: &[(&str, Arc<CheckedPackageV2>)]) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    for (name, dependency) in imports {
        evidence
            .insert_dependency_package(*name, Arc::clone(dependency))
            .expect("distinct dependency names");
    }
    CheckedPackageV2::read(
        &serde_json::to_vec(wire).expect("test wire"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn admitted(wire: &Value, imports: &[(&str, Arc<CheckedPackageV2>)]) -> Arc<CheckedPackageV2> {
    match read(wire, imports) {
        CheckedPackageV2ReadResult::Admitted(value) => Arc::from(value),
        other => panic!("structural package must admit: {other:?}"),
    }
}

fn dimension_packages(all: &Value) -> Vec<(String, Arc<CheckedPackageV2>)> {
    let named = |name: &str| {
        let row = &all[name];
        dimension_node(row["package"].as_str().expect("package"), row, all)
    };
    let rpc_name = "acme/rpc";
    let rpc_nodes = ["rpc_time", "rpc_s", "rpc_length", "rpc_speed"].map(named);
    let rpc = admitted(&package(rpc_name, &rpc_nodes, &[]), &[]);
    let flow_name = "acme/flow";
    let flow_nodes = ["flow_time", "flow_speed", "flow_accel"].map(named);
    let flow = admitted(
        &package(flow_name, &flow_nodes, &[rpc_name]),
        &[(rpc_name, Arc::clone(&rpc))],
    );
    let clock_name = "acme/clock";
    let clock = admitted(&package(clock_name, &[named("clock_time")], &[]), &[]);
    vec![
        (clock_name.to_owned(), clock),
        (flow_name.to_owned(), flow),
        (rpc_name.to_owned(), rpc),
    ]
}

/// Trace: FR-063-AC-3
#[trace("TC-233", "FR-063-AC-3")]
#[test]
fn xp_01_three_independent_equivalences_admit_as_one_class() {
    let vectors = vectors();
    let all = &vectors["nodes"];
    let xp01 = vectors["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["id"] == "XP-01")
        .expect("XP-01");
    let dimensions = dimension_packages(all);
    let dimension_imports = dimensions
        .iter()
        .map(|(name, value)| (name.as_str(), Arc::clone(value)))
        .collect::<Vec<_>>();
    let equivalences = xp01["equivalences"].as_array().expect("equivalences");
    assert_eq!(equivalences.len(), 3);
    let mut authored = Vec::new();
    for (index, case) in equivalences.iter().enumerate() {
        let name = format!("equivalence/{index}");
        // Each authored package has one anchor occurrence, starting at zero.
        let node = equivalence_node(&name, case, 0, all);
        let graph = vec![node];
        let imports = dimensions
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        let wire = package(&name, &graph, &imports);
        let read = admitted(&wire, &dimension_imports);
        authored.push((name, read));
    }
    let mut graph = Vec::new();
    for (index, case) in equivalences.iter().enumerate() {
        graph.push(equivalence_node(&authored[index].0, case, index, all));
    }
    let imports = authored
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let evidence = authored
        .iter()
        .map(|(name, value)| (name.as_str(), Arc::clone(value)))
        .collect::<Vec<_>>();
    let wire = package("example", &graph, &imports);
    let read = admitted(&wire, &evidence);
    assert_eq!(read.graph().nodes.len(), 3);
    for (index, case) in equivalences.iter().enumerate() {
        let retained = &read.graph().nodes[index];
        assert_eq!(
            serde_json::to_value(&retained.node_id).expect("node id"),
            graph[index]["node_id"]
        );
        assert_eq!(
            serde_json::to_value(&retained.body).expect("typed equivalence body")["root_scale"],
            case["root_scale"],
            "XP-01 scale {index}"
        );
        assert_eq!(
            retained.occurrences[0].ordinal,
            u64::try_from(index).expect("vector index")
        );
    }
}

/// Trace: FR-063-AC-4
#[trace("TC-233", "FR-063-AC-4")]
#[test]
fn xp_adverse_pairs_refuse_at_the_published_equivalence() {
    let vectors = vectors();
    let all = &vectors["nodes"];
    let dimensions = dimension_packages(all);
    let imported = dimensions
        .iter()
        .map(|(name, value)| (name.as_str(), Arc::clone(value)))
        .collect::<Vec<_>>();
    // All referenced dimensions are read from their selected dependency
    // packages, including the unit and derived dimensions in XP-05/08..10.
    for case in vectors["cases"].as_array().expect("cases") {
        let id = case["id"].as_str().expect("case id");
        if id == "XP-01" {
            continue;
        }
        let mut nodes = Vec::new();
        for (index, equivalence) in case["equivalences"]
            .as_array()
            .expect("equivalences")
            .iter()
            .enumerate()
        {
            nodes.push(equivalence_node("example", equivalence, index, all));
        }
        let imports = dimensions
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        let wire = package("example", &nodes, &imports);
        let result = read(&wire, &imported);
        match case["expected"].as_str().expect("outcome") {
            "admitted" => assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{id}: {result:?}"
            ),
            "invalid_semantic_graph" => {
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id}: expected invalid semantic graph, got {result:?}")
                };
                assert_eq!(
                    refusal.code,
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    "{id}"
                );
                let at = usize::try_from(case["at"].as_u64().expect("refused equivalence"))
                    .expect("vector index fits usize");
                let expected = format!("/semantic_graph/nodes/{at}");
                assert_eq!(
                    refusal.loci[0]
                        .path
                        .as_ref()
                        .map(ToString::to_string)
                        .as_deref(),
                    Some(expected.as_str()),
                    "{id}"
                );
            }
            other => panic!("unrecognized QSpec outcome {other}"),
        }
    }
}
