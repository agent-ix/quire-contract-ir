// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A genuine selected-model producer and strict-reader correspondence control.
//! QSpec's documents and vectors remain in the caller's checkout.

use qsl_foundation::{node_wire, SourceIdentity};
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{
    check::{
        wire_id::WireGraph, BindingValue, BodyTerm, LeafTerm, LiteralValue, MemberTerm,
        PackageDeclarations,
    },
    model::intake,
};
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedNodeId, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use quire_exact::{NodeKey, PathSegment};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};

const PACKAGE: &str = "std610";
const MODEL: &str = "acme/orders";

fn read(path: &Path) -> Value {
    serde_json::from_slice(
        &std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

// This expands references in the published vector's storage format only;
// it derives no language type or node identity.
fn published_key(keys: &Value, name: &str) -> NodeKey {
    fn expand(keys: &Value, name: &str) -> Value {
        let key = &keys[name];
        assert!(key.is_object(), "missing published key {name}");
        json!({"package":key["package"], "ordinal":key["ordinal"],
            "path":key["segments"].as_array().expect("published segments").iter().map(|part| {
                if let Some(reference) = part["node_key_ref"].as_str() {
                    expand(keys, reference)
                } else { part.clone() }
            }).collect::<Vec<_>>()})
    }
    node_wire::node_key_from_json(&expand(keys, name)).expect("published structural key")
}

fn selected_document(mut document: Value) -> Value {
    let order = document["types"]
        .as_array_mut()
        .expect("selected types")
        .iter_mut()
        .find(|declaration| declaration["identity"] == "ix://acme/orders/Order")
        .expect("published Order declaration");
    let fields = order["fields"].as_array_mut().expect("Order fields");
    let template = fields
        .iter()
        .find(|field| field["name"] == "floor")
        .expect("published Integer field")
        .clone();
    // FR-322 step 4's multiplicity truth table: these are ordinary authored
    // model fields, including the two kinds absent from the published Order.
    for (name, native, ordered, unique, upper) in [
        ("shape_boolean", "Boolean", false, true, 1),
        ("shape_integer", "Integer", false, true, 1),
        ("shape_set", "Integer", false, true, 8),
        ("shape_bag", "Integer", false, false, 8),
        ("shape_sequence", "Integer", true, false, 8),
        ("shape_ordered_set", "Integer", true, true, 8),
    ] {
        assert!(!fields.iter().any(|field| field["name"] == name));
        let mut field = template.clone();
        field["identity"] = json!(format!("ix://acme/orders/Order/{name}"));
        field["name"] = json!(name);
        field["typeRef"] = json!(format!("ix://quire/native/{native}"));
        field["constraints"] = json!([]);
        field["multiplicity"] = json!({"lower":if upper == 1 {1} else {0},
            "upper":upper, "ordered":ordered, "unique":unique});
        fields.push(field);
    }
    document
}

struct Shape {
    tag: &'static str,
    form: &'static str,
    key: NodeKey,
    semantic_type: NodeKey,
    body: ShapeBody,
}

enum ShapeBody {
    Empty,
    Reference(NodeKey),
    Bounds {
        integer: NodeKey,
        lower: &'static str,
        upper: &'static str,
    },
}

fn assert_producer_body(body: &BodyTerm, expected: &ShapeBody) {
    let BodyTerm::Aggregate(aggregate) = body else {
        panic!("derived body must be an aggregate: {body:?}");
    };
    match expected {
        ShapeBody::Empty => assert!(aggregate.members.is_empty()),
        ShapeBody::Reference(expected) => {
            let [MemberTerm::Leaf(LeafTerm::Reference { target })] = aggregate.members.as_slice()
            else {
                panic!("one exact type reference: {body:?}");
            };
            assert_eq!(target.admitted(), Some(expected));
        }
        ShapeBody::Bounds {
            integer,
            lower,
            upper,
        } => {
            assert_eq!(aggregate.members.len(), 2, "closed min/max body");
            for (member, name, expected) in [
                (&aggregate.members[0], "min", lower),
                (&aggregate.members[1], "max", upper),
            ] {
                let MemberTerm::Binding(binding) = member else {
                    panic!("bound binding: {body:?}");
                };
                assert_eq!(binding.name, name);
                let BindingValue::Leaf(LeafTerm::Literal {
                    ty,
                    value: LiteralValue::Integer(value),
                }) = &binding.value
                else {
                    panic!("exact typed integer literal: {body:?}");
                };
                assert_eq!(ty.admitted(), Some(integer));
                assert_eq!(value.to_string(), *expected);
            }
        }
    }
}

fn expected_body(body: &ShapeBody) -> Value {
    let members = match body {
        ShapeBody::Empty => Vec::new(),
        ShapeBody::Reference(target) => vec![json!({"term":"reference", "target":node_wire::node_key_to_json(target).expect("key representation")})],
        ShapeBody::Bounds { integer, lower, upper } => [("min", lower), ("max", upper)].into_iter()
            .map(|(name, value)| json!({"term":"binding", "name":name,
                "value":{"term":"literal", "type":node_wire::node_key_to_json(integer).expect("Integer representation"),
                    "value_kind":"integer", "value":value}})).collect(),
    };
    json!({"term":"aggregate", "members":members})
}

fn main() {
    let qspec =
        std::env::var_os("QUIRE_SPECIFICATION_DIR").expect("QSpec source checkout is required");
    let proposal = Path::new(&qspec).join("proposals/checked-package-v2");
    let vectors = read(&proposal.join("collection-population-type-vectors.json"));
    let members = read(&proposal.join("model-member-type-vectors.json"));
    assert_eq!(
        vectors["version"],
        "quire.collection-population-type-vectors/v1"
    );
    assert_eq!(members["version"], "quire.model-member-type-vectors/v1");
    let base = read(&proposal.join("fixtures/positive-all-families.json"));
    let published_range = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("published base nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "integer_range"
                && node["body"]["members"][0]["value"]["value"] == "0"
                && node["body"]["members"][1]["value"]["value"] == "9"
        })
        .expect("published member range [0,9]");
    assert_eq!(
        published_range["body"]["members"]
            .as_array()
            .expect("bounds")
            .len(),
        2
    );
    assert_eq!(published_range["body"]["members"][0]["name"], "min");
    assert_eq!(published_range["body"]["members"][1]["name"], "max");
    for (name, expected) in [
        ("MT-01", json!({"type":"Int", "min":"0", "max":"9"})),
        (
            "MT-03",
            json!({"type":"Option", "inner":{"type":"Boolean"}}),
        ),
    ] {
        let row = members["member_cases"]
            .as_array()
            .expect("member cases")
            .iter()
            .find(|row| row["name"] == name)
            .expect("published member oracle");
        assert_eq!(
            row["expected"]["member_type"], expected,
            "{name}: published selected-member type"
        );
    }
    let document = selected_document(read(&proposal.join("domain-package-acme-orders.json")));
    let document_bytes = serde_json::to_vec(&document).expect("selected document");
    let packages = intake::package_input([document_bytes.as_slice()]).expect("public model intake");
    let source = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"quire.value.complete/v1\";\n\
        model Domain = \"acme/orders\";\n\
        function read_boolean using v(o: Domain::Order): Boolean pure { o.shape_boolean }\n\
        function read_integer using v(o: Domain::Order): Integer pure { o.shape_integer }\n\
        function read_option using v(o: Domain::Order): Option<Boolean> pure { o.note }\n\
        function read_reference using v(o: Domain::Order): Domain::Invoice pure { o.invoice }\n\
        function read_range using v(o: Domain::Order): Int[0, 9] pure { o.total }\n\
        function read_set using v(o: Domain::Order): Set<Integer>[0, 8] pure { o.shape_set }\n\
        function read_bag using v(o: Domain::Order): Bag<Integer>[0, 8] pure { o.shape_bag }\n\
        function read_sequence using v(o: Domain::Order): Sequence<Integer>[0, 8] pure { o.shape_sequence }\n\
        function read_ordered_set using v(o: Domain::Order): OrderedSet<Integer>[0, 8] pure { o.shape_ordered_set }\n";
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "test:derived-shape-correspondence"),
        "derived-shapes.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("public parser");
    assert_eq!(parsed.diagnostics(), []);
    let unit = qsl_forms::build_unit(&parsed).expect("public forms");
    let models = intake::admit_unit(
        &unit.selections().models,
        &packages,
        qsl_semantics::model::accounting::ModelNormalizationLimits::default(),
    )
    .expect("selected model admission");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        PACKAGE.to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("public assembly")
    .check(CheckingLimits::default())
    .expect("genuine public checking");

    let integer = published_key(&vectors["node_keys"], "integer");
    // These three explicit expected paths are FR-322's native and option
    // rules; no expected key is obtained from IR admission or MemberType.
    let boolean = NodeKey::new(
        PACKAGE,
        vec![
            PathSegment::Text("scalar_type".into()),
            PathSegment::Text("boolean".into()),
        ],
        0,
    )
    .expect("Boolean key");
    let option = NodeKey::new(
        PACKAGE,
        vec![
            PathSegment::Text("composite_type".into()),
            PathSegment::Text("option".into()),
            PathSegment::Node(boolean.clone()),
        ],
        0,
    )
    .expect("Option key");
    let range = NodeKey::new(
        PACKAGE,
        vec![
            PathSegment::Text("bounded_domain".into()),
            PathSegment::Text("integer_range".into()),
            PathSegment::Text("min".into()),
            PathSegment::Node(integer.clone()),
            PathSegment::Text("0".into()),
            PathSegment::Text("max".into()),
            PathSegment::Text("9".into()),
        ],
        0,
    )
    .expect("range key");
    let reference_case = members["member_cases"]
        .as_array()
        .expect("member cases")
        .iter()
        .find(|case| case["name"] == "MT-06")
        .expect("published reference member");
    let invoice_id: CheckedNodeId =
        serde_json::from_value(reference_case["expected"]["member_type"]["target"].clone())
            .expect("published Invoice identity");
    let invoice =
        qsl_semantics::check::wire_id::node_key(&invoice_id).expect("declared reference target");
    let reference = NodeKey::new(
        PACKAGE,
        vec![
            PathSegment::Text("composite_type".into()),
            PathSegment::Text("reference".into()),
            PathSegment::Node(invoice.clone()),
        ],
        0,
    )
    .expect("Reference key");
    let set = published_key(&vectors["node_keys"], "set_integer");
    let bag = published_key(&vectors["node_keys"], "bag_integer");
    let sequence = published_key(&vectors["node_keys"], "sequence_integer");
    let ordered_set = published_key(&vectors["node_keys"], "ordered_set_integer");
    let bounds = published_key(&vectors["node_keys"], "set_integer_0_8");
    let shapes = [
        Shape {
            tag: "scalar_type",
            form: "boolean",
            key: boolean.clone(),
            semantic_type: boolean.clone(),
            body: ShapeBody::Empty,
        },
        Shape {
            tag: "scalar_type",
            form: "integer",
            key: integer.clone(),
            semantic_type: integer.clone(),
            body: ShapeBody::Empty,
        },
        Shape {
            tag: "composite_type",
            form: "reference",
            key: reference.clone(),
            semantic_type: reference.clone(),
            body: ShapeBody::Reference(invoice),
        },
        Shape {
            tag: "composite_type",
            form: "option",
            key: option.clone(),
            semantic_type: option.clone(),
            body: ShapeBody::Reference(boolean.clone()),
        },
        Shape {
            tag: "composite_type",
            form: "set",
            key: set.clone(),
            semantic_type: set.clone(),
            body: ShapeBody::Reference(integer.clone()),
        },
        Shape {
            tag: "composite_type",
            form: "bag",
            key: bag.clone(),
            semantic_type: bag.clone(),
            body: ShapeBody::Reference(integer.clone()),
        },
        Shape {
            tag: "composite_type",
            form: "sequence",
            key: sequence.clone(),
            semantic_type: sequence.clone(),
            body: ShapeBody::Reference(integer.clone()),
        },
        Shape {
            tag: "composite_type",
            form: "ordered_set",
            key: ordered_set.clone(),
            semantic_type: ordered_set.clone(),
            body: ShapeBody::Reference(integer.clone()),
        },
        Shape {
            tag: "bounded_domain",
            form: "integer_range",
            key: range.clone(),
            semantic_type: integer.clone(),
            body: ShapeBody::Bounds {
                integer: integer.clone(),
                lower: "0",
                upper: "9",
            },
        },
        Shape {
            tag: "bounded_domain",
            form: "collection_bounds",
            key: bounds.clone(),
            semantic_type: set,
            body: ShapeBody::Bounds {
                integer: integer.clone(),
                lower: "0",
                upper: "8",
            },
        },
    ];
    let mut witnessed = BTreeSet::new();
    for shape in &shapes {
        let node = graph
            .semantic_graph()
            .node(shape.key.clone())
            .unwrap_or_else(|| panic!("producer omitted {} at {:?}", shape.form, shape.key));
        assert_eq!(node.key(), shape.key);
        assert_eq!(node.semantic_form(), shape.form);
        assert_eq!(
            node.semantic_type().unwrap_or_else(|| node.key()),
            shape.semantic_type
        );
        assert!(node.declaration().is_none());
        assert!(node.owner().is_none());
        assert!(node.recursion().is_none());
        assert_producer_body(node.body(), &shape.body);
        assert!(witnessed.insert(shape.form));
    }
    assert_eq!(
        witnessed.len(),
        10,
        "all ten closed forms have real producer nodes"
    );
    let emission = emit_checked(&CheckedPackage::link(graph)).expect("genuine V2 emitter");
    assert_eq!(emission.omitted(), []);
    let wire: Value = serde_json::from_slice(emission.package().bytes()).expect("emitted V2");
    let ids = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("emitted nodes")
        .iter()
        .map(|node| {
            serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).expect("emitted ID")
        })
        .collect::<Vec<_>>();
    let frame = WireGraph::new(&ids).expect("producer's public indexed representation");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(MODEL, &document_bytes)
        .expect("same selected document");
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    let CheckedPackageV2ReadResult::Admitted(admitted) = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("genuine producer package must pass strict admission");
    };
    for shape in &shapes {
        let position = usize::try_from(frame.index_of(&shape.key).expect("emitted expected key"))
            .expect("graph position");
        let produced = &wire["semantic_graph"]["nodes"][position];
        assert_eq!(produced["node_tag"], shape.tag);
        assert_eq!(produced["semantic_form"], shape.form);
        assert_eq!(
            frame
                .key_of_value(&produced["node_id"])
                .expect("produced key"),
            shape.key
        );
        assert_eq!(
            frame
                .key_of_value(&produced["semantic_type"])
                .expect("produced type"),
            shape.semantic_type
        );
        assert_eq!(
            frame
                .structural_value(&produced["body"])
                .expect("produced body representation"),
            expected_body(&shape.body)
        );
        let retained = &admitted.graph().nodes[position];
        assert_eq!(retained.node_id, ids[position]);
        assert_eq!(
            qsl_semantics::check::wire_id::node_key(
                admitted
                    .structural_key_at(position)
                    .expect("reader structural key")
            )
            .expect("key representation"),
            shape.key
        );
        assert_eq!(
            frame.key(&retained.semantic_type).expect("retained type"),
            shape.semantic_type
        );
        assert_eq!(
            frame
                .structural_value(&serde_json::to_value(&retained.body).expect("retained body"))
                .expect("retained body representation"),
            expected_body(&shape.body)
        );
    }
    let order = admitted.graph().nodes.iter().find(|node| node.semantic_form.as_ref() == "object_type"
        && node.node_id.path.last().is_some_and(|part| matches!(part, quire_contract_model::CheckedNodePathSegment::Name(name) if name.as_ref() == "ix://acme/orders/Order")))
        .expect("selected Order declaration");
    let fields = admitted
        .model_object_fields(&order.node_id)
        .expect("selected declaration's effective member types");
    let expected_members = [
        ("shape_boolean", CheckedMemberType::Boolean, boolean),
        ("shape_integer", CheckedMemberType::Integer, integer),
        (
            "note",
            CheckedMemberType::Option(Box::new(CheckedMemberType::Boolean)),
            option,
        ),
        (
            "invoice",
            CheckedMemberType::Reference(invoice_id),
            reference,
        ),
        (
            "total",
            CheckedMemberType::IntRange { lower: 0, upper: 9 },
            range,
        ),
        (
            "shape_set",
            CheckedMemberType::Collection {
                kind: CheckedCollectionKind::Set,
                element: Box::new(CheckedMemberType::Integer),
                bounds: Some((0, 8)),
            },
            bounds,
        ),
        (
            "shape_bag",
            CheckedMemberType::Collection {
                kind: CheckedCollectionKind::Bag,
                element: Box::new(CheckedMemberType::Integer),
                bounds: Some((0, 8)),
            },
            published_key(&vectors["node_keys"], "bag_integer_0_8"),
        ),
        (
            "shape_sequence",
            CheckedMemberType::Collection {
                kind: CheckedCollectionKind::Sequence,
                element: Box::new(CheckedMemberType::Integer),
                bounds: Some((0, 8)),
            },
            published_key(&vectors["node_keys"], "sequence_integer_0_8"),
        ),
        (
            "shape_ordered_set",
            CheckedMemberType::Collection {
                kind: CheckedCollectionKind::OrderedSet,
                element: Box::new(CheckedMemberType::Integer),
                bounds: Some((0, 8)),
            },
            published_key(&vectors["node_keys"], "ordered_set_integer_0_8"),
        ),
    ];
    for (name, expected, type_key) in expected_members {
        assert_eq!(
            fields.field(name).expect("selected field").member_type(),
            Some(&expected),
            "{name}: independently specified member type"
        );
        let projections = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|node| {
                node["body"]["operation"]["identity"] == "quire.op.record.project"
                    && node["body"]["operation"]["member"]["name"] == name
            })
            .collect::<Vec<_>>();
        let [projection] = projections.as_slice() else {
            panic!("{name}: genuine selected-member projection must emit exactly once");
        };
        assert_eq!(
            frame
                .key_of_value(&projection["body"]["result_type"])
                .expect("projection result type"),
            type_key,
            "{name}: selected field to producer type"
        );
        assert_eq!(
            frame
                .key_of_value(&projection["semantic_type"])
                .expect("projection semantic type"),
            type_key
        );
        let projection_id: CheckedNodeId =
            serde_json::from_value(projection["node_id"].clone()).expect("projection ID");
        let retained = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == projection_id)
            .expect("admitted projection");
        assert_eq!(
            frame
                .key(&retained.semantic_type)
                .expect("reader projection type"),
            type_key,
            "{name}: strict reader type"
        );
    }
    println!("FR-038-AC-130 genuine selected-member/producer/strict-reader correspondence passed for all ten derived shapes");
}
