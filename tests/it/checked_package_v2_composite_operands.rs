// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authored operand projection through the production admitted-package API.

use crate::support::checked_package::{
    add_family_reference_target, canonical, evidence_for, family_key, fixture_source, node_id,
    nominal_fixture_members, nominal_package, over_body, rebuild_source_map, refresh_identity,
    source_owner, typed_node_id,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedAuthoredCompositeDomain as Domain, CheckedCollectionKind, CheckedCompositeDomainKey,
    CheckedCompositeOperandDomain as OperandDomain, CheckedCompositeOperandError as Error,
    CheckedCompositeOperands, CheckedCompositeOperator, CheckedNodeId, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedScalarOperandChild,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term":"aggregate", "members":members})
}

fn reference(id: &str) -> Value {
    json!({"term":"reference", "target":node_id(id)})
}

fn id_label(id: &Value) -> String {
    let path = id["path"].as_array().expect("structural path");
    let parts = path
        .iter()
        .map(|part| part.as_str().expect("path segment"))
        .collect::<Vec<_>>();
    if parts.len() == 1 {
        parts[0].to_owned()
    } else {
        format!("qualified:{}", parts.join("/"))
    }
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term":"binding", "name":name, "value":value})
}

fn bounds_body(integer: &str, min: &str, max: &str) -> Value {
    aggregate(vec![
        binding(
            "min",
            json!({"term":"literal", "type":node_id(integer),
            "value_kind":"integer", "value":min}),
        ),
        binding(
            "max",
            json!({"term":"literal", "type":node_id(integer),
            "value_kind":"integer", "value":max}),
        ),
    ])
}

fn occurrence(ordinal: u64) -> CheckedOccurrence {
    CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal,
    }
}

/// Only this test's authored wire goes into the production reader. Structural
/// paths are assigned by the fixture, independently of node bodies.
struct Fixture {
    nodes: BTreeMap<String, Value>,
    boolean: String,
    integer: String,
    text: String,
    definitions: Vec<Value>,
    include_model_target: bool,
}

impl Fixture {
    fn next_key(&self, tag: &str, form: &str) -> String {
        format!("composite/{tag}/{form}/{}", self.nodes.len())
    }

    fn new() -> Self {
        let mut fixture = Self {
            nodes: BTreeMap::new(),
            boolean: String::new(),
            integer: String::new(),
            text: String::new(),
            definitions: Vec::new(),
            include_model_target: false,
        };
        fixture.boolean = fixture.ty("scalar_type", "boolean", None, aggregate(vec![]));
        fixture.integer = fixture.ty("scalar_type", "integer", None, aggregate(vec![]));
        fixture.text = fixture.ty("scalar_type", "text", None, aggregate(vec![]));
        fixture
    }

    fn node(
        &mut self,
        id: String,
        tag: &str,
        form: &str,
        ty: &str,
        body: Value,
        dependencies: Vec<String>,
    ) -> String {
        let mut dependencies = dependencies
            .iter()
            .map(|id| node_id(id))
            .collect::<Vec<_>>();
        dependencies.sort_by_key(|id| {
            serde_json::from_value::<CheckedNodeId>(id.clone()).expect("dependency node ID")
        });
        dependencies.dedup();
        self.nodes.insert(id.clone(), json!({
            "node_id":node_id(&id), "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":tag, "semantic_form":form, "semantic_type":node_id(ty),
            "dependencies":dependencies,
            "occurrences":[{"role":if matches!(tag,"value"|"expression") {"expression"} else {"type"},"ordinal":0}],
            "body":body,
        }));
        id
    }

    fn ty(&mut self, tag: &str, form: &str, base: Option<&str>, body: Value) -> String {
        let id = self.next_key(tag, form);
        let mut dependencies = Vec::new();
        references(&body, &mut dependencies);
        dependencies.sort();
        dependencies.dedup();
        self.node(
            id.clone(),
            tag,
            form,
            base.unwrap_or(&id),
            body,
            dependencies,
        )
    }

    fn collection(&mut self, form: &str, inner: &str) -> String {
        self.ty("composite_type", form, None, over_body(inner))
    }

    fn model_reference(&mut self) -> String {
        self.include_model_target = true;
        let target = family_key("1515");
        let id = self.next_key("composite_type", "reference");
        self.node(
            id.clone(),
            "composite_type",
            "reference",
            &id,
            over_body(&target),
            vec![target],
        )
    }

    fn record(&mut self, fields: &[(&str, &str)]) -> String {
        let body = aggregate(
            fields
                .iter()
                .map(|(name, ty)| binding(name, reference(ty)))
                .collect(),
        );
        self.ty("composite_type", "record", None, body)
    }

    fn parameter(&mut self, name: &str, ty: &str) -> String {
        let body = aggregate(vec![
            binding(
                "name",
                json!({"term":"literal", "type":node_id(&self.text),"value_kind":"text","value":name}),
            ),
            binding(
                "level",
                json!({"term":"literal", "type":node_id(&self.integer),"value_kind":"integer","value":"0"}),
            ),
        ]);
        let id = format!("composite/value/parameter/{name}");
        self.node(id, "value", "parameter", ty, body, vec![])
    }

    fn graph_value(&mut self, form: &str, ty: &str, body: Value) -> String {
        let id = self.next_key("value", form);
        let mut dependencies = Vec::new();
        references(&body, &mut dependencies);
        dependencies.sort();
        dependencies.dedup();
        self.node(id, "value", form, ty, body, dependencies)
    }

    fn application(&mut self, args: Vec<Value>, operation: &str) -> String {
        self.application_with_leaves(args, operation, Vec::new())
    }

    fn application_with_leaves(
        &mut self,
        args: Vec<Value>,
        operation: &str,
        leaves: Vec<Value>,
    ) -> String {
        let body = json!({"term":"application", "operator":"binary",
            "operation":{"identity":operation,"laws":[],"mode":null,"member":null,"leaves":leaves},
            "result_type":node_id(&self.boolean),"arguments":args});
        let id = self.next_key("expression", "binary");
        let mut dependencies = Vec::new();
        references(&body, &mut dependencies);
        dependencies.sort();
        dependencies.dedup();
        let boolean = self.boolean.clone();
        self.node(id, "expression", "binary", &boolean, body, dependencies)
    }

