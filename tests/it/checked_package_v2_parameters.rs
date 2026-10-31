// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-280: the V2 reader admits the node shapes QSL's S4 emitter writes for
//! a function that takes parameters (`value`/`parameter`, QSL FR-092) and
//! for a compound-unit quantity type (`scalar_type`/`compound_unit`, QSL
//! FR-094), and checks FR-322's application-node dependency join.
//!
//! The package below is built here, node by node, in QSL FR-092's shapes for
//! `function both(a: Boolean, b: Boolean): Boolean { a and b }`, each node
//! assigned its own structural path.

use crate::support::checked_package::{
    canonical, evidence_for, fixture_source, index_fixture_node_ids, node_id,
    nominal_fixture_members, nominal_package, rebuild_source_map, refusal, refusal_at_id,
    source_owner, ExpectedRefusal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedParameterDeclaration, CheckedParameterDeclarationError,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};
use std::sync::LazyLock;

type Mutation = Box<dyn Fn(&mut Value)>;

// Pointer suffixes below `/semantic_graph/nodes/{n}`: each structural
// refusal names the member it is about.
const BODY: &str = "/body";
const DEPENDENCIES: &str = "/dependencies";
const SEMANTIC_TYPE: &str = "/semantic_type";

/// The self-typed scalar type nodes the package's literals and parameters
/// are typed at.
static BOOLEAN: LazyLock<String> = LazyLock::new(|| scalar("boolean").0);
static INTEGER: LazyLock<String> = LazyLock::new(|| scalar("integer").0);
static TEXT: LazyLock<String> = LazyLock::new(|| scalar("text").0);

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
    let key = format!("scalar_type/{form}");
    let node = wire_node(&key, "scalar_type", form, &key, &[], "type", body);
    (key, node)
}

fn parameter_body(name: &str, level: &str) -> Value {
    json!({"term": "aggregate", "members": [
        binding("name", literal(&TEXT, "text", name)),
        binding("level", literal(&INTEGER, "integer", level)),
    ]})
}

fn parameter(name: &str, level: &str) -> (String, Value) {
    let body = parameter_body(name, level);
    let key = format!("value/parameter/{name}/{level}");
    let node = wire_node(
        &key,
        "value",
        "parameter",
        &BOOLEAN,
        &[],
        "expression",
        body,
    );
    (key, node)
}

fn compound_unit_body(unit: &str, exponent: &str) -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("unit", reference(unit)),
            binding("exponent", literal(&INTEGER, "integer", exponent)),
        ]},
    ]})
}

fn set_compound_id(node: &mut Value, terms: &[(&str, &str)]) {
    let mut path = vec![json!("scalar_type"), json!("compound_unit")];
    for &(unit, exponent) in terms {
        path.push(node_id(unit));
        path.push(json!(exponent));
    }
    let id = json!({
        "package": node["node_id"]["package"],
        "path": path,
        "ordinal": node["node_id"]["ordinal"],
    });
    node["node_id"] = id.clone();
    node["semantic_type"] = id;
}

/// Positions of the QSL-shaped nodes in [`parameters_package`]'s graph.
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
        let key = format!(
            "qualified:{}",
            preimage["qualified_declaration"]
                .as_array()
                .expect("nominal declaration")
                .iter()
                .map(|part| part.as_str().expect("path segment"))
                .collect::<Vec<_>>()
                .join("/")
        );
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
fn parameters_package() -> Value {
    let members = nominal_fixture_members();
    // [2] = unit `Metre`, [3] = dimension `Length`.
    let mut nominal = vec![members[3].clone(), members[2].clone()];
    nominal.extend(extra_units(&members[3].0, &members[2].0, &members[2].1));
    let mut package = nominal_package(&nominal);
    let metre = members[2].1.clone();

    let (_, boolean_node) = scalar("boolean");
    let (_, integer_node) = scalar("integer");
    let (_, text_node) = scalar("text");
    let (a, a_node) = parameter("a", "0");
    let (b, b_node) = parameter("b", "1");

    let and_body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.boolean.and", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&BOOLEAN),
        "arguments": [reference(&a), reference(&b)],
    });
    let and = "expression/binary/and".to_owned();
    let mut and_dependencies = [a.as_str(), b.as_str()];
    and_dependencies.sort_unstable();
    let and_node = wire_node(
        &and,
        "expression",
        "binary",
        &BOOLEAN,
        &and_dependencies,
        "expression",
        and_body,
    );

    let both_body = json!({"term": "aggregate", "members": [
        binding("parameters", json!({"term": "aggregate", "members": [reference(&a), reference(&b)]})),
        binding("body", reference(&and)),
    ]});
    let owner = source_owner(&fixture_source());
    let both = "qualified:both".to_owned();
    let mut both_dependencies = [a.as_str(), b.as_str(), and.as_str()];
    both_dependencies.sort_unstable();
    let mut both_node = wire_node(
        &both,
        "function",
        "pure_function",
        &BOOLEAN,
        &both_dependencies,
        "declaration",
        both_body,
    );
    both_node["declaration"] = json!({"qualified_name": ["both"]});
    both_node["owner"] = owner;

    let compound_body = compound_unit_body(&metre, "2");
    let compound = "scalar_type/compound_unit/metre_squared".to_owned();
    let mut compound_node = wire_node(
        &compound,
        "scalar_type",
        "compound_unit",
        &compound,
        &[metre.as_str()],
        "type",
        compound_body,
    );
    set_compound_id(&mut compound_node, &[(metre.as_str(), "2")]);

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
    package
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    let mut wire = value.clone();
    index_fixture_node_ids(&mut wire);
    CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    )
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

