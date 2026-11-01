// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public-reader qualification of the owner's complete selected timed-slot package.

use super::*;
use std::collections::BTreeSet;

fn published_slots() -> (Value, Value, PathBuf) {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json")).unwrap();
    let wire = read_json(&proposal.join(vectors["selected_package"].as_str().unwrap())).unwrap();
    (wire, vectors, fixtures)
}

fn admitted_slots(wire: &Value, fixtures: &Path) -> Box<CheckedPackageV2> {
    match read_selected(wire, "timed-slot control", fixtures) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("control must admit: {other:?}"),
    }
}

fn read_selected(wire: &Value, id: &str, fixtures: &Path) -> CheckedPackageV2ReadResult {
    read_package_with_document(
        wire,
        id,
        &proposal_dir_of(fixtures).join("domain-package-acme-timed-slots.json"),
    )
    .expect("published selected timed-slot package")
}

fn check_outcome(result: CheckedPackageV2ReadResult, case: &Value) {
    let id = case["id"].as_str().expect("case id");
    match (case["expected"].as_str(), result) {
        (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {
            println!("{id}: admitted");
        }
        (Some("refused"), CheckedPackageV2ReadResult::Refused(refusal)) => {
            assert_eq!(
                Some(code_word(refusal.code)),
                case["expected_code"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                case["expected_cause"].as_str(),
                "{id}"
            );
            let path = refusal.loci[0]
                .path
                .as_ref()
                .expect("refusal pointer")
                .to_string();
            if let Some(exact) = case["locus"].as_str() {
                assert_eq!(path, exact, "{id}");
            } else {
                let suffix = case["locus_suffix"]
                    .as_str()
                    .expect("published refusal suffix");
                assert!(path.ends_with(&format!("/{suffix}")), "{id}: {path}");
            }
            println!("{id}: {:?} {:?} at {path}", refusal.code, refusal.cause);
        }
        (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
    }
}

fn check_earlier_stages(wire: &Value, id: &str, fixtures: &Path) {
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .unwrap();
    let mut malformed = wire.clone();
    malformed["semantic_graph"]["nodes"][root]["occurrences"] = json!("not-an-array");
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&malformed, id, fixtures)
    else {
        panic!("{id}: wire defect must precede the semantic defect");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::MalformedWire, None),
        "{id}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/occurrences"),
        "{id}"
    );

    let mut graph = wire.clone();
    let root_node = &mut graph["semantic_graph"]["nodes"][root];
    let first_place = root_node["body"]["members"][3]["value"]["members"][0]["target"].clone();
    let dependencies = root_node["dependencies"].as_array_mut().unwrap();
    let position = dependencies
        .iter()
        .position(|target| *target == first_place)
        .unwrap();
    dependencies.remove(position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&graph, id, fixtures) else {
        panic!("{id}: graph defect must precede the semantic defect");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None),
        "{id}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/dependencies"),
        "{id}"
    );
}

#[test]
#[trace(
    "TC-048",
    "FR-038-AC-335",
    "FR-038-AC-337",
    "FR-038-AC-338",
    "FR-038-AC-339",
    "FR-038-AC-340"
)]
fn every_published_slot_and_presence_case_uses_the_strict_reader() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json")).unwrap();
    let original =
        read_json(&proposal.join(vectors["selected_package"].as_str().unwrap())).unwrap();
    let cases = vectors["cases"].as_array().expect("slot cases");
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["expected"] == "admitted" {
            let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
            assert_eq!(
                original["semantic_graph"]["nodes"][position], case["node"],
                "{id}"
            );
            check_outcome(read_selected(&original, id, &fixtures), case);
        } else {
            let selected = cases
                .iter()
                .find(|row| row["expected"] == "admitted" && row["field"] == case["field"])
                .expect("the selected package contains this declared field's valid place");
            let position = usize::try_from(selected["package_node"].as_u64().unwrap()).unwrap();
            let mut wire = original.clone();
            let old = wire["semantic_graph"]["nodes"][position]["node_id"].clone();
            let fresh = case["node"]["node_id"].clone();
            wire["semantic_graph"]["nodes"][position] = case["node"].clone();
            fixture_support::reassign_node_id(&mut wire, &old, &fresh);
            check_outcome(read_selected(&wire, id, &fixtures), case);
            check_earlier_stages(&wire, id, &fixtures);
        }
    }
    let presence = vectors["presence"]["cases"].as_array().unwrap();
    let baseline = presence.iter().find(|row| row["id"] == "PR-03").unwrap();
    let baseline_position = usize::try_from(baseline["package_node"].as_u64().unwrap()).unwrap();
    for case in presence {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["expected"] == "admitted" {
            let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
            assert_eq!(
                original["semantic_graph"]["nodes"][position], case["node"],
                "{id}"
            );
            check_outcome(read_selected(&original, id, &fixtures), case);
        } else {
            let mut wire = original.clone();
            let old = wire["semantic_graph"]["nodes"][baseline_position]["node_id"].clone();
            let fresh = case["node"]["node_id"].clone();
            wire["semantic_graph"]["nodes"][baseline_position] = case["node"].clone();
            fixture_support::reassign_node_id(&mut wire, &old, &fresh);
            check_outcome(read_selected(&wire, id, &fixtures), case);
            check_earlier_stages(&wire, id, &fixtures);
        }
    }
    assert_eq!(
        seen,
        BTreeSet::from([
            "SL-01", "SL-02", "SL-03", "SL-04", "SL-05", "SL-06", "SL-07", "SR-01", "SR-02",
            "SR-03", "SR-04", "PR-01", "PR-03", "PR-04", "PR-05", "PR-06",
        ])
    );
}

