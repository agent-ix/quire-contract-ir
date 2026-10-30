// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public-reader controls for declared-unit, aggregate and literal families.

use crate::support::checked_package::{
    add_family_reference_target, canonical, evidence_for, node_id, nominal_fixture_members,
    nominal_package, rebuild_source_map, refresh_identity, settle, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn add_node(package: &mut Value, key: &str, tag: &str, form: &str, ty: &str, body: Value) {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(key),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form,
            "semantic_type": node_id(ty),
            "dependencies": [],
            "occurrences": [{"role": "expression", "ordinal": 0}],
            "body": body,
        }));
}

fn fixture() -> (Value, String, String, String, String, String) {
    let members = nominal_fixture_members();
    let unit = members
        .iter()
        .find(|(p, _)| p["version"] == "quire.unit-node/v1")
        .expect("unit")
        .1
        .clone();
    let mut package = nominal_package(&members);
    let empty = json!({"term": "aggregate", "members": []});
    let scalar = |form: &str| format!("operand/scalar/{form}");
    let boolean = scalar("boolean");
    let integer = scalar("integer");
    let text = scalar("text");
    for (form, key) in [
        ("boolean", &boolean),
        ("integer", &integer),
        ("text", &text),
    ] {
        add_node(&mut package, key, "scalar_type", form, key, empty.clone());
    }
    let parameter = |name: &str| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value":
                {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": name}},
            {"term": "binding", "name": "level", "value":
                {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
        ]})
    };
    let left_body = parameter("left");
    let right_body = parameter("right");
    let left = "operand/parameter/left".to_owned();
    let right = "operand/parameter/right".to_owned();
    add_node(&mut package, &left, "value", "parameter", &unit, left_body);
    add_node(
        &mut package,
        &right,
        "value",
        "parameter",
        &unit,
        right_body,
    );
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, unit, boolean, integer, left, right)
}

fn operation(identity: &str, mode: Value, member: Value) -> Value {
    json!({"identity": identity, "laws": [], "mode": mode, "member": member, "leaves": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn append_application(
    package: &mut Value,
    operator: &str,
    form: &str,
    operation: Value,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let body = json!({"term": "application", "operator": operator,
        "operation": operation, "result_type": node_id(result), "arguments": arguments});
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let key = format!("operand/application/{form}/{position}");
    add_node(package, &key, "expression", form, result, body);
    settle(package);
    position
}

fn append_literal(package: &mut Value, ty: &str, kind: &str, value: Value) -> usize {
    let body = json!({"term": "literal", "type": node_id(ty),
        "value_kind": kind, "value": value});
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let key = format!("operand/literal/{position}");
    add_node(package, &key, "value", "literal", ty, body);
    settle(package);
    position
}

fn same_named_unit_and_dimension() -> Vec<(Value, String)> {
    let mut members = nominal_fixture_members();
    let dimension = members
        .iter_mut()
        .find(|(preimage, _)| preimage["version"] == "quire.dimension-node/v1")
        .expect("dimension");
    dimension.0["qualified_declaration"] = json!(["Example", "Metre"]);
    dimension.1 = "nominal/dimension/Example/Metre".to_owned();
    let dimension_id = node_id(&dimension.1);
    let unit = members
        .iter_mut()
        .find(|(preimage, _)| preimage["version"] == "quire.unit-node/v1")
        .expect("unit");
    unit.0["dimension_node_id"] = dimension_id;
    unit.1 = "qualified:Example/Metre".to_owned();
    members
}

#[trace("TC-230", "FR-038-AC-301", "FR-038-AC-302")]
#[test]
fn tc_230_declaration_names_are_scoped_to_unit_dimension_and_type() {
    let mut package = nominal_package(&same_named_unit_and_dimension());
    let source = &package["lock"]["sources"][0];
    let owner = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    let body = json!({"term": "aggregate", "members": []});
    let key = "nominal/type/Example/Metre";
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "node_id": node_id(key), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "record",
            "semantic_type": node_id(key), "dependencies": [],
            "occurrences": [{"role": "declaration", "ordinal": 0}],
            "declaration": {"qualified_name": ["Example", "Metre"]},
            "owner": owner, "body": body,
        }));
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    admits(&package);
}

#[trace("TC-230", "FR-038-AC-301")]
#[test]
fn tc_230_repeated_unit_name_refuses_at_later_node() {
    let mut members = same_named_unit_and_dimension();
    let mut repeated = members
        .iter()
        .find(|(preimage, _)| preimage["version"] == "quire.unit-node/v1")
        .expect("unit")
        .0
        .clone();
    repeated["target_unit_node_id"] = node_id(
        &members
            .iter()
            .find(|(preimage, _)| preimage["version"] == "quire.unit-node/v1")
            .expect("unit")
            .1,
    );
    repeated["scale"]["numerator"] = json!("2");
    let key = "qualified:Example/Metre".to_owned();
    members.push((repeated, key));
    let mut package = nominal_package(&members);
    let later_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .rposition(|node| node["semantic_form"] == "unit")
        .expect("repeated unit");
    package["semantic_graph"]["nodes"][later_position]["node_id"]["ordinal"] = json!(1);
    rebuild_source_map(&mut package);
    let nodes = package["semantic_graph"]["nodes"].as_array().unwrap();
    let unit_positions: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node["semantic_form"] == "unit")
        .collect();
    let (position, later) = unit_positions.into_iter().last().expect("later unit");
    refuses(
        &package,
        CheckedPackageRefusalCode::AmbiguousDeclaration,
        CheckedPackageRefusalCause::AmbiguousName,
        &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("duplicate unit must refuse");
    };
    assert_eq!(
        refusal.locus.as_ref(),
        Some(&serde_json::from_value(later["node_id"].clone()).unwrap())
    );
}

