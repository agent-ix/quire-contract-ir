// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-056: the V2 reader admits QSpec FR-340's frame `modifies` entries,
//! FR-342's operation anchor body and FR-341's state clause and parameter
//! bodies, and exactly QSpec's fifteen `model` forms, refusing every other
//! shape with the code, cause and locus FR-040 fixes, in reader order.
//!
//! The authored cases below build their packages in-repo from this crate's
//! public vocabulary. `qspec_*` tests read QSpec's published fixtures and its
//! `frame_mutations` at run time from the checkout `QSPEC_DIR` names; nothing
//! of QSpec is copied into this repository. They skip (and pass) when
//! `QSPEC_DIR` is unset; `make qspec-vectors` requires it.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, rebuild_source_map, refresh_identity, rekey_application_node,
    v2_all_families,
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
    // a domain package; the model-owned cases are TC-280's
    // `frame_field_cases`, replayed by `make qspec-vectors`.
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
    // second parameter, refuse at the clause. The model-owned signature
    // cases (result and operation parameters) are TC-280's
    // `clause_signature_cases`, replayed by `make qspec-vectors`.
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

// QSpec's published fixtures and `frame_mutations`, read from `QSPEC_DIR`.

const FIXTURES: &str = "proposals/checked-package-v2/fixtures";
const VECTORS: &str = "proposals/checked-package-v2/node-identity-vectors.json";

/// A file of the QSpec checkout `QSPEC_DIR` names, or `None` when unset.
fn qspec(relative: &str) -> Option<Value> {
    let root = std::env::var_os("QSPEC_DIR")?;
    let path = std::path::Path::new(&root).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Some(serde_json::from_str(&text).expect("QSpec JSON"))
}

fn vector_code(name: &str) -> Code {
    match name {
        "invalid_semantic_graph" => Code::InvalidSemanticGraph,
        "missing_declaration" => Code::MissingDeclaration,
        "invalid_model_binding" => Code::InvalidModelBinding,
        other => panic!("vector code {other} is not mapped"),
    }
}

fn vector_cause(value: &Value) -> Option<Cause> {
    match value.as_str()? {
        "missing-name" => Some(Cause::MissingName),
        "malformed-declaration" => Some(Cause::MalformedDeclaration),
        other => panic!("vector cause {other} is not mapped"),
    }
}

/// Every bare digest in `value` (a digest string, or an entry whose
/// `declaration` is one) as its `NodeRef`.
fn node_refs(value: &Value) -> Value {
    Value::Array(
        value
            .as_array()
            .expect("an array")
            .iter()
            .map(|entry| match entry {
                Value::String(digest) => node_id(digest),
                Value::Object(_) => {
                    let mut entry = entry.clone();
                    let declaration = entry["declaration"].as_str().expect("a digest").to_owned();
                    entry["declaration"] = node_id(&declaration);
                    entry
                }
                other => panic!("unexpected entry {other}"),
            })
            .collect(),
    )
}

/// Tracing: TC-056
/// ACs: FR-040-AC-13
#[trace("TC-056", "FR-040-AC-13")]
#[test]
fn qspec_frame_mutations_and_published_fixtures() {
    let Some(vectors) = qspec(VECTORS) else {
        println!("skipped: QSPEC_DIR not set");
        return;
    };
    let fixtures = [
        "positive-all-families.json",
        "positive-clause-operations.json",
    ];
    for fixture in fixtures {
        let package = qspec(&format!("{FIXTURES}/{fixture}")).expect("QSPEC_DIR is set");
        match read(&package) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{fixture} admits, read {other:?}"),
        }
    }
    let base = qspec(&format!("{FIXTURES}/positive-all-families.json")).expect("QSPEC_DIR is set");
    let frame = position(&base, "state", "frame");
    let mutations = vectors["frame_mutations"]
        .as_array()
        .expect("frame_mutations");
    for mutation in mutations {
        let name = mutation["name"].as_str().expect("name");
        let mut package = base.clone();
        let node = &mut package["semantic_graph"]["nodes"][frame];
        node["dependencies"] = node_refs(&mutation["dependencies"]);
        node["body"]["modifies"] = node_refs(&mutation["modifies"]);
        node["body"]["creates"] = node_refs(&mutation["creates"]);
        node["body"]["deletes"] = node_refs(&mutation["deletes"]);
        if let Some(second) = mutation.get("second_frame") {
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(second.clone());
            let mut entry = package["source_map"]
                .as_array()
                .expect("source map")
                .iter()
                .find(|entry| {
                    entry["node_id"] == package["semantic_graph"]["nodes"][frame]["node_id"]
                })
                .expect("the frame's source map entry")
                .clone();
            entry["node_id"] = second["node_id"].clone();
            package["source_map"]
                .as_array_mut()
                .expect("source map")
                .push(entry);
        }
        refresh_identity(&mut package);
        let refusal = refused(&package);
        assert_eq!(
            refusal.code,
            vector_code(mutation["expected_code"].as_str().expect("code")),
            "{name}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause,
            vector_cause(&mutation["expected_cause"]),
            "{name}: {refusal:?}"
        );
        assert_eq!(
            refusal.locus.as_ref().map(|id| &*id.digest),
            mutation["expected_locus_digest"].as_str(),
            "{name}: {refusal:?}"
        );
    }
    println!(
        "conformance: {} published fixtures admitted + {} frame_mutations",
        fixtures.len(),
        mutations.len()
    );
}
