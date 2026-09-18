// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Exact, independent per-item lowering of admitted CheckedPackage V2 nodes.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, refresh_identity, sha256_hex, typed_node_id, v2_all_families,
    v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    CONTRACT_IR_SEMANTIC_DOMAIN,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Every family in the all-families fixture, in graph order.
const FAMILIES: [(&str, CheckedNodeTag); 13] = [
    ("aaaa", CheckedNodeTag::ScalarType),
    ("bbbb", CheckedNodeTag::CompositeType),
    ("cccc", CheckedNodeTag::BoundedDomain),
    ("dddd", CheckedNodeTag::Value),
    ("eeee", CheckedNodeTag::Expression),
    ("ffff", CheckedNodeTag::Function),
    ("1010", CheckedNodeTag::Model),
    ("2020", CheckedNodeTag::Relation),
    ("3030", CheckedNodeTag::State),
    ("4040", CheckedNodeTag::Temporal),
    ("5050", CheckedNodeTag::Protocol),
    ("6060", CheckedNodeTag::Claim),
    ("7070", CheckedNodeTag::Correspondence),
];

fn key(prefix: &str) -> String {
    prefix.repeat(16)
}

fn id(prefix: &str) -> CheckedNodeId {
    typed_node_id(&key(prefix))
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

fn profile(work_limit: u64) -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: FAMILIES.iter().map(|(_, tag)| *tag).collect(),
        require_bounds: false,
        work_limit,
    }
}

fn lowered(record: &CompleteLoweringRecordV2) -> &quire_contract_ir::CompleteContractNodeV2 {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => node,
        other => panic!("expected lowered record, got {other:?}"),
    }
}

/// Tracing: TC-050, FR-038-AC-6
#[trace("TC-050", "FR-038-AC-6")]
#[test]
fn tc_050_every_family_lowers_with_its_exact_closure_and_identity() {
    let value = v2_all_families();
    let package = admit(&value);
    let requested = FAMILIES
        .iter()
        .map(|(prefix, _)| id(prefix))
        .collect::<Vec<_>>();
    let result = package.lower(&requested, &profile(u64::MAX));
    assert_eq!(result.package_id, *package.package_id());
    assert_eq!(result.records.len(), FAMILIES.len());
    for (position, ((prefix, tag), record)) in FAMILIES.iter().zip(&result.records).enumerate() {
        let node = lowered(record);
        let wire = &value["semantic_graph"]["nodes"][position];
        assert_eq!(node.node_tag, *tag, "{prefix}");
        assert_eq!(node.node.node_id, id(prefix));
        assert_eq!(serde_json::to_value(&node.node).expect("node"), *wire);
        assert_eq!(node.semantic_type, id("aaaa"));
        let expected_dependencies = match *prefix {
            "aaaa" => vec![],
            "eeee" | "7070" => vec![id("aaaa"), id("dddd")],
            _ => vec![id("aaaa")],
        };
        assert_eq!(node.dependencies, expected_dependencies, "{prefix}");
        let only_self = |family: CheckedNodeTag| {
            if *tag == family {
                vec![id(prefix)]
            } else {
                vec![]
            }
        };
        assert_eq!(
            node.bounds,
            only_self(CheckedNodeTag::BoundedDomain),
            "{prefix}"
        );
        assert_eq!(node.claims, only_self(CheckedNodeTag::Claim), "{prefix}");
        let source_map = value["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| entry["node_id"] == wire["node_id"])
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&node.source_map).expect("source map"),
            Value::Array(source_map)
        );

        // The lowered identity is re-derivable from the wire alone.
        let mut projection = wire.clone();
        projection
            .as_object_mut()
            .expect("node")
            .remove("occurrences");
        let preimage = json!({
            "version": "quire.contract-ir.lowered-node/v1",
            "node": projection,
            "dependencies": node.dependencies,
            "bounds": node.bounds,
            "claims": node.claims,
        });
        assert_eq!(node.ir_id.domain.as_ref(), CONTRACT_IR_SEMANTIC_DOMAIN);
        assert_eq!(node.ir_id.algorithm.as_ref(), "sha256");
        assert_eq!(
            node.ir_id.digest.as_ref(),
            sha256_hex(&canonical(&preimage))
        );
    }
    let identities = result
        .records
        .iter()
        .map(|record| lowered(record).ir_id.digest.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), FAMILIES.len());

    // Nominal nodes lower with their preimages and nominal closures intact.
    let nominal_value = checked_package::v2_nominal();
    let nominal = admit(&nominal_value);
    let keys = nominal_value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| typed_node_id(node["node_id"]["digest"].as_str().expect("key")))
        .collect::<Vec<_>>();
    let result = nominal.lower(&keys, &profile(u64::MAX));
    let [member, declaration, unit, dimension] = [0, 1, 2, 3].map(|i| lowered(&result.records[i]));
    assert_eq!(member.dependencies, vec![keys[1].clone()]);
    assert_eq!(member.semantic_type, keys[1]);
    assert!(declaration.dependencies.is_empty());
    assert_eq!(unit.dependencies, vec![keys[3].clone()]);
    assert!(dimension.dependencies.is_empty());
    for (position, node) in [member, declaration, unit, dimension].iter().enumerate() {
        assert_eq!(
            serde_json::to_value(&node.node).expect("node"),
            nominal_value["semantic_graph"]["nodes"][position]
        );
        assert!(node.node.nominal_identity_preimage.is_some());
    }
}