#[trace("TC-230", "FR-038-AC-299", "FR-038-AC-302")]
#[test]
fn tc_230_quantity_literal_in_checked_unit_admits() {
    for magnitude in ["3", "0", "-5", "1/1000", "-7/2"] {
        let (mut package, unit, _, _, _, _) = fixture();
        append_literal(&mut package, &unit, "quantity", json!(magnitude));
        admits(&package);
    }
}

#[trace("TC-230", "FR-038-AC-299")]
#[test]
fn tc_230_quantity_literal_refuses_noncanonical_magnitude_and_other_family() {
    for magnitude in [
        "03", "+3", "2/4", "4/2", "1/0", "0/5", "3/1", "1/-2", " 3", "2.5", "",
    ] {
        let (mut package, unit, _, _, _, _) = fixture();
        let at = append_literal(&mut package, &unit, "quantity", json!(magnitude));
        refuses(
            &package,
            CheckedPackageRefusalCode::InvalidPackage,
            CheckedPackageRefusalCause::InvalidValue,
            &format!("/semantic_graph/nodes/{at}/body/value"),
        );
    }
    for (kind, family) in [
        ("quantity", "integer"),
        ("quantity", "boolean"),
        ("quantity", "string"),
        ("integer", "unit"),
    ] {
        let (mut package, unit, boolean, integer, _, _) = fixture();
        let string = "operand/scalar/string".to_owned();
        if family == "string" {
            add_node(
                &mut package,
                &string,
                "scalar_type",
                "string",
                &string,
                json!({"term": "aggregate", "members": []}),
            );
            settle(&mut package);
        }
        let ty = match family {
            "unit" => unit,
            "integer" => integer,
            "boolean" => boolean,
            "string" => string,
            _ => unreachable!(),
        };
        let at = append_literal(&mut package, &ty, kind, json!("3"));
        refuses(
            &package,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/type"),
        );
    }
}

#[trace("TC-230", "FR-038-AC-299")]
#[test]
fn tc_230_quantity_literal_refuses_dimension_type_without_operand_family() {
    let (mut package, _, _, _, _, _) = fixture();
    let dimension = nominal_fixture_members()
        .into_iter()
        .find(|(preimage, _)| preimage["version"] == "quire.dimension-node/v1")
        .expect("dimension fixture")
        .1;
    let at = append_literal(&mut package, &dimension, "quantity", json!("3"));
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/type"),
    );
}

