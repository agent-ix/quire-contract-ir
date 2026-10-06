// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Exact, independent per-item lowering of admitted CheckedPackage V2 nodes.

use crate::support::checked_package::{
    self, canonical, evidence_for, mint_ungrouped_structural_keys, refresh_identity, settle,
    sha256_hex, typed_node_id, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2,
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

/// Every family node's node key is `prefix.repeat(16)` except the four whose
/// body is a real `application` term (function, temporal, protocol, claim):
/// their required `operation`/`result_type` members change their preimage, so
/// the fixture gives them the real computed `quire.application-node/v1` key
/// instead of a repeated placeholder — see `checked_package::family_key`,
/// which both the fixture builder and this lookup call, so the two can never
/// drift apart. `prefix` still labels and orders them.
fn key(prefix: &str) -> String {
    checked_package::family_key(prefix)
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
    assert_eq!(result.package.source_package_id(), package.package_id());
    assert_eq!(result.records.len(), FAMILIES.len());
    for (position, ((prefix, tag), record)) in FAMILIES.iter().zip(&result.records).enumerate() {
        let node = lowered(record);
        let wire = &value["semantic_graph"]["nodes"][position];
        assert_eq!(node.node_tag, *tag, "{prefix}");
        assert_eq!(node.node.node_id, id(prefix));
        assert_eq!(serde_json::to_value(&node.node).expect("node"), *wire);
        // The `option` is its own type and the `integer_range` is typed at
        // the `Integer` node (FR-038-AC-134); every other family is typed at
        // `aaaa`.
        let expected_type = match *prefix {
            "bbbb" => id("bbbb"),
            "cccc" => id("a3a3"),
            _ => id("aaaa"),
        };
        assert_eq!(node.semantic_type, expected_type, "{prefix}");
        let mut expected_dependencies = match *prefix {
            "aaaa" => vec![],
            // The self-typed `option`'s body references `aaaa`.
            "bbbb" => vec![id("aaaa")],
            // The range's `Integer` type, which also types its two bounds.
            "cccc" => vec![id("a3a3")],
            "eeee" | "7070" => vec![id("aaaa"), id("dddd")],
            // The function node's real `application` body also argues over
            // the second function node ("8080"), so its own one-hop
            // dependency set gains that reference beside its self-typed
            // `result_type`; digest-ascending puts "8080" before "aaaa".
            "ffff" => vec![id("8080"), id("aaaa")],
            // The temporal clause reaches its `over` parameter ("a4a4") and
            // that parameter's `integer` level type ("a3a3"), the `text`
            // type of its name literal ("a1a1"), its formula ("a2a2"), the
            // formula's `holds` operand ("a5a5") and that operand's Boolean
            // literal type (the one Boolean node, "aaaa", which is also its
            // `result_type`), in digest order (two of them are application
            // keys).
            "4040" => {
                let mut reached = vec![
                    id("a1a1"),
                    id("a2a2"),
                    id("a3a3"),
                    id("a4a4"),
                    id("a5a5"),
                    id("aaaa"),
                ];
                reached.sort();
                reached
            }
            _ => vec![id("aaaa")],
        };
        expected_dependencies.sort();
        assert_eq!(node.dependencies, expected_dependencies, "{prefix}");
        // bounds/claims exclude the requested node itself; this fixture's
        // only reachable dependencies ("aaaa", and "dddd" for eeee/7070) are
        // neither a bounded_domain nor a claim, so every direct request here
        // yields empty lists regardless of the requested node's own tag.
        assert_eq!(node.bounds, Vec::<CheckedNodeId>::new(), "{prefix}");
        assert_eq!(node.claims, Vec::<CheckedNodeId>::new(), "{prefix}");
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
    let missing = typed_node_id(&"0123456789abcdef".repeat(4));

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
    // its successor edges: eeee (1 + 1 reference term + 4 edges: its own
    // semantic_type `aaaa`, its two wire `dependencies` `aaaa`/`dddd`, and its
    // `reference` body's own target `dddd`), aaaa (1 + 1 aggregate term + 1
    // edge: its own semantic_type, self),
    // dddd (1 + 1 aggregate term + 2 edges: semantic_type `aaaa` appearing
    // once via the successor list's own leading entry and once via its one
    // wire dependency) makes 1 + 6 + 3 + 4 = 14.
    let exact = package.lower(&[id("eeee")], &profile(14));
    let mut expected_dependencies = vec![id("aaaa"), id("dddd")];
    expected_dependencies.sort();
    assert_eq!(
        lowered(&exact.records[0]).dependencies,
        expected_dependencies
    );
    let result = package.lower(&[id("eeee"), id("aaaa"), id("eeee")], &profile(13));
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("eeee"),
            limit_kind: CheckedPackageLimit::Work,
            limit: 13,
            consumed: 14,
        }
    );
    assert_eq!(
        result.records[1],
        package.lower(&[id("aaaa")], &profile(13)).records[0]
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));
    assert_eq!(result.records[2], result.records[0]);
    // aaaa costs 1 + (1 + 1 aggregate term + 1 edge: its own semantic_type,
    // self) = 4: the node charge fails at a limit of 1 and the term-and-edge
    // charge fails at a limit of 3.
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
            limit_kind: CheckedPackageLimit::Work,
            limit: 3,
            consumed: 4,
        }
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(1)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit_kind: CheckedPackageLimit::Work,
            limit: 1,
            consumed: 2,
        }
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(0)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: 1,
        }
    );
}