fn node_key(package: &Value, position: usize) -> String {
    let path = package["semantic_graph"]["nodes"][position]["node_id"]["path"]
        .as_array()
        .expect("node path");
    let names = path
        .iter()
        .map(|segment| segment.as_str().expect("name segment"))
        .collect::<Vec<_>>();
    if names.len() == 1 {
        names[0].to_owned()
    } else {
        format!("qualified:{}", names.join("/"))
    }
}

/// Applies `mutate` to the node at `position`, then rebuilds its source map
/// so only the mutated rule can refuse.
fn mutated(position: usize, mutate: impl Fn(&mut Value)) -> Value {
    let mut package = parameters_package();
    mutate(&mut package["semantic_graph"]["nodes"][position]);
    rebuild_source_map(&mut package);
    package
}

/// `invalid_semantic_graph` at the node at `position`, pointing at `member`
/// (a pointer suffix below that node).
fn invalid_at(package: &Value, position: usize, member: &str) -> ExpectedRefusal {
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    refusal_at_id(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &format!("/semantic_graph/nodes/{position}{member}"),
        None,
        &wire["semantic_graph"]["nodes"][position]["node_id"],
    )
}

fn invalid_derived_at(package: &Value, position: usize) -> ExpectedRefusal {
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    refusal_at_id(
        CheckedPackageRefusalCode::InvalidPackage,
        &format!("/semantic_graph/nodes/{position}/node_id"),
        Some(CheckedPackageRefusalCause::InvalidDerivedShape),
        &wire["semantic_graph"]["nodes"][position]["node_id"],
    )
}

