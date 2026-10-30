// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-060 and TC-061: public V2 read/write vectors for the two new body forms.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, family_key, family_model_document,
    fixture_source, model_owner, node_id, refresh_identity, settle, sha256_hex, source_owner,
    structural_key, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedNodeId,
    CheckedOccurrence, CheckedPackageLimit, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSemanticGraphV2, CheckedSourceMapEntry,
    CheckedTerminalDuplicate,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// The shared V2 fixture grows as independent families become admissible.
fn base_node_index() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| {
        v2_all_families()["semantic_graph"]["nodes"]
            .as_array()
            .expect("fixture nodes")
            .len()
    })
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn word(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": value})
}

fn node(tag: &str, form: &str, body: Value, dependencies: &[String]) -> Value {
    let ty = family_key("aaaa");
    let id = structural_key(tag, form, Some(&ty), &body);
    let mut dependencies = dependencies.to_vec();
    dependencies.sort();
    dependencies.dedup();
    json!({
        "node_id": node_id(&id), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": node_id(&ty),
        "dependencies": dependencies.into_iter().map(|key| node_id(&key)).collect::<Vec<_>>(),
        "occurrences": [{"role": "claim", "ordinal": 0}], "body": body,
    })
}

fn rekey(node: &mut Value) {
    let key = structural_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        Some(&family_key("aaaa")),
        &node["body"],
    );
    node["node_id"] = node_id(&key);
}

fn state_graph(kind: &str, over: Value, from: Value, target: Value, deps: &[String]) -> Value {
    node(
        "claim",
        "state_graph",
        json!({"term": "aggregate", "members": [
            binding("kind", word(kind)), binding("over", over),
            binding("from", from), binding("target", target),
        ]}),
        deps,
    )
}

fn terminal(
    mode: &str,
    model: Value,
    parameters: Value,
    condition: Value,
    deps: &[String],
) -> Value {
    node(
        "temporal",
        "terminal",
        json!({"term": "aggregate", "members": [
            binding("model", model), binding("mode", word(mode)),
            binding("parameters", parameters), binding("condition", condition),
        ]}),
        deps,
    )
}

fn malformed_abstraction() -> Value {
    let body = json!({"term": "abstraction_relation", "objects": [{}],
        "populations": [], "frames": []});
    let key = sha256_hex(&canonical(&json!({
        "version": "quire.abstraction-relation-node/v1", "body": body
    })));
    json!({
        "node_id": node_id(&key), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "correspondence", "semantic_form": "abstraction_relation",
        "semantic_type": node_id(&key), "dependencies": [],
        "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
    })
}

fn malformed_hybrid_model() -> Value {
    let owner = source_owner(&fixture_source());
    let declaration = json!({"qualified_name": ["Tank"]});
    let target = family_key("aaaa");
    let body = json!({"term": "aggregate", "members": [
        binding("variables", json!({"term": "aggregate", "members": [
            {"term": "aggregate", "members": [
                binding("parameter", reference(&target)), binding("class", word("clock")),
            ]},
        ]})),
        binding("modes", json!({"term": "aggregate", "members": [reference(&target)]})),
    ]});
    let key = sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "hybrid",
        "semantic_form": "hybrid_model", "semantic_type": null,
        "declaration": declaration, "recursion": null, "owner": owner, "body": body,
    })));
    json!({
        "node_id": node_id(&key), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "hybrid", "semantic_form": "hybrid_model",
        "semantic_type": node_id(&key), "declaration": declaration, "owner": owner,
        "dependencies": [node_id(&target)],
        "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
    })
}

fn package_with(node: Value) -> Value {
    package_with_nodes(vec![node])
}

fn package_with_nodes(nodes: Vec<Value>) -> Value {
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    settle(&mut package);
    package
}

fn terminal_with_self(target_model: &str, parameter_name: &str, level: &str) -> Value {
    let mut package = v2_all_families();
    let model = family_key("1515");
    let reference_body = json!({"term": "aggregate", "members": [reference(target_model)]});
    let reference_type = structural_key("composite_type", "reference", None, &reference_body);
    let mut reference_node = node(
        "composite_type",
        "reference",
        reference_body,
        &[target_model.to_owned()],
    );
    reference_node["node_id"] = node_id(&reference_type);
    reference_node["semantic_type"] = node_id(&reference_type);
    reference_node["occurrences"] = json!([{"role": "generated", "ordinal": 0}]);
    let parameter_body = json!({"term": "aggregate", "members": [
        binding("name", word(parameter_name)),
        binding("level", json!({"term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": level})),
    ]});
    let parameter = structural_key("value", "parameter", Some(&reference_type), &parameter_body);
    let mut parameter_node = node("value", "parameter", parameter_body, &[]);
    parameter_node["node_id"] = node_id(&parameter);
    parameter_node["semantic_type"] = node_id(&reference_type);
    parameter_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let condition_body = reference(&parameter);
    let condition = structural_key(
        "expression",
        "reference",
        Some(&family_key("aaaa")),
        &condition_body,
    );
    let condition_node = node(
        "expression",
        "reference",
        condition_body,
        std::slice::from_ref(&parameter),
    );
    let mut condition_node = condition_node;
    condition_node["node_id"] = node_id(&condition);
    let terminal_node = terminal(
        "when",
        reference(&model),
        json!({"term": "aggregate", "members": [reference(&parameter)]}),
        reference(&condition),
        &[model, parameter, condition],
    );
    if target_model == other_object_key() {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(other_object_node());
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            reference_node,
            parameter_node,
            condition_node,
            terminal_node,
        ]);
    settle(&mut package);
    package
}