/// Tracing: TC-050, FR-038-AC-6
#[trace("TC-050", "FR-038-AC-6")]
#[test]
fn tc_050_unbounded_types_require_a_reachable_bounding_domain() {
    let value = integer_typed();
    let package = admit(&value);
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;

    let value_key = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "literal")
        .and_then(|node| node["node_id"]["digest"].as_str())
        .expect("value key");
    let result = package.lower(
        &[id("1010"), typed_node_id(value_key), id("cccc")],
        &bounded,
    );
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id("1010"),
            unbounded_type: id("a3a3"),
        }
    );
    let value_node = lowered(&result.records[1]);
    let mut reached = vec![id("a3a3"), id("cccc")];
    reached.sort();
    assert_eq!(value_node.dependencies, reached);
    assert_eq!(value_node.bounds, vec![id("cccc")]);
    // Requesting the bounded_domain node itself excludes it from its own
    // `bounds`: the only other reachable node (`a3a3`, the `Integer` node) is
    // not a bounded domain.
    assert!(lowered(&result.records[2]).bounds.is_empty());

    // Without the bound requirement the same request lowers.
    let unbounded = package.lower(&[id("1010")], &profile(u64::MAX));
    assert!(unbounded.records[0] != result.records[0]);
    assert!(lowered(&unbounded.records[0]).bounds.is_empty());
}

