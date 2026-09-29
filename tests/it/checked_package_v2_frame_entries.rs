// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-056: the V2 reader admits QSpec FR-340's frame `modifies` entries,
//! FR-342's operation anchor body and FR-341's state clause and parameter
//! bodies, and exactly QSpec's fifteen `model` forms, refusing every other
//! shape with the code, cause and locus FR-040 fixes, in reader order.
//!
//! Every case builds its package in-repo from this crate's public
//! vocabulary.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, rebuild_source_map, refresh_identity, rekey_application_node,
    sha256_hex, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

use CheckedPackageRefusalCause as Cause;
use CheckedPackageRefusalCode as Code;

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn admits(value: &Value) {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, read {other:?}"),
    }
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause, pointer and (when given) locus digest.
fn expect(
    case: &str,
    value: &Value,
    code: Code,
    cause: Option<Cause>,
    path: &str,
    locus: Option<&str>,
) {
    let refusal = refused(value);
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
    if let Some(locus) = locus {
        assert_eq!(
            refusal.locus.as_ref().map(|id| &*id.digest),
            Some(locus),
            "{case}: {refusal:?}"
        );
    }
}

fn nodes(value: &Value) -> &Vec<Value> {
    value["semantic_graph"]["nodes"].as_array().expect("nodes")
}

fn position(value: &Value, tag: &str, form: &str) -> usize {
    nodes(value)
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("a {tag}/{form} node"))
}

fn digest(value: &Value, position: usize) -> String {
    nodes(value)[position]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned()
}

fn field(declaration: &str, name: &str) -> Value {
    json!({"kind": "field", "declaration": node_id(declaration), "name": name})
}

fn relationship(declaration: &str) -> Value {
    json!({"kind": "relationship", "declaration": node_id(declaration)})
}

/// `v2_all_families()` with `edit` applied to its frame node, identity
/// re-derived.
fn frame_mutation(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut value = v2_all_families();
    let frame = position(&value, "state", "frame");
    edit(&mut value["semantic_graph"]["nodes"][frame]);
    rebuild_source_map(&mut value);
    refresh_identity(&mut value);
    (value, frame)
}

/// A join case: name, `modifies`, `creates`, `dependencies`, code, cause,
/// locus digest, member and entry index.
type JoinCase<'a> = (
    &'a str,
    Value,
    Value,
    Value,
    Code,
    Cause,
    &'a str,
    &'a str,
    usize,
);

/// An edit of one clause body.
type Edit = Box<dyn Fn(&mut Value)>;

const OBJECT: &str = "1515151515151515151515151515151515151515151515151515151515151515";
const PROCESS: &str = "1616161616161616161616161616161616161616161616161616161616161616";
const RELATIONSHIP: &str = "2020202020202020202020202020202020202020202020202020202020202020";
const NAMESPACE: &str = "1010101010101010101010101010101010101010101010101010101010101010";