fn other_object_key() -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": model_owner("test/families", "ix://test/families/Other"),
        "body": empty(),
    })))
}

fn other_object_node() -> Value {
    let key = other_object_key();
    json!({
        "node_id": node_id(&key), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "model", "semantic_form": "object_type", "semantic_type": node_id(&key),
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": model_owner("test/families", "ix://test/families/Other"), "body": empty(),
    })
}

fn terminal_for_selected_model(model: &str) -> Value {
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other_object_node());
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal(
            "any",
            reference(model),
            empty(),
            empty(),
            &[model.to_owned()],
        ));
    settle(&mut package);
    package
}

fn population_document() -> Value {
    let mut document = family_model_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "test/families", "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }));
    document["populations"] = json!([{
        "identity": "ix://test/families/Accounts", "displayName": "Accounts",
        "kind": {"module": "test/families", "name": "population"},
        "members": ["ix://test/families/Account"], "extent": "closed",
        "origin": {"source": {"sourceIdentity": "ix://test/families/spec",
            "path": "models/account.md", "startLine": 1, "startColumn": 1}},
    }]);
    let mut other = document["types"][0].clone();
    other["identity"] = json!("ix://test/families/Other");
    other["displayName"] = json!("Other");
    other["fields"] = json!([]);
    other["operations"] = json!([]);
    other
        .as_object_mut()
        .expect("object type")
        .remove("relationships");
    document["types"].as_array_mut().expect("types").push(other);
    document
}

fn read_with_population(package: &Value) -> CheckedPackageV2ReadResult {
    read_with_population_limited(package, CheckedPackageReadLimits::bounded())
}

fn read_with_population_limited(
    package: &Value,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    let document = population_document();
    let evidence = evidence_with_document(package, "test/families", &document);
    CheckedPackageV2::read(&canonical(package), limits, &evidence)
}

fn second_claim_occurrence(package: &mut Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let last = nodes.last_mut().expect("claim node");
    let occurrences = last["occurrences"].as_array_mut().expect("occurrences");
    let ordinal = occurrences.len();
    occurrences.push(json!({"role": "claim", "ordinal": ordinal}));
    let first = package["source_map"]
        .as_array()
        .expect("source map")
        .last()
        .expect("entry")
        .clone();
    let mut second = first;
    second["ordinal"] = json!(ordinal);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second);
    refresh_identity(package);
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn reread_written(package: &Value) -> CheckedPackageV2 {
    let written = encoded(package);
    assert_eq!(written, canonical(package));
    match CheckedPackageV2::read(
        &written,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    ) {
        CheckedPackageV2ReadResult::Admitted(value) => *value,
        other => panic!("expected written package to admit, got {other:?}"),
    }
}

fn refused(package: &Value, code: Code, cause: Option<Cause>, suffix: &str) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, code, "{value:?}");
            assert_eq!(value.cause, cause);
            if code != Code::AmbiguousDeclaration {
                assert_eq!(value.terminal_duplicate, None);
            }
            assert!(
                value
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().ends_with(suffix)),
                "{value:?}"
            );
        }
        other => panic!("expected refusal, got {other:?}"),
    }
}

fn terminal_duplicate(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(value) => value,
        other => panic!("expected terminal duplicate refusal, got {other:?}"),
    }
}

fn duplicate_parties(refusal: &CheckedPackageRefusal) -> (CheckedNodeId, CheckedNodeId) {
    assert_eq!(refusal.code, Code::AmbiguousDeclaration);
    assert_eq!(refusal.cause, Some(Cause::AmbiguousName));
    assert_eq!(refusal.document_pointer, None);
    assert_eq!(refusal.loci.len(), 1);
    assert_eq!(refusal.extension_detail, None);
    let Some(CheckedTerminalDuplicate::Nodes { first, second }) = &refusal.terminal_duplicate
    else {
        panic!("expected two terminal nodes: {refusal:?}");
    };
    assert_eq!(refusal.locus.as_ref(), Some(second));
    (first.clone(), second.clone())
}

