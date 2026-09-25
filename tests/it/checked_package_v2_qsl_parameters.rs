// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-280: the V2 reader admits the node shapes QSL's S4 emitter writes for
//! a function that takes parameters (`value`/`parameter`, QSL FR-092) and
//! for a compound-unit quantity type (`scalar_type`/`compound_unit`, QSL
//! FR-094), and checks FR-322's application-node dependency join.
//!
//! The package below is built here, node by node, in the shapes QSL's
//! FR-092 golden vectors fix for `function both(a: Boolean, b: Boolean):
//! Boolean { a and b }`. The keys of the nodes are recomputed from those
//! shapes and compared with the FR-092 vector digests, so the fixture is
//! checked against QSL's own output rather than against this reader.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_fixture_members, nominal_package, rebuild_source_map,
    refresh_identity, refusal, refusal_at, sha256_hex,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2,
};
use serde_json::{json, Value};

type Mutation = Box<dyn Fn(&mut Value)>;

const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const APPLICATION_NODE: &str = "quire.application-node/v1";
// Pointer suffixes below `/semantic_graph/nodes/{n}`: each structural
// refusal names the member it is about.
const BODY: &str = "/body";
const NAME_BINDING: &str = "/body/members/0";
const LEVEL_BINDING: &str = "/body/members/1";
const FIRST_TERM: &str = "/body/members/0";
const SECOND_TERM: &str = "/body/members/1";
const DEPENDENCIES: &str = "/dependencies";
const SEMANTIC_TYPE: &str = "/semantic_type";

/// QSL FR-092 golden vector keys (quire-spec-language
/// `spec/functional/FR-092-key-type-parameter-and-declared-nodes.md`).
const T1_BOOLEAN: &str = "9964390677844ad66b781babdbfa95933bc2b16ef1e86f67005966b77e6db3aa";
const T2_INTEGER: &str = "07f6dca966d22bde13d3bb198f12610e57d8e1e04d0476bbab03f405d2b04e32";
const T3_TEXT: &str = "0062ac9eee212061689152b5b0d62c5b3a6ca69ce83c1897b8c0624837ab5659";
const P1_A: &str = "838088fb2300dd016cf10707e297afbd2f6209eb7e20c24757d515fda8ee6cf1";
const P2_B: &str = "555416913f6f787765eeb816c83af4650d7a0e122b8e3202f8b8d13b144fef59";
const E1_A_AND_B: &str = "a98896386ccee595ae1cc04f4f83c8792fc04e1027e779ac7def428592c8b21e";
const F2_BOTH: &str = "2597b9bf514c3dd93654daca8fbea64d0a4622ea8bc5002888ce72ecb2520454";

/// A node's key under QSL's `quire.structural-node/v1` preimage (FR-092).
/// `semantic_type` is `None` for a self-typed node.
fn structural_key(
    tag: &str,
    form: &str,
    semantic_type: Option<&str>,
    declaration: Option<(&[&str], Value)>,
    body: &Value,
) -> String {
    let mut preimage = json!({
        "version": STRUCTURAL_NODE,
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": semantic_type.map(node_id),
        "declaration": declaration.as_ref().map(|(name, _)| json!({"qualified_name": name})),
        "recursion": null,
        "body": body,
    });
    if let Some((_, owner)) = declaration {
        preimage["owner"] = owner;
    }
    sha256_hex(&canonical(&preimage))
}

/// FR-322's `quire.application-node/v1` key of a node outside any group.
fn application_key(tag: &str, form: &str, semantic_type: &str, body: &Value) -> String {
    sha256_hex(&canonical(&json!({
        "version": APPLICATION_NODE,
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "declaration": null,
        "recursion": null,
        "body": body,
    })))
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(ty), "value_kind": value_kind, "value": value})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn scalar(form: &str) -> (String, Value) {
    let body = json!({"term": "aggregate", "members": []});
    let key = structural_key("scalar_type", form, None, None, &body);
    let node = wire_node(&key, "scalar_type", form, &key, &[], "type", body);
    (key, node)
}

fn parameter_body(name: &str, level: &str) -> Value {
    json!({"term": "aggregate", "members": [
        binding("name", literal(T3_TEXT, "text", name)),
        binding("level", literal(T2_INTEGER, "integer", level)),
    ]})
}

fn parameter(name: &str, level: &str) -> (String, Value) {
    let body = parameter_body(name, level);
    let key = structural_key("value", "parameter", Some(T1_BOOLEAN), None, &body);
    let node = wire_node(&key, "value", "parameter", T1_BOOLEAN, &[], "anchor", body);
    (key, node)
}

