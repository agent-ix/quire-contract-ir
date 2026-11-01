// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete operation declarations read through the admitted public facade.

use super::*;
use quire_contract_model::CheckedOperationReadErrorCode as Code;

fn operation(owner: &str, name: &str) -> Value {
    json!({"identity":format!("{}/{name}",orders(owner)),"name":name,
        "params":[],"pre":[],"post":[],"frame":{"modifies":[],"creates":[],"deletes":[]}})
}

fn with_operations(short: &str, parents: &[&str], operations: Vec<Value>) -> Value {
    let mut declared = object(short, parents, &[]);
    declared["operations"] = json!(operations);
    declared
}

#[trace("TC-464", "FR-355-AC-1", "FR-355-AC-5")]
#[test]
fn complete_census_includes_unreferenced_operations_without_bodies_or_anchors() {
    let doc = document(vec![
        with_operations(
            "Node",
            &[],
            vec![operation("Node", "zeta"), operation("Node", "alpha")],
        ),
        object("Empty", &[], &[]),
    ]);
    let admitted = admit(&doc, &[("Node", OBJECT), ("Empty", OBJECT)], &[]);
    let read = admitted
        .model_operations(&declaration_id("Node"), 10_000)
        .unwrap();
    assert_eq!(
        read.operations()
            .iter()
            .map(|op| op.name())
            .collect::<Vec<_>>(),
        ["alpha", "zeta"]
    );
    for op in read.operations() {
        assert_eq!(op.identity(), format!("{}/{}", orders("Node"), op.name()));
        assert_eq!(op.context(), &declaration_id("Node"));
        assert_eq!(op.declaring_node_id(), &declaration_id("Node"));
        assert!(op.parameters().is_empty());
        assert_eq!(op.result(), None);
    }
    assert!(admitted
        .model_operations(&declaration_id("Empty"), 10_000)
        .unwrap()
        .operations()
        .is_empty());
    assert!(admitted
        .graph()
        .nodes
        .iter()
        .all(|node| node.semantic_form.as_ref() != "operation_anchor"));
    let mut changed = doc.clone();
    changed["types"][0]["operations"]
        .as_array_mut()
        .unwrap()
        .pop();
    let admitted = admit(&changed, &[("Node", OBJECT), ("Empty", OBJECT)], &[]);
    let census = admitted
        .model_operations(&declaration_id("Node"), 10_000)
        .unwrap();
    assert_eq!(census.operations().len(), 1);
    assert_eq!(census.operations()[0].name(), "zeta");
}