/// Tracing: TC-050, FR-038-AC-6
#[trace("TC-050", "FR-038-AC-6")]
#[test]
fn tc_050_non_lowered_records_are_terminal_and_independent() {
    let value = v2_all_families();
    let package = admit(&value);
    let missing = typed_node_id(&"9".repeat(64));

    // invalid_input for a key outside the admitted graph.
    let result = package.lower(&[missing.clone(), id("aaaa")], &profile(u64::MAX));
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::InvalidInput { node_id: missing }
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));

    // unsupported names the first unsupported reachable key.
    let mut narrow = profile(u64::MAX);
    narrow.supported_tags.remove(&CheckedNodeTag::Value);
    let result = package.lower(&[id("7070"), id("aaaa"), id("dddd")], &narrow);
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("7070"),
            unsupported_node_id: id("dddd"),
            node_tag: CheckedNodeTag::Value,
        }
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));
    assert_eq!(
        result.records[2],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("dddd"),
            unsupported_node_id: id("dddd"),
            node_tag: CheckedNodeTag::Value,
        }
    );

    // failed at exactly one over the request's own work; siblings unaffected.
    // One for the request, then per visited node one plus its body terms plus
    // its successor edges: eeee (1 + 1 reference + 2 edges), aaaa (1 + 1
    // literal + 1 edge), dddd (1 + 1 literal + 1 edge) makes 1 + 4 + 3 + 3.
    let exact = package.lower(&[id("eeee")], &profile(11));
    assert_eq!(
        lowered(&exact.records[0]).dependencies,
        vec![id("aaaa"), id("dddd")]
    );
    let result = package.lower(&[id("eeee"), id("aaaa"), id("eeee")], &profile(10));
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("eeee"),
            limit: 10,
            consumed: 11,
        }
    );
    assert_eq!(
        result.records[1],
        package.lower(&[id("aaaa")], &profile(10)).records[0]
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));
    assert_eq!(result.records[2], result.records[0]);
    // aaaa costs 1 + (1 + 1 literal + 1 edge) = 4: the node charge fails at a
    // limit of 1 and the term-and-edge charge fails at a limit of 3.
    assert_eq!(
        lowered(&package.lower(&[id("aaaa")], &profile(4)).records[0])
            .node
            .node_id,
        id("aaaa")
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(3)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit: 3,
            consumed: 4,
        }
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(1)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit: 1,
            consumed: 2,
        }
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(0)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit: 0,
            consumed: 1,
        }
    );
}