    fn wire(&self) -> Value {
        let mut wire = nominal_package(&[]);
        wire["semantic_graph"]["nodes"] = Value::Array(self.nodes.values().cloned().collect());
        if self.include_model_target {
            add_family_reference_target(&mut wire);
        }
        wire["lock"]["definition_selections"]
            .as_array_mut()
            .expect("definitions")
            .extend(self.definitions.clone());
        rebuild_source_map(&mut wire);
        // The shared fixture helper maps each node's first occurrence. This
        // fixture also authors repeated occurrences, each of which admission
        // requires to have its own source-map entry.
        let additional = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .zip(wire["source_map"].as_array().expect("source map"))
            .flat_map(|(node, first)| {
                node["occurrences"]
                    .as_array()
                    .expect("occurrences")
                    .iter()
                    .skip(1)
                    .map(|occurrence| {
                        let mut entry = first.clone();
                        entry["role"] = occurrence["role"].clone();
                        entry["ordinal"] = occurrence["ordinal"].clone();
                        entry
                    })
            })
            .collect::<Vec<_>>();
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .extend(additional);
        refresh_identity(&mut wire);
        wire
    }

    fn read(&self) -> CheckedPackageV2 {
        admit(&self.wire())
    }
}

/// Small, independently authored test bodies only; this helper is not a
/// production traversal or a copy of a foreign fixture.
fn references(term: &Value, out: &mut Vec<String>) {
    match term.get("term").and_then(Value::as_str) {
        Some("reference") => out.push(id_label(&term["target"])),
        Some("application") => {
            for argument in term["arguments"].as_array().expect("test arguments") {
                references(argument, out);
            }
        }
        Some("aggregate") => {
            for member in term["members"].as_array().expect("test members") {
                references(member, out);
            }
        }
        Some("binding") => references(&term["value"], out),
        _ => {}
    }
}

fn admit(wire: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("self-authored fixture must admit: {other:?}"),
    }
}

fn project(package: &CheckedPackageV2, application: &str) -> CheckedCompositeOperands {
    let result = package
        .composite_application_operands(&typed_node_id(application), &occurrence(0), 100_000)
        .expect("authored projection");
    inspect_public_fields(&result);
    result
}

/// Exhaustive external patterns make every promised field and variant a
/// compile-time API obligation, without exposing or reading the wire body.
fn inspect_public_fields(result: &CheckedCompositeOperands) {
    use quire_contract_model::{
        CheckedCompositeChildEdge, CheckedCompositeDomainPosition, CheckedCompositeOperand,
        CheckedCompositeShapeEntry,
    };
    let CheckedCompositeOperands {
        application,
        occurrence,
        operator,
        operands,
        consumed_work,
    } = result;
    // An external exhaustive match fails to compile if this public enum is non-exhaustive.
    match operator {
        CheckedCompositeOperator::Eq => {}
        CheckedCompositeOperator::Ne => {}
    }
    assert!(!application.package.is_empty());
    assert!(!application.path.is_empty());
    assert_eq!(occurrence.role, CheckedOccurrenceRole::Expression);
    assert!(*consumed_work > 0);
    for operand in operands {
        let CheckedCompositeOperand {
            ordinal,
            child,
            semantic_type,
            domain,
        } = operand;
        match child {
            CheckedScalarOperandChild::GraphChild(node) => {
                assert_eq!(node.package, application.package)
            }
            CheckedScalarOperandChild::InlineLiteral {
                application,
                occurrence,
                ordinal: position,
            } => {
                assert_eq!(occurrence.role, CheckedOccurrenceRole::Expression);
                assert_eq!(position, ordinal);
                assert_eq!(application.package, semantic_type.package);
            }
        }
        match domain {
            OperandDomain::Literal => {}
            OperandDomain::Parameter { shape, positions } => {
                for entry in shape {
                    let CheckedCompositeShapeEntry {
                        type_node,
                        kind: _,
                        path: _,
                        edges,
                    } = entry;
                    assert_eq!(type_node.package, semantic_type.package);
                    for edge in edges {
                        let CheckedCompositeChildEdge {
                            target,
                            ordinal: _,
                            name: _,
                            optional_presence: _,
                        } = edge;
                        assert_eq!(target.package, type_node.package);
                    }
                }
                for entry in positions {
                    let CheckedCompositeDomainPosition {
                        key,
                        type_node,
                        authored,
                    } = entry;
                    let CheckedCompositeDomainKey::Node { node, path: _ } = key;
                    assert_eq!(node.package, type_node.package);
                    match authored {
                        Domain::IntegerRange { lower, upper } => {
                            assert!(!lower.as_str().is_empty());
                            assert!(!upper.as_str().is_empty());
                        }
                        Domain::UnboundedInteger => {}
                        Domain::Collection {
                            kind: _,
                            minimum,
                            maximum,
                        } => {
                            assert!(!minimum.as_str().is_empty());
                            assert!(!maximum.as_str().is_empty());
                        }
                        Domain::UnboundedCollection { kind: _ } => {}
                        Domain::Enum {
                            ordered: _,
                            members,
                        } => assert!(!members.is_empty()),
                        Domain::UnboundedDepth {
                            declaration,
                            reentry_paths,
                        } => {
                            assert_eq!(declaration, type_node);
                            assert!(!reentry_paths.is_empty());
                        }
                        Domain::Whole { kind: _ } => {}
                    }
                }
            }
        }
    }
}

fn parameter_domain(
    result: &CheckedCompositeOperands,
    operand: usize,
) -> (
    &[quire_contract_model::CheckedCompositeShapeEntry],
    &[quire_contract_model::CheckedCompositeDomainPosition],
) {
    match &result.operands[operand].domain {
        OperandDomain::Parameter { shape, positions } => (shape, positions),
        OperandDomain::Literal => panic!("expected parameter"),
    }
}

#[trace("FR-038-AC-246")]
#[test]
fn tc_048_structural_operator_is_typed_across_operand_shapes() {
    let mut fixture = Fixture::new();
    let boolean = fixture.boolean.clone();
    let record = fixture.record(&[("ready", &boolean)]);
    let left = fixture.parameter("left", &record);
    let right = fixture.parameter("right", &record);
    let closed = fixture.graph_value(
        "record_value",
        &record,
        aggregate(vec![binding(
            "ready",
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true}),
        )]),
    );
    let shapes = [
        (reference(&left), reference(&right)),
        (reference(&left), reference(&closed)),
        (reference(&closed), reference(&closed)),
    ];
    let applications = shapes.map(|(left, right)| {
        let arguments = vec![left, right];
        (
            fixture.application(arguments.clone(), "quire.op.structural.eq"),
            fixture.application(arguments, "quire.op.structural.ne"),
        )
    });
    let package = fixture.read();
    for (eq_id, ne_id) in applications {
        let eq = project(&package, &eq_id);
        let ne = project(&package, &ne_id);
        assert_eq!(eq.operator, CheckedCompositeOperator::Eq);
        assert_eq!(ne.operator, CheckedCompositeOperator::Ne);
        assert_ne!(eq.application, ne.application);
        assert_eq!(eq.application, typed_node_id(&eq_id));
        assert_eq!(ne.application, typed_node_id(&ne_id));
        assert_eq!(eq.occurrence, occurrence(0));
        assert_eq!(ne.occurrence, occurrence(0));
        assert_eq!(eq.operands, ne.operands);
        assert_eq!(
            eq.operands
                .iter()
                .map(|operand| operand.ordinal)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(eq.consumed_work, ne.consumed_work);
    }
}