/// Tracing: TC-056
/// ACs: FR-040-AC-1
#[trace("TC-056", "FR-040-AC-1")]
#[test]
fn tc_056_modifies_entries_admit_in_their_closed_shape_only() {
    // The base frame holds a field entry and a relationship entry.
    let base = v2_all_families();
    let frame = position(&base, "state", "frame");
    assert_eq!(
        nodes(&base)[frame]["body"]["modifies"],
        json!([field(OBJECT, "balance"), relationship(RELATIONSHIP)])
    );
    admits(&base);
    let (empty, _) = frame_mutation(|frame| {
        frame["body"] = json!({"term": "frame", "modifies": [], "creates": [], "deletes": []});
        frame["dependencies"] = json!([]);
    });
    admits(&empty);
    let entry = |value: Value| frame_mutation(move |frame| frame["body"]["modifies"][1] = value);
    for (case, value) in [
        ("a bare node key", node_id(RELATIONSHIP)),
        (
            "another kind",
            json!({"kind": "operation", "declaration": node_id(RELATIONSHIP)}),
        ),
        (
            "a field entry without name",
            json!({"kind": "field", "declaration": node_id(RELATIONSHIP)}),
        ),
        (
            "a relationship entry carrying name",
            json!({"kind": "relationship", "declaration": node_id(RELATIONSHIP), "name": "x"}),
        ),
        (
            "an extra member",
            json!({"kind": "relationship", "declaration": node_id(RELATIONSHIP), "at": 1}),
        ),
        (
            "a field name that is not an identifier",
            json!({"kind": "field", "declaration": node_id(OBJECT), "name": "a b"}),
        ),
    ] {
        let (value, frame) = entry(value);
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{frame}/body/modifies/1"),
            None,
        );
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-2, FR-040-AC-5
#[trace("TC-056", "FR-040-AC-2", "FR-040-AC-5")]
#[test]
fn tc_056_frame_entries_and_semantic_type_join_declared_dependencies() {
    let at = |frame: usize, member: &str, entry: usize| {
        format!("/semantic_graph/nodes/{frame}/body/{member}/{entry}")
    };
    let cases: [JoinCase<'_>; 6] = [
        (
            "a relationship entry naming an object type",
            json!([relationship(OBJECT)]),
            json!([]),
            json!([node_id(OBJECT)]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            OBJECT,
            "modifies",
            0,
        ),
        (
            "a field entry on a process",
            json!([field(PROCESS, "x")]),
            json!([]),
            json!([node_id(PROCESS)]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            PROCESS,
            "modifies",
            0,
        ),
        (
            "a field entry on a relationship",
            json!([field(RELATIONSHIP, "x")]),
            json!([]),
            json!([node_id(RELATIONSHIP)]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            RELATIONSHIP,
            "modifies",
            0,
        ),
        (
            "a creates entry naming a relationship",
            json!([]),
            json!([node_id(RELATIONSHIP)]),
            json!([node_id(RELATIONSHIP)]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            RELATIONSHIP,
            "creates",
            0,
        ),
        (
            "an entry naming no declared dependency",
            json!([relationship(RELATIONSHIP)]),
            json!([]),
            json!([]),
            Code::MissingDeclaration,
            Cause::MissingName,
            RELATIONSHIP,
            "modifies",
            0,
        ),
        (
            "an entry naming no node",
            json!([relationship(&"ab".repeat(32))]),
            json!([]),
            json!([node_id(&"ab".repeat(32))]),
            Code::MissingDeclaration,
            Cause::MissingName,
            &"ab".repeat(32),
            "modifies",
            0,
        ),
    ];
    for (case, modifies, creates, dependencies, code, cause, locus, member, entry) in cases {
        let (value, frame) = frame_mutation(|frame| {
            frame["body"] =
                json!({"term": "frame", "modifies": modifies, "creates": creates, "deletes": []});
            frame["dependencies"] = dependencies;
        });
        expect(
            case,
            &value,
            code,
            Some(cause),
            &at(frame, member, entry),
            Some(locus),
        );
    }

    // A `semantic_type` that is not an object type refuses before an entry
    // defect.
    let (value, frame) = frame_mutation(|frame| {
        frame["semantic_type"] = node_id(PROCESS);
        frame["body"]["modifies"] = json!([relationship(OBJECT)]);
    });
    expect(
        "semantic_type naming a process",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{frame}/semantic_type"),
        Some(PROCESS),
    );
    let (value, frame) = frame_mutation(|frame| frame["semantic_type"] = node_id(&"ab".repeat(32)));
    expect(
        "semantic_type naming no node",
        &value,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &format!("/semantic_graph/nodes/{frame}/semantic_type"),
        Some(&"ab".repeat(32)),
    );
    // A frame occurrence of another role.
    let (value, frame) =
        frame_mutation(|frame| frame["occurrences"] = json!([{"role": "claim", "ordinal": 0}]));
    expect(
        "a claim-role frame occurrence",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{frame}/occurrences/0/role"),
        None,
    );
}

/// Tracing: TC-056
/// ACs: FR-040-AC-3, FR-040-AC-4
#[trace("TC-056", "FR-040-AC-3", "FR-040-AC-4")]
#[test]
fn tc_056_field_entries_order_by_declaration_then_name() {
    // Two field entries of one declaring node, ascending by name, admit;
    // descending refuses at the frame body, located at the frame.
    let (ascending, _) = frame_mutation(|frame| {
        frame["body"]["modifies"] = json!([field(OBJECT, "balance"), field(OBJECT, "total")]);
    });
    admits(&ascending);
    let (descending, frame) = frame_mutation(|frame| {
        frame["body"]["modifies"] = json!([field(OBJECT, "total"), field(OBJECT, "balance")]);
    });
    let frame_digest = digest(&descending, frame);
    expect(
        "descending names",
        &descending,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{frame}/body"),
        Some(&frame_digest),
    );
    // A meaning-join defect in `deletes` outranks an order defect in
    // `modifies`, wherever each sits.
    let (both, frame) = frame_mutation(|frame| {
        frame["body"]["modifies"] = json!([field(OBJECT, "total"), field(OBJECT, "balance")]);
        frame["body"]["deletes"] = json!([node_id(RELATIONSHIP)]);
    });
    expect(
        "meaning join over order",
        &both,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{frame}/body/deletes/0"),
        Some(RELATIONSHIP),
    );
    // A field entry on a source-declared object type is not resolved against
    // a domain package.
    let (unresolved, _) = frame_mutation(|frame| {
        frame["body"]["modifies"][0] = field(OBJECT, "anything");
    });
    admits(&unresolved);
}

/// Tracing: TC-056
/// ACs: FR-040-AC-3
#[trace("TC-056", "FR-040-AC-3")]
#[test]
fn tc_056_record_value_type_field_entries_admit_unresolved() {
    let base = v2_all_families();
    let record = nodes(&base)
        .iter()
        .position(|node| node["node_id"]["digest"] == NAMESPACE)
        .expect("the namespace node");
    let with_record = |declared: bool| {
        let mut value = base.clone();
        let node = &mut value["semantic_graph"]["nodes"][record];
        node["semantic_form"] = json!("record_value_type");
        if declared {
            node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
            node["declaration"] = json!({"qualified_name": ["Example", "Address"]});
        }
        let frame = position(&value, "state", "frame");
        let frame = &mut value["semantic_graph"]["nodes"][frame];
        frame["dependencies"]
            .as_array_mut()
            .expect("frame dependencies")
            .push(node_id(NAMESPACE));
        frame["body"]["modifies"] = json!([
            field(NAMESPACE, "street"),
            field(OBJECT, "balance"),
            relationship(RELATIONSHIP)
        ]);
        rebuild_source_map(&mut value);
        refresh_identity(&mut value);
        value
    };
    // With no `declaration` the node is not owned by any domain package, so
    // its field name is not resolved; the source-declared node likewise.
    admits(&with_record(false));
    admits(&with_record(true));
}

/// Tracing: TC-056
/// ACs: FR-040-AC-6
#[trace("TC-056", "FR-040-AC-6")]
#[test]
fn tc_056_exactly_fifteen_model_forms_admit() {
    let base = v2_all_families();
    let model = nodes(&base)
        .iter()
        .position(|node| node["node_id"]["digest"] == NAMESPACE)
        .expect("the namespace node");
    let with_form = |form: &str| {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][model]["semantic_form"] = json!(form);
        refresh_identity(&mut value);
        value
    };
    for form in [
        "model_import",
        "object_type",
        "value_type",
        "variant_type",
        "record_value_type",
        "event_type",
        "state_machine",
        "process",
        "persistence_interface",
        "namespace",
        "systems_interface",
        "systems_part",
        "systems_port",
        "systems_connection",
        "systems_allocation",
    ] {
        admits(&with_form(form));
    }
    for form in [
        "field_declaration",
        "operation_declaration",
        "clause_member_declaration",
        "model_export",
    ] {
        expect(
            form,
            &with_form(form),
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{model}/semantic_form"),
            None,
        );
    }
}

/// A package holding an operation anchor, an invariant and a precondition
/// over a source-declared object type, built in-repo on `v2_all_families()`:
/// the frame binds the anchor, a `Reference<Account>` type and a `self`
/// parameter bind both clauses, and each clause asserts the Boolean literal.
struct StatePackage {
    value: Value,
}

const TEXT: &str = "7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e";
const INTEGER: &str = "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f";
const REFERENCE: &str = "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2";
const SELF: &str = "b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3";
const ANCHOR: &str = "b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5";

fn plain(
    digest: &str,
    tag: &str,
    form: &str,
    ty: &str,
    deps: &[&str],
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(ty),
        "dependencies": deps.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn text(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(TEXT), "value_kind": "text", "value": value})
}

fn anchor_body(context: &str, operation: Value, frame: &str) -> Value {
    json!({"term": "aggregate", "members": [
        binding("context", reference(context)),
        binding("operation", operation),
        binding("frame", reference(frame)),
    ]})
}

/// A `quire.op.state.clause` application.
fn clause_body(clause: &str, parameters: &[&str], anchor: &str, condition: &str) -> Value {
    json!({
        "term": "application",
        "operator": "state_clause",
        "operation": {"identity": "quire.op.state.clause", "laws": [], "mode": null,
            "member": {"kind": "state_clause", "clause": clause}, "leaves": []},
        "result_type": node_id(&boolean()),
        "arguments": [
            {"term": "aggregate", "members": parameters.iter().map(|p| reference(p)).collect::<Vec<_>>()},
            reference(anchor),
            reference(condition),
        ],
    })
}

fn boolean() -> String {
    "a".repeat(64)
}

fn condition() -> String {
    "d".repeat(64)
}

/// The dependency join of a clause body: its reference targets, ascending.
fn clause_dependencies(parameters: &[&str], anchor: &str) -> Vec<String> {
    let mut deps: Vec<String> = parameters
        .iter()
        .map(|p| (*p).to_owned())
        .chain([anchor.to_owned(), condition()])
        .collect();
    deps.sort();
    deps.dedup();
    deps
}

fn clause_node(clause: &str, parameters: &[&str], anchor: &str) -> Value {
    let deps = clause_dependencies(parameters, anchor);
    let deps: Vec<&str> = deps.iter().map(String::as_str).collect();
    let placeholder = match clause {
        "invariant" => "c1",
        "precondition" => "c2",
        _ => "c3",
    };
    plain(
        &placeholder.repeat(32),
        "state",
        "state_clause",
        &boolean(),
        &deps,
        "claim",
        clause_body(clause, parameters, anchor, &condition()),
    )
}

impl StatePackage {
    fn new() -> Self {
        let mut value = v2_all_families();
        let frame = position(&value, "state", "frame");
        let frame_key = digest(&value, frame);
        let added = [
            plain(
                TEXT,
                "scalar_type",
                "text",
                TEXT,
                &[],
                "type",
                json!({"term": "aggregate", "members": []}),
            ),
            plain(
                INTEGER,
                "scalar_type",
                "integer",
                INTEGER,
                &[],
                "type",
                json!({"term": "aggregate", "members": []}),
            ),
            plain(
                REFERENCE,
                "composite_type",
                "reference",
                REFERENCE,
                &[OBJECT],
                "type",
                json!({"term": "aggregate", "members": [reference(OBJECT)]}),
            ),
            plain(
                SELF,
                "value",
                "parameter",
                REFERENCE,
                &[],
                "expression",
                json!({"term": "aggregate", "members": [
                    binding("name", text("self")),
                    binding("level", json!({"term": "literal", "type": node_id(INTEGER), "value_kind": "integer", "value": "0"})),
                ]}),
            ),
            plain(
                ANCHOR,
                "state",
                "operation_anchor",
                OBJECT,
                &[OBJECT, &frame_key],
                "anchor",
                anchor_body(OBJECT, text("deposit"), &frame_key),
            ),
            clause_node("invariant", &[SELF], OBJECT),
            clause_node("precondition", &[SELF], ANCHOR),
        ];
        value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .extend(added);
        let mut package = Self { value };
        package.refresh();
        package
    }

    /// Re-keys every node whose body is a state clause application, then re-derives the source map and identity.
    fn refresh(&mut self) {
        for position in 0..nodes(&self.value).len() {
            let node = &nodes(&self.value)[position];
            if node["body"]["operation"]["identity"] == "quire.op.state.clause" {
                rekey_application_node(&mut self.value, position);
            }
        }
        rebuild_source_map(&mut self.value);
        refresh_identity(&mut self.value);
    }

    fn at(&self, tag: &str, form: &str) -> usize {
        position(&self.value, tag, form)
    }

    /// The clause of kind `clause`.
    fn clause(&self, clause: &str) -> usize {
        nodes(&self.value)
            .iter()
            .position(|node| {
                node["semantic_form"] == "state_clause"
                    && node["body"]["operation"]["member"]["clause"] == clause
            })
            .unwrap_or_else(|| panic!("a {clause}"))
    }

    fn edit(mut self, position: usize, edit: impl FnOnce(&mut Value)) -> Self {
        edit(&mut self.value["semantic_graph"]["nodes"][position]);
        self.refresh();
        self
    }

    fn digest(&self, position: usize) -> String {
        digest(&self.value, position)
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-7
#[trace("TC-056", "FR-040-AC-7")]
#[test]
fn tc_056_operation_anchors_admit_their_three_bindings_and_joins() {
    let package = StatePackage::new();
    admits(&package.value);
    let anchor = package.at("state", "operation_anchor");
    let frame = package.at("state", "frame");
    let frame_key = package.digest(frame);
    let body_at = format!("/semantic_graph/nodes/{anchor}/body");
    let shapes: [(&str, Value); 4] = [
        (
            "reordered bindings",
            json!({"term": "aggregate", "members": [
                binding("operation", text("deposit")),
                binding("context", reference(OBJECT)),
                binding("frame", reference(&frame_key)),
            ]}),
        ),
        (
            "a non-text operation",
            anchor_body(
                OBJECT,
                json!({"term": "literal", "type": node_id(INTEGER), "value_kind": "integer", "value": "1"}),
                &frame_key,
            ),
        ),
        (
            "a missing binding",
            json!({"term": "aggregate", "members": [
                binding("context", reference(OBJECT)),
                binding("operation", text("deposit")),
            ]}),
        ),
        (
            "an extra binding",
            json!({"term": "aggregate", "members": [
                binding("context", reference(OBJECT)),
                binding("operation", text("deposit")),
                binding("frame", reference(&frame_key)),
                binding("extra", reference(OBJECT)),
            ]}),
        ),
    ];
    for (case, body) in shapes {
        let value = StatePackage::new()
            .edit(anchor, |node| node["body"] = body)
            .value;
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &body_at,
            None,
        );
    }
    let value = StatePackage::new()
        .edit(anchor, |node| {
            node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        })
        .value;
    expect(
        "an expression-role anchor occurrence",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{anchor}/occurrences/0/role"),
        None,
    );

    let target = |member: usize| format!("{body_at}/members/{member}/value/target");
    let value = StatePackage::new()
        .edit(anchor, |node| {
            node["body"] = anchor_body(&frame_key, text("deposit"), &frame_key);
        })
        .value;
    expect(
        "a context naming a frame",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &target(0),
        Some(&frame_key),
    );
    let value = StatePackage::new()
        .edit(anchor, |node| {
            node["body"] = anchor_body(OBJECT, text("deposit"), OBJECT)
        })
        .value;
    expect(
        "a frame naming an object type",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &target(2),
        Some(OBJECT),
    );
    let value = StatePackage::new()
        .edit(anchor, |node| {
            node["dependencies"] = json!([node_id(OBJECT)])
        })
        .value;
    expect(
        "a frame missing from dependencies",
        &value,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &target(2),
        Some(&frame_key),
    );
    let value = StatePackage::new()
        .edit(anchor, |node| node["semantic_type"] = node_id(PROCESS))
        .value;
    expect(
        "an anchor typed at another node",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{anchor}"),
        Some(ANCHOR),
    );

    // A second anchor naming the same (context, operation) pair, or the same
    // frame, refuses at the higher-digest anchor.
    for (case, operation) in [
        ("a duplicate pair", "deposit"),
        ("a shared frame", "withdraw"),
    ] {
        let mut package = StatePackage::new();
        let mut second = nodes(&package.value)[anchor].clone();
        let second_key = "b6".repeat(32);
        second["node_id"] = node_id(&second_key);
        second["body"]["members"][1]["value"]["value"] = json!(operation);
        package.value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second);
        package.refresh();
        let second = nodes(&package.value).len() - 1;
        expect(
            case,
            &package.value,
            Code::AmbiguousDeclaration,
            Some(Cause::AmbiguousName),
            &format!("/semantic_graph/nodes/{second}"),
            Some(&second_key),
        );
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-8, FR-040-AC-9
#[trace("TC-056", "FR-040-AC-8", "FR-040-AC-9")]
#[test]
fn tc_056_state_clauses_admit_their_body_and_join_anchor_and_parameters() {
    let package = StatePackage::new();
    let invariant = package.clause("invariant");
    let precondition = package.clause("precondition");
    let arguments = |clause: usize| format!("/semantic_graph/nodes/{clause}/body/arguments");

    // An invariant anchored at an operation anchor, and a precondition at an
    // object type, refuse at the anchor's target.
    for (case, clause, kind, anchor) in [
        (
            "an invariant at an operation anchor",
            invariant,
            "invariant",
            ANCHOR,
        ),
        (
            "a precondition at an object type",
            precondition,
            "precondition",
            OBJECT,
        ),
    ] {
        let value = StatePackage::new()
            .edit(clause, |node| *node = clause_node(kind, &[SELF], anchor))
            .value;
        let clause = StatePackage {
            value: value.clone(),
        }
        .clause(kind);
        expect(
            case,
            &value,
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            &format!("{}/1/target", arguments(clause)),
            Some(anchor),
        );
    }
    // A parameter reference to a node that is not a parameter.
    let value = StatePackage::new()
        .edit(invariant, |node| {
            *node = clause_node("invariant", &[INTEGER], OBJECT)
        })
        .value;
    let clause = StatePackage {
        value: value.clone(),
    }
    .clause("invariant");
    expect(
        "a parameter that is not a value/parameter",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("{}/0/members/0/target", arguments(clause)),
        Some(INTEGER),
    );
    // Schema-level shapes refuse at the clause body.
    let shapes: [(&str, Edit); 7] = [
        (
            "another identity",
            Box::new(|body| body["operation"]["identity"] = json!("quire.op.claim.clause")),
        ),
        (
            "another operator class",
            Box::new(|body| body["operator"] = json!("claim")),
        ),
        (
            "another member kind",
            Box::new(|body| body["operation"]["member"]["kind"] = json!("profile_operator")),
        ),
        (
            "another clause value",
            Box::new(|body| body["operation"]["member"]["clause"] = json!("assertion")),
        ),
        (
            "a first argument that is not an aggregate",
            Box::new(|body| body["arguments"][0] = reference(SELF)),
        ),
        (
            "an empty parameter aggregate",
            Box::new(|body| body["arguments"][0]["members"] = json!([])),
        ),
        (
            "a second argument that is not a reference",
            Box::new(|body| {
                body["arguments"][1] = json!({"term": "aggregate", "members": [reference(OBJECT)]});
            }),
        ),
    ];
    for (case, edit) in shapes {
        let value = StatePackage::new()
            .edit(invariant, |node| edit(&mut node["body"]))
            .value;
        let clause = nodes(&value)
            .iter()
            .position(|node| {
                node["semantic_form"] == "state_clause"
                    && node["body"]["arguments"][1] != reference(ANCHOR)
            })
            .expect("the edited clause");
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{clause}/body"),
            None,
        );
    }
    let value = StatePackage::new()
        .edit(invariant, |node| {
            node["occurrences"] = json!([{"role": "anchor", "ordinal": 0}])
        })
        .value;
    let clause = StatePackage {
        value: value.clone(),
    }
    .clause("invariant");
    expect(
        "an anchor-role clause occurrence",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{clause}/occurrences/0/role"),
        None,
    );

    // Signature: `self` of another object type, and an invariant binding a
    // second parameter, refuse at the clause.
    let mut other_self = StatePackage::new();
    let other_reference = "b7".repeat(32);
    let other_self_key = "b8".repeat(32);
    let reference_node =
        nodes(&other_self.value)[other_self.at("composite_type", "reference")].clone();
    let self_node = nodes(&other_self.value)[other_self.at("value", "parameter")].clone();
    let mut other_reference_node = reference_node;
    other_reference_node["node_id"] = node_id(&other_reference);
    other_reference_node["semantic_type"] = node_id(&other_reference);
    other_reference_node["dependencies"] = json!([node_id(PROCESS)]);
    other_reference_node["body"] = json!({"term": "aggregate", "members": [reference(PROCESS)]});
    let mut other_self_node = self_node;
    other_self_node["node_id"] = node_id(&other_self_key);
    other_self_node["semantic_type"] = node_id(&other_reference);
    other_self.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([other_reference_node, other_self_node]);
    let other_self = other_self.edit(invariant, |node| {
        *node = clause_node("invariant", &[&other_self_key], OBJECT);
    });
    let clause = other_self.clause("invariant");
    expect(
        "a self of another type",
        &other_self.value,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("{}/0", arguments(clause)),
        None,
    );
    let mut second = StatePackage::new();
    let mut extra = nodes(&second.value)[second.at("value", "parameter")].clone();
    let extra_key = "b9".repeat(32);
    extra["node_id"] = node_id(&extra_key);
    extra["body"]["members"][0]["value"]["value"] = json!("n");
    extra["body"]["members"][1]["value"]["value"] = json!("1");
    second.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(extra);
    let second = second.edit(invariant, |node| {
        *node = clause_node("invariant", &[SELF, &extra_key], OBJECT);
    });
    let clause = second.clause("invariant");
    expect(
        "an invariant binding a second parameter",
        &second.value,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("{}/0", arguments(clause)),
        None,
    );
}

/// Tracing: TC-056
/// ACs: FR-040-AC-10
#[trace("TC-056", "FR-040-AC-10")]
#[test]
fn tc_056_a_state_clause_application_stands_only_as_a_clause_body_root() {
    let package = StatePackage::new();
    let invariant = package.clause("invariant");
    // Nested inside the invariant's own condition position.
    let nested = StatePackage::new()
        .edit(invariant, |node| {
            let inner = node["body"].clone();
            node["body"]["arguments"][2] = inner;
        })
        .value;
    let clause = StatePackage {
        value: nested.clone(),
    }
    .clause("invariant");
    expect(
        "a nested clause application",
        &nested,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{clause}/body/arguments/2"),
        None,
    );
    // The body root of a node that is not a state clause.
    let misplaced = StatePackage::new()
        .edit(invariant, |node| {
            node["node_tag"] = json!("claim");
            node["semantic_form"] = json!("verification_claim");
        })
        .value;
    let claim = nodes(&misplaced)
        .iter()
        .position(|node| {
            node["body"]["operation"]["identity"] == "quire.op.state.clause"
                && node["node_tag"] == "claim"
        })
        .expect("the misplaced clause");
    expect(
        "a clause application rooting a claim",
        &misplaced,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{claim}/body"),
        None,
    );
    // A non-Boolean condition refuses at the operation step.
    let non_boolean = StatePackage::new()
        .edit(invariant, |node| {
            node["body"]["arguments"][2] = reference(INTEGER);
            let deps = clause_dependencies(&[SELF], OBJECT);
            let mut deps: Vec<String> = deps
                .into_iter()
                .filter(|d| *d != condition())
                .chain([INTEGER.to_owned()])
                .collect();
            deps.sort();
            node["dependencies"] = json!(deps.iter().map(|d| node_id(d)).collect::<Vec<_>>());
        })
        .value;
    let clause = StatePackage {
        value: non_boolean.clone(),
    }
    .clause("invariant");
    expect(
        "a non-Boolean condition",
        &non_boolean,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{clause}/body/arguments/2"),
        None,
    );
}

/// Tracing: TC-056
/// ACs: FR-040-AC-11
#[trace("TC-056", "FR-040-AC-11")]
#[test]
fn tc_056_parameter_bodies_admit_name_then_level_only() {
    let package = StatePackage::new();
    let parameter = package.at("value", "parameter");
    let body_at = format!("/semantic_graph/nodes/{parameter}/body");
    let level =
        json!({"term": "literal", "type": node_id(INTEGER), "value_kind": "integer", "value": "0"});
    let shapes: [(&str, Value); 4] = [
        (
            "reordered bindings",
            json!({"term": "aggregate", "members": [binding("level", level.clone()), binding("name", text("self"))]}),
        ),
        (
            "a non-text name",
            json!({"term": "aggregate", "members": [binding("name", level.clone()), binding("level", level.clone())]}),
        ),
        (
            "a non-integer level",
            json!({"term": "aggregate", "members": [binding("name", text("self")), binding("level", text("zero"))]}),
        ),
        (
            "a missing binding",
            json!({"term": "aggregate", "members": [binding("name", text("self"))]}),
        ),
    ];
    for (case, body) in shapes {
        let value = StatePackage::new()
            .edit(parameter, |node| node["body"] = body)
            .value;
        expect(
            case,
            &value,
            Code::InvalidSemanticGraph,
            None,
            &body_at,
            None,
        );
    }
    let value = StatePackage::new()
        .edit(parameter, |node| {
            node["occurrences"] = json!([{"role": "anchor", "ordinal": 0}])
        })
        .value;
    expect(
        "an anchor-role parameter occurrence",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{parameter}/occurrences/0/role"),
        None,
    );
}

/// Tracing: TC-056
/// ACs: FR-040-AC-12
#[trace("TC-056", "FR-040-AC-12")]
#[test]
fn tc_056_frame_state_and_operation_steps_report_in_reader_order() {
    let package = StatePackage::new();
    let frame = package.at("state", "frame");
    let anchor = package.at("state", "operation_anchor");
    let invariant = package.clause("invariant");
    let frame_defect = |node: &mut Value| node["body"]["deletes"] = json!([node_id(RELATIONSHIP)]);
    let anchor_defect = |node: &mut Value| node["semantic_type"] = node_id(PROCESS);
    let clause_defect = |node: &mut Value| *node = clause_node("invariant", &[INTEGER], OBJECT);
    let operation_defect = |node: &mut Value| node["body"]["result_type"] = node_id(INTEGER);

    let value = StatePackage::new()
        .edit(frame, frame_defect)
        .edit(anchor, anchor_defect)
        .value;
    assert_eq!(refused(&value).code, Code::InvalidModelBinding);
    assert_eq!(
        refused(&value).path.map(|path| path.as_str().to_owned()),
        Some(format!("/semantic_graph/nodes/{frame}/body/deletes/0")),
        "a frame defect before a state defect"
    );
    let value = StatePackage::new()
        .edit(anchor, anchor_defect)
        .edit(invariant, clause_defect)
        .value;
    assert_eq!(
        refused(&value).path.map(|path| path.as_str().to_owned()),
        Some(format!("/semantic_graph/nodes/{anchor}")),
        "an anchor defect before a clause defect"
    );
    // A state defect before an operation defect (the precondition's
    // `result_type`, checked at the operation step).
    let precondition = package.clause("precondition");
    let value = StatePackage::new()
        .edit(anchor, anchor_defect)
        .edit(precondition, operation_defect)
        .value;
    assert_eq!(
        refused(&value).path.map(|path| path.as_str().to_owned()),
        Some(format!("/semantic_graph/nodes/{anchor}")),
        "a state defect before an operation defect"
    );
    // The operation defect alone refuses at the operation step.
    let value = StatePackage::new()
        .edit(precondition, operation_defect)
        .value;
    let clause = StatePackage {
        value: value.clone(),
    }
    .clause("precondition");
    expect(
        "a non-Boolean result_type",
        &value,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{clause}/body/result_type"),
        None,
    );
}

// Model-owned resolution: a package whose lock selects a domain package
// document built here, holding model declaration nodes for its object types.

const ORDERS: &str = "acme/orders";
const ORDERS_VERSION: &str = "1.0.0";
const ORDER_NODE: &str = "ix://acme/orders/Order";
const SUB_NODE: &str = "ix://acme/orders/Sub";
const LEFT_NODE: &str = "ix://acme/orders/Left";
const RIGHT_NODE: &str = "ix://acme/orders/Right";
const BOTH_NODE: &str = "ix://acme/orders/Both";
const NATIVE_INTEGER: &str = "ix://quire/native/Integer";

/// A member slot of type `type_ref` with the multiplicity `[1, 1]`.
fn slot(type_ref: &str) -> Value {
    json!({"typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}})
}

fn domain_field(owner: &str, name: &str) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": NATIVE_INTEGER,
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    })
}

fn domain_type(node: &str, supertypes: &[&str], fields: Vec<Value>, operations: Value) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": ORDERS, "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": operations,
    })
}