fn compound_unit_body(unit: &str, exponent: &str) -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("unit", reference(unit)),
            binding("exponent", literal(T2_INTEGER, "integer", exponent)),
        ]},
    ]})
}

/// Positions of the QSL-shaped nodes in [`qsl_package`]'s graph.
const DIMENSION: usize = 0;
const METRE: usize = 1;
const SECOND: usize = 3;
const KILOMETRE: usize = 4;
const P1: usize = 8;
const E1: usize = 10;
const COMPOUND: usize = 12;

/// Nominal `(preimage, key)` pairs beside the fixture's `Length`/`Metre`:
/// the base dimension `Time` with its root unit `Second`, and `Kilometre`,
/// a non-root unit of `Length` whose target is `Metre`.
fn extra_units(dimension: &Value, metre: &Value, metre_key: &str) -> Vec<(Value, String)> {
    let keyed = |preimage: Value| {
        let key = sha256_hex(&canonical(&preimage));
        (preimage, key)
    };
    let time = keyed(json!({
        "version": "quire.dimension-node/v1",
        "owner": dimension["owner"],
        "qualified_declaration": ["Example", "Time"],
        "terms": [],
    }));
    let second = keyed(json!({
        "version": "quire.unit-node/v1",
        "owner": metre["owner"],
        "qualified_declaration": ["Example", "Second"],
        "dimension_node_id": node_id(&time.1),
        "target_unit_node_id": null,
        "scale": {"numerator": "1", "denominator": "1"},
        "offset": {"numerator": "0", "denominator": "1"},
    }));
    let kilometre = keyed(json!({
        "version": "quire.unit-node/v1",
        "owner": metre["owner"],
        "qualified_declaration": ["Example", "Kilometre"],
        "dimension_node_id": metre["dimension_node_id"],
        "target_unit_node_id": node_id(metre_key),
        "scale": {"numerator": "1000", "denominator": "1"},
        "offset": {"numerator": "0", "denominator": "1"},
    }));
    vec![time, second, kilometre]
}

/// A QSL-shaped V2 package: the fixture's `Length` dimension and root unit
/// `Metre`, the [`extra_units`], the Boolean, Integer and text scalars, the parameters `a` and
/// `b`, the expression `a and b`, the function `both` that takes them, and
/// the compound unit `Metre^2`.
fn qsl_package() -> Value {
    let members = nominal_fixture_members();
    // [2] = unit `Metre`, [3] = dimension `Length`.
    let mut nominal = vec![members[3].clone(), members[2].clone()];
    nominal.extend(extra_units(&members[3].0, &members[2].0, &members[2].1));
    let mut package = nominal_package(&nominal);
    let metre = members[2].1.clone();

    let (boolean, boolean_node) = scalar("boolean");
    let (integer, integer_node) = scalar("integer");
    let (text, text_node) = scalar("text");
    let (a, a_node) = parameter("a", "0");
    let (b, b_node) = parameter("b", "1");

    let and_body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.boolean.and", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(T1_BOOLEAN),
        "arguments": [reference(&a), reference(&b)],
    });
    let and = application_key("expression", "binary", T1_BOOLEAN, &and_body);
    let mut and_dependencies = [a.as_str(), b.as_str()];
    and_dependencies.sort_unstable();
    let and_node = wire_node(
        &and,
        "expression",
        "binary",
        T1_BOOLEAN,
        &and_dependencies,
        "expression",
        and_body,
    );

    let both_body = json!({"term": "aggregate", "members": [
        binding("parameters", json!({"term": "aggregate", "members": [reference(&a), reference(&b)]})),
        binding("body", reference(&and)),
    ]});
    let owner = json!({"kind": "source", "authority": "a", "identity": "u"});
    let both = structural_key(
        "function",
        "pure_function",
        Some(T1_BOOLEAN),
        Some((&["both"], owner)),
        &both_body,
    );
    let mut both_dependencies = [a.as_str(), b.as_str(), and.as_str()];
    both_dependencies.sort_unstable();
    let mut both_node = wire_node(
        &both,
        "function",
        "pure_function",
        T1_BOOLEAN,
        &both_dependencies,
        "declaration",
        both_body,
    );
    both_node["declaration"] = json!({"qualified_name": ["both"]});

    let compound_body = compound_unit_body(&metre, "2");
    let compound = structural_key("scalar_type", "compound_unit", None, None, &compound_body);
    let compound_node = wire_node(
        &compound,
        "scalar_type",
        "compound_unit",
        &compound,
        &[metre.as_str()],
        "type",
        compound_body,
    );

    // The recomputed keys are QSL's FR-092 vectors: this is QSL's output.
    assert_eq!(
        [&boolean, &integer, &text, &a, &b, &and, &both],
        [T1_BOOLEAN, T2_INTEGER, T3_TEXT, P1_A, P2_B, E1_A_AND_B, F2_BOTH]
    );

    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    nodes.extend([
        boolean_node,
        integer_node,
        text_node,
        a_node,
        b_node,
        and_node,
        both_node,
        compound_node,
    ]);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