/// The all-families fixture with the namespace node `1010` and the value
/// node `dddd` typed at the unbounded `Integer` node (`a3a3`): `1010` reaches
/// no bounded domain and `dddd` reaches the `integer_range` node `cccc`.
fn integer_typed() -> Value {
    let mut value = v2_all_families();
    let nodes = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    for node in nodes.iter_mut() {
        if node["node_id"] == json!(checked_package::node_id(&key("1010"))) {
            node["semantic_type"] = checked_package::node_id(&key("a3a3"));
            node["dependencies"] = json!([checked_package::node_id(&key("a3a3"))]);
        } else if node["node_id"] == json!(checked_package::node_id(&key("dddd"))) {
            node["semantic_type"] = checked_package::node_id(&key("a3a3"));
            node["dependencies"] = json!([checked_package::node_id(&key("cccc"))]);
        }
    }
    mint_ungrouped_structural_keys(&mut value);
    settle(&mut value);
    value
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
    let integer_package = admit(&integer_typed());
    let observed = [
        record_kind(&package.lower(&[id("aaaa")], &profile(u64::MAX)).records[0]),
        record_kind(&package.lower(&[id("dddd")], &narrow).records[0]),
        record_kind(&integer_package.lower(&[id("1010")], &bounded).records[0]),
        record_kind(
            &package
                .lower(
                    &[typed_node_id(&"0123456789abcdef".repeat(4))],
                    &profile(u64::MAX),
                )
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
    // `scalar` is unbounded `text` and `composite` an unbounded
    // `sequence` over it, so a closure can hold an unsupported tag and an
    // unbounded type at once. `value` is a node typed at the scalar.
    let forms = forms_package("text", "sequence");
    let package = admit(&forms.value);
    let (scalar, composite, value) = (&forms.scalar, &forms.composite, &forms.value_node);

    // FR-038-AC-8: unsupported is decided before requires_bound and wins
    // outright. The composite's closure is {composite, scalar}; both are
    // unbounded, and with scalar_type out of profile the scalar is also
    // unsupported.
    let mut both = profile(u64::MAX);
    both.require_bounds = true;
    both.supported_tags.remove(&CheckedNodeTag::ScalarType);
    assert_eq!(
        package
            .lower(std::slice::from_ref(composite), &both)
            .records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: composite.clone(),
            unsupported_node_id: scalar.clone(),
            node_tag: CheckedNodeTag::ScalarType,
        }
    );

    // FR-038-AC-8: the per-request work unit is charged before the key is
    // looked up, so a zero work limit returns failed for an *absent* key
    // rather than invalid_input. With any budget the same key is invalid_input.
    let missing = typed_node_id(&"0123456789abcdef".repeat(4));
    assert_eq!(
        package
            .lower(std::slice::from_ref(&missing), &profile(0))
            .records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: missing.clone(),
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: 1,
        }
    );
    assert_eq!(
        package
            .lower(std::slice::from_ref(&missing), &profile(1))
            .records[0],
        CompleteLoweringRecordV2::InvalidInput { node_id: missing }
    );

    // FR-038-AC-8: "first" is the least key in ascending order over the whole
    // closure, not the first node the traversal reached. The value node is
    // the requested node and therefore the first visited; the scalar is the
    // lesser key. Both are out of profile, and the record names the scalar.
    let mut two_unsupported = profile(u64::MAX);
    two_unsupported
        .supported_tags
        .remove(&CheckedNodeTag::Value);
    two_unsupported
        .supported_tags
        .remove(&CheckedNodeTag::ScalarType);
    assert!(scalar < value);
    assert_eq!(
        package
            .lower(std::slice::from_ref(value), &two_unsupported)
            .records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: value.clone(),
            unsupported_node_id: scalar.clone(),
            node_tag: CheckedNodeTag::ScalarType,
        }
    );

    // The same tie-break governs requires_bound: the composite is visited
    // first and the scalar is the lesser key, and both are unbounded.
    assert!(scalar < composite);
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;
    assert_eq!(
        package
            .lower(std::slice::from_ref(composite), &bounded)
            .records[0],
        CompleteLoweringRecordV2::RequiresBound {
            node_id: composite.clone(),
            unbounded_type: scalar.clone(),
        }
    );
}

/// A package of one scalar node of `scalar_form`, one composite node of
/// `composite_form` over it and one value node typed at the scalar, in place
/// of the all-families graph. The undeclared nodes of the ten derived shapes
/// carry their derived key and closed body (FR-038-AC-134); a form the reader
/// does not derive carries a placeholder key.
struct Forms {
    value: Value,
    scalar: CheckedNodeId,
    composite: CheckedNodeId,
    value_node: CheckedNodeId,
}