/// `Order { total; scaled(Integer): Integer; reset() }`, `Sub: Order`, and
/// `Both: Left, Right` where `Left` and `Right` each declare `shared`.
fn orders_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": ORDERS, "version": ORDERS_VERSION},
        "constructs": [{
            "kind": {"module": ORDERS, "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [
            domain_type(ORDER_NODE, &[], vec![domain_field(ORDER_NODE, "total")], json!([
                {"identity": format!("{ORDER_NODE}/scaled"), "params": [slot(NATIVE_INTEGER)],
                 "returns": slot(NATIVE_INTEGER)},
                {"identity": format!("{ORDER_NODE}/reset"), "params": []},
            ])),
            domain_type(SUB_NODE, &[ORDER_NODE], vec![], json!([])),
            domain_type(LEFT_NODE, &[], vec![domain_field(LEFT_NODE, "shared")], json!([])),
            domain_type(RIGHT_NODE, &[], vec![domain_field(RIGHT_NODE, "shared")], json!([])),
            domain_type(BOTH_NODE, &[LEFT_NODE, RIGHT_NODE], vec![], json!([])),
        ],
    })
}

/// A node key: the SHA-256 of the canonical bytes of its structural preimage.
fn structural(tag: &str, form: &str, owner: Option<Value>) -> String {
    let mut preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": tag, "semantic_form": form,
        "semantic_type": null, "declaration": null, "recursion": null,
        "body": {"term": "aggregate", "members": []},
    });
    if let Some(owner) = owner {
        preimage["owner"] = owner;
    }
    sha256_hex(&canonical(&preimage))
}