fn node_key(package: &Value, position: usize) -> String {
    package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("node digest")
        .to_owned()
}

/// Applies `mutate` to the node at `position`, then re-derives the source
/// map and the package identity so only the mutated rule can refuse.
fn mutated(position: usize, mutate: impl Fn(&mut Value)) -> Value {
    let mut package = qsl_package();
    mutate(&mut package["semantic_graph"]["nodes"][position]);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

/// `invalid_semantic_graph` at the node at `position`, pointing at `member`
/// (a pointer suffix below that node).
fn invalid_at(package: &Value, position: usize, member: &str) -> CheckedPackageRefusal {
    refusal_at(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &format!("/semantic_graph/nodes/{position}{member}"),
        None,
        &node_key(package, position),
    )
}

/// Tracing: TC-048, FR-038-AC-22, FR-038-AC-23
#[trace("TC-048", "FR-038-AC-22", "FR-038-AC-23")]
#[test]
fn tc_048_a_qsl_function_with_parameters_reads_and_lowers() {
    let package = qsl_package();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!(
            "the QSL-shaped package must admit, got {:?}",
            read(&package)
        );
    };
    let forms = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| (node.node_tag.as_ref(), node.semantic_form.as_ref()))
        .collect::<Vec<_>>();
    assert!(forms.contains(&("value", "parameter")));
    assert!(forms.contains(&("scalar_type", "compound_unit")));

    let every_node = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let lowered = admitted.lower(
        &every_node,
        &CompleteLoweringProfileV2 {
            supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
            require_bounds: false,
            work_limit: 10_000,
        },
    );
    for record in &lowered.records {
        assert!(
            matches!(record, CompleteLoweringRecordV2::Lowered { .. }),
            "{record:?}"
        );
    }
}

/// Tracing: TC-048, FR-038-AC-22
#[trace("TC-048", "FR-038-AC-22")]
#[test]
fn tc_048_a_malformed_parameter_node_refuses() {
    let integer_literal_name = |node: &mut Value| {
        node["body"]["members"][0]["value"]["type"] = node_id(T2_INTEGER);
    };
    // A body that is not the two-binding aggregate refuses at the body; a
    // defective binding refuses at that binding.
    let body_cases: [(&str, Mutation, &str); 9] = [
        (
            "reference body",
            Box::new(|node| node["body"] = reference(T1_BOOLEAN)),
            BODY,
        ),
        (
            "literal body",
            Box::new(|node| node["body"] = literal(T3_TEXT, "text", "a")),
            BODY,
        ),
        (
            "negative level",
            Box::new(|node| node["body"] = parameter_body("a", "-1")),
            LEVEL_BINDING,
        ),
        (
            "non-canonical level",
            Box::new(|node| node["body"] = parameter_body("a", "00")),
            LEVEL_BINDING,
        ),
        (
            "empty name",
            Box::new(|node| node["body"] = parameter_body("", "0")),
            NAME_BINDING,
        ),
        (
            "missing level",
            Box::new(|node| {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .pop();
            }),
            BODY,
        ),
        (
            "bindings swapped",
            Box::new(|node| {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .reverse();
            }),
            NAME_BINDING,
        ),
        (
            "name typed at Integer",
            Box::new(integer_literal_name),
            NAME_BINDING,
        ),
        (
            "extra binding",
            Box::new(|node| {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .push(binding("level", literal(T2_INTEGER, "integer", "2")));
            }),
            BODY,
        ),
    ];
    for (case, mutate, member) in body_cases {
        let package = mutated(P1, mutate);
        assert_eq!(
            refused(&package),
            invalid_at(&package, P1, member),
            "{case}"
        );
    }

    let with_dependency = mutated(P1, |node| {
        node["dependencies"] = json!([node_id(T1_BOOLEAN)]);
    });
    assert_eq!(
        refused(&with_dependency),
        invalid_at(&with_dependency, P1, DEPENDENCIES)
    );

    let self_typed = mutated(P1, |node| node["semantic_type"] = node["node_id"].clone());
    assert_eq!(
        refused(&self_typed),
        invalid_at(&self_typed, P1, SEMANTIC_TYPE)
    );

    // A parameter declares no name, even with a `declaration` occurrence.
    let declared = mutated(P1, |node| {
        node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
        node["declaration"] = json!({"qualified_name": ["a"]});
    });
    assert_eq!(
        refused(&declared),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{P1}/declaration")
        )
    );
}