#[trace("TC-048", "FR-038-AC-177", "FR-038-AC-182", "FR-038-AC-197")]
#[test]
fn tc_048_positional_children_and_selected_occurrences_survive_repeated_calls_and_clone() {
    let mut f = Fixture::new();
    let boolean = f.boolean.clone();
    let record = f.record(&[("ready", &boolean)]);
    let left = f.parameter("left", &record);
    let right = f.parameter("right", &record);
    let app = f.application(
        vec![reference(&left), reference(&right)],
        "quire.op.structural.eq",
    );
    f.nodes.get_mut(&app).expect("application")["occurrences"] = json!([
        {"role":"expression","ordinal":0},{"role":"expression","ordinal":1}]);
    let package = f.read();
    let before = package.clone();
    let result = project(&package, &app);
    assert_eq!(result.application, typed_node_id(&app));
    assert_eq!(result.occurrence, occurrence(0));
    assert_eq!(result.operands.len(), 2);
    for (entry, (ordinal, id)) in result.operands.iter().zip([(0, &left), (1, &right)]) {
        assert_eq!(entry.ordinal, ordinal);
        assert_eq!(
            entry.child,
            CheckedScalarOperandChild::GraphChild(typed_node_id(id))
        );
        assert_eq!(entry.semantic_type, typed_node_id(&record));
    }
    let selected = package
        .composite_application_operands(&typed_node_id(&app), &occurrence(1), 100_000)
        .expect("second authentic occurrence");
    assert_eq!(selected.occurrence, occurrence(1));
    assert_eq!(selected.operands, result.operands);
    assert_eq!(
        package.composite_application_operands(&typed_node_id(&app), &occurrence(2), 100_000),
        Err(Error::MissingOccurrence {
            application: typed_node_id(&app),
            occurrence: occurrence(2)
        })
    );
    assert_eq!(project(&before, &app), result);
    assert_eq!(package, before);
    let swapped = f.application(
        vec![reference(&right), reference(&left)],
        "quire.op.structural.eq",
    );
    let repeated = f.application(
        vec![reference(&left), reference(&left)],
        "quire.op.structural.eq",
    );
    let ne = f.application(
        vec![reference(&left), reference(&right)],
        "quire.op.structural.ne",
    );
    f.nodes.get_mut(&ne).expect("inequality application")["occurrences"] = json!([
        {"role":"expression","ordinal":0},{"role":"expression","ordinal":1}]);
    let ne_swapped = f.application(
        vec![reference(&right), reference(&left)],
        "quire.op.structural.ne",
    );
    let ne_repeated = f.application(
        vec![reference(&left), reference(&left)],
        "quire.op.structural.ne",
    );
    let package = f.read();
    let swap = project(&package, &swapped);
    let repeat = project(&package, &repeated);
    assert_eq!(swap.operands[0].child, result.operands[1].child);
    assert_eq!(swap.operands[1].child, result.operands[0].child);
    assert_eq!(repeat.operands[0].child, repeat.operands[1].child);
    assert_ne!(repeat.operands[0].ordinal, repeat.operands[1].ordinal);
    let inequality = project(&package, &ne);
    assert_eq!(inequality.application, typed_node_id(&ne));
    assert_eq!(inequality.occurrence, occurrence(0));
    assert_eq!(inequality.operands, result.operands);
    let selected = package
        .composite_application_operands(&typed_node_id(&ne), &occurrence(1), 100_000)
        .expect("second authentic inequality occurrence");
    assert_eq!(selected.occurrence, occurrence(1));
    assert_eq!(selected.operands, inequality.operands);
    assert_eq!(
        package.composite_application_operands(&typed_node_id(&ne), &occurrence(2), 100_000),
        Err(Error::MissingOccurrence {
            application: typed_node_id(&ne),
            occurrence: occurrence(2)
        })
    );
    let swap_ne = project(&package, &ne_swapped);
    let repeat_ne = project(&package, &ne_repeated);
    assert_eq!(swap_ne.operands, swap.operands);
    assert_eq!(repeat_ne.operands, repeat.operands);
}