/// Tracing: TC-050, FR-038-AC-6
#[trace("TC-050", "FR-038-AC-6")]
#[test]
fn tc_050_unbounded_types_require_a_reachable_bounding_domain() {
    let mut value = v2_all_families();
    value["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    value["semantic_graph"]["nodes"][3]["dependencies"] =
        json!([value["semantic_graph"]["nodes"][2]["node_id"]]);
    refresh_identity(&mut value);
    let package = admit(&value);
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;

    let result = package.lower(&[id("bbbb"), id("dddd"), id("cccc")], &bounded);
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id("bbbb"),
            unbounded_type: id("aaaa"),
        }
    );
    let value_node = lowered(&result.records[1]);
    assert_eq!(value_node.dependencies, vec![id("aaaa"), id("cccc")]);
    assert_eq!(value_node.bounds, vec![id("cccc")]);
    assert_eq!(lowered(&result.records[2]).bounds, vec![id("cccc")]);

    // Without the bound requirement the same request lowers.
    let unbounded = package.lower(&[id("bbbb")], &profile(u64::MAX));
    assert!(unbounded.records[0] != result.records[0]);
    assert!(lowered(&unbounded.records[0]).bounds.is_empty());
}

/// The closed FR-038 lowering record vocabulary. The match has no wildcard
/// arm, so an eighth record kind fails to compile here rather than reaching a
/// consumer written to seven.
fn record_kind(record: &CompleteLoweringRecordV2) -> &'static str {
    match record {
        CompleteLoweringRecordV2::Lowered { .. } => "lowered",
        CompleteLoweringRecordV2::Unsupported { .. } => "unsupported",
        CompleteLoweringRecordV2::RequiresBound { .. } => "requires_bound",
        CompleteLoweringRecordV2::InvalidInput { .. } => "invalid_input",
        CompleteLoweringRecordV2::Failed { .. } => "failed",
        CompleteLoweringRecordV2::InvalidBody { .. } => "invalid_body",
        CompleteLoweringRecordV2::BodyIncomplete { .. } => "body_incomplete",
    }
}

fn graph_keys(value: &Value) -> Vec<CheckedNodeId> {
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .map(|node| typed_node_id(node["node_id"]["digest"].as_str().expect("node id digest")))
        .collect()
}