#[test]
#[trace("TC-048", "FR-038-AC-336", "FR-038-AC-340")]
fn every_published_guard_case_uses_the_strict_reader() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json")).unwrap();
    let mut seen = BTreeSet::new();
    for case in vectors["guards"].as_array().expect("guard vectors") {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut wire = read_json(&proposal.join(case["fixture"].as_str().unwrap())).unwrap();
        for patch in case["patch"].as_array().into_iter().flatten() {
            super::timed_model::apply_published_timed_patch(&mut wire, patch);
        }
        if id == "GD-01" {
            let result = read_selected(&wire, id, &fixtures);
            let CheckedPackageV2ReadResult::Admitted(package) = result else {
                panic!("GD-01 must admit: {result:?}");
            };
            let root = package
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    node.node_tag.as_ref() == "timed_model"
                        && node.semantic_form.as_ref() == "timed_model"
                })
                .expect("admitted timed root");
            let CheckedTerm::Aggregate { members } = &root.body else {
                panic!("typed root aggregate");
            };
            for name in [
                "clock_places",
                "resets",
                "guards",
                "invariants",
                "urgencies",
                "constraints",
            ] {
                assert!(members.iter().any(|member| matches!(member,
                    CheckedTerm::Binding { name: actual, value } if actual.as_ref() == name
                        && matches!(value.as_ref(), CheckedTerm::Aggregate { members } if members.is_empty())
                )), "{name}");
            }
            println!("{id}: admitted with all six empty timed aggregates");
        } else {
            assert!(matches!(id, "GD-02" | "GD-03"));
            let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&wire, id, &fixtures)
            else {
                panic!("{id} must refuse");
            };
            assert_eq!(
                Some(code_word(refusal.code)),
                case["expected_code"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                case["expected_cause"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                case["locus"].as_str().unwrap(),
                "{id}"
            );
            println!(
                "{id}: {:?} {:?} at {}",
                refusal.code,
                refusal.cause,
                refusal.loci[0].path.as_ref().unwrap()
            );
        }
    }
    assert_eq!(seen, BTreeSet::from(["GD-01", "GD-02", "GD-03"]));
}