fn duplicate_kind(duplicate: &CheckedTerminalDuplicate) -> &'static str {
    match duplicate {
        CheckedTerminalDuplicate::Nodes { .. } => "nodes",
        CheckedTerminalDuplicate::Occurrences { .. } => "occurrences",
    }
}

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture")
}

fn encoded(package: &Value) -> Vec<u8> {
    let lock: CheckedPackageLockV2 = typed(&package["lock"]);
    let graph: CheckedSemanticGraphV2<Value> = typed(&package["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&package["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&package["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&package["diagnostics"]);
    encode_checked_package_v2(
        package["name"].as_str().expect("package name"),
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        &capabilities,
        &diagnostics,
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("writer")
}

fn source_correspondence(package: &Value, node_position: usize, start: u64) -> Value {
    let mut changed = package.clone();
    let node_id = &package["semantic_graph"]["nodes"][node_position]["node_id"];
    let entry = changed["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| &entry["node_id"] == node_id)
        .expect("node source correspondence");
    entry["regions"][0]["start"] = json!(start);
    entry["regions"][0]["end"] = json!(start + 1);
    assert_ne!(changed["source_map"], package["source_map"]);
    changed
}

#[trace("FR-059-AC-1", "FR-059-AC-5")]
#[test]
fn state_graph_forms_read_and_write() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    for (kind, from, deps) in [
        ("possible", empty(), vec![over.clone(), predicate.clone()]),
        (
            "always-possible",
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
        ),
        (
            "unique-path",
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
        ),
    ] {
        let graph_node = state_graph(kind, reference(&over), from, reference(&predicate), &deps);
        let package = package_with(graph_node.clone());
        let admitted = admitted(&package);
        assert_eq!(
            admitted
                .graph()
                .nodes
                .last()
                .expect("claim")
                .semantic_form
                .as_ref(),
            "state_graph"
        );
        let written = encoded(&package);
        assert_eq!(written, canonical(&package));
        let reread = match CheckedPackageV2::read(
            &written,
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&package),
        ) {
            CheckedPackageV2ReadResult::Admitted(value) => value,
            other => panic!("writer output refused: {other:?}"),
        };
        assert_eq!(reread.graph(), admitted.graph());
        assert_eq!(reread.source_map(), admitted.source_map());
    }
}

#[trace("FR-060-AC-1", "FR-060-AC-5")]
#[test]
fn terminal_any_reads_and_writes() {
    let model = family_key("1515");
    let terminal_node = terminal("any", reference(&model), empty(), empty(), &[model]);
    let package = package_with(terminal_node);
    let admitted = admitted(&package);
    assert_eq!(
        admitted
            .graph()
            .nodes
            .last()
            .expect("terminal")
            .semantic_form
            .as_ref(),
        "terminal"
    );
    let written = encoded(&package);
    assert_eq!(written, canonical(&package));
    let reread = match CheckedPackageV2::read(
        &written,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("writer output refused: {other:?}"),
    };
    assert_eq!(reread.graph(), admitted.graph());
    assert_eq!(reread.source_map(), admitted.source_map());
}

#[trace("FR-059-AC-2", "FR-059-AC-5")]
#[test]
fn state_graph_refuses_kind_fit_and_target_meaning() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let baseline = || (reference(&over), empty(), reference(&predicate));
    for (case, kind, over_term, from_term, target_term, deps, code, cause, suffix) in [
        {
            let (over_term, from_term, target_term) = baseline();
            (
                "SN-09",
                "eventually-possible",
                over_term,
                from_term,
                target_term,
                vec![over.clone(), predicate.clone()],
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                "/body/members/0",
            )
        },
        (
            "SN-10",
            "possible",
            reference(&over),
            reference(&predicate),
            reference(&predicate),
            vec![over.clone(), predicate.clone()],
            Code::InvalidSemanticGraph,
            None,
            "/body/members/2",
        ),
        (
            "SN-11",
            "possible",
            reference(&over),
            empty(),
            reference(&over),
            vec![over.clone()],
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            "/body/members/3/value/target",
        ),
        (
            "SN-14",
            "possible",
            word("over"),
            empty(),
            reference(&predicate),
            vec![predicate.clone()],
            Code::InvalidSemanticGraph,
            None,
            "/body/members/1",
        ),
    ] {
        let package = package_with(state_graph(kind, over_term, from_term, target_term, &deps));
        match read(&package) {
            CheckedPackageV2ReadResult::Refused(value) => {
                assert_eq!(value.code, code, "{case}");
                assert_eq!(value.cause, cause, "{case}");
                assert!(
                    value
                        .path
                        .as_ref()
                        .is_some_and(|path| path.as_str().ends_with(suffix)),
                    "{case}: {value:?}"
                );
            }
            other => panic!("{case}: expected refusal, got {other:?}"),
        }
    }
    let mut missing = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&predicate),
        std::slice::from_ref(&over),
    ));
    // The body's target remains present, while the written dependency is absent.
    settle(&mut missing);
    refused(&missing, Code::InvalidSemanticGraph, None, "/dependencies");
    let mut reordered = state_graph(
        "always-possible",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    reordered["body"]["members"]
        .as_array_mut()
        .expect("members")
        .swap(2, 3);
    rekey(&mut reordered);
    refused(
        &package_with(reordered),
        Code::InvalidSemanticGraph,
        None,
        "/body",
    );
    let absent_from = package_with(state_graph(
        "unique-path",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    ));
    refused(
        &absent_from,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/2",
    );
    let temporal_target = v2_all_families()["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let wrong_predicate = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&temporal_target),
        &[over, temporal_target],
    ));
    refused(
        &wrong_predicate,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/members/3/value/target",
    );
}