/// Tracing: TC-052, FR-038-AC-7
#[trace("TC-052", "FR-038-AC-7")]
#[test]
fn tc_052_record_vocabulary_is_seven_and_defensive_kinds_are_unreachable() {
    // FR-038-AC-7: the vocabulary is exactly seven distinct declared kinds.
    let declared = [
        "lowered",
        "unsupported",
        "requires_bound",
        "invalid_input",
        "failed",
        "invalid_body",
        "body_incomplete",
    ];
    assert_eq!(
        declared.iter().collect::<BTreeSet<_>>().len(),
        declared.len()
    );

    // FR-038-AC-7: lowering every node of every vendored fixture under a
    // profile supporting every tag yields neither defensive kind. They exist so
    // the walk reports a refusing body instead of guessing past it; an admitted
    // package must never produce one.
    for value in [v2_all_families(), v2_nominal()] {
        let package = admit(&value);
        let keys = graph_keys(&value);
        assert!(!keys.is_empty(), "fixture has no nodes to lower");
        let result = package.lower(&keys, &profile(u64::MAX));
        assert_eq!(result.records.len(), keys.len());
        for record in &result.records {
            let kind = record_kind(record);
            assert!(
                declared.contains(&kind),
                "record kind {kind} is outside the declared vocabulary"
            );
            assert_ne!(
                kind, "invalid_body",
                "admitted package yielded invalid_body"
            );
            assert_ne!(
                kind, "body_incomplete",
                "admitted package yielded body_incomplete"
            );
        }
    }

    // FR-038-AC-7: the four reachable non-lowered kinds are each named and
    // each distinct, so "seven declared" is not seven spellings of one state.
    let value = v2_all_families();
    let package = admit(&value);
    let mut narrow = profile(u64::MAX);
    narrow.supported_tags.remove(&CheckedNodeTag::Value);
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;
    let mut integer = v2_all_families();
    integer["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    refresh_identity(&mut integer);
    let integer_package = admit(&integer);
    let observed = [
        record_kind(&package.lower(&[id("aaaa")], &profile(u64::MAX)).records[0]),
        record_kind(&package.lower(&[id("dddd")], &narrow).records[0]),
        record_kind(&integer_package.lower(&[id("bbbb")], &bounded).records[0]),
        record_kind(
            &package
                .lower(&[typed_node_id(&"9".repeat(64))], &profile(u64::MAX))
                .records[0],
        ),
        record_kind(&package.lower(&[id("aaaa")], &profile(0)).records[0]),
    ];
    assert_eq!(
        observed,
        [
            "lowered",
            "unsupported",
            "requires_bound",
            "invalid_input",
            "failed"
        ]
    );
}

/// Tracing: TC-052, FR-038-AC-8
#[trace("TC-052", "FR-038-AC-8")]
#[test]
fn tc_052_lowering_outcome_selection_is_a_total_order() {
    let mut value = v2_all_families();
    // aaaa becomes an unbounded integer and bbbb an unbounded sequence, so a
    // closure can hold an unsupported tag and an unbounded type at once.
    value["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    value["semantic_graph"]["nodes"][1]["semantic_form"] = json!("sequence");
    refresh_identity(&mut value);
    let package = admit(&value);

    // FR-038-AC-8: unsupported is decided before requires_bound and wins
    // outright. bbbb's closure is {bbbb, aaaa}; both are unbounded, and with
    // scalar_type out of profile aaaa is also unsupported.
    let mut both = profile(u64::MAX);
    both.require_bounds = true;
    both.supported_tags.remove(&CheckedNodeTag::ScalarType);
    assert_eq!(
        package.lower(&[id("bbbb")], &both).records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("bbbb"),
            unsupported_node_id: id("aaaa"),
            node_tag: CheckedNodeTag::ScalarType,
        }
    );

    // FR-038-AC-8: the per-request work unit is charged before the key is
    // looked up, so a zero work limit returns failed for an *absent* key
    // rather than invalid_input. With any budget the same key is invalid_input.
    let missing = typed_node_id(&"9".repeat(64));
    assert_eq!(
        package.lower(&[missing.clone()], &profile(0)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: missing.clone(),
            limit: 0,
            consumed: 1,
        }
    );
    assert_eq!(
        package.lower(&[missing.clone()], &profile(1)).records[0],
        CompleteLoweringRecordV2::InvalidInput { node_id: missing }
    );

    // FR-038-AC-8: "first" is the least key in ascending order over the whole
    // closure, not the first node the traversal reached. dddd is the requested
    // node and therefore the first visited; aaaa is the lesser key. Both are
    // out of profile, and the record names aaaa.
    let mut two_unsupported = profile(u64::MAX);
    two_unsupported
        .supported_tags
        .remove(&CheckedNodeTag::Value);
    two_unsupported
        .supported_tags
        .remove(&CheckedNodeTag::ScalarType);
    assert!(id("aaaa") < id("dddd"));
    assert_eq!(
        package.lower(&[id("dddd")], &two_unsupported).records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("dddd"),
            unsupported_node_id: id("aaaa"),
            node_tag: CheckedNodeTag::ScalarType,
        }
    );

    // The same tie-break governs requires_bound: bbbb is visited first and
    // aaaa is the lesser key, and both are unbounded.
    assert!(id("aaaa") < id("bbbb"));
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;
    assert_eq!(
        package.lower(&[id("bbbb")], &bounded).records[0],
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id("bbbb"),
            unbounded_type: id("aaaa"),
        }
    );
}