#[trace(
    "TC-048",
    "FR-038-AC-179",
    "FR-038-AC-181",
    "FR-038-AC-198",
    "FR-038-AC-200"
)]
#[test]
fn tc_048_exact_authored_decimal_bounds_and_paths_have_a_reproducible_work_boundary() {
    let mut f = Fixture::new();
    let integer = f.integer.clone();
    let low = "-40000000000000000000000000000000000000000000000000";
    let high = "50000000000000000000000000000000000000000000000000";
    let maximum = "184467440737095516160000";
    let range = f.ty(
        "bounded_domain",
        "integer_range",
        Some(&integer),
        bounds_body(&integer, low, high),
    );
    let sequence = f.collection("sequence", &range);
    let bounded = f.ty(
        "bounded_domain",
        "collection_bounds",
        Some(&sequence),
        bounds_body(&integer, "2", maximum),
    );
    let record = f.record(&[("items", &bounded)]);
    let p = f.parameter("p", &record);
    let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let ne = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.ne");
    let package = f.read();
    let result = project(&package, &app);
    let inequality = project(&package, &ne);
    assert_eq!(inequality.operands, result.operands);
    assert_eq!(inequality.consumed_work, result.consumed_work);
    let (shape, positions) = parameter_domain(&result, 0);
    assert_eq!(
        shape
            .iter()
            .map(|entry| entry.type_node.clone())
            .collect::<Vec<_>>(),
        [&record, &bounded, &sequence, &range, &integer].map(|id| typed_node_id(id))
    );
    assert_eq!(positions.len(), 2);
    for (entry, path) in positions.iter().zip([vec![0], vec![0, 0]]) {
        assert_eq!(
            entry.key,
            CheckedCompositeDomainKey::Node {
                node: typed_node_id(&p),
                path
            }
        );
    }
    match &positions[0].authored {
        Domain::Collection {
            kind,
            minimum,
            maximum: actual,
        } => {
            assert_eq!(*kind, CheckedCollectionKind::Sequence);
            assert_eq!(minimum.as_str(), "2");
            assert_eq!(actual.as_str(), maximum);
        }
        other => panic!("collection bounds: {other:?}"),
    }
    match &positions[1].authored {
        Domain::IntegerRange { lower, upper } => {
            assert_eq!(lower.as_str(), low);
            assert_eq!(upper.as_str(), high);
        }
        other => panic!("integer bounds: {other:?}"),
    }
    let id = typed_node_id(&app);
    let occurrence = occurrence(0);
    // Independent semantic visits for each operand (five shape nodes):
    // 1 operand + 1 child resolution + 1 family resolution;
    // 5 shape resolutions + 5 shape visits + 2 position visits;
    // 4 child/forward edges + 2 descriptor-family resolutions;
    // 0+1+1+2+2 retained shape path elements + 1+2 position path elements;
    // field name plus four exact decimal endpoint spellings.
    let expected_per_operand = 3
        + 5
        + 5
        + 2
        + 4
        + 2
        + 6
        + 3
        + u64::try_from("items".len() + "2".len() + maximum.len() + low.len() + high.len())
            .expect("small strings");
    let expected = 1 + 2 * expected_per_operand;
    assert_eq!(result.consumed_work, expected);
    assert_eq!(inequality.consumed_work, expected);
    let consumed = expected;
    assert_eq!(
        package.composite_application_operands(&id, &occurrence, consumed),
        Ok(result.clone())
    );
    assert_eq!(
        package.composite_application_operands(&id, &occurrence, consumed - 1),
        Err(Error::WorkLimit {
            limit: consumed - 1,
            consumed
        })
    );
    assert_eq!(
        package.composite_application_operands(&id, &occurrence, 0),
        Err(Error::WorkLimit {
            limit: 0,
            consumed: 1
        })
    );
    let ne_id = typed_node_id(&ne);
    assert_eq!(
        package.composite_application_operands(&ne_id, &occurrence, expected),
        Ok(inequality.clone())
    );
    assert_eq!(
        package.composite_application_operands(&ne_id, &occurrence, expected - 1),
        Err(Error::WorkLimit {
            limit: expected - 1,
            consumed: expected
        })
    );
    assert_eq!(
        package.composite_application_operands(&ne_id, &occurrence, 0),
        Err(Error::WorkLimit {
            limit: 0,
            consumed: 1
        })
    );
    let mut reversed = f.wire();
    reversed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .reverse();
    rebuild_source_map(&mut reversed);
    refresh_identity(&mut reversed);
    assert_eq!(project(&admit(&reversed), &app), result);
    assert_eq!(project(&admit(&reversed), &ne), inequality);

    // Re-author only the element range's lower endpoint and derive all parent
    // identities again. Its collection descriptor remains the same, while
    // the element descriptor at the same path names the new authored range.
    let changed_lower = "-60000000000000000000000000000000000000000000000000";
    let changed_range = f.ty(
        "bounded_domain",
        "integer_range",
        Some(&integer),
        bounds_body(&integer, changed_lower, high),
    );
    let changed_sequence = f.collection("sequence", &changed_range);
    let changed_bounded = f.ty(
        "bounded_domain",
        "collection_bounds",
        Some(&changed_sequence),
        bounds_body(&integer, "2", maximum),
    );
    let changed_record = f.record(&[("items", &changed_bounded)]);
    let changed_parameter = f.parameter("p", &changed_record);
    let changed_app = f.application(
        vec![reference(&changed_parameter), reference(&changed_parameter)],
        "quire.op.structural.eq",
    );
    let changed = project(&f.read(), &changed_app);
    let (_, changed_positions) = parameter_domain(&changed, 0);
    assert_eq!(changed_positions.len(), positions.len());
    assert_eq!(changed_positions[0].authored, positions[0].authored);
    assert_ne!(changed_positions[1].authored, positions[1].authored);
    assert_eq!(
        changed_positions[1].type_node,
        typed_node_id(&changed_range)
    );
    assert_ne!(changed_positions[1].type_node, positions[1].type_node);
    for (entry, path) in changed_positions.iter().zip([vec![0], vec![0, 0]]) {
        assert_eq!(
            entry.key,
            CheckedCompositeDomainKey::Node {
                node: typed_node_id(&changed_parameter),
                path,
            }
        );
    }
    match &changed_positions[1].authored {
        Domain::IntegerRange { lower, upper } => {
            assert_eq!(lower.as_str(), changed_lower);
            assert_eq!(upper.as_str(), high);
        }
        other => panic!("changed integer bounds: {other:?}"),
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-178",
    "FR-038-AC-181",
    "FR-038-AC-197",
    "FR-038-AC-198",
    "FR-038-AC-199"
)]
#[test]
fn tc_048_closed_graph_values_preserve_identity_while_free_values_and_unions_refuse() {
    let mut f = Fixture::new();
    let boolean = f.boolean.clone();
    let record = f.record(&[("ready", &boolean)]);
    let value = f.graph_value(
        "record_value",
        &record,
        aggregate(vec![binding(
            "ready",
            json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true}),
        )]),
    );
    let app = f.application(
        vec![reference(&value), reference(&value)],
        "quire.op.structural.eq",
    );
    let ne = f.application(
        vec![reference(&value), reference(&value)],
        "quire.op.structural.ne",
    );
    let package = f.read();
    let result = project(&package, &app);
    assert_eq!(project(&package, &ne).operands, result.operands);
    for operand in &result.operands {
        assert_eq!(
            operand.child,
            CheckedScalarOperandChild::GraphChild(typed_node_id(&value))
        );
        assert_eq!(operand.domain, OperandDomain::Literal);
    }
    let literal =
        json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true});
    let tuple = f.ty(
        "composite_type",
        "tuple",
        None,
        aggregate(vec![reference(&boolean)]),
    );
    let sequence = f.collection("sequence", &boolean);
    let option = f.collection("option", &boolean);
    for (form, ty) in [
        ("tuple_value", &tuple),
        ("collection_value", &sequence),
        ("option_value", &option),
    ] {
        let child = f.graph_value(form, ty, aggregate(vec![literal.clone()]));
        let application = f.application(
            vec![reference(&child), reference(&child)],
            "quire.op.structural.eq",
        );
        let ne = f.application(
            vec![reference(&child), reference(&child)],
            "quire.op.structural.ne",
        );
        let package = f.read();
        let projected = project(&package, &application);
        assert_eq!(project(&package, &ne).operands, projected.operands);
        for operand in projected.operands {
            assert_eq!(
                operand.child,
                CheckedScalarOperandChild::GraphChild(typed_node_id(&child))
            );
            assert_eq!(operand.domain, OperandDomain::Literal);
        }
    }

    // The owning catalog publishes collection.sequence as a collection-valued
    // operation. Its graph reference admits; flat wire rejects nested applications.
    let application_body = json!({"term":"application","operator":"collection",
        "operation":{"identity":"quire.op.collection.sequence","laws":[],"mode":null,"member":null,"leaves":[]},
        "result_type":node_id(&sequence),"arguments":[literal.clone()]});
    let expression_key = f.next_key("expression", "collection");
    let expression = f.node(
        expression_key,
        "expression",
        "collection",
        &sequence,
        application_body.clone(),
        vec![],
    );
    let closed_sequence = f.graph_value(
        "collection_value",
        &sequence,
        aggregate(vec![literal.clone()]),
    );
    let referenced_app = f.application(
        vec![reference(&closed_sequence), reference(&expression)],
        "quire.op.structural.eq",
    );
    let referenced_ne = f.application(
        vec![reference(&closed_sequence), reference(&expression)],
        "quire.op.structural.ne",
    );
    assert_eq!(
        f.read().composite_application_operands(
            &typed_node_id(&referenced_app),
            &occurrence(0),
            100_000
        ),
        Err(Error::UnsupportedOperand {
            ordinal: 1,
            type_node: Some(typed_node_id(&sequence)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::ApplicationSubterm
        })
    );
    assert_eq!(
        f.read().composite_application_operands(
            &typed_node_id(&referenced_ne),
            &occurrence(0),
            100_000
        ),
        Err(Error::UnsupportedOperand {
            ordinal: 1,
            type_node: Some(typed_node_id(&sequence)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::ApplicationSubterm
        })
    );
    // An independently authored nested application is rejected by the reader's
    // flat grammar. The model unit exercises the defensive accessor branch
    // only after admitting its fixture and then mutating argument one.
    let inline_subterm = f.application(
        vec![reference(&closed_sequence), application_body],
        "quire.op.structural.eq",
    );
    let nested_wire = f.wire();
    let nested_position = nested_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == node_id(&inline_subterm))
        .expect("nested application");
    match CheckedPackageV2::read(
        &canonical(&nested_wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&nested_wire),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                quire_contract_model::CheckedPackageRefusalCode::MalformedWire
            );
            assert_eq!(refusal.cause, None);
            assert_eq!(
                refusal.loci[0]
                    .path
                    .clone()
                    .expect("nested argument pointer")
                    .as_str(),
                format!("/semantic_graph/nodes/{nested_position}/body/arguments/1")
            );
        }
        other => panic!("nested application must retain flat-wire refusal: {other:?}"),
    }
    // Keep subsequent public-accessor cases genuinely reader-admitted.
    f.nodes
        .remove(&inline_subterm)
        .expect("authored nested application");
    let containing_record = f.record(&[("items", &sequence)]);
    let closed_record = f.graph_value(
        "record_value",
        &containing_record,
        aggregate(vec![binding("items", reference(&closed_sequence))]),
    );
    let application_member = f.graph_value(
        "record_value",
        &containing_record,
        aggregate(vec![binding("items", reference(&expression))]),
    );
    let member_application = f.application(
        vec![reference(&closed_record), reference(&application_member)],
        "quire.op.structural.eq",
    );
    assert_eq!(
        f.read().composite_application_operands(
            &typed_node_id(&member_application),
            &occurrence(0),
            100_000
        ),
        Err(Error::UnsupportedOperand {
            ordinal: 1,
            type_node: Some(typed_node_id(&sequence)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::NonliteralGraphValue
        })
    );

    // This separately authored input goes through admission; it is not a
    // post-admission defensive mutation or an eligible inline-integer success.
    let mut integer_input = Fixture::new();
    let integer_literal = json!({"term":"literal","type":node_id(&integer_input.integer),"value_kind":"integer","value":"1"});
    let integer_application = integer_input.application(
        vec![integer_literal.clone(), integer_literal],
        "quire.op.structural.eq",
    );
    let wire = integer_input.wire();
    let position = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == node_id(&integer_application))
        .expect("application");
    match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                quire_contract_model::CheckedPackageRefusalCode::IllTyped
            );
            assert_eq!(
                refusal.cause,
                Some(quire_contract_model::CheckedPackageRefusalCause::OperatorIneligible)
            );
            assert_eq!(
                refusal.loci[0]
                    .path
                    .clone()
                    .expect("argument pointer")
                    .as_str(),
                format!("/semantic_graph/nodes/{position}/body/arguments/0")
            );
            assert_eq!(refusal.locus, Some(typed_node_id(&integer_application)));
        }
        other => panic!("integer inline must retain reader refusal: {other:?}"),
    }
    let free = f.parameter("free", &boolean);
    let open = f.graph_value(
        "record_value",
        &record,
        aggregate(vec![binding("ready", reference(&free))]),
    );
    let open_app = f.application(
        vec![reference(&open), reference(&open)],
        "quire.op.structural.eq",
    );
    assert_eq!(
        f.read()
            .composite_application_operands(&typed_node_id(&open_app), &occurrence(0), 100_000),
        Err(Error::UnsupportedOperand {
            ordinal: 0,
            type_node: Some(typed_node_id(&boolean)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::NonliteralGraphValue
        })
    );
    let union = f.ty(
        "composite_type",
        "union",
        None,
        aggregate(vec![binding("Ready", aggregate(vec![reference(&boolean)]))]),
    );
    let union_value = f.graph_value("union_value",&union,aggregate(vec![binding("Ready",aggregate(vec![
        json!({"term":"literal","type":node_id(&boolean),"value_kind":"boolean","value":true})]))]));
    let union_app = f.application(
        vec![reference(&union_value), reference(&union_value)],
        "quire.op.structural.eq",
    );
    let enclosing = f.record(&[("choice", &union)]);
    let nested = f.graph_value(
        "record_value",
        &enclosing,
        aggregate(vec![binding("choice", reference(&union_value))]),
    );
    let nested_app = f.application(
        vec![reference(&nested), reference(&nested)],
        "quire.op.structural.eq",
    );
    let union_parameter = f.parameter("choice", &union);
    let parameter_app = f.application(
        vec![reference(&union_parameter), reference(&union_parameter)],
        "quire.op.structural.eq",
    );
    let union_ne = f.application(
        vec![reference(&union_parameter), reference(&union_parameter)],
        "quire.op.structural.ne",
    );
    let package = f.read();
    for app in [union_app, nested_app, parameter_app, union_ne] {
        assert_eq!(
            package.composite_application_operands(&typed_node_id(&app), &occurrence(0), 100_000),
            Err(Error::UnsupportedDomain {
                ordinal: 0,
                type_node: typed_node_id(&union)
            })
        );
    }
    let option = f.collection("option", &boolean);
    let none = json!({"term":"literal","type":node_id(&option),"value_kind":"none","value":null});
    let inline_app = f.application(vec![none.clone(), none.clone()], "quire.op.structural.eq");
    let closed_option = f.graph_value("option_value", &option, aggregate(vec![literal]));
    let inline_ne = f.application(
        vec![reference(&closed_option), none],
        "quire.op.structural.ne",
    );
    assert_eq!(
        f.read().composite_application_operands(
            &typed_node_id(&inline_app),
            &occurrence(0),
            100_000
        ),
        Err(Error::UnsupportedOperand {
            ordinal: 0,
            type_node: Some(typed_node_id(&option)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::InlineNonInteger
        })
    );
    assert_eq!(
        f.read().composite_application_operands(
            &typed_node_id(&inline_ne),
            &occurrence(0),
            100_000
        ),
        Err(Error::UnsupportedOperand {
            ordinal: 1,
            type_node: Some(typed_node_id(&option)),
            reason: quire_contract_model::CheckedUnsupportedCompositeOperand::InlineNonInteger
        })
    );
}

#[trace("TC-048", "FR-038-AC-180", "FR-038-AC-198")]
#[test]
fn tc_048_optional_wrapper_paths_and_all_reentries_preserve_distinct_source_routes() {
    for wrapped in [false, true] {
        let mut f = Fixture::new();
        let boolean = f.boolean.clone();
        let option = f.collection("option", &boolean);
        let field = if wrapped {
            aggregate(vec![binding("optional", reference(&option))])
        } else {
            reference(&option)
        };
        let record = f.ty(
            "composite_type",
            "record",
            None,
            aggregate(vec![binding("maybe", field)]),
        );
        let parameter = f.parameter("optional_budget", &record);
        let app = f.application(
            vec![reference(&parameter), reference(&parameter)],
            "quire.op.structural.eq",
        );
        let ne = f.application(
            vec![reference(&parameter), reference(&parameter)],
            "quire.op.structural.ne",
        );
        let package = f.read();
        let result = project(&package, &app);
        assert_eq!(project(&package, &ne).operands, result.operands);
        // Per operand: operand/child/family=3, three node resolutions+shape
        // visits=6, two edges=2, retained shape path elements=0+1+2=3,
        // copied field name=5. The optional wrapper adds no visit or segment.
        assert_eq!(result.consumed_work, 1 + 2 * (3 + 6 + 2 + 3 + 5));
    }
    for wrapped in [false, true] {
        let mut f = Fixture::new();
        let integer = f.integer.clone();
        let list = "composite/group/list/record".to_owned();
        let option = "composite/group/list/option".to_owned();
        let tail = if wrapped {
            aggregate(vec![binding("optional", reference(&option))])
        } else {
            reference(&option)
        };
        f.node(
            list.clone(),
            "composite_type",
            "record",
            &list,
            aggregate(vec![
                binding("head", reference(&integer)),
                binding("tail", tail),
            ]),
            vec![integer, option.clone()],
        );
        f.node(
            option.clone(),
            "composite_type",
            "option",
            &option,
            over_body(&list),
            vec![list.clone()],
        );
        for id in [&list, &option] {
            f.nodes.get_mut(id).expect("cycle member")["recursion_group"] = json!("list");
        }
        let p = f.parameter("list", &list);
        let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
        let ne = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.ne");
        let package = f.read();
        let result = project(&package, &app);
        assert_eq!(project(&package, &ne).operands, result.operands);
        let (shape, positions) = parameter_domain(&result, 0);
        assert_eq!(shape[0].edges[1].optional_presence, wrapped);
        assert_eq!(shape[0].edges[1].ordinal, Some(1));
        assert_eq!(shape[0].edges[1].name.as_deref(), Some("tail"));
        let depth = positions
            .iter()
            .find(|p| matches!(p.authored, Domain::UnboundedDepth { .. }))
            .expect("depth position");
        assert_eq!(
            depth.key,
            CheckedCompositeDomainKey::Node {
                node: typed_node_id(&p),
                path: vec![]
            }
        );
        assert_eq!(depth.type_node, typed_node_id(&list));
        assert_eq!(
            depth.authored,
            Domain::UnboundedDepth {
                declaration: typed_node_id(&list),
                reentry_paths: vec![vec![1, 0]]
            }
        );
    }
    let mut f = Fixture::new();
    let tree = "composite/group/tree/record".to_owned();
    let option = "composite/group/tree/option".to_owned();
    let optional = aggregate(vec![binding("optional", reference(&option))]);
    f.node(
        tree.clone(),
        "composite_type",
        "record",
        &tree,
        aggregate(vec![
            binding("left", optional.clone()),
            binding("right", optional),
        ]),
        vec![option.clone()],
    );
    f.node(
        option.clone(),
        "composite_type",
        "option",
        &option,
        over_body(&tree),
        vec![tree.clone()],
    );
    for id in [&tree, &option] {
        f.nodes.get_mut(id).expect("cycle member")["recursion_group"] = json!("tree");
    }
    let p = f.parameter("tree", &tree);
    let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let result = project(&f.read(), &app);
    let (_, positions) = parameter_domain(&result, 0);
    assert_eq!(positions.len(), 1);
    assert_eq!(
        positions[0].key,
        CheckedCompositeDomainKey::Node {
            node: typed_node_id(&p),
            path: vec![]
        }
    );
    assert_eq!(
        positions[0].authored,
        Domain::UnboundedDepth {
            declaration: typed_node_id(&tree),
            reentry_paths: vec![vec![0, 0], vec![1, 0]]
        }
    );
    let integer = f.integer.clone();
    let option = f.collection("option", &integer);
    let tuple = f.ty(
        "composite_type",
        "tuple",
        None,
        aggregate(vec![reference(&option), reference(&option)]),
    );
    let root = f.record(&[("first", &tuple), ("second", &tuple)]);
    let p = f.parameter("shared", &root);
    let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let result = project(&f.read(), &app);
    let (_, positions) = parameter_domain(&result, 0);
    assert_eq!(
        positions
            .iter()
            .map(|position| position.key.clone())
            .collect::<Vec<_>>(),
        [vec![0, 0, 0], vec![0, 1, 0], vec![1, 0, 0], vec![1, 1, 0]].map(|path| {
            CheckedCompositeDomainKey::Node {
                node: typed_node_id(&p),
                path,
            }
        })
    );
    assert!(positions
        .iter()
        .all(|p| p.authored == Domain::UnboundedInteger));
}

#[trace("TC-048", "FR-038-AC-180", "FR-038-AC-182", "FR-038-AC-198")]
#[test]
fn tc_048_eligible_scalar_and_bounded_forms_keep_authored_positions_and_source_kinds() {
    use quire_contract_model::{BoundedDomainForm, CheckedNodeKind, ScalarTypeForm};
    let mut f = Fixture::new();
    let mut types: BTreeMap<String, String> = BTreeMap::new();
    for form in ["boolean", "integer", "rational", "decimal"] {
        let id = f.ty("scalar_type", form, None, aggregate(vec![]));
        types.insert(form.into(), id);
    }
    // Nominal supporting nodes are this repository's own test-authored inputs.
    let nominal = nominal_package(&nominal_fixture_members());
    for node in nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes")
    {
        let id = id_label(&node["node_id"]);
        if node["node_tag"] == "scalar_type" {
            types.insert(
                node["semantic_form"].as_str().expect("form").into(),
                id.clone(),
            );
        }
        f.nodes.insert(id, node.clone());
    }
    let color = json!({"version":"quire.enum-declaration-node/v1","owner":source_owner(&fixture_source()),
        "qualified_declaration":["Color"],"ordered":false,"members":["Blue","Green","Red"]});
    let color_key = "Color".to_owned();
    let color_package = nominal_package(&[(color, color_key.clone())]);
    for node in color_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("enum nodes")
    {
        f.nodes.insert(id_label(&node["node_id"]), node.clone());
    }
    types.insert("enum".into(), color_key);
    let compound = f.ty("scalar_type", "compound_unit", None, aggregate(vec![]));
    types.insert("compound_unit".into(), compound);
    let text = f.text.clone();
    let profile = aggregate(vec![binding(
        "text_profile",
        json!({"term":"literal","type":node_id(&text),"value_kind":"text","value":"nfc"}),
    )]);
    let profiled = f.ty("scalar_type", "text", None, profile.clone());
    types.insert("text".into(), profiled);
    let integer = f.integer.clone();
    let range = f.ty(
        "bounded_domain",
        "integer_range",
        Some(&integer),
        bounds_body(&integer, "-2", "3"),
    );
    types.insert("integer_range".into(), range);
    let sequence = f.collection("sequence", &integer);
    let bounded = f.ty(
        "bounded_domain",
        "collection_bounds",
        Some(&sequence),
        bounds_body(&integer, "1", "4"),
    );
    types.insert("collection_bounds".into(), bounded);
    for (form, base) in [("rational_range", "rational"), ("decimal_range", "decimal")] {
        let ty = types.get(base).expect("base").clone();
        let id = f.ty("bounded_domain", form, Some(&ty), aggregate(vec![]));
        types.insert(form.into(), id);
    }
    let text_bounds = f.ty("bounded_domain", "text_bounds", Some(&text), profile);
    types.insert("text_bounds".into(), text_bounds);
    let model_reference = f.model_reference();
    let model_set = f.collection("set", &model_reference);
    let population = f.ty(
        "bounded_domain",
        "model_population",
        Some(&model_set),
        aggregate(vec![]),
    );
    types.insert("model_population".into(), population);
    // Float32, Float64 and float_rounding would make the compared record
    // ineligible under FR-038-AC-231. This projection covers the fifteen
    // forms whose comparison admits through the public reader.
    assert_eq!(types.len(), 15, "all comparison-eligible forms");
    let names = types.keys().cloned().collect::<Vec<_>>();
    let fields = names
        .iter()
        .map(|name| (name.as_str(), types.get(name).expect("type").as_str()))
        .collect::<Vec<_>>();
    let root = f.record(&fields);
    let p = f.parameter("all_forms", &root);
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("catalog");
    let law = catalog["law_roles"]["text_profile"][0].clone();
    f.definitions.push(law.clone());
    let leaves=["text","text_bounds"].map(|field|json!({"path":[format!("field:{field}")],
        "laws":[{"role":"text_profile","definition":law}],"mode":{"kind":"text_profile","value":"nfc"}})).to_vec();
    let app = f.application_with_leaves(
        vec![reference(&p), reference(&p)],
        "quire.op.structural.eq",
        leaves.clone(),
    );
    let ne = f.application_with_leaves(
        vec![reference(&p), reference(&p)],
        "quire.op.structural.ne",
        leaves,
    );
    let package = f.read();
    let result = project(&package, &app);
    assert_eq!(project(&package, &ne).operands, result.operands);
    let (_, positions) = parameter_domain(&result, 0);
    // This key oracle comes from authored fields and expected form semantics.
    let expected = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_str() != "boolean")
        .flat_map(|(index, name)| {
            if name == "collection_bounds" {
                vec![
                    vec![u32::try_from(index).expect("index")],
                    vec![u32::try_from(index).expect("index"), 0],
                ]
            } else {
                vec![vec![u32::try_from(index).expect("index")]]
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        positions
            .iter()
            .map(|entry| entry.key.clone())
            .collect::<Vec<_>>(),
        expected
            .into_iter()
            .map(|path| CheckedCompositeDomainKey::Node {
                node: typed_node_id(&p),
                path
            })
            .collect::<Vec<_>>()
    );
    for (index, name) in names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_str() != "boolean")
    {
        let path = vec![u32::try_from(index).expect("index")];
        let position = positions
            .iter()
            .find(|entry| {
                entry.key
                    == CheckedCompositeDomainKey::Node {
                        node: typed_node_id(&p),
                        path: path.clone(),
                    }
            })
            .expect("introduced position");
        assert_eq!(
            position.type_node,
            typed_node_id(types.get(name).expect("original source"))
        );
        match (name.as_str(), &position.authored) {
            ("integer", Domain::UnboundedInteger) => {}
            ("integer_range", Domain::IntegerRange { lower, upper }) => {
                assert_eq!(lower.as_str(), "-2");
                assert_eq!(upper.as_str(), "3");
            }
            (
                "collection_bounds",
                Domain::Collection {
                    kind,
                    minimum,
                    maximum,
                },
            ) => {
                assert_eq!(*kind, CheckedCollectionKind::Sequence);
                assert_eq!(minimum.as_str(), "1");
                assert_eq!(maximum.as_str(), "4");
            }
            ("enum", Domain::Enum { ordered, members }) => {
                assert!(!ordered);
                assert_eq!(
                    members.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    ["Blue", "Green", "Red"]
                );
            }
            (name, Domain::Whole { kind }) => {
                let expected = ScalarTypeForm::from_wire(name)
                    .map(CheckedNodeKind::ScalarType)
                    .or_else(|| {
                        BoundedDomainForm::from_wire(name).map(CheckedNodeKind::BoundedDomain)
                    })
                    .expect("closed source form");
                assert_eq!(*kind, expected);
            }
            other => panic!("wrong authored descriptor {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-180", "FR-038-AC-179", "FR-038-AC-198")]
#[test]
fn tc_048_collection_kinds_remain_unbounded_and_ordered_enums_keep_semantic_order() {
    let mut f = Fixture::new();
    let integer = f.integer.clone();
    let mut fields = Vec::new();
    for (name, kind) in [
        ("sequence", CheckedCollectionKind::Sequence),
        ("set", CheckedCollectionKind::Set),
        ("bag", CheckedCollectionKind::Bag),
        ("ordered_set", CheckedCollectionKind::OrderedSet),
    ] {
        let id = f.collection(name, &integer);
        fields.push((name, id, kind));
    }
    let root = f.record(
        &fields
            .iter()
            .map(|(name, id, _)| (*name, id.as_str()))
            .collect::<Vec<_>>(),
    );
    let p = f.parameter("collections", &root);
    let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let ne = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.ne");
    let package = f.read();
    let result = project(&package, &app);
    assert_eq!(project(&package, &ne).operands, result.operands);
    let (_, positions) = parameter_domain(&result, 0);
    assert_eq!(positions.len(), 8);
    for (index, (_, id, kind)) in fields.iter().enumerate() {
        assert_eq!(
            positions[2 * index].authored,
            Domain::UnboundedCollection { kind: *kind }
        );
        assert_eq!(positions[2 * index].type_node, typed_node_id(id));
        assert_eq!(positions[2 * index + 1].authored, Domain::UnboundedInteger);
    }
    let color = json!({"version":"quire.enum-declaration-node/v1","owner":source_owner(&fixture_source()),
        "qualified_declaration":["Priority"],"ordered":true,"members":["Red","Blue","Green"]});
    let color_key = "Priority".to_owned();
    let nominal = nominal_package(&[(color.clone(), color_key.clone())]);
    for node in nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes")
    {
        f.nodes.insert(id_label(&node["node_id"]), node.clone());
    }
    let record = f.record(&[("priority", &color_key)]);
    let p = f.parameter("priority", &record);
    let app = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let ne = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.ne");
    let package = f.read();
    let result = project(&package, &app);
    assert_eq!(project(&package, &ne).operands, result.operands);
    let (_, positions) = parameter_domain(&result, 0);
    assert_eq!(positions.len(), 1);
    assert_eq!(
        positions[0].authored,
        Domain::Enum {
            ordered: true,
            members: ["Red", "Blue", "Green"].map(Into::into).to_vec()
        }
    );
    let member = json!({"version":"quire.enum-member-node/v1","declaration_node_id":node_id(&color_key),"case":"Blue"});
    let member_key = "value/enum_value/Priority/Blue".to_owned();
    let declaration = color;
    let nominal = nominal_package(&[
        (declaration, color_key.clone()),
        (member, member_key.clone()),
    ]);
    for node in nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes")
    {
        f.nodes.insert(id_label(&node["node_id"]), node.clone());
    }
    let literal = f.graph_value(
        "record_value",
        &record,
        aggregate(vec![binding("priority", reference(&member_key))]),
    );
    let app = f.application(
        vec![reference(&literal), reference(&literal)],
        "quire.op.structural.eq",
    );
    let result = project(&f.read(), &app);
    assert!(result
        .operands
        .iter()
        .all(|operand| operand.domain == OperandDomain::Literal));
    assert_eq!(
        result.operands[0].child,
        CheckedScalarOperandChild::GraphChild(typed_node_id(&literal))
    );
}

/// External exhaustive matches prove the published payloads are readable;
/// errors are selected by variants, never Display text.
fn error_payload(error: Error) -> (&'static str, Option<u64>, Option<CheckedNodeId>) {
    match error {
        Error::UnknownNode { node } => ("unknown", None, Some(node)),
        Error::NotApplication { node } => ("application", None, Some(node)),
        Error::MissingOccurrence {
            application,
            occurrence,
        } => {
            assert_eq!(occurrence.role, CheckedOccurrenceRole::Expression);
            ("occurrence", None, Some(application))
        }
        Error::UnknownOperator { application } => ("operator", None, Some(application)),
        Error::IneligibleOperator { application } => ("ineligible", None, Some(application)),
        Error::MissingChild { ordinal, child } => ("child", Some(ordinal), Some(child)),
        Error::MalformedChild { ordinal } => ("malformed child", Some(ordinal), None),
        Error::UnsupportedOperand {
            ordinal,
            type_node,
            reason,
        } => {
            match reason {
                quire_contract_model::CheckedUnsupportedCompositeOperand::InlineInteger
                | quire_contract_model::CheckedUnsupportedCompositeOperand::InlineNonInteger
                | quire_contract_model::CheckedUnsupportedCompositeOperand::ApplicationSubterm
                | quire_contract_model::CheckedUnsupportedCompositeOperand::NonStructuralType
                | quire_contract_model::CheckedUnsupportedCompositeOperand::NonliteralGraphValue => {
                }
            }
            ("operand", Some(ordinal), type_node)
        }
        Error::MalformedDomain { ordinal, type_node } => ("domain", Some(ordinal), type_node),
        Error::UnsupportedDomain { ordinal, type_node } => {
            ("unsupported domain", Some(ordinal), Some(type_node))
        }
        Error::PositionOutOfRange { ordinal, type_node } => ("position", ordinal, type_node),
        Error::WorkLimit { limit, consumed } => {
            assert!(consumed > limit || consumed == u64::MAX);
            ("work", None, None)
        }
    }
}

#[trace("TC-048", "FR-038-AC-181", "FR-038-AC-182", "FR-038-AC-199")]
#[test]
fn tc_048_external_error_consumer_retains_authentic_node_and_application_loci() {
    let mut f = Fixture::new();
    let integer = f.integer.clone();
    let record = f.record(&[("number", &integer)]);
    let p = f.parameter("p", &record);
    let eq = f.application(vec![reference(&p), reference(&p)], "quire.op.structural.eq");
    let bool_parameter = f.parameter("flag", &f.boolean.clone());
    let ineligible = f.application(
        vec![reference(&bool_parameter), reference(&bool_parameter)],
        "quire.op.boolean.and",
    );
    let package = f.read();
    let unknown = typed_node_id("composite/absent/application");
    let err = package
        .composite_application_operands(&unknown, &occurrence(0), 100_000)
        .expect_err("unknown");
    assert_eq!(error_payload(err), ("unknown", None, Some(unknown)));
    let err = package
        .composite_application_operands(&typed_node_id(&p), &occurrence(0), 100_000)
        .expect_err("not application");
    assert_eq!(
        error_payload(err),
        ("application", None, Some(typed_node_id(&p)))
    );
    let err = package
        .composite_application_operands(&typed_node_id(&eq), &occurrence(2), 100_000)
        .expect_err("absent occurrence");
    assert_eq!(
        error_payload(err),
        ("occurrence", None, Some(typed_node_id(&eq)))
    );
    let err = package
        .composite_application_operands(&typed_node_id(&ineligible), &occurrence(0), 100_000)
        .expect_err("admitted nonstructural operation");
    assert_eq!(
        error_payload(err),
        ("ineligible", None, Some(typed_node_id(&ineligible)))
    );
}