fn parameter(owner: &str, operation: &str, name: &str, ty: &str) -> Value {
    json!({"identity":format!("{}/{operation}/{name}",orders(owner)),"name":name,"typeRef":ty,
        "presence":"required","nullable":false,"defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}})
}

#[trace("TC-464", "FR-355-AC-2")]
#[test]
fn parameter_order_domains_and_optional_result_are_independent() {
    let mut typed = operation("Node", "typed");
    let mut many = parameter("Node", "typed", "middle", &orders("Small"));
    many["multiplicity"] = json!({"lower":1,"upper":3,"ordered":true,"unique":true});
    typed["params"] = json!([
        parameter("Node", "typed", "z_flag", "ix://quire/native/Boolean"),
        many,
        parameter("Node", "typed", "a_count", &orders("Small")),
    ]);
    typed["returns"] = parameter("Node", "typed", "result", &orders("Small"));
    let mut unsupported = operation("Node", "unsupported");
    unsupported["params"] = json!([parameter(
        "Node",
        "unsupported",
        "real",
        "ix://quire/native/Decimal"
    )]);
    unsupported["returns"] =
        parameter("Node", "unsupported", "result", "ix://quire/native/Decimal");
    let doc = document(vec![
        with_operations(
            "Node",
            &[],
            vec![typed, unsupported, operation("Node", "unit")],
        ),
        value_type("Small", "integer", Some(("-2", "7"))),
    ]);
    let admitted = admit(&doc, &[("Node", OBJECT)], &[]);
    let census = admitted
        .model_operations(&declaration_id("Node"), 10_000)
        .unwrap();
    let op = &census.operations()[0];
    assert_eq!(op.name(), "typed");
    let parameters = op.parameters();
    assert_eq!(
        parameters.iter().map(|p| p.name()).collect::<Vec<_>>(),
        ["z_flag", "middle", "a_count"]
    );
    assert_eq!(
        parameters[0].member_type(),
        Some(&CheckedMemberType::Boolean)
    );
    assert_eq!(
        parameters[1].member_type(),
        Some(&CheckedMemberType::Collection {
            kind: CheckedCollectionKind::OrderedSet,
            element: Box::new(CheckedMemberType::IntRange {
                lower: -2,
                upper: 7
            }),
            bounds: Some((1, 3)),
        })
    );
    assert_eq!(
        parameters[2].member_type(),
        Some(&CheckedMemberType::IntRange {
            lower: -2,
            upper: 7
        })
    );
    assert_eq!(
        op.result(),
        Some(Some(&CheckedMemberType::IntRange {
            lower: -2,
            upper: 7
        }))
    );
    assert_eq!(census.operations()[1].name(), "unit");
    assert_eq!(census.operations()[1].result(), None);
    assert_eq!(census.operations()[2].name(), "unsupported");
    assert_eq!(census.operations()[2].parameters()[0].member_type(), None);
    assert_eq!(census.operations()[2].result(), Some(None));
}

#[trace("TC-464", "FR-355-AC-2", "FR-355-AC-3")]
#[test]
fn same_names_in_distinct_selected_packages_do_not_join_contexts_or_domains() {
    let mut step = operation("Node", "step");
    step["params"] = json!([parameter(
        "Node",
        "step",
        "value",
        "ix://quire/native/Boolean"
    )]);
    let first = document(vec![with_operations("Node", &[], vec![step])]);
    let mut second: Value = serde_json::from_str(
        &serde_json::to_string(&first)
            .unwrap()
            .replace(IDENTITY, "acme/other"),
    )
    .unwrap();
    second["types"][0]["operations"][0]["params"][0]["typeRef"] =
        json!("ix://quire/native/Integer");
    let (mut wire, _, _) = package(&first, &[("Node", OBJECT)], &[]);
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    let mut other = nodes
        .iter()
        .find(|node| node["owner"]["node"] == orders("Node"))
        .unwrap()
        .clone();
    other["node_id"] = structural_id(
        "acme/other",
        vec![json!("model"), json!(OBJECT), json!("ix://acme/other/Node")],
        0,
    );
    other["semantic_type"] = other["node_id"].clone();
    other["owner"] = json!({"kind":"model","identity":"acme/other","node":"ix://acme/other/Node"});
    let other_id: CheckedNodeId = serde_json::from_value(other["node_id"].clone()).unwrap();
    nodes.push(other);
    select_model(&mut wire, "acme/other");
    settle(&mut wire);
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    for (identity, document) in [
        (IDENTITY, first),
        ("acme/other", second),
        (
            "test/families",
            crate::support::checked_package::family_model_document(),
        ),
    ] {
        evidence
            .insert_domain_package_document(identity, canonical(&document))
            .unwrap();
    }
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("independent selections admit: {other:?}"),
    };
    let a = admitted
        .model_operations(&declaration_id("Node"), 10_000)
        .unwrap();
    let b = admitted.model_operations(&other_id, 10_000).unwrap();
    assert_eq!(a.operations()[0].name(), b.operations()[0].name());
    assert_ne!(a.operations()[0].identity(), b.operations()[0].identity());
    assert_ne!(
        a.operations()[0].declaring_node_id(),
        b.operations()[0].declaring_node_id()
    );
    assert_eq!(
        a.operations()[0].parameters()[0].member_type(),
        Some(&CheckedMemberType::Boolean)
    );
    assert_eq!(
        b.operations()[0].parameters()[0].member_type(),
        Some(&CheckedMemberType::Integer)
    );
}

#[trace("TC-464", "FR-355-AC-2")]
#[test]
fn unavailable_parameter_name_refuses_instead_of_inventing_a_binder() {
    let mut op = operation("Node", "step");
    let mut slot = parameter("Node", "step", "value", "ix://quire/native/Boolean");
    slot.as_object_mut().unwrap().remove("name");
    slot.as_object_mut().unwrap().remove("identity");
    op["params"] = json!([slot]);
    let doc = document(vec![with_operations("Node", &[], vec![op])]);
    let admitted = admit(&doc, &[("Node", OBJECT)], &[]);
    assert_eq!(
        admitted
            .model_operations(&declaration_id("Node"), 10_000)
            .unwrap_err()
            .code,
        Code::UnavailableParameterName
    );
}

#[trace("TC-464", "FR-355-AC-3")]
#[test]
fn inheritance_diamond_and_most_derived_redefinition_use_actual_declaring_nodes() {
    let mut middle = operation("Mid", "step");
    middle["redefines"] = json!(format!("{}/step", orders("Base")));
    let mut leaf = operation("Leaf", "step");
    leaf["redefines"] = json!(format!("{}/step", orders("Base")));
    let doc = document(vec![
        with_operations(
            "Base",
            &[],
            vec![operation("Base", "step"), operation("Base", "inherited")],
        ),
        with_operations("Mid", &["Base"], vec![middle]),
        object("Side", &["Base"], &[]),
        with_operations("Leaf", &["Mid", "Side"], vec![leaf]),
    ]);
    let declared = [
        ("Base", OBJECT),
        ("Mid", OBJECT),
        ("Side", OBJECT),
        ("Leaf", OBJECT),
    ];
    let admitted = admit(&doc, &declared, &[]);
    let census = admitted
        .model_operations(&declaration_id("Leaf"), 10_000)
        .unwrap();
    assert_eq!(census.operations().len(), 2);
    assert_eq!(
        census.operations()[0].identity(),
        format!("{}/inherited", orders("Base"))
    );
    assert_eq!(
        census.operations()[0].declaring_node_id(),
        &declaration_id("Base")
    );
    assert_eq!(
        census.operations()[1].identity(),
        format!("{}/step", orders("Leaf"))
    );
    assert_eq!(
        census.operations()[1].declaring_node_id(),
        &declaration_id("Leaf")
    );
    assert_eq!(census.operations()[0].context(), &declaration_id("Leaf"));
    let mut renamed = doc.clone();
    for (index, name) in [(1, "mid_step"), (3, "leaf_step")] {
        let owner = if index == 1 { "Mid" } else { "Leaf" };
        renamed["types"][index]["operations"][0]["identity"] =
            json!(format!("{}/{name}", orders(owner)));
        renamed["types"][index]["operations"][0]["name"] = json!(name);
    }
    let renamed = admit(&renamed, &declared, &[]);
    let read = renamed
        .model_operations(&declaration_id("Leaf"), 10_000)
        .unwrap();
    assert_eq!(
        read.operations()
            .iter()
            .map(|op| op.name())
            .collect::<Vec<_>>(),
        ["inherited", "leaf_step"]
    );
    let missing = admit(
        &doc,
        &[("Mid", OBJECT), ("Side", OBJECT), ("Leaf", OBJECT)],
        &[],
    );
    assert_eq!(
        missing
            .model_operations(&declaration_id("Leaf"), 10_000)
            .unwrap_err()
            .code,
        Code::MissingDeclaringNode
    );
}

#[trace("TC-464", "FR-355-AC-3")]
#[test]
fn ambiguous_operation_and_duplicate_declaring_node_refuse_the_whole_census() {
    let doc = document(vec![
        with_operations("Left", &[], vec![operation("Left", "same")]),
        with_operations("Right", &[], vec![operation("Right", "same")]),
        object("Leaf", &["Left", "Right"], &[]),
    ]);
    let admitted = admit(
        &doc,
        &[("Left", OBJECT), ("Right", OBJECT), ("Leaf", OBJECT)],
        &[],
    );
    assert_eq!(
        admitted
            .model_operations(&declaration_id("Leaf"), 10_000)
            .unwrap_err()
            .code,
        Code::AmbiguousOperation
    );
    let doc = document(vec![with_operations(
        "Node",
        &[],
        vec![operation("Node", "step")],
    )]);
    let (mut wire, evidence, _) = package(&doc, &[("Node", OBJECT)], &[]);
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    let mut duplicate = nodes
        .iter()
        .find(|node| node["semantic_form"] == OBJECT)
        .unwrap()
        .clone();
    duplicate["node_id"]["ordinal"] = json!(1);
    duplicate["semantic_type"] = duplicate["node_id"].clone();
    nodes.push(duplicate);
    settle(&mut wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("distinct structural ordinal admits: {other:?}"),
    };
    assert_eq!(
        admitted
            .model_operations(&declaration_id("Node"), 10_000)
            .unwrap_err()
            .code,
        Code::AmbiguousDeclaringNode
    );
}

#[trace("TC-464", "FR-355-AC-4")]
#[test]
fn context_refusals_and_exact_work_boundary_do_not_expose_partial_tables() {
    let doc = document(vec![with_operations(
        "Node",
        &[],
        vec![operation("Node", "step")],
    )]);
    let admitted = admit(&doc, &[("Node", OBJECT)], &[]);
    let id = declaration_id("Node");
    let census = admitted.model_operations(&id, 10_000).unwrap();
    let required = census.work();
    assert!(required > 1);
    assert_eq!(
        admitted.model_operations(&id, required).unwrap().work(),
        required
    );
    let denied = admitted.model_operations(&id, required - 1).unwrap_err();
    assert_eq!(denied.code, Code::WorkLimit);
    assert_eq!(denied.limit, required - 1);
    assert!(denied.work > denied.limit);
    assert_eq!(
        admitted.model_operations(&id, 0).unwrap_err().code,
        Code::WorkLimit
    );
    let mut other = id.clone();
    other.package = "foreign/package".into();
    assert_eq!(
        admitted.model_operations(&other, 10_000).unwrap_err().code,
        Code::ForeignContext
    );
    let mut unknown = id;
    unknown.ordinal = 99;
    assert_eq!(
        admitted
            .model_operations(&unknown, 10_000)
            .unwrap_err()
            .code,
        Code::UnknownNode
    );
    let root = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "model_root")
        .unwrap();
    assert_eq!(
        admitted
            .model_operations(&root.node_id, 10_000)
            .unwrap_err()
            .code,
        Code::WrongContextKind
    );
}