/// Tracing: TC-052, FR-038-AC-8
#[trace("TC-052", "FR-038-AC-8")]
#[test]
fn tc_052_unbounded_forms_are_exactly_the_eight_declared_forms() {
    // FR-038-AC-8: each of the eight forms raises requires_bound and each
    // other declared form of those two families does not. bbbb's closure is
    // {bbbb, aaaa} and holds no reachable bounded_domain, so the outcome is
    // decided by the two forms alone.
    let scalar_unbounded = ["integer", "rational", "decimal", "text"];
    let scalar_bounded = ["boolean", "float32", "float64"];
    let composite_unbounded = ["sequence", "set", "bag", "ordered_set"];
    let composite_bounded = ["option", "record", "tuple", "alias", "reference"];

    let mut bounded_profile = profile(u64::MAX);
    bounded_profile.require_bounds = true;

    // enum, dimension and unit are the three remaining declared scalar forms.
    // They are nominal: the reader requires a closed identity preimage, so
    // they cannot be produced by rewriting a non-nominal node's form and are
    // checked on the fixture that carries them for real. None requires a bound.
    let nominal = v2_nominal();
    let nominal_package = admit(&nominal);
    let nominal_keys = graph_keys(&nominal);
    for record in &nominal_package
        .lower(&nominal_keys, &bounded_profile)
        .records
    {
        assert_eq!(
            record_kind(record),
            "lowered",
            "no nominal scalar form requires a bound"
        );
    }

    for (scalar_form, composite_form, expect_unbounded, expected_key) in scalar_unbounded
        .iter()
        .map(|form| (*form, "record", true, "aaaa"))
        .chain(
            scalar_bounded
                .iter()
                .map(|form| (*form, "record", false, "")),
        )
        .chain(
            composite_unbounded
                .iter()
                .map(|form| ("boolean", *form, true, "bbbb")),
        )
        .chain(
            composite_bounded
                .iter()
                .map(|form| ("boolean", *form, false, "")),
        )
    {
        let mut value = v2_all_families();
        value["semantic_graph"]["nodes"][0]["semantic_form"] = json!(scalar_form);
        value["semantic_graph"]["nodes"][1]["semantic_form"] = json!(composite_form);
        refresh_identity(&mut value);
        let package = admit(&value);
        let record = package.lower(&[id("bbbb")], &bounded_profile).records[0].clone();
        if expect_unbounded {
            assert_eq!(
                record,
                CompleteLoweringRecordV2::RequiresBound {
                    node_id: id("bbbb"),
                    unbounded_type: id(expected_key),
                },
                "scalar {scalar_form} / composite {composite_form} must require a bound"
            );
        } else {
            assert_eq!(
                record_kind(&record),
                "lowered",
                "scalar {scalar_form} / composite {composite_form} must not require a bound"
            );
        }
    }
}

/// Tracing: TC-052, FR-038-AC-8
#[trace("TC-052", "FR-038-AC-8")]
#[test]
fn tc_052_dependencies_contain_every_bound_and_claim_key() {
    // FR-038-AC-8: bounds and claims are filtered views of the reachable set,
    // not a partition of it. The lowered-node preimage carries all three lists
    // as written, so a re-derivation that treats them as disjoint disagrees.
    let mut value = v2_all_families();
    // Reach the bounded_domain and the claim from the value node, so at least
    // one lowered record has a nonempty bounds and claims list to check.
    let bound_key = value["semantic_graph"]["nodes"][2]["node_id"].clone();
    let claim_key = value["semantic_graph"]["nodes"][11]["node_id"].clone();
    value["semantic_graph"]["nodes"][3]["dependencies"] = json!([bound_key, claim_key]);
    refresh_identity(&mut value);
    let package = admit(&value);
    let keys = graph_keys(&value);
    let result = package.lower(&keys, &profile(u64::MAX));
    let mut saw_bound = false;
    let mut saw_claim = false;
    for record in &result.records {
        let node = lowered(record);
        let dependencies = node.dependencies.iter().collect::<BTreeSet<_>>();
        for bound in &node.bounds {
            assert!(
                bound == &node.node.node_id || dependencies.contains(bound),
                "bound {bound:?} is absent from dependencies"
            );
            saw_bound |= bound != &node.node.node_id;
        }
        for claim in &node.claims {
            assert!(
                claim == &node.node.node_id || dependencies.contains(claim),
                "claim {claim:?} is absent from dependencies"
            );
            saw_claim |= claim != &node.node.node_id;
        }
        // dependencies is every reachable key except the node itself.
        assert!(!dependencies.contains(&node.node.node_id));
        let mut ascending = node.dependencies.clone();
        ascending.sort();
        assert_eq!(node.dependencies, ascending);
    }
    assert!(saw_bound, "no lowered record reached a bounding domain");
    assert!(saw_claim, "no lowered record reached a claim");
}