fn forms_package(scalar_form: &str, composite_form: &str) -> Forms {
    let node = |key: &str, tag: &str, form: &str, ty: &str, dependencies: &[&str], body: Value| {
        json!({
            "node_id": checked_package::node_id(key),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": checked_package::node_id(ty),
            "dependencies": dependencies
                .iter()
                .map(|key| checked_package::node_id(key))
                .collect::<Vec<_>>(),
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        })
    };
    let empty = || json!({"term": "aggregate", "members": []});
    let scalar_key = checked_package::structural_key("scalar_type", scalar_form, None, &empty());
    let value_key =
        checked_package::structural_key("value", "literal", Some(&scalar_key), &empty());
    let self_typed = matches!(
        composite_form,
        "option" | "reference" | "set" | "bag" | "sequence" | "ordered_set"
    );
    let (composite_key, composite) = if self_typed {
        let body = checked_package::over_body(&scalar_key);
        let key = checked_package::structural_key("composite_type", composite_form, None, &body);
        let composite = node(
            &key,
            "composite_type",
            composite_form,
            &key,
            &[&scalar_key],
            body,
        );
        (key, composite)
    } else {
        let key = checked_package::structural_key(
            "composite_type",
            composite_form,
            Some(&scalar_key),
            &empty(),
        );
        let composite = node(
            &key,
            "composite_type",
            composite_form,
            &scalar_key,
            &[],
            empty(),
        );
        (key, composite)
    };
    let mut value = v2_all_families();
    value["semantic_graph"]["nodes"] = json!([
        node(
            &scalar_key,
            "scalar_type",
            scalar_form,
            &scalar_key,
            &[],
            empty()
        ),
        composite,
        node(
            &value_key,
            "value",
            "literal",
            &scalar_key,
            &[&scalar_key],
            empty()
        ),
    ]);
    checked_package::rebuild_source_map(&mut value);
    refresh_identity(&mut value);
    Forms {
        value,
        scalar: typed_node_id(&scalar_key),
        composite: typed_node_id(&composite_key),
        value_node: typed_node_id(&value_key),
    }
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

    // Which node is the unbounded type: the scalar, the composite or neither.
    #[derive(Clone, Copy)]
    enum Unbounded {
        Scalar,
        Composite,
        Neither,
    }
    for (scalar_form, composite_form, unbounded) in scalar_unbounded
        .iter()
        .map(|form| (*form, "record", Unbounded::Scalar))
        .chain(
            scalar_bounded
                .iter()
                .map(|form| (*form, "record", Unbounded::Neither)),
        )
        .chain(
            composite_unbounded
                .iter()
                .map(|form| ("boolean", *form, Unbounded::Composite)),
        )
        .chain(
            composite_bounded
                .iter()
                .map(|form| ("boolean", *form, Unbounded::Neither)),
        )
    {
        let forms = forms_package(scalar_form, composite_form);
        let package = admit(&forms.value);
        let record = package
            .lower(std::slice::from_ref(&forms.composite), &bounded_profile)
            .records[0]
            .clone();
        let expect_unbounded = !matches!(unbounded, Unbounded::Neither);
        if expect_unbounded {
            let unbounded_type = match unbounded {
                Unbounded::Scalar => forms.scalar.clone(),
                _ => forms.composite.clone(),
            };
            assert_eq!(
                record,
                CompleteLoweringRecordV2::RequiresBound {
                    node_id: forms.composite.clone(),
                    unbounded_type,
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
                dependencies.contains(bound),
                "bound {bound:?} is absent from dependencies"
            );
            saw_bound = true;
        }
        for claim in &node.claims {
            assert!(
                dependencies.contains(claim),
                "claim {claim:?} is absent from dependencies"
            );
            saw_claim = true;
        }
        // dependencies, bounds and claims are every reachable key except the
        // node itself: the requested node is never its own dependency, bound
        // or claim.
        assert!(!dependencies.contains(&node.node.node_id));
        assert!(!node.bounds.contains(&node.node.node_id));
        assert!(!node.claims.contains(&node.node.node_id));
        let mut ascending = node.dependencies.clone();
        ascending.sort();
        assert_eq!(node.dependencies, ascending);
    }
    assert!(saw_bound, "no lowered record reached a bounding domain");
    assert!(saw_claim, "no lowered record reached a claim");
}