#[trace("TC-464", "FR-355-AC-1", "FR-355-AC-4")]
#[test]
fn every_operation_bearing_model_form_can_expose_its_declared_census() {
    for (kind, form, meaning) in [
        (
            "entity",
            "object_type",
            "quire.meaning.model.object-type/v1",
        ),
        (
            "interface",
            "systems_interface",
            "quire.meaning.systems.interface/v1",
        ),
        ("process", "process", "quire.meaning.model.process/v1"),
        (
            "machine",
            "state_machine",
            "quire.meaning.model.state-machine/v1",
        ),
        (
            "persistence",
            "persistence_interface",
            "quire.meaning.model.persistence-interface/v1",
        ),
    ] {
        let mut ty = if kind == "process" {
            process("Context", &[], &[])
        } else {
            typed_object("Context", kind, &[], &[])
        };
        let name = if kind == "process" { "advance" } else { "step" };
        ty["operations"] = json!([operation("Context", name)]);
        if kind == "persistence" {
            ty["persists"] = json!([orders("Stored")]);
        }
        if kind == "machine" {
            ty["states"] =
                json!([{"identity":format!("{}/ready",orders("Context")),"name":"ready"}]);
            ty["transitions"] = json!([{"identity":format!("{}/transition",orders("Context")),
                "from":format!("{}/ready",orders("Context")),"to":format!("{}/ready",orders("Context")),
                "trigger":format!("{}/step",orders("Context")),"emits":[]}]);
        }
        let mut types = vec![ty];
        if kind == "persistence" {
            let mut stored = object("Stored", &[], &[field("id", "ix://quire/native/UUID")]);
            stored["identityFields"] = json!([format!("{}/id", orders("Stored"))]);
            types.push(stored);
        }
        let mut doc = document(types);
        doc["constructs"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":{"module":IDENTITY,"name":kind},"construct":{"meaning":meaning}}));
        let (wire, evidence, _) = package(&doc, &[("Context", form)], &[]);
        let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("{form} fixture must admit: {other:?}"),
        };
        let id: CheckedNodeId =
            serde_json::from_value(node_id(&declaration_key("Context", form))).unwrap();
        let read = admitted.model_operations(&id, 10_000).unwrap();
        assert_eq!(read.operations().len(), 1, "{form}");
        assert_eq!(
            read.operations()[0].identity(),
            format!("{}/{name}", orders("Context"))
        );
        assert_eq!(read.operations()[0].declaring_node_id(), &id);
    }
}