/// The model declaration node key of `node` owned by `acme/orders` at `version`.
fn model_key(node: &str, version: &str) -> String {
    structural(
        "model",
        "object_type",
        Some(json!({"kind": "model", "identity": ORDERS, "version": version, "node": node})),
    )
}

/// Reads `value` with [`orders_document`] supplied under its digest.
fn read_model(value: &Value) -> CheckedPackageV2ReadResult {
    let document = orders_document();
    let mut evidence = evidence_for(value);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document));
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn model_admits(case: &str, value: &Value) {
    match read_model(value) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause and pointer.
fn model_expect(case: &str, value: &Value, code: Code, cause: Cause, path: &str) {
    let refusal = match read_model(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    };
    assert_eq!(
        (
            refusal.code,
            refusal.cause,
            refusal.path.as_ref().map(|path| path.as_str())
        ),
        (code, Some(cause), Some(path)),
        "{case}: {refusal:?}"
    );
}

/// [`StatePackage`] over the selected `Order`: its frame and anchor bind
/// `Order` and `scaled`, `self` is a `Reference<Order>`, and a precondition
/// `[self, n]` and a postcondition `[self, result, n]` bind the anchor.
struct ModelPackage {
    value: Value,
}

const MODEL_INTEGER_PARAMETER: &str =
    "c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4c4";
const MODEL_RESULT_PARAMETER: &str =
    "c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5";
const MODEL_TEXT_PARAMETER: &str =
    "c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6c6";

fn model_parameter(key: &str, name: &str, level: &str, ty: &str) -> Value {
    plain(
        key,
        "value",
        "parameter",
        ty,
        &[],
        "expression",
        json!({"term": "aggregate", "members": [
            binding("name", text(name)),
            binding("level", json!({"term": "literal", "type": node_id(INTEGER), "value_kind": "integer", "value": level})),
        ]}),
    )
}

impl ModelPackage {
    fn new() -> Self {
        let order = model_key(ORDER_NODE, ORDERS_VERSION);
        let integer = structural("scalar_type", "integer", None);
        let mut state = StatePackage::new();
        let frame = state.at("state", "frame");
        let frame_key = state.digest(frame);
        let value = &mut state.value;
        let document = orders_document();
        value["lock"]["model_selections"] = json!([{
            "identity": ORDERS, "version": ORDERS_VERSION,
            "digest_domain": "sha256-jcs", "digest": sha256_hex(&canonical(&document)),
        }]);
        let empty = || json!({"term": "aggregate", "members": []});
        let mut added = vec![plain(
            &integer,
            "scalar_type",
            "integer",
            &integer,
            &[],
            "type",
            empty(),
        )];
        for (node, version) in [
            (ORDER_NODE, ORDERS_VERSION),
            (SUB_NODE, ORDERS_VERSION),
            (LEFT_NODE, ORDERS_VERSION),
            (RIGHT_NODE, ORDERS_VERSION),
            (BOTH_NODE, ORDERS_VERSION),
            (ORDER_NODE, "2.0.0"),
        ] {
            let key = model_key(node, version);
            added.push(plain(
                &key,
                "model",
                "object_type",
                &key,
                &[],
                "type",
                empty(),
            ));
        }
        added.push(model_parameter(MODEL_INTEGER_PARAMETER, "n", "1", &integer));
        added.push(model_parameter(
            MODEL_RESULT_PARAMETER,
            "result",
            "1",
            &integer,
        ));
        added.push(model_parameter(MODEL_TEXT_PARAMETER, "t", "1", TEXT));
        added.push(clause_node(
            "postcondition",
            &[SELF, MODEL_RESULT_PARAMETER, MODEL_INTEGER_PARAMETER],
            ANCHOR,
        ));
        let nodes = value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        for node in nodes.iter_mut() {
            let key = node["node_id"]["digest"].as_str().expect("key").to_owned();
            if key == REFERENCE {
                node["dependencies"] = json!([node_id(&order)]);
                node["body"] = json!({"term": "aggregate", "members": [reference(&order)]});
            } else if key == frame_key {
                node["semantic_type"] = node_id(&order);
                let mut deps = node["dependencies"].as_array().expect("deps").clone();
                deps.push(node_id(&order));
                deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
                node["dependencies"] = Value::Array(deps);
            } else if key == ANCHOR {
                let mut deps = vec![order.clone(), frame_key.clone()];
                deps.sort();
                node["semantic_type"] = node_id(&order);
                node["dependencies"] = json!(deps.iter().map(|d| node_id(d)).collect::<Vec<_>>());
                node["body"] = anchor_body(&order, text("scaled"), &frame_key);
            } else if node["body"]["operation"]["member"]["clause"] == "invariant" {
                *node = clause_node("invariant", &[SELF], &order);
            } else if node["body"]["operation"]["member"]["clause"] == "precondition" {
                *node = clause_node("precondition", &[SELF, MODEL_INTEGER_PARAMETER], ANCHOR);
            }
        }
        nodes.extend(added);
        state.refresh();
        Self { value: state.value }
    }

    fn position_of(&self, key: &str) -> usize {
        nodes(&self.value)
            .iter()
            .position(|node| node["node_id"]["digest"] == key)
            .unwrap_or_else(|| panic!("node {key}"))
    }

    fn frame(&self) -> usize {
        position(&self.value, "state", "frame")
    }

    /// Edits the node at `position`, then re-keys and re-derives identity.
    fn edit(mut self, position: usize, edit: impl FnOnce(&mut Value)) -> Self {
        edit(&mut self.value["semantic_graph"]["nodes"][position]);
        let mut state = StatePackage { value: self.value };
        state.refresh();
        self.value = state.value;
        self
    }

    /// Replaces the frame's `modifies` with one field entry of `declaring`,
    /// which joins the frame's dependencies.
    fn modifying(self, declaring: &str, name: &str) -> Self {
        let frame = self.frame();
        let declaring = declaring.to_owned();
        let name = name.to_owned();
        self.edit(frame, move |node| {
            node["body"]["modifies"] = json!([field(&declaring, &name)]);
            let mut deps = node["dependencies"].as_array().expect("deps").clone();
            if !deps.contains(&node_id(&declaring)) {
                deps.push(node_id(&declaring));
            }
            deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            node["dependencies"] = Value::Array(deps);
        })
    }
}

impl ModelPackage {
    /// Binds the anchor, and the frame it names, to `context` and `operation`.
    fn anchored(self, context: &str, operation: &str) -> Self {
        let frame = self.frame();
        let frame_key = digest(&self.value, frame);
        let anchor = self.position_of(ANCHOR);
        let context = context.to_owned();
        let with_context = context.clone();
        let operation = operation.to_owned();
        self.edit(frame, move |node| {
            node["semantic_type"] = node_id(&with_context);
            let mut deps = node["dependencies"].as_array().expect("deps").clone();
            if !deps.contains(&node_id(&with_context)) {
                deps.push(node_id(&with_context));
            }
            deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            node["dependencies"] = Value::Array(deps);
        })
        .edit(anchor, move |node| {
            let mut deps = vec![context.clone(), frame_key.clone()];
            deps.sort();
            node["semantic_type"] = node_id(&context);
            node["dependencies"] = json!(deps.iter().map(|d| node_id(d)).collect::<Vec<_>>());
            node["body"] = anchor_body(&context, text(&operation), &frame_key);
        })
    }

    /// Replaces the parameters of the clause of kind `clause`.
    fn binding(self, clause: &str, parameters: &[&str]) -> Self {
        let position = nodes(&self.value)
            .iter()
            .position(|node| node["body"]["operation"]["member"]["clause"] == clause)
            .unwrap_or_else(|| panic!("a {clause}"));
        let replacement = clause_node(clause, parameters, ANCHOR);
        self.edit(position, move |node| *node = replacement)
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-3
#[trace("TC-056", "FR-040-AC-3")]
#[test]
fn tc_056_a_field_entry_resolves_among_the_selected_object_types_fields() {
    let order = model_key(ORDER_NODE, ORDERS_VERSION);
    let sub = model_key(SUB_NODE, ORDERS_VERSION);
    let both = model_key(BOTH_NODE, ORDERS_VERSION);
    let unselected = model_key(ORDER_NODE, "2.0.0");
    model_admits("the base package", &ModelPackage::new().value);
    model_admits(
        "an own field",
        &ModelPackage::new().modifying(&order, "total").value,
    );
    model_admits(
        "an inherited field",
        &ModelPackage::new().modifying(&sub, "total").value,
    );
    for (case, declaring, name, code, cause) in [
        (
            "an undeclared name",
            &order,
            "missing",
            Code::MissingDeclaration,
            Cause::MissingName,
        ),
        (
            "an operation's name",
            &order,
            "scaled",
            Code::MissingDeclaration,
            Cause::MissingName,
        ),
        (
            "a name two supertypes expose",
            &both,
            "shared",
            Code::AmbiguousDeclaration,
            Cause::AmbiguousName,
        ),
        (
            "an unselected version",
            &unselected,
            "total",
            Code::MissingDeclaration,
            Cause::MissingSelection,
        ),
    ] {
        let package = ModelPackage::new().modifying(declaring, name);
        let frame = package.frame();
        model_expect(
            case,
            &package.value,
            code,
            cause,
            &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
        );
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-7
#[trace("TC-056", "FR-040-AC-7")]
#[test]
fn tc_056_an_anchor_operation_resolves_to_one_its_context_declares() {
    let order = model_key(ORDER_NODE, ORDERS_VERSION);
    let sub = model_key(SUB_NODE, ORDERS_VERSION);
    let unselected = model_key(ORDER_NODE, "2.0.0");
    model_admits(
        "Order.scaled",
        &ModelPackage::new().anchored(&order, "scaled").value,
    );
    for (case, context, operation, code, cause) in [
        (
            "an undeclared name",
            &order,
            "missing",
            Code::MissingDeclaration,
            Cause::MissingName,
        ),
        (
            "a field's name",
            &order,
            "total",
            Code::MissingDeclaration,
            Cause::MissingName,
        ),
        (
            "an operation the context only inherits",
            &sub,
            "scaled",
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
        ),
        (
            "an unselected version",
            &unselected,
            "scaled",
            Code::MissingDeclaration,
            Cause::MissingSelection,
        ),
    ] {
        let package = ModelPackage::new().anchored(context, operation);
        let anchor = package.position_of(ANCHOR);
        model_expect(
            case,
            &package.value,
            code,
            cause,
            &format!("/semantic_graph/nodes/{anchor}/body/members/1"),
        );
    }
}

/// Tracing: TC-056
/// ACs: FR-040-AC-8, FR-040-AC-9
#[trace("TC-056", "FR-040-AC-8", "FR-040-AC-9")]
#[test]
fn tc_056_a_clause_binds_self_the_result_and_the_operation_parameters() {
    // The base package's precondition `[self, n]` and postcondition
    // `[self, result, n]` over `Order.scaled(n: Integer): Integer` admit.
    model_admits("precondition and postcondition", &ModelPackage::new().value);
    for (case, clause, parameters) in [
        ("a missing operation parameter", "precondition", vec![SELF]),
        (
            "a missing result",
            "postcondition",
            vec![SELF, MODEL_INTEGER_PARAMETER],
        ),
        (
            "a parameter of another type",
            "precondition",
            vec![SELF, MODEL_TEXT_PARAMETER],
        ),
    ] {
        let package = ModelPackage::new().binding(clause, &parameters);
        let position = nodes(&package.value)
            .iter()
            .position(|node| node["body"]["operation"]["member"]["clause"] == clause)
            .expect("the clause");
        model_expect(
            case,
            &package.value,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0"),
        );
    }
}