#[test]
#[trace("TC-048", "FR-038-AC-337", "FR-038-AC-339")]
fn admitted_presence_retains_boolean_owner_slot_and_resolved_path_facts() {
    let (wire, vectors, fixtures) = published_slots();
    let package = admitted_slots(&wire, &fixtures);
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    assert_eq!(package.graph().nodes.len(), nodes.len());
    for (admitted, authored) in package.graph().nodes.iter().zip(nodes) {
        assert_eq!(
            admitted.semantic_form.as_ref(),
            authored["semantic_form"].as_str().unwrap()
        );
        assert_eq!(
            admitted.semantic_type,
            serde_json::from_value(authored["semantic_type"].clone()).unwrap()
        );
    }
    for case in vectors["presence"]["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["expected"] == "admitted")
    {
        let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
        let node = &package.graph().nodes[position];
        assert_eq!(node.semantic_form.as_ref(), "presence_read");
        assert_eq!(
            node.node_id,
            serde_json::from_value(nodes[position]["node_id"].clone()).unwrap()
        );
        let CheckedTerm::Application {
            operation,
            arguments,
            result_type,
            ..
        } = &node.body
        else {
            panic!("presence application")
        };
        assert_eq!(operation.identity(), Some("quire.op.option.present"));
        assert_eq!(result_type, &node.semantic_type);
        assert_eq!(
            node.semantic_type,
            fixture_support::resolved_fixture_id(&case["node"]["semantic_type"], nodes)
        );
        let expected = nodes[position]["body"]["arguments"].as_array().unwrap();
        assert_eq!(arguments.len(), expected.len());
        for (actual, expected) in arguments.iter().zip(expected) {
            let CheckedTerm::Reference { target } = actual else {
                panic!("ordered place, owner and slot references")
            };
            assert_eq!(
                *target,
                serde_json::from_value(expected["target"].clone()).unwrap()
            );
        }
        if case["id"] == "PR-03" {
            let slot = usize::try_from(
                vectors["presence"]["parameter_package_nodes"]["e"]
                    .as_u64()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                package.graph().nodes[slot].semantic_type,
                fixture_support::resolved_fixture_id(&vectors["type_nodes"]["integer"], nodes)
            );
        }
    }
    for name in ["v", "q"] {
        let place = &vectors["presence"]["places"][name];
        let position = usize::try_from(place["package_node"].as_u64().unwrap()).unwrap();
        let CheckedTerm::Aggregate { members } = &package.graph().nodes[position].body else {
            panic!("place body")
        };
        assert_eq!(members.len(), 3);
        let CheckedTerm::Binding { value, .. } = &members[2] else {
            panic!("resolved_path")
        };
        let CheckedTerm::Aggregate { members: rows } = value.as_ref() else {
            panic!("resolved rows")
        };
        assert_eq!(rows.len(), if name == "v" { 1 } else { 2 });
        let actual = serde_json::to_value(value.as_ref()).unwrap();
        assert_eq!(actual, place["node"]["body"]["members"][2]["value"]);
    }
}

#[test]
#[trace("TC-048", "FR-038-AC-338")]
fn wrong_typed_presence_slot_refuses_at_the_slot_argument() {
    let (mut wire, vectors, fixtures) = published_slots();
    let case = vectors["presence"]["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "PR-03")
        .unwrap();
    let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
    let other = usize::try_from(
        vectors["presence"]["parameter_package_nodes"]["other"]
            .as_u64()
            .unwrap(),
    )
    .unwrap();
    let old = wire["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["target"].clone();
    let replacement = wire["semantic_graph"]["nodes"][other]["node_id"].clone();
    let node = &mut wire["semantic_graph"]["nodes"][position];
    node["body"]["arguments"][2]["target"] = replacement.clone();
    for dependency in node["dependencies"].as_array_mut().unwrap() {
        if *dependency == old {
            *dependency = replacement.clone();
        }
    }
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_selected(&wire, "wrong slot type", &fixtures)
    else {
        panic!("wrong slot must refuse")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::TypeMismatch))
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{position}/body/arguments/2")
    );
}

#[test]
#[trace("TC-048", "FR-038-AC-339")]
fn optional_clock_path_refuses_missing_member_presence_and_inner_type() {
    let (wire, vectors, fixtures) = published_slots();
    let document = proposal_dir_of(&fixtures).join("domain-package-acme-timed-slots.json");
    let original = read_json(&document).unwrap();
    let place = usize::try_from(
        vectors["presence"]["places"]["v"]["package_node"]
            .as_u64()
            .unwrap(),
    )
    .unwrap();
    for change in ["missing member", "required presence", "wrong inner type"] {
        let mut selected = original.clone();
        let fields = selected["types"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|ty| ty["identity"] == "ix://acme/orders/Order")
            .unwrap()["fields"]
            .as_array_mut()
            .unwrap();
        let index = fields
            .iter()
            .position(|field| field["name"] == "v")
            .unwrap();
        match change {
            "missing member" => {
                fields.remove(index);
            }
            "required presence" => fields[index]["presence"] = json!("required"),
            "wrong inner type" => fields[index]["typeRef"] = json!("ix://quire/native/Integer"),
            _ => unreachable!(),
        }
        let bytes = serde_json::to_vec(&selected).unwrap();
        let result = read_package_with_document_override(
            &wire,
            change,
            &document,
            true,
            CheckedPackageReadLimits::bounded(),
            Some(&bytes),
        )
        .unwrap();
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{change}: must refuse {result:?}")
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::MalformedClockPlace)),
            "{change}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            format!(
                "/semantic_graph/nodes/{place}/body/members/{}/value/members/0",
                if change == "missing member" { 1 } else { 2 }
            ),
            "{change}"
        );
    }
}