fn change_literal_type(package: &mut Value, position: usize, ty: &str) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["type"] = node_id(ty);
    settle(package);
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admits(package: &Value) {
    assert!(
        matches!(read(package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(package)
    );
}

fn refuses(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    path: &str,
) {
    let CheckedPackageV2ReadResult::Refused(actual) = read(package) else {
        panic!("expected refusal, got {:?}", read(package));
    };
    assert_eq!(
        (
            actual.code,
            actual.cause,
            actual.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (code, Some(cause), Some(path.to_owned()))
    );
}

fn native_fixture() -> (Value, String, [String; 3], String) {
    let (mut package, _, boolean, _, _, _) = fixture();
    let empty = json!({"term": "aggregate", "members": []});
    let native = ["string", "uuid", "timestamp"].map(|form| {
        let key = format!("operand/scalar/{form}");
        add_node(&mut package, &key, "scalar_type", form, &key, empty.clone());
        key
    });
    let text = "operand/scalar/text".to_owned();
    settle(&mut package);
    (package, boolean, native, text)
}

fn native_literal(package: &mut Value, ty: &str, kind: &str, value: Value) -> String {
    let at = append_literal(package, ty, kind, value);
    package["semantic_graph"]["nodes"][at]["node_id"]["path"][0]
        .as_str()
        .expect("literal path")
        .to_owned()
}

fn native_comparison(
    package: &mut Value,
    boolean: &str,
    kind: &str,
    left: &str,
    right: &str,
    verb: &str,
) -> usize {
    append_application(
        package,
        "binary",
        "binary",
        operation(&format!("quire.op.{kind}.{verb}"), Value::Null, Value::Null),
        boolean,
        vec![reference(left), reference(right)],
    )
}

fn native_refuses(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    path: &str,
    position: usize,
) {
    let CheckedPackageV2ReadResult::Refused(actual) = read(package) else {
        panic!("expected refusal, got {:?}", read(package));
    };
    assert_eq!(actual.code, code, "{actual:?}");
    assert_eq!(actual.cause, Some(cause));
    assert_eq!(
        actual.loci[0].path.as_ref().map(ToString::to_string),
        Some(path.to_owned())
    );
    assert_eq!(
        serde_json::to_value(actual.locus.expect("node locus")).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
}

#[trace("TC-048", "FR-038-AC-242", "FR-038-AC-243")]
#[test]
fn native_scalar_comparisons_keep_distinct_types_and_operands() {
    let (base, boolean, native, text) = native_fixture();
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalog");
    let mut native_identities = catalog["operations"]
        .as_array()
        .expect("operations")
        .iter()
        .filter_map(|entry| entry["identity"].as_str())
        .filter(|identity| {
            ["string", "uuid", "timestamp"]
                .iter()
                .any(|kind| identity.starts_with(&format!("quire.op.{kind}.")))
        })
        .collect::<Vec<_>>();
    native_identities.sort_unstable();
    assert_eq!(
        native_identities,
        [
            "quire.op.string.eq",
            "quire.op.string.ne",
            "quire.op.timestamp.eq",
            "quire.op.timestamp.ne",
            "quire.op.uuid.eq",
            "quire.op.uuid.ne",
        ]
    );
    for (index, kind, equal, unequal) in [
        (0, "string", "é", "e\u{301}"),
        (
            1,
            "uuid",
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-000000000001",
        ),
        (2, "timestamp", "-1", "1"),
        (2, "timestamp", "0", "1"),
    ] {
        for verb in ["eq", "ne"] {
            let mut package = base.clone();
            let left = native_literal(&mut package, &native[index], kind, json!(equal));
            let right = native_literal(&mut package, &native[index], kind, json!(unequal));
            let at = native_comparison(&mut package, &boolean, kind, &left, &right, verb);
            let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
                panic!("native {kind}.{verb} refused: {:?}", read(&package));
            };
            let nodes = &admitted.graph().nodes;
            assert_ne!(nodes[at - 2].node_id, nodes[at - 1].node_id);
            assert_eq!(
                serde_json::to_value(&nodes[at].body).expect("typed body")["arguments"][0]
                    ["target"],
                node_id(&left)
            );
            assert_eq!(
                serde_json::to_value(&nodes[at].body).expect("typed body")["arguments"][1]
                    ["target"],
                node_id(&right)
            );
            for key in [
                node_id(&left),
                node_id(&right),
                serde_json::to_value(&nodes[at].node_id).expect("application id"),
            ] {
                assert!(admitted.source_map().iter().any(|entry| {
                    serde_json::to_value(&entry.node_id).expect("source-map id") == key
                        && !entry.regions.is_empty()
                }));
            }
            assert_eq!(
                serde_json::to_value(&nodes[at - 2].body).expect("typed body")["value"],
                equal
            );
            assert_eq!(
                serde_json::to_value(&nodes[at - 1].body).expect("typed body")["value"],
                unequal
            );
            let mut equal_package = package.clone();
            equal_package["semantic_graph"]["nodes"][at]["body"]["arguments"][1] = reference(&left);
            settle(&mut equal_package);
            let CheckedPackageV2ReadResult::Admitted(equal_read) = read(&equal_package) else {
                panic!(
                    "equal native {kind}.{verb} refused: {:?}",
                    read(&equal_package)
                );
            };
            assert_eq!(
                serde_json::to_value(&equal_read.graph().nodes[at].body).expect("typed body")
                    ["arguments"][0]["target"],
                serde_json::to_value(&equal_read.graph().nodes[at].body).expect("typed body")
                    ["arguments"][1]["target"]
            );
            assert_eq!(
                admitted.node_kinds()[at - 2],
                quire_contract_model::CheckedNodeKind::Value(
                    quire_contract_model::ValueForm::Literal
                )
            );
            let foreign = if index == 0 {
                &text
            } else {
                &native[index - 1]
            };
            for argument in 0..2 {
                let mut changed = package.clone();
                changed["semantic_graph"]["nodes"][at]["body"]["arguments"][argument] =
                    reference(foreign);
                settle(&mut changed);
                native_refuses(
                    &changed,
                    CheckedPackageRefusalCode::IllTyped,
                    CheckedPackageRefusalCause::OperatorIneligible,
                    &format!("/semantic_graph/nodes/{at}/body/arguments/{argument}"),
                    at,
                );
            }
            if verb == "eq" {
                for foreign_operation in ["quire.op.integer.lt", "quire.op.integer.add"] {
                    let mut changed = package.clone();
                    changed["semantic_graph"]["nodes"][at]["body"]["operation"]["identity"] =
                        json!(foreign_operation);
                    settle(&mut changed);
                    native_refuses(
                        &changed,
                        CheckedPackageRefusalCode::IllTyped,
                        CheckedPackageRefusalCause::OperatorIneligible,
                        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
                        at,
                    );
                }
            }
        }
    }
}

fn change_native_literal_value(package: &mut Value, position: usize, value: Value) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["value"] = value;
    settle(package);
}

#[trace("TC-048", "FR-038-AC-245")]
#[test]
fn native_literal_payloads_are_canonical_after_type_validation() {
    let (base, _, native, text) = native_fixture();
    let cases: [(usize, &str, Vec<Value>, Vec<Value>); 3] = [
        (
            0,
            "string",
            vec![json!("é"), json!("e\u{301}")],
            vec![json!(5)],
        ),
        (
            1,
            "uuid",
            vec![
                json!("00000000-0000-0000-0000-000000000000"),
                json!("ffffffff-ffff-ffff-ffff-ffffffffffff"),
            ],
            vec![
                json!("FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF"),
                json!("00000000000000000000000000000000"),
                json!("0000000-00000-0000-0000-000000000000"),
                json!("00000000-0000-0000-0000-00000000000g"),
            ],
        ),
        (
            2,
            "timestamp",
            vec![
                json!(i128::MIN.to_string()),
                json!("-1"),
                json!("0"),
                json!("1"),
                json!(i128::MAX.to_string()),
            ],
            vec![
                json!("-0"),
                json!("+1"),
                json!("01"),
                json!(""),
                json!(" 1"),
                json!("1.0"),
                json!("2026-01-01"),
                json!("170141183460469231731687303715884105728"),
                json!("-170141183460469231731687303715884105729"),
            ],
        ),
    ];
    for (index, kind, good, bad) in cases {
        let mut package = base.clone();
        let key = native_literal(&mut package, &native[index], kind, good[0].clone());
        let at = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
            panic!("native literal refused: {:?}", read(&package));
        };
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[at].node_id).expect("literal id"),
            node_id(&key)
        );
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[at].body).expect("typed body")["value"],
            good[0]
        );
        for value in good.iter().skip(1) {
            let mut changed = package.clone();
            change_native_literal_value(&mut changed, at, value.clone());
            admits(&changed);
        }
        let mut wrong_type = package.clone();
        change_native_literal_value(&mut wrong_type, at, bad[0].clone());
        change_literal_type(&mut wrong_type, at, &text);
        native_refuses(
            &wrong_type,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/type"),
            at,
        );
        for value in bad {
            let mut changed = package.clone();
            change_native_literal_value(&mut changed, at, value);
            native_refuses(
                &changed,
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::InvalidValue,
                &format!("/semantic_graph/nodes/{at}/body/value"),
                at,
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-245")]
#[test]
fn native_literal_work_limit_precedes_later_payload_defect() {
    let (mut package, _, native, _) = native_fixture();
    native_literal(&mut package, &native[2], "timestamp", json!("0"));
    let mut lower = 0;
    let mut upper = CheckedPackageReadLimits::bounded().work;
    while lower + 1 < upper {
        let middle = lower + (upper - lower) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match CheckedPackageV2::read(&canonical(&package), limits, &evidence_for(&package)) {
            CheckedPackageV2ReadResult::Admitted(_) => upper = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => lower = middle,
            other => panic!("valid native package refused: {other:?}"),
        }
    }
    assert!(upper > 0);
    let mut malformed = package;
    let at = malformed["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    change_native_literal_value(&mut malformed, at, json!("-0"));
    let mut low = 0;
    let mut high = upper;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match CheckedPackageV2::read(&canonical(&malformed), limits, &evidence_for(&malformed)) {
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            CheckedPackageV2ReadResult::Refused(_) => high = middle,
            other => panic!("malformed native package admitted: {other:?}"),
        }
    }
    assert!(high > 0);
    assert!(
        high < upper,
        "the malformed payload must refuse before a valid package finishes"
    );
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high - 1;
    let CheckedPackageV2ReadResult::Incomplete(incomplete) =
        CheckedPackageV2::read(&canonical(&malformed), limits, &evidence_for(&malformed))
    else {
        panic!("malformed native package must exhaust work before its payload refusal");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(incomplete.limit, high - 1);
    assert!(incomplete.consumed > incomplete.limit);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high;
    let CheckedPackageV2ReadResult::Refused(actual) =
        CheckedPackageV2::read(&canonical(&malformed), limits, &evidence_for(&malformed))
    else {
        panic!("malformed native package did not refuse at its payload threshold");
    };
    assert_eq!(actual.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(actual.cause, Some(CheckedPackageRefusalCause::InvalidValue));
    assert_eq!(
        actual.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{at}/body/value"))
    );
    native_refuses(
        &malformed,
        CheckedPackageRefusalCode::InvalidPackage,
        CheckedPackageRefusalCause::InvalidValue,
        &format!("/semantic_graph/nodes/{at}/body/value"),
        at,
    );
}

#[trace("TC-048", "FR-038-AC-244")]
#[test]
fn native_comparisons_reject_text_laws_and_modes() {
    let (base, boolean, native, _) = native_fixture();
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalog");
    let law = catalog["law_roles"]["text_profile"][0].clone();
    for (index, kind, left_value, right_value) in [
        (0, "string", "a", "b"),
        (
            1,
            "uuid",
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-000000000001",
        ),
        (2, "timestamp", "0", "1"),
    ] {
        for verb in ["eq", "ne"] {
            let mut package = base.clone();
            package["lock"]["definition_selections"]
                .as_array_mut()
                .expect("selections")
                .push(law.clone());
            let left = native_literal(&mut package, &native[index], kind, json!(left_value));
            let right = native_literal(&mut package, &native[index], kind, json!(right_value));
            let at = native_comparison(&mut package, &boolean, kind, &left, &right, verb);
            package["semantic_graph"]["nodes"][at]["body"]["operation"]["laws"] =
                json!([{"role": "text_profile", "definition": law}]);
            settle(&mut package);
            native_refuses(
                &package,
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::OperationLawMismatch,
                &format!("/semantic_graph/nodes/{at}/body/operation/laws/0"),
                at,
            );
            package["semantic_graph"]["nodes"][at]["body"]["operation"]["laws"] = json!([]);
            package["semantic_graph"]["nodes"][at]["body"]["operation"]["mode"] =
                json!({"kind": "text_profile", "value": "nfc"});
            settle(&mut package);
            native_refuses(
                &package,
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::OperationModeMismatch,
                &format!("/semantic_graph/nodes/{at}/body/operation/mode"),
                at,
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-244")]
#[test]
fn text_comparison_rejects_string_at_its_mode_value() {
    let (base, boolean, native, text) = native_fixture();
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalog");
    let law = catalog["law_roles"]["text_profile"][0].clone();
    for string_position in 0..2 {
        let mut package = base.clone();
        package["lock"]["definition_selections"]
            .as_array_mut()
            .expect("selections")
            .push(law.clone());
        let string = native_literal(&mut package, &native[0], "string", json!("a"));
        let profiled_text = native_literal(&mut package, &text, "text", json!("a"));
        let mut arguments = vec![reference(&profiled_text), reference(&profiled_text)];
        arguments[string_position] = reference(&string);
        let at = append_application(
            &mut package,
            "binary",
            "binary",
            json!({"identity": "quire.op.text.eq",
                "laws": [{"role": "text_profile", "definition": law}],
                "mode": {"kind": "text_profile", "value": "nfc"},
                "member": null, "leaves": []}),
            &boolean,
            arguments,
        );
        native_refuses(
            &package,
            CheckedPackageRefusalCode::InvalidPackage,
            CheckedPackageRefusalCause::OperationModeTypeMismatch,
            &format!("/semantic_graph/nodes/{at}/body/operation/mode/value"),
            at,
        );
    }
}

#[trace("TC-048", "FR-038-AC-244")]
#[test]
fn text_comparison_checks_arity_and_other_families_before_string_mode() {
    let (mut package, boolean, native, text) = native_fixture();
    let integer = "operand/scalar/integer".to_owned();
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalog");
    let law = catalog["law_roles"]["text_profile"][0].clone();
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("selections")
        .push(law.clone());
    let integer = native_literal(&mut package, &integer, "integer", json!("1"));
    let string = native_literal(&mut package, &native[0], "string", json!("a"));
    let profiled_text = native_literal(&mut package, &text, "text", json!("a"));
    let at = append_application(
        &mut package,
        "binary",
        "binary",
        json!({"identity": "quire.op.text.eq",
            "laws": [{"role": "text_profile", "definition": law}],
            "mode": {"kind": "text_profile", "value": "nfc"},
            "member": null, "leaves": []}),
        &boolean,
        vec![reference(&integer), reference(&string)],
    );
    native_refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
        at,
    );
    package["semantic_graph"]["nodes"][at]["body"]["arguments"] = json!([
        reference(&string),
        reference(&profiled_text),
        reference(&profiled_text)
    ]);
    settle(&mut package);
    native_refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments"),
        at,
    );
}

#[trace("TC-048", "FR-038-AC-210")]
#[test]
fn quantity_family_checks_every_catalogued_quantity_operand() {
    let (base, unit, boolean, integer, left, right) = fixture();
    for identity in [
        "add", "sub", "mul", "div", "pow", "eq", "ne", "lt", "le", "gt", "ge", "convert",
    ] {
        let mut package = base.clone();
        let convert = identity == "convert";
        let result = if matches!(identity, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
            &boolean
        } else {
            &unit
        };
        let second = if identity == "pow" { &integer } else { &right };
        let args = if convert {
            vec![reference(&left)]
        } else {
            vec![reference(&left), reference(second)]
        };
        let mode = if convert {
            json!({"kind": "rounding", "value": "exact"})
        } else {
            Value::Null
        };
        let member = if convert {
            json!({"kind": "type_argument", "declaration": node_id(&unit)})
        } else {
            Value::Null
        };
        let at = append_application(
            &mut package,
            if convert { "convert" } else { "binary" },
            if convert { "conversion" } else { "binary" },
            operation(&format!("quire.op.quantity.{identity}"), mode, member),
            result,
            args,
        );
        admits(&package);
        if !matches!(identity, "pow" | "convert") {
            let mut second = package.clone();
            second["semantic_graph"]["nodes"][at]["body"]["arguments"][1] = reference(&boolean);
            settle(&mut second);
            refuses(
                &second,
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                &format!("/semantic_graph/nodes/{at}/body/arguments/1"),
            );
        }
        package["semantic_graph"]["nodes"][at]["body"]["arguments"][0] = reference(&boolean);
        settle(&mut package);
        refuses(
            &package,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
        );
    }
    let mut pow = base.clone();
    let at = append_application(
        &mut pow,
        "binary",
        "binary",
        operation("quire.op.quantity.pow", Value::Null, Value::Null),
        &unit,
        vec![reference(&left), reference(&integer)],
    );
    admits(&pow);
    pow["semantic_graph"]["nodes"][at]["body"]["arguments"][1] = reference(&right);
    settle(&mut pow);
    refuses(
        &pow,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/1"),
    );

    let mut integer_add = base;
    let at = append_application(
        &mut integer_add,
        "binary",
        "binary",
        operation("quire.op.integer.add", Value::Null, Value::Null),
        &integer,
        vec![reference(&left), reference(&integer)],
    );
    refuses(
        &integer_add,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
    );
}

#[trace("TC-048", "FR-038-AC-211")]
#[test]
fn control_if_checks_quantity_branch_family() {
    let (mut package, unit, boolean, integer, left, right) = fixture();
    let at = append_application(
        &mut package,
        "conditional",
        "conditional",
        operation("quire.op.control.if", Value::Null, Value::Null),
        &unit,
        vec![reference(&boolean), reference(&left), reference(&right)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["arguments"][2] = reference(&integer);
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/2"),
    );
}

#[trace("TC-048", "FR-038-AC-211")]
#[test]
fn aggregate_arguments_admit_at_clause_positions_and_refuse_at_boolean_position() {
    let temporal = v2_all_families();
    let temporal_clause = temporal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.temporal.clause")
        .expect("temporal clause");
    for position in [2, 3, 4] {
        assert_eq!(
            temporal_clause["body"]["arguments"][position]["term"],
            "aggregate"
        );
    }
    admits(&temporal);

    let (mut state, at) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    assert_eq!(
        state["semantic_graph"]["nodes"][at]["body"]["arguments"][0]["term"],
        "aggregate"
    );
    admits(&state);
    let boolean = state["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean")["node_id"]
        .clone();
    state["semantic_graph"]["nodes"][at]["body"]["arguments"][0] =
        json!({"term": "reference", "target": boolean});
    settle(&mut state);
    let CheckedPackageV2ReadResult::Refused(actual) = read(&state) else {
        panic!("state shape must refuse before operation family");
    };
    assert_eq!(actual.code, CheckedPackageRefusalCode::InvalidSemanticGraph);
    assert_eq!(
        actual.loci[0].path.clone().expect("path").as_str(),
        format!("/semantic_graph/nodes/{at}/body")
    );

    let (mut package, _, boolean, _, _, _) = fixture();
    let at = append_application(
        &mut package,
        "binary",
        "binary",
        operation("quire.op.boolean.and", Value::Null, Value::Null),
        &boolean,
        vec![
            json!({"term": "aggregate", "members": []}),
            reference(&boolean),
        ],
    );
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
    );
}

#[trace("TC-048", "FR-038-AC-212")]
#[test]
fn literal_kind_is_checked_at_its_own_type_even_outside_family_operands() {
    let (base, unit, boolean, integer, _, _) = fixture();
    let enum_key = nominal_fixture_members()
        .iter()
        .find(|(preimage, _)| preimage["version"] == "quire.enum-declaration-node/v1")
        .expect("enum")
        .1
        .clone();
    let mut with_option = base.clone();
    let option_body = json!({"term": "aggregate", "members": [reference(&integer)]});
    let option = "operand/type/option".to_owned();
    add_node(
        &mut with_option,
        &option,
        "composite_type",
        "option",
        &option,
        option_body,
    );
    settle(&mut with_option);

    for (name, package, ty, kind, value, wrong) in [
        (
            "integer",
            base.clone(),
            integer.as_str(),
            "integer",
            json!("1"),
            boolean.as_str(),
        ),
        (
            "enum",
            base.clone(),
            enum_key.as_str(),
            "enum",
            json!("ACTIVE"),
            boolean.as_str(),
        ),
        (
            "none",
            with_option,
            option.as_str(),
            "none",
            Value::Null,
            boolean.as_str(),
        ),
        (
            "unit",
            base,
            integer.as_str(),
            "integer",
            json!("2"),
            unit.as_str(),
        ),
    ] {
        let mut positive = package;
        let at = append_literal(&mut positive, ty, kind, value);
        admits(&positive);
        change_literal_type(&mut positive, at, wrong);
        refuses(
            &positive,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/type"),
        );
        assert_eq!(
            positive["semantic_graph"]["nodes"][at]["body"]["value_kind"], kind,
            "{name}"
        );
    }

    let mut ordered_members = nominal_fixture_members();
    let declaration = ordered_members
        .iter()
        .position(|(preimage, _)| preimage["version"] == "quire.enum-declaration-node/v1")
        .expect("declaration");
    ordered_members[declaration].0["ordered"] = json!(true);
    let ordered_key = ordered_members[declaration].1.clone();
    let member = ordered_members
        .iter()
        .position(|(preimage, _)| preimage["version"] == "quire.enum-member-node/v1")
        .expect("member");
    ordered_members[member].0["declaration_node_id"] = node_id(&ordered_key);
    let mut ordered = nominal_package(&ordered_members);
    let at = append_literal(&mut ordered, &ordered_key, "enum", json!("ACTIVE"));
    admits(&ordered);
    change_literal_type(&mut ordered, at, &unit);
    refuses(
        &ordered,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/type"),
    );

    // The enclosing `any_value` operand accepts either branch's family, so
    // only the literal's own kind/type rule can reject this mutation.
    let (mut control, _, boolean, integer, _, _) = fixture();
    let integer_literal = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "3"});
    let at = append_application(
        &mut control,
        "conditional",
        "conditional",
        operation("quire.op.control.if", Value::Null, Value::Null),
        &integer,
        vec![reference(&boolean), integer_literal, reference(&integer)],
    );
    admits(&control);
    control["semantic_graph"]["nodes"][at]["body"]["arguments"][1]["type"] = node_id(&boolean);
    settle(&mut control);
    refuses(
        &control,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/1/type"),
    );
}

#[trace("TC-048", "FR-038-AC-213")]
#[test]
fn quantity_conversion_defaults_to_exact_rounding() {
    let (mut package, unit, _, _, left, _) = fixture();
    let at = append_application(
        &mut package,
        "convert",
        "conversion",
        operation(
            "quire.op.quantity.convert",
            json!({"kind": "rounding", "value": "exact"}),
            json!({"kind": "type_argument", "declaration": node_id(&unit)}),
        ),
        &unit,
        vec![reference(&left)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["operation"]["mode"]["value"] =
        json!("toward-zero");
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        CheckedPackageRefusalCause::OperationModeTypeMismatch,
        &format!("/semantic_graph/nodes/{at}/body/operation/mode/value"),
    );
}

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-248", "FR-038-AC-250")]
#[test]
fn quantity_convert_checks_member_result_before_rounding_type() {
    let (mut valid, unit, _, integer, left, _) = fixture();
    let at = append_application(
        &mut valid,
        "convert",
        "conversion",
        operation(
            "quire.op.quantity.convert",
            json!({"kind": "rounding", "value": "exact"}),
            json!({"kind": "type_argument", "declaration": node_id(&unit)}),
        ),
        &unit,
        vec![reference(&left)],
    );
    admits(&valid);
    for (declaration, result, wrong_mode) in [
        (unit.as_str(), integer.as_str(), false),
        // The unit result also requires exact rounding, so this is a real
        // mode-type defect if the result check is moved later.
        (integer.as_str(), unit.as_str(), true),
        (integer.as_str(), integer.as_str(), false),
    ] {
        let mut adverse = valid.clone();
        let node = &mut adverse["semantic_graph"]["nodes"][at];
        node["body"]["operation"]["member"]["declaration"] = node_id(declaration);
        node["body"]["result_type"] = node_id(result);
        node["semantic_type"] = node_id(result);
        if wrong_mode {
            node["body"]["operation"]["mode"]["value"] = json!("toward-zero");
        }
        settle(&mut adverse);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&adverse) else {
            panic!("quantity.convert must refuse member/result {declaration}/{result}");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::IllTyped,
            "{refusal:?}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{at}/body/result_type"))
        );
        assert_eq!(
            serde_json::to_value(refusal.locus).expect("locus"),
            adverse["semantic_graph"]["nodes"][at]["node_id"]
        );
    }

    let mut operand_and_result = valid.clone();
    let node = &mut operand_and_result["semantic_graph"]["nodes"][at];
    node["body"]["result_type"] = node_id(&integer);
    node["semantic_type"] = node_id(&integer);
    node["body"]["arguments"] = json!([]);
    settle(&mut operand_and_result);
    refuses(
        &operand_and_result,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments"),
    );
}

#[trace("TC-048", "FR-038-AC-250")]
#[test]
fn position_and_element_member_results_do_not_require_declaration_equality() {
    let (mut tuple_package, _, _, integer, _, _) = fixture();
    let tuple_body = json!({"term": "aggregate", "members": [reference(&integer)]});
    let tuple = "operand/type/tuple".to_owned();
    add_node(
        &mut tuple_package,
        &tuple,
        "composite_type",
        "tuple",
        &tuple,
        tuple_body,
    );
    append_application(
        &mut tuple_package,
        "query",
        "query",
        operation(
            "quire.op.tuple.project",
            Value::Null,
            json!({"kind": "position", "declaration": node_id(&tuple), "index": 0}),
        ),
        &integer,
        vec![reference(&tuple)],
    );
    admits(&tuple_package);

    let (mut index_package, _, _, integer, _, _) = fixture();
    let set_body = json!({"term": "aggregate", "members": [reference(&integer)]});
    let set = "operand/type/set".to_owned();
    add_node(
        &mut index_package,
        &set,
        "composite_type",
        "set",
        &set,
        set_body,
    );
    append_application(
        &mut index_package,
        "query",
        "query",
        operation(
            "quire.op.collection.index",
            Value::Null,
            json!({"kind": "element", "declaration": node_id(&set)}),
        ),
        &integer,
        vec![
            reference(&set),
            json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"0"}),
        ],
    );
    admits(&index_package);
}

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-248")]
#[test]
fn quantity_convert_admits_equal_compound_unit_result() {
    let (mut package, unit, _, integer, left, _) = fixture();
    let body = json!({"term":"aggregate","members":[{"term":"aggregate","members":[
        {"term":"binding","name":"unit","value":reference(&unit)},
        {"term":"binding","name":"exponent","value":{"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"2"}}
    ]}]});
    let compound = "operand/type/compound_unit".to_owned();
    add_node(
        &mut package,
        &compound,
        "scalar_type",
        "compound_unit",
        &compound,
        body,
    );
    let compound_at = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][compound_at]["dependencies"] = json!([node_id(&unit)]);
    let at = append_application(
        &mut package,
        "convert",
        "conversion",
        operation(
            "quire.op.quantity.convert",
            json!({"kind":"rounding","value":"exact"}),
            json!({"kind":"type_argument","declaration":node_id(&compound)}),
        ),
        &compound,
        vec![reference(&left)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["result_type"] = node_id(&unit);
    package["semantic_graph"]["nodes"][at]["semantic_type"] = node_id(&unit);
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/result_type"),
    );
}

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-248")]
#[test]
fn collection_size_admits_equal_model_population_result() {
    let (mut package, _, _, integer, _, _) = fixture();
    let target = add_family_reference_target(&mut package);
    let reference_body = json!({"term":"aggregate","members":[reference(&target)]});
    let reference_type = "operand/type/reference".to_owned();
    add_node(
        &mut package,
        &reference_type,
        "composite_type",
        "reference",
        &reference_type,
        reference_body,
    );
    let set_body = json!({"term":"aggregate","members":[reference(&reference_type)]});
    let set = "operand/type/set".to_owned();
    add_node(&mut package, &set, "composite_type", "set", &set, set_body);
    let population_body = json!({"term":"aggregate","members":[]});
    let population = "operand/type/model_population".to_owned();
    add_node(
        &mut package,
        &population,
        "bounded_domain",
        "model_population",
        &set,
        population_body,
    );
    let at = append_application(
        &mut package,
        "collection",
        "collection",
        operation(
            "quire.op.collection.size",
            Value::Null,
            json!({"kind":"type_argument","declaration":node_id(&population)}),
        ),
        &population,
        vec![reference(&set)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["result_type"] = node_id(&integer);
    package["semantic_graph"]["nodes"][at]["semantic_type"] = node_id(&integer);
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/result_type"),
    );
}

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-248")]
#[test]
fn fold_and_reduce_admit_without_member_family_and_refuse_unequal_result() {
    for identity in ["quire.op.collection.fold", "quire.op.collection.reduce"] {
        let (mut package, _, boolean, integer, _, _) = fixture();
        let set_body = json!({"term":"aggregate","members":[reference(&integer)]});
        let set = "operand/type/set".to_owned();
        add_node(&mut package, &set, "composite_type", "set", &set, set_body);
        let value =
            json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"1"});
        let mut arguments = vec![
            reference(&set),
            json!({"term":"binding","name":"item","value":value}),
        ];
        if identity.ends_with("fold") {
            arguments.push(json!({"term":"literal","type":node_id(&integer),"value_kind":"integer","value":"0"}));
        }
        let at = append_application(
            &mut package,
            "collection",
            "collection",
            operation(
                identity,
                Value::Null,
                json!({"kind":"type_argument","declaration":node_id(&integer)}),
            ),
            &integer,
            arguments,
        );
        admits(&package);
        package["semantic_graph"]["nodes"][at]["body"]["result_type"] = node_id(&boolean);
        package["semantic_graph"]["nodes"][at]["semantic_type"] = node_id(&boolean);
        settle(&mut package);
        refuses(
            &package,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/result_type"),
        );
    }
}

#[trace("TC-048", "TC-229", "FR-038-AC-247", "FR-038-AC-248", "FR-038-AC-305")]
#[test]
fn non_member_model_results_remain_admitted() {
    for lookup in [false, true] {
        let (mut package, unit, _, integer, left, _) = fixture();
        let target = add_family_reference_target(&mut package);
        let reference_body = json!({"term":"aggregate","members":[reference(&target)]});
        let reference_type = "operand/query/type/reference".to_owned();
        add_node(
            &mut package,
            &reference_type,
            "composite_type",
            "reference",
            &reference_type,
            reference_body,
        );
        let result_form = if lookup { "option" } else { "set" };
        let result_body = json!({"term":"aggregate","members":[reference(if lookup { &unit } else { &reference_type })]});
        let result = format!("operand/query/result/{result_form}");
        add_node(
            &mut package,
            &result,
            "composite_type",
            result_form,
            &result,
            result_body,
        );
        let text = "operand/scalar/text".to_owned();
        let set_body = json!({"term":"aggregate","members":[reference(&reference_type)]});
        let set = if lookup {
            "operand/query/type/input_set".to_owned()
        } else {
            result.clone()
        };
        if lookup {
            add_node(&mut package, &set, "composite_type", "set", &set, set_body);
        }
        let population_body = json!({"term":"aggregate","members":[]});
        let population_type = "operand/query/type/model_population".to_owned();
        add_node(
            &mut package,
            &population_type,
            "bounded_domain",
            "model_population",
            &set,
            population_body,
        );
        let parameter_body = json!({"term":"aggregate","members":[
            {"term":"binding","name":"name","value":{
                "term":"literal","type":node_id(&text),"value_kind":"text","value":"population_a"}},
            {"term":"binding","name":"level","value":{
                "term":"literal","type":node_id(&integer),"value_kind":"integer","value":"0"}}
        ]});
        let population = "operand/query/parameter/population_a".to_owned();
        add_node(
            &mut package,
            &population,
            "value",
            "parameter",
            &population_type,
            parameter_body,
        );
        let mut arguments = vec![reference(&population)];
        if lookup {
            let body = reference(&left);
            let expression = "operand/query/expression/object".to_owned();
            add_node(
                &mut package,
                &expression,
                "expression",
                "reference",
                &reference_type,
                body,
            );
            arguments.push(reference(&expression));
        }
        append_application(
            &mut package,
            "query",
            "query",
            operation(
                if lookup {
                    "quire.op.model.lookup"
                } else {
                    "quire.op.model.all_instances"
                },
                if lookup {
                    json!({"kind":"absence","value":"option"})
                } else {
                    Value::Null
                },
                json!({"kind":"type_argument","declaration":node_id(&unit)}),
            ),
            &result,
            arguments,
        );
        admits(&package);
    }
}

pub(crate) fn quantity_sum_control() -> (Value, usize) {
    let (mut package, unit, _, _, left, _) = fixture();
    let set_body = json!({"term": "aggregate", "members": [reference(&unit)]});
    let set = "operand/type/set".to_owned();
    add_node(&mut package, &set, "composite_type", "set", &set, set_body);
    let at = append_application(
        &mut package,
        "collection",
        "collection",
        operation(
            "quire.op.collection.sum.quantity",
            json!({"kind": "rounding", "value": "exact"}),
            json!({"kind": "type_argument", "declaration": node_id(&unit)}),
        ),
        &unit,
        vec![
            reference(&set),
            json!({"term": "binding", "name": "item", "value": reference(&left)}),
        ],
    );
    (package, at)
}

#[trace("TC-048", "FR-038-AC-213")]
#[test]
fn quantity_sum_uses_its_result_type_exact_rounding_pin() {
    let (mut package, at) = quantity_sum_control();
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["operation"]["mode"]["value"] =
        json!("toward-zero");
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        CheckedPackageRefusalCause::OperationModeTypeMismatch,
        &format!("/semantic_graph/nodes/{at}/body/operation/mode/value"),
    );
}