#[trace("FR-060-AC-1", "FR-060-AC-2", "FR-060-AC-5")]
#[test]
fn terminal_when_and_mode_refusals() {
    let model = family_key("1515");
    let predicate = family_key("eeee");
    let when = terminal(
        "when",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    );
    let package = package_with(when);
    let _ = admitted(&package);
    let bad_mode = package_with(terminal(
        "some",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    refused(
        &bad_mode,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let bad_fit = package_with(terminal(
        "any",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    refused(
        &bad_fit,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/3",
    );
    let wrong_model = family_key("a4a4");
    let bad_model = package_with(terminal(
        "when",
        reference(&wrong_model),
        empty(),
        reference(&predicate),
        &[wrong_model, predicate],
    ));
    refused(
        &bad_model,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/members/0/value/target",
    );
    let mut reordered = terminal(
        "when",
        reference(&model),
        empty(),
        reference(&family_key("eeee")),
        &[model.clone(), family_key("eeee")],
    );
    reordered["body"]["members"]
        .as_array_mut()
        .expect("members")
        .swap(1, 3);
    rekey(&mut reordered);
    refused(
        &package_with(reordered),
        Code::InvalidSemanticGraph,
        None,
        "/body",
    );
    let missing_condition = package_with(terminal(
        "when",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    ));
    refused(
        &missing_condition,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/3",
    );
    let extra_parameters = json!({"term": "aggregate", "members": [
        reference(&family_key("a4a4")), reference(&family_key("a4a4"))]});
    let two_parameters = package_with(terminal(
        "when",
        reference(&model),
        extra_parameters,
        reference(&family_key("eeee")),
        &[model.clone(), family_key("a4a4"), family_key("eeee")],
    ));
    refused(
        &two_parameters,
        Code::InvalidSemanticGraph,
        None,
        "/body/members/2",
    );
    let both = package_with(terminal(
        "any",
        reference(&model),
        json!({"term": "aggregate", "members": [reference(&family_key("a4a4"))]}),
        reference(&family_key("eeee")),
        &[model, family_key("a4a4"), family_key("eeee")],
    ));
    refused(&both, Code::InvalidSemanticGraph, None, "/body/members/2");
    let integer = family_key("a3a3");
    let integer_body = reference(&integer);
    let integer_condition =
        structural_key("expression", "reference", Some(&integer), &integer_body);
    let mut expression = node(
        "expression",
        "reference",
        integer_body,
        std::slice::from_ref(&integer),
    );
    expression["node_id"] = node_id(&integer_condition);
    expression["semantic_type"] = node_id(&integer);
    let mut wrong_condition = package_with(terminal(
        "when",
        reference(&family_key("1515")),
        empty(),
        reference(&integer_condition),
        &[family_key("1515"), integer_condition],
    ));
    wrong_condition["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .insert(base_node_index(), expression);
    settle(&mut wrong_condition);
    refused(
        &wrong_condition,
        Code::IllTyped,
        Some(Cause::NonBooleanRoot),
        "/body/members/3/value/target",
    );
}

#[trace("FR-059-AC-1", "FR-059-AC-4", "FR-059-AC-5")]
#[test]
fn state_graph_identity_and_shared_occurrences() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let deps = [over.clone(), predicate.clone()];
    let possible = package_with(state_graph(
        "possible",
        reference(&over),
        empty(),
        reference(&predicate),
        &deps,
    ));
    let always = package_with(state_graph(
        "always-possible",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &deps,
    ));
    assert_ne!(
        possible["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        always["semantic_graph"]["nodes"][base_node_index()]["node_id"]
    );
    assert_ne!(encoded(&possible), encoded(&always));
    let moved = source_correspondence(&possible, base_node_index(), 100);
    assert_eq!(
        possible["semantic_graph"]["nodes"][base_node_index()],
        moved["semantic_graph"]["nodes"][base_node_index()]
    );
    assert_eq!(possible["name"], moved["name"]);
    assert_ne!(possible["source_map"], moved["source_map"]);
    let moved_read = admitted(&moved);
    let moved_reread = reread_written(&moved);
    let expected_source: Vec<CheckedSourceMapEntry> = typed(&moved["source_map"]);
    assert_eq!(moved_read.source_map(), expected_source);
    assert_eq!(moved_reread.source_map(), moved_read.source_map());
    assert_eq!(
        moved_reread.graph().nodes[base_node_index()],
        moved_read.graph().nodes[base_node_index()]
    );
    let mut instance_a = possible.clone();
    instance_a["source_map"][base_node_index()]["regions"][0]["start"] = json!(100);
    instance_a["source_map"][base_node_index()]["regions"][0]["end"] = json!(101);
    let mut instance_b = possible.clone();
    instance_b["source_map"][base_node_index()]["regions"][0]["start"] = json!(200);
    instance_b["source_map"][base_node_index()]["regions"][0]["end"] = json!(201);
    for instance in [&instance_a, &instance_b] {
        let read = admitted(instance);
        let written = reread_written(instance);
        assert_eq!(
            read.graph().nodes[base_node_index()].node_id,
            admitted(&possible).graph().nodes[base_node_index()].node_id
        );
        assert_eq!(read.name(), admitted(&possible).name());
        assert_eq!(written.source_map(), read.source_map());
        assert_eq!(
            written.graph().nodes[base_node_index()],
            read.graph().nodes[base_node_index()]
        );
    }
    assert_ne!(instance_a["source_map"], instance_b["source_map"]);
    assert_eq!(
        instance_a["semantic_graph"]["nodes"][base_node_index()],
        instance_b["semantic_graph"]["nodes"][base_node_index()]
    );
    let mut shared = possible.clone();
    second_claim_occurrence(&mut shared);
    let admitted = admitted(&shared);
    let claim = admitted.graph().nodes.last().expect("claim");
    assert_eq!(claim.occurrences.len(), 2);
    assert_eq!(
        admitted
            .source_map()
            .iter()
            .filter(|entry| entry.node_id == claim.node_id)
            .count(),
        2
    );
    assert_eq!(possible["name"], shared["name"]);
    assert!(matches!(
        CheckedPackageV2::read(
            &encoded(&shared),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&shared)
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("FR-059-AC-1", "FR-059-AC-5")]
#[test]
fn state_graph_constant_predicate_and_absent_over() {
    let boolean = family_key("aaaa");
    let truth_body = json!({"term": "literal", "type": node_id(&boolean),
        "value_kind": "boolean", "value": true});
    let truth = structural_key("value", "literal", Some(&boolean), &truth_body);
    let mut truth_node = node("value", "literal", truth_body, &[]);
    truth_node["node_id"] = node_id(&truth);
    truth_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let possible = package_with_nodes(vec![
        truth_node.clone(),
        state_graph(
            "possible",
            empty(),
            empty(),
            reference(&truth),
            std::slice::from_ref(&truth),
        ),
    ]);
    assert_eq!(
        admitted(&possible)
            .graph()
            .nodes
            .last()
            .expect("claim")
            .dependencies
            .len(),
        1
    );
    let predicate = family_key("eeee");
    let always = package_with_nodes(vec![
        truth_node,
        state_graph(
            "always-possible",
            empty(),
            reference(&truth),
            reference(&predicate),
            &[truth, predicate],
        ),
    ]);
    let _ = admitted(&always);
}

#[trace(
    "FR-060-AC-2",
    "FR-060-AC-4",
    "FR-060-AC-5",
    "FR-062-AC-1",
    "FR-062-AC-2",
    "FR-062-AC-3",
    "FR-062-AC-4"
)]
#[test]
fn terminal_identity_and_uniqueness_per_model() {
    let model = family_key("1515");
    let predicate = family_key("eeee");
    let any = package_with(terminal(
        "any",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    ));
    let when = package_with(terminal(
        "when",
        reference(&model),
        empty(),
        reference(&predicate),
        &[model.clone(), predicate.clone()],
    ));
    assert_ne!(
        any["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        when["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        "forms: {:?} {:?}",
        any["semantic_graph"]["nodes"][base_node_index()]["semantic_form"],
        when["semantic_graph"]["nodes"][base_node_index()]["semantic_form"]
    );
    assert_ne!(encoded(&any), encoded(&when));
    let done = source_correspondence(&any, base_node_index(), 100);
    let stop = source_correspondence(&any, base_node_index(), 200);
    assert_eq!(
        done["semantic_graph"]["nodes"][base_node_index()],
        stop["semantic_graph"]["nodes"][base_node_index()]
    );
    assert_eq!(done["name"], stop["name"]);
    assert_ne!(done["source_map"], stop["source_map"]);
    for named in [&done, &stop] {
        let read = admitted(named);
        let written = reread_written(named);
        assert_eq!(
            read.graph().nodes[base_node_index()].node_id,
            admitted(&any).graph().nodes[base_node_index()].node_id
        );
        assert_eq!(read.name(), admitted(&any).name());
        assert_eq!(written.source_map(), read.source_map());
        assert_eq!(
            written.graph().nodes[base_node_index()],
            read.graph().nodes[base_node_index()]
        );
    }
    assert_ne!(encoded(&any), encoded(&v2_all_families()));
    let changed_body = reference(&predicate);
    let changed_condition = structural_key(
        "expression",
        "reference",
        Some(&family_key("aaaa")),
        &changed_body,
    );
    let mut changed_node = node(
        "expression",
        "reference",
        changed_body,
        std::slice::from_ref(&predicate),
    );
    changed_node["node_id"] = node_id(&changed_condition);
    changed_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let changed = package_with_nodes(vec![
        changed_node,
        terminal(
            "when",
            reference(&model),
            empty(),
            reference(&changed_condition),
            &[model.clone(), changed_condition],
        ),
    ]);
    let _ = admitted(&changed);
    assert_ne!(
        when["semantic_graph"]["nodes"][base_node_index()]["node_id"],
        changed["semantic_graph"]["nodes"][base_node_index() + 1]["node_id"]
    );
    assert_ne!(encoded(&when), encoded(&changed));
    let mut duplicate = any.clone();
    duplicate["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(when["semantic_graph"]["nodes"][base_node_index()].clone());
    settle(&mut duplicate);
    let duplicate_result = terminal_duplicate(&duplicate);
    assert_eq!(
        duplicate_kind(
            duplicate_result
                .terminal_duplicate
                .as_ref()
                .expect("payload")
        ),
        "nodes"
    );
    let (first, second) = duplicate_parties(&duplicate_result);
    let any_id: CheckedNodeId =
        typed(&any["semantic_graph"]["nodes"][base_node_index()]["node_id"]);
    let when_id: CheckedNodeId =
        typed(&when["semantic_graph"]["nodes"][base_node_index()]["node_id"]);
    assert_eq!(
        first,
        if any_id < when_id {
            any_id.clone()
        } else {
            when_id.clone()
        }
    );
    assert_eq!(second, if first == any_id { when_id } else { any_id });
    let second_position = duplicate["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == json!(second))
        .expect("second terminal position");
    assert_eq!(
        duplicate_result.loci[0].path.as_ref().map(AsRef::as_ref),
        Some(format!("/semantic_graph/nodes/{second_position}").as_str())
    );
    let mut reversed = duplicate.clone();
    reversed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(base_node_index(), base_node_index() + 1);
    settle(&mut reversed);
    let reversed_result = terminal_duplicate(&reversed);
    assert_eq!(duplicate_parties(&reversed_result), (first, second));
    let mut bad_target = duplicate.clone();
    let terminal = &mut bad_target["semantic_graph"]["nodes"][base_node_index() + 1];
    terminal["body"]["members"][3]["value"] = reference(&model);
    terminal["dependencies"] = json!([node_id(&model)]);
    rekey(terminal);
    settle(&mut bad_target);
    refused(
        &bad_target,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/members/3/value/target",
    );
    let mut repeated = any;
    second_claim_occurrence(&mut repeated);
    let repeated_result = terminal_duplicate(&repeated);
    assert_eq!(
        duplicate_kind(
            repeated_result
                .terminal_duplicate
                .as_ref()
                .expect("payload")
        ),
        "occurrences"
    );
    assert_eq!(repeated_result.code, Code::AmbiguousDeclaration);
    assert_eq!(repeated_result.cause, Some(Cause::AmbiguousName));
    assert_eq!(
        repeated_result.loci[0].path.as_ref().map(AsRef::as_ref),
        Some(format!("/semantic_graph/nodes/{}", base_node_index()).as_str())
    );
    assert_eq!(repeated_result.document_pointer, None);
    assert_eq!(repeated_result.loci.len(), 1);
    assert_eq!(repeated_result.extension_detail, None);
    let repeated_id: CheckedNodeId =
        typed(&repeated["semantic_graph"]["nodes"][base_node_index()]["node_id"]);
    let (first_occurrence, second_occurrence) = match repeated_result.terminal_duplicate {
        Some(CheckedTerminalDuplicate::Occurrences {
            node,
            first,
            second,
        }) => {
            assert_eq!(node, repeated_id);
            assert_eq!(repeated_result.locus, Some(node));
            (first, second)
        }
        other => panic!("expected duplicate occurrences, got {other:?}"),
    };
    assert_eq!(
        first_occurrence,
        (CheckedOccurrence {
            role: quire_contract_model::CheckedOccurrenceRole::Claim,
            ordinal: 0
        })
    );
    assert_eq!(
        second_occurrence,
        (CheckedOccurrence {
            role: quire_contract_model::CheckedOccurrenceRole::Claim,
            ordinal: 1
        })
    );
    second_claim_occurrence(&mut repeated);
    let third_result = terminal_duplicate(&repeated);
    assert_eq!(
        third_result.terminal_duplicate,
        Some(CheckedTerminalDuplicate::Occurrences {
            node: repeated_id,
            first: first_occurrence,
            second: second_occurrence,
        })
    );
    let first_model = terminal_for_selected_model(&model);
    let second_model = terminal_for_selected_model(&other_object_key());
    assert!(matches!(
        read_with_population(&first_model),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert!(matches!(
        read_with_population(&second_model),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_ne!(
        first_model["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "terminal")
            .expect("first terminal")["node_id"],
        second_model["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "terminal")
            .expect("second terminal")["node_id"]
    );
    assert_ne!(encoded(&first_model), encoded(&second_model));
}

#[trace("FR-060-AC-1", "FR-060-AC-2", "FR-060-AC-5")]
#[test]
fn terminal_self_parameter_must_have_right_name_level_and_type() {
    let model = family_key("1515");
    let valid = terminal_with_self(&model, "self", "0");
    assert!(matches!(
        read_with_population(&valid),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut omitted = valid.clone();
    let terminal = omitted["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "terminal")
        .expect("terminal");
    let parameter_id = terminal["body"]["members"][2]["value"]["members"][0]["target"].clone();
    terminal["body"]["members"][2]["value"] = empty();
    terminal["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|id| *id != parameter_id);
    rekey(terminal);
    settle(&mut omitted);
    match read_with_population(&omitted) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::InvalidSemanticGraph);
            assert!(
                value
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().ends_with("/body/members/2")),
                "{value:?}"
            );
        }
        other => panic!("expected omitted self refusal, got {other:?}"),
    }
    for (name, level) in [("result", "1"), ("self", "1")] {
        let invalid = terminal_with_self(&model, name, level);
        match read_with_population(&invalid) {
            CheckedPackageV2ReadResult::Refused(value) => {
                assert_eq!(value.code, Code::IllTyped);
                assert_eq!(value.cause, Some(Cause::OperatorIneligible));
                assert!(
                    value.loci[0].path.as_ref().is_some_and(|path| path
                        .as_str()
                        .ends_with("/body/members/2/value/members/0/target")),
                    "{value:?}"
                );
            }
            other => panic!("expected self refusal, got {other:?}"),
        }
    }
    let other_model = other_object_key();
    let mistyped = terminal_with_self(&other_model, "self", "0");
    match read_with_population(&mistyped) {
        CheckedPackageV2ReadResult::Refused(value) => {
            assert_eq!(value.code, Code::IllTyped);
            assert_eq!(value.cause, Some(Cause::OperatorIneligible));
        }
        other => panic!("expected mistyped self refusal, got {other:?}"),
    }
}

#[trace("FR-060-AC-1", "FR-038-AC-121")]
#[test]
fn terminal_self_population_scan_obeys_work_limit() {
    let package = terminal_with_self(&family_key("1515"), "self", "0");
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        read_with_population_limited(&package, limits)
    };
    let (mut lower, mut upper) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read_at(lower),
        CheckedPackageV2ReadResult::Incomplete(_)
    ));
    assert!(matches!(
        read_at(upper),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while lower + 1 < upper {
        let middle = lower + (upper - lower) / 2;
        if matches!(read_at(middle), CheckedPackageV2ReadResult::Incomplete(_)) {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    assert!(matches!(
        read_at(upper),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut charged_population = false;
    for work in (upper.saturating_sub(2_048)..upper).rev() {
        let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(work) else {
            panic!("work below exact decision boundary must be incomplete");
        };
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
        if incomplete
            .path
            .as_ref()
            .is_some_and(|path| path.as_str().ends_with("/body/members/2"))
        {
            assert_eq!(incomplete.consumed, work + 1);
            charged_population = true;
            break;
        }
    }
    assert!(
        charged_population,
        "self population traversal must have a charged boundary"
    );
}

#[trace("FR-059-AC-3", "FR-060-AC-3")]
#[test]
fn graph_shape_then_state_graph_then_terminal_order() {
    let over = family_key("a4a4");
    let predicate = family_key("eeee");
    let model = family_key("1515");
    let bad_claim = state_graph(
        "not-a-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    let bad_terminal = terminal(
        "not-a-mode",
        reference(&model),
        empty(),
        empty(),
        std::slice::from_ref(&model),
    );
    let both = package_with_nodes(vec![bad_terminal.clone(), bad_claim.clone()]);
    refused(
        &both,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let mixed = package_with(state_graph(
        "not-a-kind",
        reference(&over),
        reference(&predicate),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    ));
    refused(
        &mixed,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let second_bad_claim = state_graph(
        "another-invalid-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over.clone(), predicate.clone()],
    );
    let first_digest = bad_claim["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let second_digest = second_bad_claim["node_id"]["path"][0]
        .as_str()
        .expect("digest")
        .to_owned();
    let by_digest = package_with_nodes(vec![second_bad_claim, bad_claim.clone()]);
    refused(
        &by_digest,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        &format!(
            "/semantic_graph/nodes/{}/body/members/0",
            if second_digest < first_digest {
                base_node_index()
            } else {
                base_node_index() + 1
            }
        ),
    );
    let no_dependency = state_graph(
        "not-a-kind",
        reference(&over),
        empty(),
        reference(&predicate),
        &[over],
    );
    let graph_first = package_with_nodes(vec![bad_terminal.clone(), no_dependency]);
    refused(
        &graph_first,
        Code::InvalidSemanticGraph,
        None,
        "/dependencies",
    );
    let valid_terminal = terminal("any", reference(&model), empty(), empty(), &[model]);
    let terminal_first = package_with_nodes(vec![
        valid_terminal,
        terminal(
            "not-a-mode",
            reference(&family_key("1515")),
            empty(),
            empty(),
            &[family_key("1515")],
        ),
    ]);
    refused(
        &terminal_first,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let mut bad_temporal = package_with_nodes(vec![bad_claim.clone(), bad_terminal.clone()]);
    let temporal_position = bad_temporal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "temporal_clause")
        .expect("temporal clause");
    bad_temporal["semantic_graph"]["nodes"][temporal_position]["body"]["arguments"][0] =
        reference(&family_key("1515"));
    settle(&mut bad_temporal);
    refused(
        &bad_temporal,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let mut temporal_before_terminal = bad_temporal.clone();
    temporal_before_terminal["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| node["semantic_form"] != "state_graph");
    settle(&mut temporal_before_terminal);
    refused(
        &temporal_before_terminal,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/arguments/0",
    );
    let abstraction_before_terminal =
        package_with_nodes(vec![malformed_abstraction(), bad_terminal.clone()]);
    refused(
        &abstraction_before_terminal,
        Code::InvalidSemanticGraph,
        None,
        "/body/objects/0",
    );
    let claim_before_abstraction = package_with_nodes(vec![bad_claim, malformed_abstraction()]);
    refused(
        &claim_before_abstraction,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/0",
    );
    let wrong_protocol_target = family_key("aaaa");
    let mut bad_protocol_reference = node(
        "protocol",
        "protocol_reference",
        reference(&wrong_protocol_target),
        std::slice::from_ref(&wrong_protocol_target),
    );
    bad_protocol_reference["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let bad_protocol_only = package_with(bad_protocol_reference.clone());
    refused(
        &bad_protocol_only,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/target",
    );
    let protocol_before_terminal =
        package_with_nodes(vec![bad_terminal.clone(), bad_protocol_reference]);
    refused(
        &protocol_before_terminal,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        "/body/target",
    );
    let bad_hybrid_only = package_with(malformed_hybrid_model());
    refused(
        &bad_hybrid_only,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/members/0/value/members/0/members/0/value/target",
    );
    let terminal_before_hybrid =
        package_with_nodes(vec![bad_terminal.clone(), malformed_hybrid_model()]);
    refused(
        &terminal_before_hybrid,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
    let mut terminal_before_operations = package_with(bad_terminal);
    let function_position = terminal_before_operations["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function")
        .expect("function node");
    terminal_before_operations["semantic_graph"]["nodes"][function_position]["body"]["operation"]
        ["identity"] = json!("quire.op.unknown");
    settle(&mut terminal_before_operations);
    refused(
        &terminal_before_operations,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/body/members/1",
    );
}

#[trace("FR-059-AC-2", "FR-060-AC-2")]
#[test]
fn unknown_forms_remain_closed() {
    for (tag, form) in [("claim", "other_claim"), ("temporal", "other_temporal")] {
        let package = package_with(node(tag, form, empty(), &[]));
        refused(&package, Code::InvalidSemanticGraph, None, "/semantic_form");
    }
}