#[test]
#[trace("TC-048", "FR-038-AC-336")]
fn removing_a_guard_preserves_the_real_place_template() {
    let (original, _, fixtures) = published_slots();
    let before = admitted_slots(&original, &fixtures);
    let mut wire = original.clone();
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    let guard = nodes
        .iter()
        .position(|node| node["semantic_form"] == "clock_guard")
        .unwrap();
    let root = nodes
        .iter()
        .position(|node| node["semantic_form"] == "timed_model")
        .unwrap();
    let old = nodes[guard]["node_id"].clone();
    let mut literal = nodes
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "literal"
                && node["body"]["value_kind"] == "text"
        })
        .unwrap()
        .clone();
    let fresh = fixture_support::structural_id(
        wire["name"].as_str().unwrap(),
        vec![json!("guard-free-control")],
        0,
    );
    literal["node_id"] = fresh.clone();
    literal["occurrences"] = nodes[guard]["occurrences"].clone();
    literal.as_object_mut().unwrap().remove("declaration");
    literal.as_object_mut().unwrap().remove("owner");
    // Retain the graph-local index pool: replace the removed guard with an unrelated literal.
    wire["semantic_graph"]["nodes"][guard] = literal;
    fixture_support::reassign_node_id(&mut wire, &old, &fresh);
    let node = &mut wire["semantic_graph"]["nodes"][root];
    node["body"]["members"][5]["value"]["members"] = json!([]);
    node["dependencies"]
        .as_array_mut()
        .unwrap()
        .retain(|id| *id != fresh);
    let after = admitted_slots(&wire, &fixtures);
    assert_ne!(
        before.graph().nodes[root].body,
        after.graph().nodes[root].body
    );
    assert_ne!(
        before.graph().nodes[guard].body,
        after.graph().nodes[guard].body
    );
    for (left, right) in before
        .graph()
        .nodes
        .iter()
        .zip(after.graph().nodes.iter())
        .filter(|(left, _)| left.semantic_form.as_ref() == "clock_place")
    {
        assert_eq!(left.node_id, right.node_id);
        assert_eq!(left.body, right.body);
    }
}

#[test]
#[trace("TC-048", "FR-038-AC-337", "FR-038-AC-340")]
fn public_reader_keeps_the_one_operand_option_presence_form() {
    let mut wire = fixture_support::v2_all_families();
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    let boolean = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .unwrap()["node_id"]
        .clone();
    let option = nodes
        .iter()
        .find(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == "option")
        .unwrap()["node_id"]
        .clone();
    let value_id = fixture_support::structural_id(
        wire["name"].as_str().unwrap(),
        vec![json!("option-value-control")],
        0,
    );
    let app_id = fixture_support::structural_id(
        wire["name"].as_str().unwrap(),
        vec![json!("option-presence-control")],
        0,
    );
    let mut value = nodes
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "literal")
        .unwrap()
        .clone();
    value["node_id"] = value_id.clone();
    value["semantic_form"] = json!("option_value");
    value["semantic_type"] = option;
    value["dependencies"] = json!([]);
    value["body"] = json!({"term":"aggregate", "members":[]});
    value["occurrences"] = json!([{"role":"expression", "ordinal":0}]);
    let mut application = nodes
        .iter()
        .find(|node| node["body"]["term"] == "application")
        .unwrap()
        .clone();
    application["node_id"] = app_id.clone();
    application["node_tag"] = json!("expression");
    application["semantic_form"] = json!("presence_read");
    application["semantic_type"] = boolean.clone();
    application["dependencies"] = json!([value_id]);
    application["body"] = json!({"term":"application", "operator":"present",
        "operation":{"identity":"quire.op.option.present", "laws":[], "mode":null, "member":null, "leaves":[]}, "result_type":boolean,
        "arguments":[{"term":"reference", "target":value_id}]});
    application["occurrences"] = json!([{"role":"expression", "ordinal":0}]);
    for mut node in [value, application] {
        node.as_object_mut().unwrap().remove("declaration");
        node.as_object_mut().unwrap().remove("owner");
        let mut source = wire["source_map"][0].clone();
        source["node_id"] = node["node_id"].clone();
        source["role"] = json!("expression");
        source["ordinal"] = json!(0);
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node);
        wire["source_map"].as_array_mut().unwrap().push(source);
    }
    let result = CheckedPackageV2::read(
        &fixture_support::canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &fixture_support::evidence_for(&wire),
    );
    let CheckedPackageV2ReadResult::Admitted(package) = result else {
        panic!("one-operand Option form must admit: {result:?}")
    };
    let node = package.graph().nodes.last().unwrap();
    assert_eq!(node.node_id, serde_json::from_value(app_id).unwrap());
    let CheckedTerm::Application {
        operation,
        arguments,
        ..
    } = &node.body
    else {
        panic!("presence application")
    };
    assert_eq!(operation.identity(), Some("quire.op.option.present"));
    assert_eq!(arguments.len(), 1);
}