/// Compound-unit body rules remain independent of structural node paths.
#[test]
fn compound_unit_references_and_exponents_are_validated() {
    let independent = mutated(COMPOUND, |node| {
        node["body"]["members"][0]["members"][1]["value"]["value"] = json!("0");
    });
    assert_eq!(
        refused(&independent),
        invalid_derived_at(&independent, COMPOUND),
    );

    let kilometre = node_key(&parameters_package(), KILOMETRE);
    let names_non_root = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&kilometre, "2");
        node["dependencies"] = json!([node_id(&kilometre)]);
    });
    assert_eq!(
        refused(&names_non_root),
        invalid_derived_at(&names_non_root, COMPOUND)
    );

    let dimensionless = mutated(COMPOUND, |node| {
        node["body"] = json!({"term": "aggregate", "members": []});
        node["dependencies"] = json!([]);
        set_compound_id(node, &[]);
    });
    assert!(matches!(
        read(&dimensionless),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048", "TC-050", "FR-038-AC-22", "FR-038-AC-23", "FR-038-AC-308")]
#[test]
fn tc_048_a_function_with_parameters_reads_and_lowers() {
    let package = parameters_package();
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
    let profile = CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: 10_000,
    };
    let lowered = admitted.lower(&every_node, &profile);
    for record in &lowered.records {
        assert!(
            matches!(record, CompleteLoweringRecordV2::Lowered { .. }),
            "{record:?}"
        );
    }
    for (request, record) in every_node.iter().zip(&lowered.records) {
        let single = admitted.lower(std::slice::from_ref(request), &profile);
        assert_eq!(record, &single.records[0]);
    }
    let mut stopped = profile;
    stopped.work_limit = 4;
    let batch_stopped = admitted.lower(&every_node, &stopped);
    for (request, record) in every_node.iter().zip(&batch_stopped.records) {
        let single = admitted.lower(std::slice::from_ref(request), &stopped);
        assert_eq!(record, &single.records[0]);
    }
}

#[trace("TC-048", "FR-038-AC-22")]
#[test]
fn tc_048_a_malformed_parameter_node_refuses() {
    let integer_literal_name = |node: &mut Value| {
        node["body"]["members"][0]["value"]["type"] = node_id(&INTEGER);
    };
    // A body that is not the two-binding aggregate, and a defective binding,
    // each refuse at the body.
    let body_cases: [(&str, Mutation, &str); 10] = [
        (
            "reference body",
            Box::new(|node| node["body"] = reference(&BOOLEAN)),
            BODY,
        ),
        (
            "literal body",
            Box::new(|node| node["body"] = literal(&TEXT, "text", "a")),
            BODY,
        ),
        (
            "negative level",
            Box::new(|node| node["body"] = parameter_body("a", "-1")),
            BODY,
        ),
        (
            "non-canonical level",
            Box::new(|node| node["body"] = parameter_body("a", "00")),
            BODY,
        ),
        (
            "empty name",
            Box::new(|node| node["body"] = parameter_body("", "0")),
            BODY,
        ),
        (
            "name that is not an identifier",
            Box::new(|node| node["body"] = parameter_body("a b", "0")),
            BODY,
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
            BODY,
        ),
        (
            "name typed at Integer",
            Box::new(integer_literal_name),
            BODY,
        ),
        (
            "extra binding",
            Box::new(|node| {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .push(binding("level", literal(&INTEGER, "integer", "2")));
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
        node["dependencies"] = json!([node_id(&BOOLEAN)]);
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
        node["owner"] = source_owner(&fixture_source());
    });
    assert_eq!(
        refused(&declared),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{P1}/declaration")
        )
    );
}

#[trace("TC-048", "FR-038-AC-22")]
#[test]
fn tc_048_a_malformed_compound_unit_node_refuses() {
    let metre = node_key(&parameters_package(), METRE);
    let dimension = node_key(&parameters_package(), DIMENSION);

    let zero_exponent = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&metre, "0");
    });
    assert_eq!(
        refused(&zero_exponent),
        invalid_derived_at(&zero_exponent, COMPOUND)
    );

    // A term names a root unit, never a dimension.
    let names_dimension = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&dimension, "2");
        node["dependencies"] = json!([node_id(&dimension)]);
    });
    assert_eq!(
        refused(&names_dimension),
        invalid_derived_at(&names_dimension, COMPOUND)
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
        invalid_at(&repeated_unit, COMPOUND, DEPENDENCIES)
    );

    let undeclared_dependency = mutated(COMPOUND, |node| node["dependencies"] = json!([]));
    assert_eq!(
        refused(&undeclared_dependency),
        invalid_at(&undeclared_dependency, COMPOUND, DEPENDENCIES)
    );

    let typed_by_another = mutated(COMPOUND, |node| {
        node["semantic_type"] = node_id(&INTEGER);
    });
    assert_eq!(
        refused(&typed_by_another),
        invalid_derived_at(&typed_by_another, COMPOUND)
    );

    // Terms naming distinct root units must still ascend by unit key.
    let second = node_key(&parameters_package(), SECOND);
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
        set_compound_id(node, &[(low.as_str(), "1"), (high.as_str(), "-1")]);
    });
    assert!(matches!(
        read(&ascending),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let descending = mutated(COMPOUND, |node| {
        node["body"] = two_terms(&high, &low);
        node["dependencies"] = json!([node_id(&high), node_id(&low)]);
        set_compound_id(node, &[(high.as_str(), "1"), (low.as_str(), "-1")]);
    });
    assert_eq!(
        refused(&descending),
        invalid_derived_at(&descending, COMPOUND)
    );

    // A term names a root unit, never a unit with a target.
    let kilometre = node_key(&parameters_package(), KILOMETRE);
    let names_non_root = mutated(COMPOUND, |node| {
        node["body"] = compound_unit_body(&kilometre, "2");
        node["dependencies"] = json!([node_id(&kilometre)]);
    });
    assert_eq!(
        refused(&names_non_root),
        invalid_derived_at(&names_non_root, COMPOUND)
    );

    // The empty body is the dimensionless unit, with no dependencies.
    let dimensionless = mutated(COMPOUND, |node| {
        node["body"] = json!({"term": "aggregate", "members": []});
        node["dependencies"] = json!([]);
        set_compound_id(node, &[]);
    });
    assert!(matches!(
        read(&dimensionless),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048", "FR-038-AC-23")]
#[test]
fn tc_048_an_application_node_dependency_list_is_its_exact_body_join() {
    let (a, b) = (
        node_key(&parameters_package(), P1),
        node_key(&parameters_package(), P1 + 1),
    );
    let cases: [(&str, Value); 4] = [
        ("missing target", json!([node_id(&a)])),
        (
            "not path-ascending",
            json!([
                node_id(&b.clone().max(a.clone())),
                node_id(&b.clone().min(a.clone()))
            ]),
        ),
        (
            "result type is not a dependency",
            json!([node_id(&BOOLEAN), node_id(&a), node_id(&b)]),
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

/// The parameters package with `extra` nodes appended, admitted.
fn admitted_with(extra: Vec<Value>) -> CheckedPackageV2 {
    let mut package = parameters_package();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(extra);
    rebuild_source_map(&mut package);
    match read(&package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn id_of(key: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(key)).expect("node id")
}

/// An integer `value`/`literal` node, which is not a parameter.
fn integer_literal_node() -> (String, Value) {
    let body = literal(&INTEGER, "integer", "7");
    let key = "value/literal/integer-seven".to_owned();
    let node = wire_node(
        &key,
        "value",
        "literal",
        &INTEGER,
        &[&INTEGER],
        "expression",
        body,
    );
    (key, node)
}

fn declared(package: &CheckedPackageV2, key: &str) -> (String, String) {
    let declaration = package
        .parameter_declaration(&id_of(key))
        .expect("a declaration");
    assert_eq!(declaration.node, id_of(key));
    (declaration.name.to_string(), declaration.level.to_string())
}

#[trace("TC-048", "FR-038-AC-258")]
#[test]
fn tc_048_parameter_declaration_returns_name_and_level_text() {
    let (ten, ten_node) = parameter("c", "10");
    let package = admitted_with(vec![ten_node]);
    let (a, b) = (
        node_key(&parameters_package(), P1),
        node_key(&parameters_package(), P1 + 1),
    );
    assert_eq!(declared(&package, &a), ("a".to_owned(), "0".to_owned()));
    assert_eq!(declared(&package, &b), ("b".to_owned(), "1".to_owned()));
    assert_eq!(declared(&package, &ten), ("c".to_owned(), "10".to_owned()));
}

#[trace("TC-048", "FR-038-AC-259")]
#[test]
fn tc_048_parameter_declaration_refuses_unknown_and_non_parameter_nodes() {
    let (literal_key, literal_node) = integer_literal_node();
    let package = admitted_with(vec![literal_node]);
    let parameters = parameters_package();
    assert_eq!(
        package.parameter_declaration(&id_of("absent-parameter")),
        Err(CheckedParameterDeclarationError::UnknownNode)
    );
    for (label, key) in [
        ("literal", literal_key),
        ("application", node_key(&parameters, E1)),
        ("function", node_key(&parameters, E1 + 1)),
    ] {
        assert_eq!(
            package.parameter_declaration(&id_of(&key)),
            Err(CheckedParameterDeclarationError::NotParameter),
            "{label}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-260")]
#[test]
fn tc_048_parameters_of_one_name_differ_by_level_and_carry_no_function() {
    let (outer, outer_node) = parameter("x", "0");
    let (inner, inner_node) = parameter("x", "1");
    assert_ne!(outer, inner);
    let package = admitted_with(vec![outer_node, inner_node]);
    assert_eq!(declared(&package, &outer), ("x".to_owned(), "0".to_owned()));
    assert_eq!(declared(&package, &inner), ("x".to_owned(), "1".to_owned()));

    // Field-by-field construction compiles only if these are all the members.
    let declaration = CheckedParameterDeclaration {
        node: id_of(&outer),
        name: "x".into(),
        level: "0".into(),
    };
    let CheckedParameterDeclaration { node, name, level } = declaration;
    assert_eq!(
        (node, name.as_ref(), level.as_ref()),
        (id_of(&outer), "x", "0")
    );
}