/// Tracing: TC-048, FR-038-AC-22
#[trace("TC-048", "FR-038-AC-22")]
#[test]
fn tc_048_a_malformed_compound_unit_node_refuses() {
    let metre = node_key(&qsl_package(), METRE);
    let dimension = node_key(&qsl_package(), DIMENSION);

    let zero_exponent = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&metre, "0");
    });
    assert_eq!(
        refused(&zero_exponent),
        invalid_at(&zero_exponent, COMPOUND, FIRST_TERM)
    );

    // A term names a root unit, never a dimension.
    let names_dimension = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&dimension, "2");
        node["dependencies"] = json!([node_id(&dimension)]);
    });
    assert_eq!(
        refused(&names_dimension),
        invalid_at(&names_dimension, COMPOUND, FIRST_TERM)
    );

    let repeated_unit = mutated(COMPOUND, |node| {
        let term = node["body"]["members"][0].clone();
        node["body"]["members"]
            .as_array_mut()
            .expect("members")
            .push(term);
    });
    assert_eq!(
        refused(&repeated_unit),
        invalid_at(&repeated_unit, COMPOUND, SECOND_TERM)
    );

    let undeclared_dependency = mutated(COMPOUND, |node| node["dependencies"] = json!([]));
    assert_eq!(
        refused(&undeclared_dependency),
        invalid_at(&undeclared_dependency, COMPOUND, DEPENDENCIES)
    );

    let typed_by_another = mutated(COMPOUND, |node| {
        node["semantic_type"] = node_id(T2_INTEGER);
    });
    assert_eq!(
        refused(&typed_by_another),
        invalid_at(&typed_by_another, COMPOUND, SEMANTIC_TYPE)
    );

    // Terms naming distinct root units must still ascend by unit key.
    let second = node_key(&qsl_package(), SECOND);
    let (low, high) = if metre < second {
        (metre.clone(), second.clone())
    } else {
        (second.clone(), metre.clone())
    };
    let two_terms = |first: &str, second: &str| {
        let mut body = compound_unit_body(first, "1");
        let term = compound_unit_body(second, "-1")["members"][0].clone();
        body["members"].as_array_mut().expect("members").push(term);
        body
    };
    let ascending = mutated(COMPOUND, |node| {
        node["body"] = two_terms(&low, &high);
        node["dependencies"] = json!([node_id(&low), node_id(&high)]);
    });
    assert!(matches!(
        read(&ascending),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let descending = mutated(COMPOUND, |node| {
        node["body"] = two_terms(&high, &low);
        node["dependencies"] = json!([node_id(&high), node_id(&low)]);
    });
    assert_eq!(
        refused(&descending),
        invalid_at(&descending, COMPOUND, SECOND_TERM)
    );

    // A term names a root unit, never a unit with a target.
    let kilometre = node_key(&qsl_package(), KILOMETRE);
    let names_non_root = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&kilometre, "2");
        node["dependencies"] = json!([node_id(&kilometre)]);
    });
    assert_eq!(
        refused(&names_non_root),
        invalid_at(&names_non_root, COMPOUND, FIRST_TERM)
    );

    // The empty body is the dimensionless unit, with no dependencies.
    let dimensionless = mutated(COMPOUND, |node| {
        node["body"] = json!({"term": "aggregate", "members": []});
        node["dependencies"] = json!([]);
    });
    assert!(matches!(
        read(&dimensionless),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-23
#[trace("TC-048", "FR-038-AC-23")]
#[test]
fn tc_048_an_application_node_dependency_list_is_its_exact_body_join() {
    let (a, b) = (
        node_key(&qsl_package(), P1),
        node_key(&qsl_package(), P1 + 1),
    );
    let cases: [(&str, Value); 4] = [
        ("missing target", json!([node_id(&a)])),
        (
            "not digest-ascending",
            json!([
                node_id(&b.clone().max(a.clone())),
                node_id(&b.clone().min(a.clone()))
            ]),
        ),
        (
            "result type is not a dependency",
            json!([node_id(T1_BOOLEAN), node_id(&a), node_id(&b)]),
        ),
        (
            "repeated target",
            json!([node_id(&a), node_id(&a), node_id(&b)]),
        ),
    ];
    for (case, dependencies) in cases {
        let package = mutated(E1, |node| node["dependencies"] = dependencies.clone());
        assert_eq!(
            refused(&package),
            invalid_at(&package, E1, DEPENDENCIES),
            "{case}"
        );
    }
}
