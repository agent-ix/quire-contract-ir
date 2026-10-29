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
    canonical, domain_package_digest, evidence_for, family_key, family_model_document,
    fixture_source, mint_ungrouped_structural_keys, node_id, over_body, rebuild_source_map,
    refresh_identity, rename_node, settle, sha256_hex, source_owner, structural_key, typed_node_id,
    v2_all_families, BOOLEAN_KEY, INTEGER_KEY,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedApplicationOperation, CheckedComparisonPath,
    CheckedComparisonRefusal, CheckedExecutionPointRefusal, CheckedMemberType, CheckedNodeId,
    CheckedNodeTag, CheckedOccurrence, CheckedOccurrenceRole, CheckedOperationAnchorBody,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm, CompleteLoweringProfileV2, CompleteLoweringRecordV2, ExecutionPoint,
    ExecutionPointName, StateClauseKind,
};
use serde_json::{json, Value};
use std::sync::LazyLock;

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

fn admitted_frame_body(value: &Value) -> Value {
    match read(value) {
        CheckedPackageV2ReadResult::Admitted(package) => serde_json::to_value(
            &package
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "frame"
                })
                .expect("admitted frame")
                .body,
        )
        .expect("typed frame body"),
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

/// Restores the genuine selected Account declaration after a frame test has
/// minted a distinct source-declared key for its edited Account node.
fn select_other_protocol_owner(value: &mut Value) {
    let model = nodes(&v2_all_families())
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
        .expect("selected Account declaration")
        .clone();
    let owner = model["node_id"].clone();
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(model);
    let clause = position(value, "protocol", "protocol_clause");
    value["semantic_graph"]["nodes"][clause]["owner_model"] = owner;
}

fn digest(value: &Value, position: usize) -> String {
    nodes(value)[position]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned()
}

fn field(declaration: &str, name: &str) -> Value {
    json!({"kind": "field", "declaration": node_id(declaration), "name": name,
        "scope": "every-object"})
}

/// ACs: FR-040-AC-18, FR-040-AC-19, FR-040-AC-20
#[trace("TC-056", "FR-040-AC-18", "FR-040-AC-19", "FR-040-AC-20")]
#[test]
fn tc_056_scoped_frame_shape_refusals_precede_frame_semantics() {
    let base = v2_all_families();
    let frame = position(&base, "state", "frame");
    let mut missing = base.clone();
    missing["semantic_graph"]["nodes"][frame]["body"]
        .as_object_mut()
        .expect("body")
        .remove("deletes_scoped");
    refresh_identity(&mut missing);
    expect(
        "missing deletes_scoped",
        &missing,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{frame}/body"),
        None,
    );
    for (case, entry) in [
        (
            "missing scope",
            json!({"kind": "field", "declaration": node_id(object_key()), "name": "balance"}),
        ),
        (
            "unknown scope",
            json!({"kind": "field", "declaration": node_id(object_key()), "name": "balance",
                "scope": "other"}),
        ),
        (
            "relationship scope",
            json!({"kind": "relationship", "declaration": node_id(relationship_key()),
                "scope": "receiver"}),
        ),
    ] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][frame]["body"]["modifies"][0] = entry;
        refresh_identity(&mut package);
        expect(
            case,
            &package,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
            None,
        );
    }
    for entry in [
        json!({"kind": "other"}),
        json!({"kind": "parameter"}),
        json!({"kind": "receiver", "name": "x"}),
    ] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][frame]["body"]["deletes_scoped"] = json!([entry]);
        refresh_identity(&mut package);
        expect(
            "invalid scoped delete",
            &package,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{frame}/body/deletes_scoped/0"),
            None,
        );
    }
    let mut repeated = base.clone();
    repeated["semantic_graph"]["nodes"][frame]["body"]["deletes_scoped"] =
        json!([{"kind": "receiver"}, {"kind": "receiver"}]);
    refresh_identity(&mut repeated);
    expect(
        "repeated scoped deletion",
        &repeated,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{frame}/body/deletes_scoped/1"),
        None,
    );
}

/// ACs: FR-040-AC-17, FR-040-AC-21, FR-040-AC-22, FR-040-AC-23, FR-040-AC-24, FR-040-AC-25,
/// FR-040-AC-26
#[trace(
    "TC-056",
    "FR-040-AC-17",
    "FR-040-AC-21",
    "FR-040-AC-22",
    "FR-040-AC-23",
    "FR-040-AC-24",
    "FR-040-AC-25",
    "FR-040-AC-26"
)]
#[test]
fn tc_056_scoped_frame_order_conflict_and_stale_key() {
    let (empty, _) = frame_mutation(|frame| {
        frame["body"] = json!({"term": "frame", "modifies": [], "creates": [], "deletes": [],
            "deletes_scoped": []});
        frame["dependencies"] = json!([]);
    });
    admits(&empty);
    let (ordered, _) = frame_mutation(|frame| {
        frame["body"]["deletes_scoped"] = json!([{"kind": "receiver"}, {"kind": "parameter",
            "name": "a"}, {"kind": "parameter", "name": "b"}]);
    });
    admits(&ordered);
    let exposed = admitted_frame_body(&ordered);
    assert_eq!(exposed["deletes_scoped"][0]["kind"], "receiver");
    assert_eq!(exposed["deletes_scoped"][1]["kind"], "parameter");
    let (wrong_scoped, at) = frame_mutation(|frame| {
        frame["body"]["deletes_scoped"] =
            json!([{"kind": "parameter", "name": "a"}, {"kind": "receiver"}]);
    });
    expect(
        "receiver after parameter",
        &wrong_scoped,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{at}/body"),
        Some(&digest(&wrong_scoped, at)),
    );
    let (wrong_parameter_order, at) = frame_mutation(|frame| {
        frame["body"]["deletes_scoped"] = json!([
            {"kind": "receiver"},
            {"kind": "parameter", "name": "b"},
            {"kind": "parameter", "name": "a"}
        ]);
    });
    expect(
        "parameter names descending",
        &wrong_parameter_order,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{at}/body"),
        Some(&digest(&wrong_parameter_order, at)),
    );

    let (ordered_names, _) = frame_mutation(|frame| {
        frame["body"]["modifies"] =
            json!([field(object_key(), "balance"), field(object_key(), "total")]);
    });
    admits(&ordered_names);
    let (mixed_scopes, _) = frame_mutation(|frame| {
        frame["body"]["modifies"] = json!([
            {"kind": "field", "declaration": node_id(object_key()), "name": "balance",
                "scope": "receiver"},
            field(object_key(), "total")
        ]);
    });
    admits(&mixed_scopes);
    let mixed = admitted_frame_body(&mixed_scopes);
    assert_eq!(mixed["modifies"][0]["scope"], "receiver");
    assert_eq!(mixed["modifies"][0]["kind"], "field");
    assert_eq!(mixed["modifies"][1]["scope"], "every-object");
    let (wrong_names, at) = frame_mutation(|frame| {
        frame["body"]["modifies"] =
            json!([field(object_key(), "total"), field(object_key(), "balance")]);
    });
    expect(
        "reverse field names",
        &wrong_names,
        Code::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{at}/body"),
        Some(&digest(&wrong_names, at)),
    );

    let receiver = || {
        json!({"kind": "field", "declaration": node_id(object_key()),
        "name": "balance", "scope": "receiver"})
    };
    let every = || field(object_key(), "balance");
    for entries in [json!([every(), receiver()]), json!([receiver(), every()])] {
        let (package, frame) = frame_mutation(|node| node["body"]["modifies"] = entries);
        let receiver_index = nodes(&package)[frame]["body"]["modifies"]
            .as_array()
            .expect("entries")
            .iter()
            .position(|entry| entry["scope"] == "receiver")
            .expect("receiver");
        expect(
            "scope conflict",
            &package,
            Code::InvalidModelBinding,
            Some(Cause::ConflictingBinding),
            &format!("/semantic_graph/nodes/{frame}/body/modifies/{receiver_index}"),
            Some(object_key()),
        );
    }
    let (two_conflicts, frame) = frame_mutation(|node| {
        node["body"]["modifies"] = json!([
            field(object_key(), "total"),
            {"kind": "field", "declaration": node_id(object_key()),
                "name": "total", "scope": "receiver"},
            field(object_key(), "balance"),
            receiver()
        ]);
    });
    expect(
        "least order key conflict",
        &two_conflicts,
        Code::InvalidModelBinding,
        Some(Cause::ConflictingBinding),
        &format!("/semantic_graph/nodes/{frame}/body/modifies/3"),
        Some(object_key()),
    );
    let (other_defect, frame) = frame_mutation(|node| {
        node["body"]["modifies"] = json!([every(), receiver()]);
        node["body"]["creates"] = json!([node_id(&"ef".repeat(32))]);
    });
    expect(
        "meaning join before conflict",
        &other_defect,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &format!("/semantic_graph/nodes/{frame}/body/creates/0"),
        Some(&"ef".repeat(32)),
    );
    let mut stale = ordered.clone();
    let frame = position(&stale, "state", "frame");
    stale["semantic_graph"]["nodes"][frame]["body"]["deletes_scoped"] = json!([]);
    refresh_identity(&mut stale);
    expect(
        "stale scoped deletion",
        &stale,
        Code::InvalidPackage,
        Some(Cause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{frame}/node_id"),
        None,
    );
    let mut stale_scope = ordered_names.clone();
    let frame = position(&stale_scope, "state", "frame");
    stale_scope["semantic_graph"]["nodes"][frame]["body"]["modifies"][0]["scope"] =
        json!("receiver");
    refresh_identity(&mut stale_scope);
    expect(
        "stale field scope",
        &stale_scope,
        Code::InvalidPackage,
        Some(Cause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{frame}/node_id"),
        None,
    );
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
    mint_ungrouped_structural_keys(&mut value);
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

/// FR-092/FR-094 keys computed here from the declared preimages, independent
/// of the shared fixture's `family_key` mapping and the reader.
fn fixture_key(tag: &str, form: &str, ty: Option<&str>, owner: Option<Value>) -> String {
    let mut preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": tag,
        "semantic_form": form, "semantic_type": ty.map(node_id),
        "declaration": null, "recursion": null,
        "body": {"term": "aggregate", "members": []},
    });
    if let Some(owner) = owner {
        preimage["owner"] = owner;
    }
    sha256_hex(&canonical(&preimage))
}

fn object_key() -> &'static str {
    static KEY: LazyLock<String> = LazyLock::new(|| {
        fixture_key(
            "model",
            "object_type",
            None,
            Some(json!({
                "kind": "model", "identity": "test/families",
                "node": "ix://test/families/Account",
            })),
        )
    });
    &KEY
}

fn process_key() -> &'static str {
    static KEY: LazyLock<String> = LazyLock::new(|| family_key("1616"));
    &KEY
}

fn relationship_key() -> &'static str {
    static KEY: LazyLock<String> = LazyLock::new(|| {
        fixture_key(
            "relation",
            "relationship",
            None,
            Some(json!({
                "kind": "model", "identity": "test/families",
                "node": "ix://test/families/relationship/Account-balance-Account",
            })),
        )
    });
    &KEY
}

fn namespace_key() -> &'static str {
    static KEY: LazyLock<String> =
        LazyLock::new(|| fixture_key("model", "namespace", Some(BOOLEAN_KEY), None));
    &KEY
}

/// ACs: FR-040-AC-1
#[trace("TC-056", "FR-040-AC-1")]
#[test]
fn tc_056_modifies_entries_admit_in_their_closed_shape_only() {
    // The base frame holds both kinds in FR-340 declaration-digest order.
    let base = v2_all_families();
    let frame = position(&base, "state", "frame");
    assert!(relationship_key() < object_key());
    assert_eq!(
        nodes(&base)[frame]["body"]["modifies"],
        json!([
            relationship(relationship_key()),
            field(object_key(), "balance")
        ])
    );
    admits(&base);
    let (empty, _) = frame_mutation(|frame| {
        frame["body"] = json!({"term": "frame", "modifies": [], "creates": [], "deletes": [],
            "deletes_scoped": []});
        frame["dependencies"] = json!([]);
    });
    admits(&empty);
    let entry = |value: Value| frame_mutation(move |frame| frame["body"]["modifies"][1] = value);
    for (case, value) in [
        ("a bare node key", node_id(relationship_key())),
        (
            "another kind",
            json!({"kind": "operation", "declaration": node_id(relationship_key())}),
        ),
        (
            "a field entry without name",
            json!({"kind": "field", "declaration": node_id(relationship_key())}),
        ),
        (
            "a relationship entry carrying name",
            json!({"kind": "relationship", "declaration": node_id(relationship_key()),
                "name": "x"}),
        ),
        (
            "an extra member",
            json!({"kind": "relationship", "declaration": node_id(relationship_key()), "at": 1}),
        ),
        (
            "a field name that is not an identifier",
            json!({"kind": "field", "declaration": node_id(object_key()), "name": "a b"}),
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
            json!([relationship(object_key())]),
            json!([]),
            json!([node_id(object_key())]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            object_key(),
            "modifies",
            0,
        ),
        (
            "a field entry on a namespace",
            json!([field(namespace_key(), "x")]),
            json!([]),
            json!([node_id(namespace_key())]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            namespace_key(),
            "modifies",
            0,
        ),
        (
            "a field entry on a relationship",
            json!([field(relationship_key(), "x")]),
            json!([]),
            json!([node_id(relationship_key())]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            relationship_key(),
            "modifies",
            0,
        ),
        (
            "a creates entry naming a relationship",
            json!([]),
            json!([node_id(relationship_key())]),
            json!([node_id(relationship_key())]),
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            relationship_key(),
            "creates",
            0,
        ),
        (
            "an entry naming no declared dependency",
            json!([relationship(relationship_key())]),
            json!([]),
            json!([]),
            Code::MissingDeclaration,
            Cause::MissingName,
            relationship_key(),
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
            frame["body"] = json!({"term": "frame", "modifies": modifies, "creates": creates,
                "deletes": [], "deletes_scoped": []});
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

    // A `semantic_type` that is not an operation-bearing type refuses before an entry
    // defect.
    let (value, frame) = frame_mutation(|frame| {
        frame["semantic_type"] = node_id(relationship_key());
        frame["body"]["modifies"] = json!([relationship(object_key())]);
    });
    expect(
        "semantic_type naming a relationship",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{frame}/semantic_type"),
        Some(relationship_key()),
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

/// ACs: FR-040-AC-3, FR-040-AC-4
#[trace("TC-056", "FR-040-AC-3", "FR-040-AC-4")]
#[test]
fn tc_056_field_entries_order_by_declaration_then_name() {
    // Two field entries of one declaring node, ascending by name, admit;
    // descending refuses at the frame body, located at the frame.
    let (ascending, _) = frame_mutation(|frame| {
        frame["body"]["modifies"] =
            json!([field(object_key(), "balance"), field(object_key(), "total")]);
    });
    admits(&ascending);
    let (descending, frame) = frame_mutation(|frame| {
        frame["body"]["modifies"] =
            json!([field(object_key(), "total"), field(object_key(), "balance")]);
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
        frame["body"]["modifies"] =
            json!([field(object_key(), "total"), field(object_key(), "balance")]);
        frame["body"]["deletes"] = json!([node_id(relationship_key())]);
    });
    expect(
        "meaning join over order",
        &both,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{frame}/body/deletes/0"),
        Some(relationship_key()),
    );
    // The selected model owns this object, so an undeclared field name refuses.
    let (missing, frame) = frame_mutation(|frame| {
        frame["body"]["modifies"] = json!([field(object_key(), "anything")]);
    });
    expect(
        "a model-owned missing field",
        &missing,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
        Some(object_key()),
    );
}

/// A source-declared record keeps unresolved frame names after selected
/// record field resolution is enabled.
#[trace("TC-056", "FR-040-AC-3", "FR-040-AC-16")]
#[test]
fn tc_056_record_source_declared_field_entries_admit_unresolved() {
    let base = v2_all_families();
    let record = nodes(&base)
        .iter()
        .position(|node| node["node_id"]["digest"] == namespace_key())
        .expect("the namespace node");
    let with_record = |declared: bool| {
        let mut value = base.clone();
        let node = &mut value["semantic_graph"]["nodes"][record];
        node["semantic_form"] = json!("record_value_type");
        let mut preimage = json!({
            "version": "quire.structural-node/v1", "node_tag": "model",
            "semantic_form": "record_value_type", "semantic_type": node_id(BOOLEAN_KEY),
            "declaration": null, "recursion": null,
            "body": {"term": "aggregate", "members": []},
        });
        if declared {
            node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
            node["declaration"] = json!({"qualified_name": ["Example", "Address"]});
            node["owner"] = source_owner(&fixture_source());
            preimage["declaration"] = node["declaration"].clone();
            preimage["owner"] = node["owner"].clone();
        }
        let record_key = sha256_hex(&canonical(&preimage));
        rename_node(&mut value, namespace_key(), &record_key);
        let frame = position(&value, "state", "frame");
        let frame = &mut value["semantic_graph"]["nodes"][frame];
        frame["dependencies"]
            .as_array_mut()
            .expect("frame dependencies")
            .push(node_id(&record_key));
        let mut modifies = vec![
            field(&record_key, "street"),
            field(object_key(), "balance"),
            relationship(relationship_key()),
        ];
        modifies.sort_by(|left, right| {
            left["declaration"]["digest"]
                .as_str()
                .cmp(&right["declaration"]["digest"].as_str())
        });
        frame["body"]["modifies"] = json!(modifies);
        frame["dependencies"]
            .as_array_mut()
            .expect("frame dependencies")
            .sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
        mint_ungrouped_structural_keys(&mut value);
        settle(&mut value);
        value
    };
    // Neither an anonymous nor a source-declared record has selected model
    // fields to resolve through.
    admits(&with_record(false));
    admits(&with_record(true));
}

/// ACs: FR-040-AC-6
#[trace("TC-056", "FR-040-AC-6")]
#[test]
fn tc_056_exactly_fifteen_model_forms_admit() {
    let base = v2_all_families();
    let model = nodes(&base)
        .iter()
        .position(|node| node["node_id"]["digest"] == namespace_key())
        .expect("the namespace node");
    let with_form = |form: &str| {
        if matches!(form, "object_type" | "process" | "namespace") {
            // The shared fixture already holds these admitted forms.
            return base.clone();
        }
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][model]["semantic_form"] = json!(form);
        if form == "systems_interface" {
            let node = &mut value["semantic_graph"]["nodes"][model];
            node["owner"] = json!({
                "kind": "model", "identity": "test/families",
                "node": "ix://test/families/Interface",
            });
            let key = fixture_key("model", form, None, Some(node["owner"].clone()));
            rename_node(&mut value, namespace_key(), &key);
            value["semantic_graph"]["nodes"][model]["semantic_type"] = node_id(&key);
        } else {
            let key = fixture_key("model", form, Some(BOOLEAN_KEY), None);
            rename_node(&mut value, namespace_key(), &key);
        }
        rebuild_source_map(&mut value);
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
        "process",
        "namespace",
        "systems_interface",
        "systems_part",
        "systems_port",
        "systems_connection",
        "systems_allocation",
    ] {
        admits(&with_form(form));
    }
    // A state machine is owner dependent: use a selected declaration rather
    // than changing only the form of the anonymous all-family node.
    let (state_machine, evidence) = crate::checked_package_v2_refinement::machine_package();
    assert!(matches!(
        CheckedPackageV2::read(
            &canonical(&state_machine),
            CheckedPackageReadLimits::bounded(),
            &evidence
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    // Persistence interfaces also require an owner from a selected model.
    let (persistence, evidence) = crate::checked_package_v2_refinement::selected_forms_package();
    let result = CheckedPackageV2::read(
        &canonical(&persistence),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("selected persistence interface must admit: {result:?}");
    };
    assert!(admitted.graph().nodes.iter().any(|node| {
        node.node_tag.as_ref() == "model" && node.semantic_form.as_ref() == "persistence_interface"
    }));
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

fn text_key() -> &'static str {
    static KEY: LazyLock<String> =
        LazyLock::new(|| crate::support::checked_package::family_key("a1a1"));
    &KEY
}

/// The derived key of the anonymous `Integer` node the fixture already holds.
const INTEGER: &str = INTEGER_KEY;

/// The derived key of the anonymous `Reference<target>` node (FR-038-AC-134).
fn reference_key(target: &str) -> String {
    structural_key("composite_type", "reference", None, &over_body(target))
}

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

fn self_key() -> &'static str {
    static KEY: LazyLock<String> = LazyLock::new(|| {
        let ty = reference_key(object_key());
        let body = json!({"term": "aggregate", "members": [
            binding("name", text("self")),
            binding("level", json!({"term": "literal", "type": node_id(INTEGER),
                "value_kind": "integer", "value": "0"})),
        ]});
        structural_key("value", "parameter", Some(&ty), &body)
    });
    &KEY
}

fn anchor_key() -> &'static str {
    static KEY: LazyLock<String> = LazyLock::new(|| {
        let package = v2_all_families();
        let frame = position(&package, "state", "frame");
        let body = anchor_body(object_key(), text("deposit"), &digest(&package, frame));
        structural_key("state", "operation_anchor", Some(object_key()), &body)
    });
    &KEY
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn text(value: &str) -> Value {
    json!({"term": "literal", "type": node_id(text_key()), "value_kind": "text", "value": value})
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
            {"term": "aggregate",
                "members": parameters.iter().map(|p| reference(p)).collect::<Vec<_>>()},
            reference(anchor),
            reference(condition),
        ],
    })
}

fn boolean() -> String {
    BOOLEAN_KEY.to_owned()
}

fn condition() -> String {
    crate::support::checked_package::family_key("dddd")
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
        let reference_type = reference_key(object_key());
        let added = [
            plain(
                &reference_type,
                "composite_type",
                "reference",
                &reference_type,
                &[object_key()],
                "type",
                json!({"term": "aggregate", "members": [reference(object_key())]}),
            ),
            plain(
                self_key(),
                "value",
                "parameter",
                &reference_type,
                &[],
                "expression",
                json!({"term": "aggregate", "members": [
                    binding("name", text("self")),
                    binding("level", json!({"term": "literal", "type": node_id(INTEGER),
                        "value_kind": "integer", "value": "0"})),
                ]}),
            ),
            plain(
                anchor_key(),
                "state",
                "operation_anchor",
                object_key(),
                &[object_key(), &frame_key],
                "anchor",
                anchor_body(object_key(), text("deposit"), &frame_key),
            ),
            clause_node("invariant", &[self_key()], object_key()),
            clause_node("precondition", &[self_key()], anchor_key()),
        ];
        value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .extend(added);
        mint_ungrouped_structural_keys(&mut value);
        settle(&mut value);
        Self { value }
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
        mint_ungrouped_structural_keys(&mut self.value);
        settle(&mut self.value);
        self
    }

    fn digest(&self, position: usize) -> String {
        digest(&self.value, position)
    }
}

fn pointed_state_package(anchor_point: Option<Value>, clause_point: Option<Value>) -> StatePackage {
    let mut package = StatePackage::new();
    let anchor = package.at("state", "operation_anchor");
    let clause = package.clause("precondition");
    if let Some(point) = anchor_point {
        package.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
            .as_array_mut()
            .expect("anchor members")
            .push(binding("execution_point", point));
    }
    if let Some(point) = clause_point {
        package.value["semantic_graph"]["nodes"][clause]["body"]["operation"]["member"]
            ["execution_point"] = point;
    }
    mint_ungrouped_structural_keys(&mut package.value);
    settle(&mut package.value);
    package
}

fn with_postcondition(mut package: StatePackage, point: Option<Value>) -> StatePackage {
    let anchor = package.at("state", "operation_anchor");
    let mut node = clause_node("postcondition", &[self_key()], &package.digest(anchor));
    if let Some(point) = point {
        node["body"]["operation"]["member"]["execution_point"] = point;
    }
    package.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    settle(&mut package.value);
    package
}

fn pointed_invariant(point: Value) -> StatePackage {
    let mut package = StatePackage::new();
    let clause = package.clause("invariant");
    package.value["semantic_graph"]["nodes"][clause]["body"]["operation"]["member"]
        ["execution_point"] = point;
    mint_ungrouped_structural_keys(&mut package.value);
    settle(&mut package.value);
    package
}

fn pointed_constraint(point: Value) -> StatePackage {
    let mut package = StatePackage::new();
    let mut node = clause_node("constraint", &[self_key()], object_key());
    node["body"]["operation"]["member"]["execution_point"] = point;
    package.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    mint_ungrouped_structural_keys(&mut package.value);
    settle(&mut package.value);
    package
}

fn assert_pointed_writer_round_trip(package: &StatePackage, positions: &[usize]) {
    let admitted = match read(&package.value) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("pointed package refused: {other:?}"),
    };
    let preimage = admitted.identity_preimage();
    let borrowed = CheckedPackageIdentityPreimageRefV2 {
        version: &preimage.version,
        edition: &preimage.edition,
        profile_selections: &preimage.profile_selections,
        definition_selections: &preimage.definition_selections,
        model_selections: &preimage.model_selections,
        required_features: &preimage.required_features,
        dependency_selections: &preimage.dependency_selections,
        identity_projection: &admitted.graph().nodes,
    };
    let encoded = encode_checked_package_v2(
        &borrowed,
        admitted.package_id(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        Limits::new(u64::MAX),
    )
    .expect("encode admitted package");
    let encoded_json: Value = serde_json::from_slice(&encoded).expect("writer JSON");
    for &position in positions {
        assert_eq!(
            encoded_json["semantic_graph"]["nodes"][position]["body"],
            package.value["semantic_graph"]["nodes"][position]["body"]
        );
    }
    let read_back = match CheckedPackageV2::read(
        &encoded,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package.value),
    ) {
        CheckedPackageV2ReadResult::Admitted(read_back) => read_back,
        other => panic!("writer output refused: {other:?}"),
    };
    assert_eq!(read_back.package_id(), admitted.package_id());
    assert_eq!(read_back.graph(), admitted.graph());
}

#[trace("TC-056", "FR-040-AC-33", "FR-040-AC-34", "FR-066-AC-4")]
#[test]
fn tc_056_model_clause_points_retain_name_and_change_application_identity() {
    let absent = StatePackage::new();
    let initialized = pointed_invariant(json!({"kind": "initialization", "name": "startup"}));
    let handled = pointed_invariant(json!({"kind": "handler", "name": "tick"}));
    for package in [&initialized, &handled] {
        let admitted = match read(&package.value) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("pointed invariant refused: {other:?}"),
        };
        let clause = package.clause("invariant");
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[clause].body).expect("typed clause"),
            package.value["semantic_graph"]["nodes"][clause]["body"]
        );
        assert_ne!(
            digest(&absent.value, absent.clause("invariant")),
            digest(&package.value, clause)
        );
        assert_ne!(absent.value["package_id"], package.value["package_id"]);
        assert_pointed_writer_round_trip(package, &[clause]);
    }
    assert_ne!(
        digest(&initialized.value, initialized.clause("invariant")),
        digest(&handled.value, handled.clause("invariant"))
    );
    for point in [
        json!({"kind": "initialization", "name": "startup"}),
        json!({"kind": "handler", "name": "tick"}),
    ] {
        let package = pointed_constraint(point.clone());
        let clause = package.clause("constraint");
        let admitted = match read(&package.value) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("pointed constraint refused: {other:?}"),
        };
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[clause].body).expect("typed constraint")
                ["operation"]["member"]["execution_point"],
            point
        );
        assert_pointed_writer_round_trip(&package, &[clause]);
    }
    let mut absent_constraint = StatePackage::new();
    absent_constraint.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(clause_node("constraint", &[self_key()], object_key()));
    mint_ungrouped_structural_keys(&mut absent_constraint.value);
    settle(&mut absent_constraint.value);
    let pointed = pointed_constraint(json!({"kind": "handler", "name": "tick"}));
    let constraint = pointed.clause("constraint");
    assert_ne!(
        absent_constraint.value["package_id"],
        pointed.value["package_id"]
    );
    assert_ne!(
        absent_constraint.digest(absent_constraint.clause("constraint")),
        pointed.digest(constraint)
    );
    let wrong_constraint = pointed_constraint(json!({"kind": "pre", "operation": "deposit"}));
    let constraint = wrong_constraint.clause("constraint");
    expect(
        "operation point on constraint",
        &wrong_constraint.value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{constraint}"),
        Some(&wrong_constraint.digest(constraint)),
    );
    let constraint = pointed.clause("constraint");
    let operation_anchor = pointed.digest(pointed.at("state", "operation_anchor"));
    let target = operation_anchor.clone();
    let wrong_anchor = StatePackage {
        value: pointed.value.clone(),
    }
    .edit(constraint, move |node| {
        node["body"]["arguments"][1]["target"] = node_id(&target);
    });
    expect(
        "constraint requires an object anchor",
        &wrong_anchor.value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{constraint}/body/arguments/1/target"),
        Some(&operation_anchor),
    );
    let wrong_signature = pointed.edit(constraint, |node| {
        node["body"]["arguments"][0]["members"] =
            json!([reference(self_key()), reference(self_key())]);
    });
    expect(
        "constraint binds self alone",
        &wrong_signature.value,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &format!("/semantic_graph/nodes/{constraint}/body/arguments/0"),
        Some(&wrong_signature.digest(constraint)),
    );
    let wrong = pointed_invariant(json!({"kind": "pre", "operation": "deposit"}));
    let clause = wrong.clause("invariant");
    expect(
        "operation point on invariant",
        &wrong.value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{clause}"),
        Some(&wrong.digest(clause)),
    );
    for malformed in [
        json!({"kind": "unknown", "name": "startup"}),
        json!({"kind": "handler", "name": ""}),
        json!({"kind": "handler", "name": "tick", "extra": true}),
    ] {
        let mut package = StatePackage::new();
        let clause = package.clause("invariant");
        package.value["semantic_graph"]["nodes"][clause]["body"]["operation"]["member"]
            ["execution_point"] = malformed;
        settle(&mut package.value);
        expect(
            "malformed clause point",
            &package.value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{clause}/body/operation/member/execution_point"),
            Some(&package.digest(clause)),
        );
    }
}

#[trace("TC-056", "FR-066-AC-4")]
#[test]
fn tc_056_duplicate_clause_point_precedes_version_and_identity_checks() {
    let mut package = pointed_invariant(json!({"kind": "handler", "name": "tick"}));
    let clause = package.clause("invariant");
    package.value["contract_version"] = json!("wrong-version");
    let bytes = String::from_utf8(canonical(&package.value)).expect("canonical UTF-8");
    let member = r#""execution_point":{"kind":"handler","name":"tick"}"#;
    assert_eq!(
        bytes.matches(member).count(),
        1,
        "one clause point to duplicate"
    );
    let duplicate = bytes.replacen(member, &format!("{member},{member}"), 1);
    let refusal = match CheckedPackageV2::read(
        duplicate.as_bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package.value),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("duplicate point should refuse: {other:?}"),
    };
    assert_eq!(refusal.code, Code::DuplicateMember);
    assert_eq!(refusal.cause, None);
    let expected_path =
        format!("/semantic_graph/nodes/{clause}/body/operation/member/execution_point");
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(expected_path.as_str())
    );
}

#[trace(
    "TC-056",
    "FR-040-AC-31",
    "FR-040-AC-33",
    "FR-040-AC-36",
    "FR-066-AC-3"
)]
#[test]
fn tc_056_execution_points_rekey_and_round_trip_as_bare_wire_objects() {
    let absent = pointed_state_package(None, None);
    admits(&absent.value);
    let pre = pointed_state_package(
        Some(json!({"kind": "pre", "operation": "deposit"})),
        Some(json!({"kind": "pre", "operation": "deposit"})),
    );
    let post = pointed_state_package(
        Some(json!({"kind": "post", "operation": "deposit"})),
        Some(json!({"kind": "post", "operation": "deposit"})),
    );
    for (package, expected_point) in [
        (&absent, None),
        (
            &pre,
            Some(ExecutionPoint::Pre {
                operation: ExecutionPointName::new("deposit").unwrap(),
            }),
        ),
        (
            &post,
            Some(ExecutionPoint::Post {
                operation: ExecutionPointName::new("deposit").unwrap(),
            }),
        ),
    ] {
        let admitted = match read(&package.value) {
            CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
            other => panic!("pointed package refused: {other:?}"),
        };
        let anchor = package.at("state", "operation_anchor");
        let clause = package.clause("precondition");
        let point = package.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
            .as_array()
            .and_then(|members| members.get(3))
            .map(|member| &member["value"]);
        let CheckedTerm::OperationAnchorBody(typed_anchor) = &admitted.graph().nodes[anchor].body
        else {
            panic!("admitted anchor must have one typed form");
        };
        assert_eq!(typed_anchor.execution_point, expected_point);
        let constructed_anchor =
            CheckedTerm::OperationAnchorBody(Box::new(CheckedOperationAnchorBody {
                members: typed_anchor.members.clone(),
                execution_point: expected_point.clone(),
            }));
        assert_eq!(constructed_anchor, admitted.graph().nodes[anchor].body);
        let CheckedTerm::Application {
            operator,
            operation,
            result_type,
            arguments,
        } = &admitted.graph().nodes[clause].body
        else {
            panic!("admitted clause must have a typed application");
        };
        assert_eq!(operation.execution_point(), expected_point.as_ref());
        let constructed = CheckedTerm::Application {
            operator: *operator,
            operation: CheckedApplicationOperation::state_clause(
                StateClauseKind::Precondition,
                expected_point.clone(),
            ),
            result_type: result_type.clone(),
            arguments: arguments.clone(),
        };
        assert_eq!(constructed, admitted.graph().nodes[clause].body);
        if let Some(point) = point {
            assert!(point.get("term").is_none(), "anchor point is not a term");
        }
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[anchor].body).expect("typed anchor"),
            package.value["semantic_graph"]["nodes"][anchor]["body"]
        );
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[clause].body).expect("typed clause"),
            package.value["semantic_graph"]["nodes"][clause]["body"]
        );
        assert_pointed_writer_round_trip(package, &[anchor, clause]);
    }
    for package in [&pre, &post] {
        assert_ne!(
            digest(&absent.value, absent.at("state", "operation_anchor")),
            digest(&package.value, package.at("state", "operation_anchor"))
        );
        assert_ne!(absent.value["package_id"], package.value["package_id"]);
    }
    assert_ne!(
        digest(&pre.value, pre.at("state", "operation_anchor")),
        digest(&post.value, post.at("state", "operation_anchor"))
    );
    assert_ne!(pre.value["package_id"], post.value["package_id"]);
    for point in [
        json!({"kind": "pre", "operation": "deposit"}),
        json!({"kind": "post", "operation": "deposit"}),
    ] {
        let package = with_postcondition(
            pointed_state_package(Some(point.clone()), Some(point.clone())),
            Some(point.clone()),
        );
        let clause = package.clause("postcondition");
        let admitted = match read(&package.value) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("postcondition with either operation point refused: {other:?}"),
        };
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[clause].body)
                .expect("typed postcondition")["operation"]["member"]["execution_point"],
            point
        );
    }
}

#[trace("TC-056", "FR-040-AC-32", "FR-040-AC-35", "FR-066-AC-4")]
#[test]
fn tc_056_same_operation_admits_distinct_anchor_points() {
    let mut package = pointed_state_package(
        Some(json!({"kind": "pre", "operation": "deposit"})),
        Some(json!({"kind": "pre", "operation": "deposit"})),
    );
    let anchor = package.at("state", "operation_anchor");
    let mut second = package.value["semantic_graph"]["nodes"][anchor].clone();
    second["node_id"] = node_id(&"d9".repeat(32));
    second["body"]["members"][3]["value"]["kind"] = json!("post");
    package.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    mint_ungrouped_structural_keys(&mut package.value);
    settle(&mut package.value);
    admits(&package.value);
    let anchors = nodes(&package.value)
        .iter()
        .filter(|node| node["semantic_form"] == "operation_anchor")
        .collect::<Vec<_>>();
    assert_eq!(anchors.len(), 2);
    assert_ne!(anchors[0]["node_id"], anchors[1]["node_id"]);

    let mut duplicate = package.value.clone();
    let second = nodes(&duplicate)
        .iter()
        .position(|node| {
            node["semantic_form"] == "operation_anchor"
                && node["body"]["members"][3]["value"]["kind"] == "post"
        })
        .expect("post anchor");
    let frame = package.at("state", "frame");
    let mut alternate_frame = nodes(&duplicate)[frame].clone();
    let alternate_key = "da".repeat(32);
    alternate_frame["node_id"] = node_id(&alternate_key);
    alternate_frame["body"] = json!({"term": "frame", "modifies": [], "creates": [],
        "deletes": [], "deletes_scoped": []});
    alternate_frame["dependencies"] = json!([]);
    duplicate["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(alternate_frame);
    duplicate["semantic_graph"]["nodes"][second]["body"]["members"][2]["value"]["target"] =
        node_id(&alternate_key);
    duplicate["semantic_graph"]["nodes"][second]["dependencies"] =
        json!([node_id(object_key()), node_id(&alternate_key)]);
    duplicate["semantic_graph"]["nodes"][second]["body"]["members"][3]["value"]["kind"] =
        json!("pre");
    mint_ungrouped_structural_keys(&mut duplicate);
    settle(&mut duplicate);
    let later = nodes(&duplicate)
        .iter()
        .enumerate()
        .filter(|(_, node)| node["semantic_form"] == "operation_anchor")
        .map(|(position, _)| position)
        .max_by_key(|position| digest(&duplicate, *position))
        .expect("later anchor");
    expect(
        "duplicate context, operation and point",
        &duplicate,
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        &format!("/semantic_graph/nodes/{later}"),
        Some(&digest(&duplicate, later)),
    );

    for node in duplicate["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .filter(|node| node["semantic_form"] == "operation_anchor")
    {
        node["body"]["members"][3]["value"]["operation"] = json!("withdraw");
    }
    mint_ungrouped_structural_keys(&mut duplicate);
    settle(&mut duplicate);
    let first = nodes(&duplicate)
        .iter()
        .enumerate()
        .filter(|(_, node)| node["semantic_form"] == "operation_anchor")
        .map(|(position, _)| position)
        .min_by_key(|position| digest(&duplicate, *position))
        .expect("first anchor");
    expect(
        "anchor point applicability precedes duplicate tuple detection",
        &duplicate,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{first}"),
        Some(&digest(&duplicate, first)),
    );
}

#[trace(
    "TC-056",
    "FR-040-AC-32",
    "FR-040-AC-34",
    "FR-040-AC-35",
    "FR-066-AC-4"
)]
#[test]
fn tc_056_execution_point_shape_and_agreement_refuse_at_their_stage() {
    let anchor = StatePackage::new().at("state", "operation_anchor");
    for (case, edit) in [
        ("point binding in third position", 0_usize),
        ("extra fifth binding", 1_usize),
    ] {
        let mut package = StatePackage::new();
        let members = package.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
            .as_array_mut()
            .expect("members");
        let point = binding(
            "execution_point",
            json!({"kind": "pre", "operation": "deposit"}),
        );
        if edit == 0 {
            members.insert(2, point);
        } else {
            members.push(point);
            members.push(binding("extra", reference(object_key())));
        }
        settle(&mut package.value);
        expect(
            case,
            &package.value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{anchor}/body"),
            None,
        );
    }
    for nonpoint in [json!("pre"), json!(7), reference(object_key())] {
        let mut package = StatePackage::new();
        package.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
            .as_array_mut()
            .expect("members")
            .push(binding("execution_point", nonpoint));
        settle(&mut package.value);
        expect(
            "non-point fourth value",
            &package.value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{anchor}/body"),
            None,
        );
    }
    let mut nested = StatePackage::new();
    let members = nested.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
        .as_array_mut()
        .expect("members");
    members.insert(
        2,
        binding(
            "execution_point",
            json!({"kind": "pre", "operation": "deposit"}),
        ),
    );
    members.push(binding(
        "extra",
        clause_body("precondition", &[self_key()], anchor_key(), &condition()),
    ));
    settle(&mut nested.value);
    expect(
        "nested state application precedes wrong point position",
        &nested.value,
        Code::MalformedWire,
        None,
        &format!("/semantic_graph/nodes/{anchor}/body/members/4/value"),
        None,
    );
    for malformed in [
        json!({"kind": "unknown", "operation": "deposit"}),
        json!({"kind": "pre", "operation": ""}),
        json!({"kind": "pre", "operation": "deposit", "extra": true}),
    ] {
        let mut package = StatePackage::new();
        package.value["semantic_graph"]["nodes"][anchor]["body"]["members"]
            .as_array_mut()
            .expect("members")
            .push(binding("execution_point", malformed));
        settle(&mut package.value);
        expect(
            "malformed anchor point",
            &package.value,
            Code::InvalidSemanticGraph,
            None,
            &format!("/semantic_graph/nodes/{anchor}/body/members/3/value"),
            None,
        );
    }
    let mut stale = pointed_state_package(
        Some(json!({"kind": "pre", "operation": "deposit"})),
        Some(json!({"kind": "pre", "operation": "deposit"})),
    );
    stale.value["semantic_graph"]["nodes"][anchor]["body"]["members"][3]["value"]["kind"] =
        json!("post");
    settle(&mut stale.value);
    expect(
        "stale anchor key precedes point agreement",
        &stale.value,
        Code::InvalidPackage,
        Some(Cause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{anchor}/node_id"),
        None,
    );
    for point in [
        json!({"kind": "initialization", "name": "startup"}),
        json!({"kind": "handler", "name": "tick"}),
        json!({"kind": "pre", "operation": "withdraw"}),
        json!({"kind": "post", "operation": "withdraw"}),
    ] {
        let invalid = pointed_state_package(Some(point.clone()), Some(point));
        expect(
            "inapplicable or wrong-operation anchor point",
            &invalid.value,
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{anchor}"),
            Some(&invalid.digest(anchor)),
        );
    }
    let mismatch = pointed_state_package(
        Some(json!({"kind": "pre", "operation": "deposit"})),
        Some(json!({"kind": "post", "operation": "deposit"})),
    );
    let clause = mismatch.clause("precondition");
    expect(
        "clause-anchor point mismatch",
        &mismatch.value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{clause}"),
        Some(&mismatch.digest(clause)),
    );
    for (case, anchor_point, clause_point) in [
        (
            "only anchor has point",
            Some(json!({"kind": "pre", "operation": "deposit"})),
            None,
        ),
        (
            "only clause has point",
            None,
            Some(json!({"kind": "pre", "operation": "deposit"})),
        ),
    ] {
        let package = pointed_state_package(anchor_point, clause_point);
        let clause = package.clause("precondition");
        expect(
            case,
            &package.value,
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{clause}"),
            Some(&package.digest(clause)),
        );
    }

    let mut stale_clause = pointed_state_package(
        Some(json!({"kind": "pre", "operation": "deposit"})),
        Some(json!({"kind": "pre", "operation": "deposit"})),
    );
    let clause = stale_clause.clause("precondition");
    stale_clause.value["semantic_graph"]["nodes"][clause]["body"]["operation"]["member"]
        ["execution_point"]["kind"] = json!("post");
    refresh_identity(&mut stale_clause.value);
    expect(
        "stale clause application key precedes agreement",
        &stale_clause.value,
        Code::InvalidPackage,
        Some(Cause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{clause}/node_id"),
        None,
    );
}

#[trace("TC-056", "FR-040-AC-35", "FR-066-AC-4")]
#[test]
fn tc_056_point_checks_follow_anchor_joins_and_clause_signature() {
    let anchor = position(&ModelPackage::new().value, "state", "operation_anchor");
    let frame = ModelPackage::new().frame();
    let bad_operation = ModelPackage::new().edit(anchor, |node| {
        node["body"]["members"][1]["value"]["value"] = json!("missing");
        node["body"]["members"]
            .as_array_mut()
            .expect("anchor members")
            .push(binding(
                "execution_point",
                json!({"kind": "pre", "operation": "other"}),
            ));
    });
    model_expect(
        "operation resolution precedes point applicability",
        &bad_operation.value,
        Code::MissingDeclaration,
        Cause::MissingName,
        &format!("/semantic_graph/nodes/{anchor}/body/members/1"),
    );

    let pointed = ModelPackage::new().edit(anchor, |node| {
        node["body"]["members"]
            .as_array_mut()
            .expect("anchor members")
            .push(binding(
                "execution_point",
                json!({"kind": "pre", "operation": "other"}),
            ));
    });
    let bad_scoped_delete = pointed.edit(frame, |node| {
        node["body"]["deletes_scoped"] = json!([{"kind": "parameter", "name": "ghost"}]);
    });
    model_expect(
        "scoped deletes precede point applicability",
        &bad_scoped_delete.value,
        Code::MissingDeclaration,
        Cause::MissingName,
        &format!("/semantic_graph/nodes/{frame}"),
    );

    let precondition = ModelPackage::new().value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["member"]["clause"] == "precondition")
        .expect("precondition");
    let pointed = ModelPackage::new().edit(anchor, |node| {
        node["body"]["members"]
            .as_array_mut()
            .expect("anchor members")
            .push(binding(
                "execution_point",
                json!({"kind": "pre", "operation": "scaled"}),
            ));
    });
    let postcondition = nodes(&pointed.value)
        .iter()
        .position(|node| node["body"]["operation"]["member"]["clause"] == "postcondition")
        .expect("postcondition");
    let pointed = pointed.edit(postcondition, |node| {
        node["body"]["operation"]["member"]["execution_point"] =
            json!({"kind": "pre", "operation": "scaled"});
    });
    let self_parameter = pointed.parameter_key("self");
    let bad_signature = pointed.edit(precondition, move |node| {
        node["body"]["operation"]["member"]["execution_point"] =
            json!({"kind": "post", "operation": "scaled"});
        node["body"]["arguments"][0]["members"] = json!([reference(&self_parameter)]);
    });
    model_expect(
        "signature precedes clause-anchor point agreement",
        &bad_signature.value,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{precondition}/body/arguments/0"),
    );
}

/// Shared admitted state-clause fixture for the operation-family reader controls.
pub(crate) fn state_operand_fixture() -> (Value, usize) {
    let package = StatePackage::new();
    let clause = package.clause("invariant");
    (package.value, clause)
}

/// ACs: FR-040-AC-7, FR-040-AC-28
#[trace("TC-056", "FR-040-AC-7", "FR-040-AC-28")]
#[test]
fn tc_056_operation_anchors_admit_their_three_bindings_and_joins() {
    let package = StatePackage::new();
    admits(&package.value);
    let anchor = package.at("state", "operation_anchor");
    let frame = package.at("state", "frame");
    let frame_key = package.digest(frame);
    let body_at = format!("/semantic_graph/nodes/{anchor}/body");
    let shapes: [(&str, Value); 5] = [
        ("a root reference", reference(object_key())),
        (
            "reordered bindings",
            json!({"term": "aggregate", "members": [
                binding("operation", text("deposit")),
                binding("context", reference(object_key())),
                binding("frame", reference(&frame_key)),
            ]}),
        ),
        (
            "a non-text operation",
            anchor_body(
                object_key(),
                json!({"term": "literal", "type": node_id(INTEGER), "value_kind": "integer",
                    "value": "1"}),
                &frame_key,
            ),
        ),
        (
            "a missing binding",
            json!({"term": "aggregate", "members": [
                binding("context", reference(object_key())),
                binding("operation", text("deposit")),
            ]}),
        ),
        (
            "an extra binding",
            json!({"term": "aggregate", "members": [
                binding("context", reference(object_key())),
                binding("operation", text("deposit")),
                binding("frame", reference(&frame_key)),
                binding("extra", reference(object_key())),
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
    let clause = package.clause("invariant");
    let clause_body = nodes(&package.value)[clause]["body"].clone();
    let value = StatePackage::new()
        .edit(anchor, |node| node["body"] = clause_body)
        .value;
    expect(
        "a root state-clause application",
        &value,
        Code::InvalidSemanticGraph,
        None,
        &body_at,
        None,
    );
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
            node["body"] = anchor_body(object_key(), text("deposit"), object_key())
        })
        .value;
    expect(
        "a frame naming an object type",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &target(2),
        Some(object_key()),
    );
    let value = StatePackage::new()
        .edit(anchor, |node| {
            node["dependencies"] = json!([node_id(object_key())])
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
        .edit(anchor, |node| {
            node["semantic_type"] = node_id(process_key())
        })
        .value;
    expect(
        "an anchor typed at another node",
        &value,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{anchor}"),
        Some(digest(&value, anchor).as_str()),
    );

    // A duplicate (context, operation) pair refuses; distinct operations may
    // share one frame.
    for (case, operation) in [
        ("a duplicate pair", "deposit"),
        ("a shared frame", "withdraw"),
    ] {
        let mut package = StatePackage::new();
        let mut second = nodes(&package.value)[anchor].clone();
        second["node_id"] = node_id(&"b6".repeat(32));
        second["body"]["members"][1]["value"]["value"] = json!(operation);
        if case == "a duplicate pair" {
            let mut alternate_frame = nodes(&package.value)[frame].clone();
            let alternate_key = "b7".repeat(32);
            alternate_frame["node_id"] = node_id(&alternate_key);
            alternate_frame["body"] = json!({"term": "frame", "modifies": [], "creates": [],
                "deletes": [], "deletes_scoped": []});
            alternate_frame["dependencies"] = json!([]);
            second["body"]["members"][2]["value"]["target"] = node_id(&alternate_key);
            second["dependencies"] = json!([node_id(object_key()), node_id(&alternate_key)]);
            package.value["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(alternate_frame);
        }
        package.value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second);
        mint_ungrouped_structural_keys(&mut package.value);
        settle(&mut package.value);
        let second = nodes(&package.value).len() - 1;
        let second_key = digest(&package.value, second);
        if case == "a shared frame" {
            admits(&package.value);
        } else {
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
}

/// ACs: FR-040-AC-28
#[trace("TC-056", "FR-040-AC-28")]
#[test]
fn tc_056_shared_frame_checks_scoped_deletes_for_each_anchor() {
    let mut package = StatePackage::new().value;
    let anchor = position(&package, "state", "operation_anchor");
    let frame = position(&package, "state", "frame");
    let mut second = nodes(&package)[anchor].clone();
    second["node_id"] = node_id(&"b6".repeat(32));
    second["body"]["members"][1]["value"]["value"] = json!("withdraw");
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    package["semantic_graph"]["nodes"][frame]["body"]["deletes_scoped"] =
        json!([{"kind": "parameter", "name": "a"}]);
    // Remove the unrelated clause signature; the test concerns anchor joins.
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| node["body"]["operation"]["member"]["clause"] != "precondition");
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    let mut anchors = nodes(&package)
        .iter()
        .filter(|node| node["semantic_form"] == "operation_anchor")
        .map(|node| {
            (
                node["node_id"]["digest"].as_str().expect("key").to_owned(),
                node["body"]["members"][1]["value"]["value"]
                    .as_str()
                    .expect("operation")
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    anchors.sort_by(|left, right| left.0.cmp(&right.0));
    let mut document = family_model_document();
    let operations = document["types"][0]["operations"]
        .as_array_mut()
        .expect("operations");
    let valid = operations
        .iter_mut()
        .find(|operation| {
            operation["identity"]
                .as_str()
                .expect("identity")
                .ends_with(&anchors[0].1)
        })
        .expect("first operation");
    valid["params"] = json!([{
        "name": "a", "typeRef": "ix://test/families/Account",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}
    }]);
    let document_digest = domain_package_digest(&document);
    package["lock"]["model_selections"][0]["digest"] = json!(document_digest.clone());
    refresh_identity(&mut package);
    let read_custom = |value: &Value| {
        let mut evidence = evidence_for(value);
        evidence
            .insert_domain_package_document(document_digest.clone(), canonical(&document))
            .expect("selected document");
        CheckedPackageV2::read(
            &canonical(value),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        )
    };
    let mut first_only = package.clone();
    first_only["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| {
            node["semantic_form"] != "operation_anchor"
                || node["body"]["members"][1]["value"]["value"] == anchors[0].1
        });
    rebuild_source_map(&mut first_only);
    refresh_identity(&mut first_only);
    assert!(
        matches!(
            read_custom(&first_only),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the first anchor's reference parameter is valid"
    );
    let frame = position(&package, "state", "frame");
    match read_custom(&package) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                Some(format!("/semantic_graph/nodes/{frame}").as_str())
            )
        ),
        other => panic!("the second anchor must check the shared frame: {other:?}"),
    }
}

/// ACs: FR-040-AC-8, FR-040-AC-9
#[trace("TC-056", "FR-040-AC-37", "TC-048", "FR-038-AC-341")]
#[test]
fn tc_056_public_state_clause_kind_tracks_reader_admission() {
    let mut package = with_postcondition(StatePackage::new(), None);
    package.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(clause_node("constraint", &[self_key()], object_key()));
    settle(&mut package.value);
    let admitted = match read(&package.value) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("state clause fixture refused: {other:?}"),
    };
    for (name, expected) in [
        ("invariant", StateClauseKind::Invariant),
        ("constraint", StateClauseKind::Constraint),
        ("precondition", StateClauseKind::Precondition),
        ("postcondition", StateClauseKind::Postcondition),
    ] {
        let node = &admitted.graph().nodes[package.clause(name)];
        let CheckedTerm::Application { operation, .. } = &node.body else {
            panic!("{name} clause is an application");
        };
        assert_eq!(
            operation.state_clause_kind().expect("admitted views"),
            Some(expected)
        );
    }
    let plain = admitted
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() != Some("quire.op.state.clause") =>
            {
                Some(operation)
            }
            _ => None,
        })
        .expect("fixture includes a non-clause operation");
    assert_eq!(plain.state_clause_kind().expect("admitted views"), None);
    let malformed = StatePackage::new().edit(package.clause("invariant"), |node| {
        node["body"]["operation"]["member"]["clause"] = json!("unknown");
    });
    assert!(matches!(
        read(&malformed.value),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

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
            anchor_key(),
        ),
        (
            "a precondition at an object type",
            precondition,
            "precondition",
            object_key(),
        ),
    ] {
        let value = StatePackage::new()
            .edit(clause, |node| {
                *node = clause_node(kind, &[self_key()], anchor)
            })
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
            *node = clause_node("invariant", &[INTEGER], object_key())
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
            Box::new(|body| body["arguments"][0] = reference(self_key())),
        ),
        (
            "an empty parameter aggregate",
            Box::new(|body| body["arguments"][0]["members"] = json!([])),
        ),
        (
            "a second argument that is not a reference",
            Box::new(|body| {
                body["arguments"][1] =
                    json!({"term": "aggregate", "members": [reference(object_key())]});
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
                    && node["body"]["arguments"][1] != reference(anchor_key())
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
    let other_reference = reference_key(process_key());
    let other_self_key = "b8".repeat(32);
    let reference_node =
        nodes(&other_self.value)[other_self.at("composite_type", "reference")].clone();
    let self_node = nodes(&other_self.value)[other_self.at("value", "parameter")].clone();
    let mut other_reference_node = reference_node;
    other_reference_node["node_id"] = node_id(&other_reference);
    other_reference_node["semantic_type"] = node_id(&other_reference);
    other_reference_node["dependencies"] = json!([node_id(process_key())]);
    other_reference_node["body"] =
        json!({"term": "aggregate", "members": [reference(process_key())]});
    let mut other_self_node = self_node;
    other_self_node["node_id"] = node_id(&other_self_key);
    other_self_node["semantic_type"] = node_id(&other_reference);
    other_self.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([other_reference_node, other_self_node]);
    let other_self = other_self.edit(invariant, |node| {
        *node = clause_node("invariant", &[&other_self_key], object_key());
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
        *node = clause_node("invariant", &[self_key(), &extra_key], object_key());
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

/// ACs: FR-040-AC-10, FR-038-AC-114
#[trace("TC-056", "FR-040-AC-10", "FR-038-AC-114")]
#[test]
fn tc_056_a_state_clause_application_stands_only_as_a_clause_body_root() {
    let package = StatePackage::new();
    let invariant = package.clause("invariant");
    // Nested inside the invariant's own condition position: an application
    // outside the body grammar, `malformed_wire` at the nested application.
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
        Code::MalformedWire,
        None,
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
            let deps = clause_dependencies(&[self_key()], object_key());
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
            json!({"term": "aggregate", "members": [binding("level", level.clone()),
                binding("name", text("self"))]}),
        ),
        (
            "a non-text name",
            json!({"term": "aggregate", "members": [binding("name", level.clone()),
                binding("level", level.clone())]}),
        ),
        (
            "a non-integer level",
            json!({"term": "aggregate", "members": [binding("name", text("self")),
                binding("level", text("zero"))]}),
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

/// ACs: FR-040-AC-12
#[trace("TC-056", "FR-040-AC-12")]
#[test]
fn tc_056_frame_state_and_operation_steps_report_in_reader_order() {
    let package = StatePackage::new();
    let frame = package.at("state", "frame");
    let anchor = package.at("state", "operation_anchor");
    let invariant = package.clause("invariant");
    let frame_defect =
        |node: &mut Value| node["body"]["deletes"] = json!([node_id(relationship_key())]);
    let anchor_defect = |node: &mut Value| node["semantic_type"] = node_id(process_key());
    let clause_defect =
        |node: &mut Value| *node = clause_node("invariant", &[INTEGER], object_key());
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
/// A domain package identity the lock never selects.
const OTHER_PACKAGE: &str = "acme/other";
const ORDER_NODE: &str = "ix://acme/orders/Order";
const SUB_NODE: &str = "ix://acme/orders/Sub";
const LEFT_NODE: &str = "ix://acme/orders/Left";
const RIGHT_NODE: &str = "ix://acme/orders/Right";
const BOTH_NODE: &str = "ix://acme/orders/Both";
const ACCOUNT_NODE: &str = "ix://acme/orders/Account";
const ADDRESS_NODE: &str = "ix://acme/orders/Address";
const STREET_TEXT_NODE: &str = "ix://acme/orders/StreetText";
const NATIVE_INTEGER: &str = "ix://quire/native/Integer";

/// A member slot of type `type_ref` with the multiplicity `[1, 1]`.
fn slot(type_ref: &str) -> Value {
    json!({"typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}})
}

fn named_slot(type_ref: &str, name: &str) -> Value {
    let mut value = slot(type_ref);
    value["name"] = json!(name);
    value
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
                {"identity": format!("{ORDER_NODE}/scaled"),
                    "params": [named_slot(NATIVE_INTEGER, "n")],
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

/// A selected object with a record-valued field, and the record's own field.
fn records_document() -> Value {
    let mut document = orders_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": ORDERS, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": ORDERS, "name": "text"},
            "construct": {"meaning": "quire.meaning.model.value-type/v1"},
        }));
    let mut street = domain_field(ADDRESS_NODE, "street");
    street["typeRef"] = json!(STREET_TEXT_NODE);
    let mut address = domain_type(ADDRESS_NODE, &[], vec![street], json!([]));
    address["kind"]["name"] = json!("record");
    let mut text_type = domain_type(STREET_TEXT_NODE, &[], vec![], json!([]));
    text_type["kind"]["name"] = json!("text");
    text_type["scalar"] = json!("text");
    text_type["constraints"] = json!([
        {"keyword": "min", "operands": {"value": 1}},
        {"keyword": "max", "operands": {"value": 80}},
        {"keyword": "profile", "operands": {"value": "unicode-scalars"}},
    ]);
    let mut account = domain_type(
        ACCOUNT_NODE,
        &[],
        vec![domain_field(ACCOUNT_NODE, "address")],
        json!([]),
    );
    account["fields"][0]["typeRef"] = json!(ADDRESS_NODE);
    document["types"]
        .as_array_mut()
        .expect("types")
        .extend([account, address]);
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(text_type);
    document
}

fn record_key() -> String {
    record_key_for(ADDRESS_NODE)
}

fn record_key_for(node: &str) -> String {
    structural(
        "model",
        "record_value_type",
        Some(json!({"kind": "model", "identity": ORDERS, "node": node})),
    )
}

fn record_package() -> Value {
    record_package_for(&records_document())
}

fn record_package_for(document: &Value) -> Value {
    let mut package = ModelPackage::new().value;
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(document)));
    let account = model_key(ACCOUNT_NODE);
    let address = record_key();
    for (key, form, owner) in [
        (&account, "object_type", ACCOUNT_NODE),
        (&address, "record_value_type", ADDRESS_NODE),
    ] {
        let mut node = plain(
            key,
            "model",
            form,
            key,
            &[],
            "type",
            json!({"term": "aggregate", "members": []}),
        );
        node["owner"] = json!({"kind": "model", "identity": ORDERS, "node": owner});
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
    }
    let frame = position(&package, "state", "frame");
    let mut entries = vec![field(&account, "address"), field(&address, "street")];
    entries.sort_by(|a, b| {
        a["declaration"]["digest"]
            .as_str()
            .cmp(&b["declaration"]["digest"].as_str())
    });
    package["semantic_graph"]["nodes"][frame]["body"]["modifies"] = json!(entries);
    let mut deps = package["semantic_graph"]["nodes"][frame]["dependencies"]
        .as_array()
        .expect("deps")
        .clone();
    deps.extend([node_id(&account), node_id(&address)]);
    deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
    package["semantic_graph"]["nodes"][frame]["dependencies"] = json!(deps);
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    package
}

fn read_record_package(package: &Value, document: &Value, work: u64) -> CheckedPackageV2ReadResult {
    let mut evidence = evidence_for(package);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(document)), canonical(document))
        .expect("distinct record document key");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    CheckedPackageV2::read(&canonical(package), limits, &evidence)
}

/// The object field and the record's own field resolve against separate
/// selected owners. Replacing only the record name reaches the frame step.
#[trace("TC-056", "FR-040-AC-14")]
#[test]
fn tc_056_selected_record_frame_fields_resolve_their_own_names() {
    let package = record_package();
    let document = records_document();
    let admitted = match read_record_package(
        &package,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("selected record frame must admit: {other:?}"),
    };
    let account = node_id(&model_key(ACCOUNT_NODE));
    let fields = admitted
        .model_object_fields(&serde_json::from_value(account).expect("node id"))
        .expect("selected Account fields");
    assert!(matches!(
        fields.field("address").and_then(|field| field.member_type()),
        Some(CheckedMemberType::Record(target)) if target.digest.as_ref() == record_key()
    ));
    let mut ghost = package.clone();
    let frame = position(&ghost, "state", "frame");
    let entries = ghost["semantic_graph"]["nodes"][frame]["body"]["modifies"]
        .as_array_mut()
        .expect("modifies");
    let index = entries
        .iter()
        .position(|entry| entry["declaration"]["digest"] == record_key())
        .expect("record entry");
    entries[index]["name"] = json!("ghost");
    mint_ungrouped_structural_keys(&mut ghost);
    settle(&mut ghost);
    match read_record_package(&ghost, &document, CheckedPackageReadLimits::bounded().work) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|p| p.as_str())
            ),
            (
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                Some(format!("/semantic_graph/nodes/{frame}/body/modifies/{index}").as_str())
            )
        ),
        other => panic!("unknown record field must refuse: {other:?}"),
    }
    let mut operation_name = package.clone();
    operation_name["semantic_graph"]["nodes"][frame]["body"]["modifies"][index]["name"] =
        json!("scaled");
    mint_ungrouped_structural_keys(&mut operation_name);
    settle(&mut operation_name);
    match read_record_package(
        &operation_name,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                Some(format!("/semantic_graph/nodes/{frame}/body/modifies/{index}").as_str())
            )
        ),
        other => panic!("an operation name cannot resolve as a record field: {other:?}"),
    }
}

/// An inherited collision remains ambiguous when the declaring frame owner
/// is a selected record; an unrelated object field cannot resolve it.
#[trace("TC-056", "FR-040-AC-14", "FR-040-AC-15")]
#[test]
fn tc_056_selected_record_frame_field_keeps_inherited_ambiguity() {
    let left = "ix://acme/orders/RecordLeft";
    let right = "ix://acme/orders/RecordRight";
    let both = "ix://acme/orders/RecordBoth";
    let mut document = records_document();
    for (node, supertypes, fields) in [
        (left, vec![], vec![domain_field(left, "shared")]),
        (right, vec![], vec![domain_field(right, "shared")]),
        (both, vec![left, right], vec![]),
    ] {
        let mut record = domain_type(node, &supertypes, fields, json!([]));
        record["kind"]["name"] = json!("record");
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(record);
    }
    let mut package = record_package_for(&document);
    let key = record_key_for(both);
    let mut owner = plain(
        &key,
        "model",
        "record_value_type",
        &key,
        &[],
        "type",
        json!({"term": "aggregate", "members": []}),
    );
    owner["owner"] = json!({"kind": "model", "identity": ORDERS, "node": both});
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(owner);
    let frame = position(&package, "state", "frame");
    let entries = package["semantic_graph"]["nodes"][frame]["body"]["modifies"]
        .as_array_mut()
        .expect("modifies");
    entries.push(field(&key, "shared"));
    entries.sort_by(|a, b| {
        a["declaration"]["digest"]
            .as_str()
            .cmp(&b["declaration"]["digest"].as_str())
    });
    let at = entries
        .iter()
        .position(|entry| entry["declaration"]["digest"] == key)
        .expect("ambiguous record entry");
    let deps = package["semantic_graph"]["nodes"][frame]["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    deps.push(node_id(&key));
    deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    match read_record_package(
        &package,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::AmbiguousDeclaration,
                Some(Cause::AmbiguousName),
                Some(format!("/semantic_graph/nodes/{frame}/body/modifies/{at}").as_str())
            )
        ),
        other => panic!("record inherited ambiguity must refuse: {other:?}"),
    }
}

/// The whole reader admits at its exact work budget and returns typed
/// incomplete at the next lower budget. Its last charge is a later state
/// clause target; the selected-row charges are asserted beside the tables.
#[trace("TC-056", "FR-040-AC-16")]
#[test]
fn tc_056_selected_record_fields_obey_exact_work_limit() {
    let package = record_package();
    let document = records_document();
    let read = |work| read_record_package(&package, &document, work);
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read(high),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while low < high {
        let middle = low + (high - low) / 2;
        match read(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            other => panic!("expected admission or incomplete: {other:?}"),
        }
    }
    assert!(matches!(read(low), CheckedPackageV2ReadResult::Admitted(_)));
    match read(low - 1) {
        CheckedPackageV2ReadResult::Incomplete(record) => {
            assert_eq!(
                record.limit_kind,
                quire_contract_model::CheckedPackageLimit::Work
            );
            assert_eq!(
                record.path.as_ref().map(|path| path.as_str()),
                Some("/semantic_graph/nodes/39/body/arguments/2/target")
            );
        }
        other => panic!("one unit below exact must be incomplete: {other:?}"),
    }
}

/// A record name defect is a frame meaning defect, before frame ordering;
/// a broken selected owner is decided before any frame entry.
#[trace("TC-056", "FR-040-AC-16")]
#[test]
fn tc_056_record_field_defects_keep_reader_precedence() {
    let document = records_document();
    let mut package = record_package();
    let frame = position(&package, "state", "frame");
    let entries = package["semantic_graph"]["nodes"][frame]["body"]["modifies"]
        .as_array_mut()
        .expect("modifies");
    let record = entries
        .iter()
        .position(|entry| entry["declaration"]["digest"] == record_key())
        .expect("record entry");
    entries[record]["name"] = json!("ghost");
    entries.reverse();
    let reversed = entries.len() - 1 - record;
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    match read_record_package(
        &package,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                Some(format!("/semantic_graph/nodes/{frame}/body/modifies/{reversed}").as_str())
            )
        ),
        other => panic!("record meaning defect must win: {other:?}"),
    }
    let record_node = nodes(&package)
        .iter()
        .position(|node| node["node_id"]["digest"] == record_key())
        .expect("selected record node");
    package["semantic_graph"]["nodes"][record_node]["owner"]["identity"] = json!(OTHER_PACKAGE);
    // Keep the old key deliberately: the owner/key join must be checked before
    // the later frame name defect.
    refresh_identity(&mut package);
    match read_record_package(
        &package,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, Code::MissingDeclaration);
            assert_eq!(refusal.cause, Some(Cause::MissingSelection));
            assert_eq!(
                refusal.path.as_ref().map(|path| path.as_str()),
                Some(format!("/semantic_graph/nodes/{record_node}/node_id").as_str())
            );
        }
        other => panic!("owner join must precede frame: {other:?}"),
    }
    let mut stale = record_package();
    let stale_frame = position(&stale, "state", "frame");
    let stale_entry = stale["semantic_graph"]["nodes"][stale_frame]["body"]["modifies"]
        .as_array()
        .expect("modifies")
        .iter()
        .position(|entry| entry["declaration"]["digest"] == record_key())
        .expect("record entry");
    stale["semantic_graph"]["nodes"][stale_frame]["body"]["modifies"][stale_entry]["name"] =
        json!("ghost");
    refresh_identity(&mut stale);
    match read_record_package(&stale, &document, CheckedPackageReadLimits::bounded().work) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::InvalidPackage,
                Some(Cause::StaleNodeKey),
                Some(format!("/semantic_graph/nodes/{stale_frame}/node_id").as_str())
            )
        ),
        other => panic!("a stale frame key must precede record field resolution: {other:?}"),
    }
}

/// A name defect in each of two frames is reported from the lower-key frame,
/// even when the other frame occurs first in the graph array.
#[trace("TC-056", "FR-040-AC-16")]
#[test]
fn tc_056_record_name_defects_follow_frame_node_key_order() {
    let document = records_document();
    let mut package = record_package();
    let frame = position(&package, "state", "frame");
    let mut other_frame = package["semantic_graph"]["nodes"][frame].clone();
    let entry = |frame: &Value| {
        frame["body"]["modifies"]
            .as_array()
            .expect("modifies")
            .iter()
            .position(|entry| entry["declaration"]["digest"] == record_key())
            .expect("record field entry")
    };
    let first_entry = entry(&other_frame);
    package["semantic_graph"]["nodes"][frame]["body"]["modifies"][first_entry]["name"] =
        json!("ghost_first");
    other_frame["body"]["modifies"][first_entry]["name"] = json!("ghost_second");
    other_frame["node_id"] = node_id(&"e".repeat(64));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other_frame);
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    let frames: Vec<(usize, &Value)> = nodes(&package)
        .iter()
        .enumerate()
        .filter(|(_, node)| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .collect();
    assert_eq!(frames.len(), 2);
    let (earlier, _) = frames
        .into_iter()
        .min_by_key(|(_, node)| node["node_id"]["digest"].as_str().expect("frame key"))
        .expect("the lower-key frame");
    match read_record_package(
        &package,
        &document,
        CheckedPackageReadLimits::bounded().work,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                Some(
                    format!("/semantic_graph/nodes/{earlier}/body/modifies/{first_entry}").as_str()
                )
            )
        ),
        other => panic!("the lower-key frame's record name must refuse: {other:?}"),
    }
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

/// The model declaration node key of `node` owned by the domain package
/// `identity`. The owner is content-only: it carries no package version.
fn model_key_in(identity: &str, node: &str) -> String {
    structural(
        "model",
        "object_type",
        Some(json!({"kind": "model", "identity": identity, "node": node})),
    )
}

/// The model declaration node key of `node` owned by the selected `acme/orders`.
fn model_key(node: &str) -> String {
    model_key_in(ORDERS, node)
}

/// The key of `ORDER_NODE` owned by a domain package the lock does not select.
fn unselected_key() -> String {
    model_key_in(OTHER_PACKAGE, ORDER_NODE)
}

/// Reads `value` with [`orders_document`] supplied under its digest.
fn read_model(value: &Value) -> CheckedPackageV2ReadResult {
    read_model_with_work(value, CheckedPackageReadLimits::bounded().work)
}

fn read_model_with_work(value: &Value, work: u64) -> CheckedPackageV2ReadResult {
    let document = orders_document();
    let mut evidence = evidence_for(value);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document))
        .expect("domain evidence declarations agree on exact bytes");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    CheckedPackageV2::read(&canonical(value), limits, &evidence)
}

fn model_admits(case: &str, value: &Value) {
    match read_model(value) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause and pointer.
fn model_expect(
    case: &str,
    value: &Value,
    code: Code,
    cause: Cause,
    path: &str,
) -> CheckedPackageRefusal {
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
    refusal
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
            binding("level", json!({"term": "literal", "type": node_id(INTEGER),
                "value_kind": "integer", "value": level})),
        ]}),
    )
}

impl ModelPackage {
    fn new() -> Self {
        let order = model_key(ORDER_NODE);
        // The fixture already holds the one anonymous `Integer` node.
        let integer = INTEGER_KEY.to_owned();
        let mut state = StatePackage::new();
        let frame = state.at("state", "frame");
        let frame_key = state.digest(frame);
        let value = &mut state.value;
        let document = orders_document();
        value["lock"]["model_selections"]
            .as_array_mut()
            .expect("model selections")
            .insert(
                0,
                json!({
                    "identity": ORDERS,
                    "digest_domain": "sha256-jcs", "digest": sha256_hex(&canonical(&document)),
                }),
            );
        let empty = || json!({"term": "aggregate", "members": []});
        let mut added = Vec::new();
        for declared in [ORDER_NODE, SUB_NODE, LEFT_NODE, RIGHT_NODE, BOTH_NODE] {
            let key = model_key(declared);
            let mut node = plain(&key, "model", "object_type", &key, &[], "type", empty());
            node["owner"] = json!({"kind": "model", "identity": ORDERS, "node": declared});
            added.push(node);
        }
        added.push(model_parameter(MODEL_INTEGER_PARAMETER, "n", "1", &integer));
        added.push(model_parameter(
            MODEL_RESULT_PARAMETER,
            "result",
            "1",
            &integer,
        ));
        added.push(model_parameter(MODEL_TEXT_PARAMETER, "t", "1", text_key()));
        added.push(clause_node(
            "postcondition",
            &[self_key(), MODEL_RESULT_PARAMETER, MODEL_INTEGER_PARAMETER],
            anchor_key(),
        ));
        let nodes = value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        for node in nodes.iter_mut() {
            let key = node["node_id"]["digest"].as_str().expect("key").to_owned();
            if key == reference_key(object_key()) {
                node["dependencies"] = json!([node_id(&order)]);
                node["body"] = json!({"term": "aggregate", "members": [reference(&order)]});
            } else if key == frame_key {
                node["semantic_type"] = node_id(&order);
                let mut deps = node["dependencies"].as_array().expect("deps").clone();
                deps.push(node_id(&order));
                deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
                node["dependencies"] = Value::Array(deps);
            } else if key == anchor_key() {
                let mut deps = [order.clone(), frame_key.clone()];
                deps.sort();
                node["semantic_type"] = node_id(&order);
                node["dependencies"] = json!(deps.iter().map(|d| node_id(d)).collect::<Vec<_>>());
                node["body"] = anchor_body(&order, text("scaled"), &frame_key);
            } else if node["body"]["operation"]["member"]["clause"] == "invariant" {
                *node = clause_node("invariant", &[self_key()], &order);
            } else if node["body"]["operation"]["member"]["clause"] == "precondition" {
                *node = clause_node(
                    "precondition",
                    &[self_key(), MODEL_INTEGER_PARAMETER],
                    anchor_key(),
                );
            }
        }
        nodes.extend(added);
        // The `Reference` node now names the selected object type, so its
        // derived key changes with its body.
        rename_node(
            &mut state.value,
            &reference_key(object_key()),
            &reference_key(&order),
        );
        mint_ungrouped_structural_keys(&mut state.value);
        settle(&mut state.value);
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

    fn parameter_key(&self, name: &str) -> String {
        nodes(&self.value)
            .iter()
            .find(|node| {
                node["semantic_form"] == "parameter"
                    && node["body"]["members"][0]["value"]["value"] == name
            })
            .and_then(|node| node["node_id"]["digest"].as_str())
            .unwrap_or_else(|| panic!("parameter {name}"))
            .to_owned()
    }

    fn with_unselected_owner(mut self) -> Self {
        let key = unselected_key();
        let mut node = plain(
            &key,
            "model",
            "object_type",
            &key,
            &[],
            "type",
            json!({"term": "aggregate", "members": []}),
        );
        node["owner"] = json!({"kind": "model", "identity": OTHER_PACKAGE, "node": ORDER_NODE});
        self.value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        rebuild_source_map(&mut self.value);
        refresh_identity(&mut self.value);
        self
    }

    /// Edits the node at `position`, then re-keys and re-derives identity.
    fn edit(mut self, position: usize, edit: impl FnOnce(&mut Value)) -> Self {
        edit(&mut self.value["semantic_graph"]["nodes"][position]);
        mint_ungrouped_structural_keys(&mut self.value);
        settle(&mut self.value);
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

/// ACs: FR-040-AC-27, FR-040-AC-29, FR-040-AC-30
#[trace("TC-056", "FR-040-AC-27", "FR-040-AC-29", "FR-040-AC-30")]
#[test]
fn tc_056_scoped_deletes_check_resolved_operation_after_frame_step() {
    let base = ModelPackage::new();
    let frame = base.frame();
    for name in ["n", "missing"] {
        let package = ModelPackage::new().edit(frame, |node| {
            node["body"]["deletes_scoped"] = json!([{"kind": "parameter", "name": name}]);
        });
        let refusal = model_expect(
            name,
            &package.value,
            Code::MissingDeclaration,
            Cause::MissingName,
            &format!("/semantic_graph/nodes/{frame}"),
        );
        assert_eq!(
            refusal.locus.as_ref().map(|id| id.digest.as_ref()),
            Some(digest(&package.value, frame).as_str()),
            "{name}: frame-node locus"
        );
    }
    let package = ModelPackage::new().edit(frame, |node| {
        node["body"]["deletes_scoped"] = json!([{"kind": "parameter", "name": "missing"}]);
        node["body"]["modifies"] = json!([field(object_key(), "missing")]);
    });
    model_expect(
        "frame defect first",
        &package.value,
        Code::MissingDeclaration,
        Cause::MissingName,
        &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
    );
    let mut state = StatePackage::new();
    let object = state.at("model", "object_type");
    state.value["semantic_graph"]["nodes"][object]["owner"] = source_owner(&fixture_source());
    state.value["semantic_graph"]["nodes"][object]["declaration"] =
        json!({"qualified_name": ["Account"]});
    state.value["semantic_graph"]["nodes"][object]["occurrences"] =
        json!([{"role": "declaration", "ordinal": 0}]);
    mint_ungrouped_structural_keys(&mut state.value);
    select_other_protocol_owner(&mut state.value);
    settle(&mut state.value);
    let frame = position(&state.value, "state", "frame");
    let package = state.edit(frame, |node| {
        node["body"]["deletes_scoped"] = json!([{"kind": "parameter", "name": "missing"}]);
    });
    admits(&package.value);
}

#[trace("TC-056", "FR-040-AC-27")]
#[test]
fn tc_056_scoped_delete_parameter_scan_charges_validation_work() {
    let frame = ModelPackage::new().frame();
    let parameter = ModelPackage::new().edit(frame, |node| {
        node["body"]["deletes_scoped"] = json!([{"kind": "parameter", "name": "n"}]);
    });
    // The three protocol nodes, selected root, and selected Process owner join
    // charge before this scoped scan.
    let incomplete = match read_model_with_work(&parameter.value, 360) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => incomplete,
        other => panic!("expected work limit at scoped parameter scan, got {other:?}"),
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(incomplete.consumed, 361);
    assert_eq!(
        incomplete.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{frame}/body").as_str())
    );
    let refusal = match read_model_with_work(&parameter.value, 361) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected missing-name after paying for scan, got {other:?}"),
    };
    assert_eq!(refusal.code, Code::MissingDeclaration);
    assert_eq!(refusal.cause, Some(Cause::MissingName));
}

impl ModelPackage {
    /// Binds the anchor, and the frame it names, to `context` and `operation`.
    fn anchored(self, context: &str, operation: &str) -> Self {
        let frame = self.frame();
        let anchor = position(&self.value, "state", "operation_anchor");
        let context = context.to_owned();
        let with_context = context.clone();
        let operation = operation.to_owned();
        let package = self.edit(frame, move |node| {
            node["semantic_type"] = node_id(&with_context);
            let mut deps = node["dependencies"].as_array().expect("deps").clone();
            if !deps.contains(&node_id(&with_context)) {
                deps.push(node_id(&with_context));
            }
            deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            node["dependencies"] = Value::Array(deps);
        });
        let frame_key = digest(&package.value, frame);
        package.edit(anchor, move |node| {
            let mut deps = [context.clone(), frame_key.clone()];
            deps.sort();
            node["semantic_type"] = node_id(&context);
            node["dependencies"] = json!(deps.iter().map(|d| node_id(d)).collect::<Vec<_>>());
            node["body"] = anchor_body(&context, text(&operation), &frame_key);
        })
    }

    /// Replaces the parameters of the clause of kind `clause`.
    fn binding(self, clause: &str, parameters: &[&str]) -> Self {
        let clause_position = nodes(&self.value)
            .iter()
            .position(|node| node["body"]["operation"]["member"]["clause"] == clause)
            .unwrap_or_else(|| panic!("a {clause}"));
        let anchor = digest(
            &self.value,
            position(&self.value, "state", "operation_anchor"),
        );
        let replacement = clause_node(clause, parameters, &anchor);
        self.edit(clause_position, move |node| *node = replacement)
    }
}

/// ACs: FR-040-AC-3
#[trace("TC-056", "FR-040-AC-3")]
#[test]
fn tc_056_a_field_entry_resolves_among_the_selected_object_types_fields() {
    let order = model_key(ORDER_NODE);
    let sub = model_key(SUB_NODE);
    let both = model_key(BOTH_NODE);
    let unselected = unselected_key();
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
    // FR-038-AC-155: every owner joins before frame entry resolution.
    let package = ModelPackage::new()
        .with_unselected_owner()
        .modifying(&unselected, "total");
    let owner = package.position_of(&unselected);
    model_expect(
        "another domain package",
        &package.value,
        Code::MissingDeclaration,
        Cause::MissingSelection,
        &format!("/semantic_graph/nodes/{owner}/node_id"),
    );
}

/// A frame entry naming an inherited field is charged for the selected
/// subtype's whole field table by the package reader. `Sub` has two effective
/// fields here and `Order` has one. The subtype variant adds one graph
/// dependency edge as well; after that edge's unit is subtracted, its
/// admission budget differs by one field-table entry.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_frame_field_entry_charges_its_effective_table_end_to_end() {
    let mut document = orders_document();
    document["types"][1]["fields"] = json!([domain_field(SUB_NODE, "extra")]);
    let digest = sha256_hex(&canonical(&document));
    let smallest = |declaring: &str| {
        let mut value = ModelPackage::new().modifying(declaring, "total").value;
        let frame = position(&value, "state", "frame");
        let dependencies = value["semantic_graph"]["nodes"][frame]["dependencies"]
            .as_array()
            .expect("frame dependencies")
            .len();
        value["lock"]["model_selections"][0]["digest"] = json!(digest);
        refresh_identity(&mut value);
        let mut evidence = evidence_for(&value);
        evidence
            .insert_domain_package_document(digest.clone(), canonical(&document))
            .expect("domain evidence declarations agree on exact bytes");
        let read = |work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            CheckedPackageV2::read(&canonical(&value), limits, &evidence)
        };
        let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
        assert!(matches!(
            read(high),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        while low < high {
            let middle = low + (high - low) / 2;
            match read(middle) {
                CheckedPackageV2ReadResult::Admitted(_) => high = middle,
                CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
                other => panic!("expected admission or work limit, got {other:?}"),
            }
        }
        assert!(matches!(
            read(low - 1),
            CheckedPackageV2ReadResult::Incomplete(_)
        ));
        (low, dependencies)
    };
    let (sub_work, sub_edges) = smallest(&model_key(SUB_NODE));
    let (order_work, order_edges) = smallest(&model_key(ORDER_NODE));
    assert_eq!(sub_edges, order_edges + 1);
    assert_eq!(sub_work - order_work - 1, 1, "one extra table entry");
}

/// ACs: FR-040-AC-7
#[trace("TC-056", "FR-040-AC-7")]
#[test]
fn tc_056_an_anchor_operation_resolves_to_one_its_context_declares() {
    let order = model_key(ORDER_NODE);
    let sub = model_key(SUB_NODE);
    let unselected = unselected_key();
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
    ] {
        let package = ModelPackage::new().anchored(context, operation);
        let anchor = position(&package.value, "state", "operation_anchor");
        model_expect(
            case,
            &package.value,
            code,
            cause,
            &format!("/semantic_graph/nodes/{anchor}/body/members/1"),
        );
    }
    // FR-038-AC-155: the unselected owner refuses before anchor lookup.
    let package = ModelPackage::new()
        .with_unselected_owner()
        .anchored(&unselected, "scaled");
    let owner = package.position_of(&unselected);
    model_expect(
        "another domain package",
        &package.value,
        Code::MissingDeclaration,
        Cause::MissingSelection,
        &format!("/semantic_graph/nodes/{owner}/node_id"),
    );
}

/// ACs: FR-040-AC-8, FR-040-AC-9
#[trace("TC-056", "FR-040-AC-8", "FR-040-AC-9")]
#[test]
fn tc_056_a_clause_binds_self_the_result_and_the_operation_parameters() {
    // The base package's precondition `[self, n]` and postcondition
    // `[self, result, n]` over `Order.scaled(n: Integer): Integer` admit.
    model_admits("precondition and postcondition", &ModelPackage::new().value);
    let base = ModelPackage::new();
    let self_key = base.parameter_key("self");
    let integer_parameter = base.parameter_key("n");
    let text_parameter = base.parameter_key("t");
    for (case, clause, parameters) in [
        (
            "a missing operation parameter",
            "precondition",
            vec![self_key.as_str()],
        ),
        (
            "a missing result",
            "postcondition",
            vec![self_key.as_str(), integer_parameter.as_str()],
        ),
        (
            "a parameter of another type",
            "precondition",
            vec![self_key.as_str(), text_parameter.as_str()],
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

#[trace("FR-038-AC-203", "FR-038-AC-205", "FR-038-AC-207")]
#[test]
fn tc_445_source_declared_operation_input_has_no_selected_model_range() {
    let mut source = StatePackage::new();
    let object = source.at("model", "object_type");
    source.value["semantic_graph"]["nodes"][object]["owner"] = source_owner(&fixture_source());
    source.value["semantic_graph"]["nodes"][object]["declaration"] =
        json!({"qualified_name": ["Account"]});
    source.value["semantic_graph"]["nodes"][object]["occurrences"] =
        json!([{"role": "declaration", "ordinal": 0}]);
    let input = "e1".repeat(32);
    let comparison = "e2".repeat(32);
    source.value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            plain(
                &input,
                "value",
                "parameter",
                INTEGER,
                &[],
                "expression",
                json!({"term": "aggregate", "members": [
                    binding("name", text("delta")),
                    binding("level", json!({"term": "literal", "type": node_id(INTEGER),
                        "value_kind": "integer", "value": "0"})),
                ]}),
            ),
            plain(
                &comparison,
                "expression",
                "binary",
                &boolean(),
                &[&input],
                "expression",
                json!({"term": "application", "operator": "binary",
                    "operation": {"identity": "quire.op.integer.eq", "laws": [],
                        "mode": null, "member": null, "leaves": []},
                    "result_type": node_id(&boolean()),
                    "arguments": [reference(&input),
                        {"term": "literal", "type": node_id(INTEGER),
                            "value_kind": "integer", "value": "0"}]}),
            ),
        ]);
    let pre = source.clause("precondition");
    source.value["semantic_graph"]["nodes"][pre]["body"]["arguments"][0]["members"]
        .as_array_mut()
        .expect("parameter aggregate")
        .push(reference(&input));
    source.value["semantic_graph"]["nodes"][pre]["body"]["arguments"][2] = reference(&comparison);
    mint_ungrouped_structural_keys(&mut source.value);
    select_other_protocol_owner(&mut source.value);
    settle(&mut source.value);
    let admitted = match read(&source.value) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("source-declared input fixture must admit: {other:?}"),
    };
    let clause: CheckedNodeId =
        serde_json::from_value(source.value["semantic_graph"]["nodes"][pre]["node_id"].clone())
            .expect("clause id");
    let node = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == clause)
        .expect("admitted clause");
    let claim = node.occurrences.first().expect("authentic claim");
    let refusal = admitted
        .direct_integer_comparison(&clause, claim)
        .expect_err("no selected operation slot bound");
    assert_eq!(refusal.kind, CheckedComparisonRefusal::MissingRange);
    assert_eq!(refusal.path, CheckedComparisonPath::Operand(0));
    assert_eq!(refusal.clause, clause);
    assert_eq!(&refusal.claim, claim);
}

#[trace("TC-050", "FR-038-AC-6")]
#[test]
fn tc_050_admitted_frame_and_state_clause_nodes_lower_under_a_state_profile() {
    let package = StatePackage::new();
    let value = package.value.clone();
    let frame = package.at("state", "frame");
    let clauses = [package.clause("invariant"), package.clause("precondition")];
    let admitted = match read(&value) {
        CheckedPackageV2ReadResult::Admitted(admitted) => *admitted,
        other => panic!("expected admission, read {other:?}"),
    };
    let profile = CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        // Lowering needs every tag in a node's dependency closure supported,
        // so the profile admits each tag the package uses, state included.
        supported_tags: nodes(&value)
            .iter()
            .map(|node| {
                serde_json::from_value(node["node_tag"].clone()).expect("a checked node tag")
            })
            .collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let anchor = package.at("state", "operation_anchor");
    let positions = [frame, anchor, clauses[0], clauses[1]];
    let requested = positions
        .iter()
        .map(|&at| typed_node_id(&digest(&value, at)))
        .collect::<Vec<_>>();
    let result = admitted.lower(&requested, &profile);
    assert_eq!(result.records.len(), positions.len());
    for (&at, record) in positions.iter().zip(&result.records) {
        let CompleteLoweringRecordV2::Lowered { node } = record else {
            panic!("node {at} did not lower: {record:?}");
        };
        assert_eq!(node.node_tag, CheckedNodeTag::State);
        assert_eq!(
            serde_json::to_value(&node.node).expect("node"),
            nodes(&value)[at]
        );
    }
}

/// Every applicable clause kind and point pair admits and the public accessor
/// returns the retained variant, including Post on a precondition and Pre on
/// a postcondition.
#[trace("TC-462", "FR-058-AC-8")]
#[test]
fn tc_462_clause_point_applicability_admits_all_eight_pairs() {
    let initialization = json!({"kind": "initialization", "name": "startup"});
    let handler = json!({"kind": "handler", "name": "tick"});
    let pre = json!({"kind": "pre", "operation": "deposit"});
    let post = json!({"kind": "post", "operation": "deposit"});
    let cases = [
        (
            pointed_invariant(initialization.clone()),
            "invariant",
            ExecutionPoint::Initialization {
                name: ExecutionPointName::new("startup").unwrap(),
            },
        ),
        (
            pointed_invariant(handler.clone()),
            "invariant",
            ExecutionPoint::Handler {
                name: ExecutionPointName::new("tick").unwrap(),
            },
        ),
        (
            pointed_constraint(initialization),
            "constraint",
            ExecutionPoint::Initialization {
                name: ExecutionPointName::new("startup").unwrap(),
            },
        ),
        (
            pointed_constraint(handler),
            "constraint",
            ExecutionPoint::Handler {
                name: ExecutionPointName::new("tick").unwrap(),
            },
        ),
        (
            pointed_state_package(Some(pre.clone()), Some(pre.clone())),
            "precondition",
            ExecutionPoint::Pre {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
        (
            pointed_state_package(Some(post.clone()), Some(post.clone())),
            "precondition",
            ExecutionPoint::Post {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
        (
            with_postcondition(
                pointed_state_package(Some(pre.clone()), Some(pre.clone())),
                Some(pre),
            ),
            "postcondition",
            ExecutionPoint::Pre {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
        (
            with_postcondition(
                pointed_state_package(Some(post.clone()), Some(post.clone())),
                Some(post),
            ),
            "postcondition",
            ExecutionPoint::Post {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
    ];
    for (source, kind, expected) in cases {
        let position = source.clause(kind);
        let admitted = match read(&source.value) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("{kind} point refused: {other:?}"),
        };
        let clause = &admitted.graph().nodes[position];
        let claim = clause.occurrences.first().expect("clause claim");
        assert_eq!(
            admitted.clause_execution_point(&clause.node_id, (&clause.node_id, claim)),
            Ok(expected),
            "{kind} point must retain its exact variant"
        );
    }
}

/// Well-formed inapplicable points refuse at the clause, with no admitted
/// package. Operation anchors remain valid so the clause check is reached.
#[trace("TC-462", "FR-058-AC-8")]
#[test]
fn tc_462_clause_point_applicability_refuses_all_eight_wrong_pairs() {
    let pre = json!({"kind": "pre", "operation": "deposit"});
    let post = json!({"kind": "post", "operation": "deposit"});
    let initialization = json!({"kind": "initialization", "name": "startup"});
    let handler = json!({"kind": "handler", "name": "tick"});
    let cases = [
        (pointed_invariant(pre.clone()), "invariant"),
        (pointed_invariant(post.clone()), "invariant"),
        (pointed_constraint(pre.clone()), "constraint"),
        (pointed_constraint(post.clone()), "constraint"),
        (
            pointed_state_package(Some(pre.clone()), Some(initialization.clone())),
            "precondition",
        ),
        (
            pointed_state_package(Some(pre.clone()), Some(handler.clone())),
            "precondition",
        ),
        (
            with_postcondition(
                pointed_state_package(Some(pre.clone()), Some(pre.clone())),
                Some(initialization),
            ),
            "postcondition",
        ),
        (
            with_postcondition(
                pointed_state_package(Some(post.clone()), Some(post)),
                Some(handler),
            ),
            "postcondition",
        ),
    ];
    for (source, kind) in cases {
        let clause = source.clause(kind);
        expect(
            kind,
            &source.value,
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{clause}"),
            Some(&source.digest(clause)),
        );
    }
}

/// The V2 point is returned from the admitted typed clause, including a name
/// that the authored-contract AnchorName grammar does not admit.
#[trace("TC-462", "FR-058-AC-5", "FR-058-AC-9")]
#[test]
fn tc_462_public_accessor_returns_exact_admitted_points_without_source_bytes() {
    fn cg_consumer(
        package: &CheckedPackageV2,
        clause: &CheckedNodeId,
        claim: (&CheckedNodeId, &CheckedOccurrence),
    ) -> Result<ExecutionPoint, CheckedExecutionPointRefusal> {
        package
            .clause_execution_point(clause, claim)
            .map_err(|error| match error.code {
                CheckedExecutionPointRefusal::UnknownClause => error.code,
                CheckedExecutionPointRefusal::NotStateClause => error.code,
                CheckedExecutionPointRefusal::MissingClaimOccurrence => error.code,
                CheckedExecutionPointRefusal::MissingExecutionPoint => error.code,
            })
    }

    let cases = [
        (
            pointed_invariant(json!({"kind": "initialization", "name": "startup"})),
            "invariant",
            ExecutionPoint::Initialization {
                name: ExecutionPointName::new("startup").unwrap(),
            },
        ),
        (
            pointed_constraint(json!({"kind": "handler", "name": "tick now"})),
            "constraint",
            ExecutionPoint::Handler {
                name: ExecutionPointName::new("tick now").unwrap(),
            },
        ),
        (
            pointed_state_package(
                Some(json!({"kind": "pre", "operation": "deposit"})),
                Some(json!({"kind": "pre", "operation": "deposit"})),
            ),
            "precondition",
            ExecutionPoint::Pre {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
        (
            pointed_state_package(
                Some(json!({"kind": "post", "operation": "deposit"})),
                Some(json!({"kind": "post", "operation": "deposit"})),
            ),
            "precondition",
            ExecutionPoint::Post {
                operation: ExecutionPointName::new("deposit").unwrap(),
            },
        ),
    ];
    for (source, clause_kind, expected) in cases {
        let position = source.clause(clause_kind);
        let admitted = match read(&source.value) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("pointed package refused: {other:?}"),
        };
        let clause = admitted.graph().nodes[position].node_id.clone();
        let claim = admitted.graph().nodes[position].occurrences[0].clone();
        let unchanged = admitted.clone();
        let identity = admitted.package_id().clone();
        drop(source);
        assert_eq!(
            cg_consumer(&admitted, &clause, (&clause, &claim)),
            Ok(expected.clone())
        );
        assert_eq!(
            cg_consumer(&admitted, &clause, (&clause, &claim)),
            Ok(expected.clone())
        );
        assert_eq!(
            cg_consumer(&unchanged, &clause, (&clause, &claim)),
            Ok(expected)
        );
        assert_eq!(admitted, unchanged);
        assert_eq!(*admitted.package_id(), identity);
    }
}

/// Refusals are selected by node, form, authentic claim, then retained point.
#[trace("TC-462", "FR-058-AC-3", "FR-058-AC-4", "FR-058-AC-10")]
#[test]
fn tc_462_public_accessor_refuses_in_closed_precedence_order() {
    let source = pointed_invariant(json!({"kind": "handler", "name": "tick"}));
    let pointed = source.clause("invariant");
    let unpointed = source.clause("precondition");
    let anchor = source.at("state", "operation_anchor");
    let admitted = match read(&source.value) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("pointed package refused: {other:?}"),
    };
    let clause = admitted.graph().nodes[pointed].node_id.clone();
    let unpointed = admitted.graph().nodes[unpointed].node_id.clone();
    let anchor = admitted.graph().nodes[anchor].node_id.clone();
    let claim = admitted.graph().nodes[pointed].occurrences[0].clone();
    let other_claim = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == unpointed)
        .and_then(|node| node.occurrences.first())
        .expect("other clause claim")
        .clone();
    assert_eq!(
        claim, other_claim,
        "colliding role and ordinal are required"
    );
    let wrong_role = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: claim.ordinal,
    };
    let wrong_ordinal = CheckedOccurrence {
        role: CheckedOccurrenceRole::Claim,
        ordinal: claim.ordinal + 1,
    };
    let absent = typed_node_id(&"ee".repeat(32));
    for (selected, owner, occurrence, expected) in [
        (
            &absent,
            &unpointed,
            &wrong_role,
            CheckedExecutionPointRefusal::UnknownClause,
        ),
        (
            &anchor,
            &unpointed,
            &wrong_role,
            CheckedExecutionPointRefusal::NotStateClause,
        ),
        (
            &clause,
            &clause,
            &wrong_role,
            CheckedExecutionPointRefusal::MissingClaimOccurrence,
        ),
        (
            &clause,
            &clause,
            &wrong_ordinal,
            CheckedExecutionPointRefusal::MissingClaimOccurrence,
        ),
        (
            &clause,
            &unpointed,
            &other_claim,
            CheckedExecutionPointRefusal::MissingClaimOccurrence,
        ),
        (
            &unpointed,
            &unpointed,
            &other_claim,
            CheckedExecutionPointRefusal::MissingExecutionPoint,
        ),
    ] {
        let error = admitted
            .clause_execution_point(selected, (owner, occurrence))
            .expect_err("selection must refuse");
        assert_eq!(error.code, expected);
        assert_eq!(&error.clause, selected);
        assert_eq!(&error.claim_owner, owner);
        assert_eq!(&error.claim, occurrence);
    }
    assert_eq!(
        admitted.clause_execution_point(&clause, (&clause, &claim)),
        Ok(ExecutionPoint::Handler {
            name: ExecutionPointName::new("tick").unwrap()
        })
    );
    for &code in CheckedExecutionPointRefusal::all() {
        assert_eq!(
            CheckedExecutionPointRefusal::from_code(code.as_str()),
            Some(code)
        );
    }
    assert_eq!(CheckedExecutionPointRefusal::from_code("unknown"), None);
}

/// Every authentic claim of one pointed node reads its one retained point.
#[trace("TC-462", "FR-058-AC-4", "FR-058-AC-10")]
#[test]
fn tc_462_multiple_claims_of_one_node_share_the_admitted_point() {
    let mut source = pointed_invariant(json!({"kind": "handler", "name": "tick"}));
    let position = source.clause("invariant");
    source.value["semantic_graph"]["nodes"][position]["occurrences"]
        .as_array_mut()
        .expect("clause occurrences")
        .push(json!({"role": "claim", "ordinal": 1}));
    settle(&mut source.value);
    let second = source.value["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| {
            entry["node_id"] == source.value["semantic_graph"]["nodes"][position]["node_id"]
        })
        .expect("clause source entry")
        .clone();
    let mut second = second;
    second["ordinal"] = json!(1);
    source.value["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second);
    refresh_identity(&mut source.value);
    let admitted = match read(&source.value) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two-claim package refused: {other:?}"),
    };
    let clause = &admitted.graph().nodes[position];
    assert_eq!(clause.occurrences.len(), 2);
    for claim in &clause.occurrences {
        assert_eq!(
            admitted.clause_execution_point(&clause.node_id, (&clause.node_id, claim)),
            Ok(ExecutionPoint::Handler {
                name: ExecutionPointName::new("tick").unwrap()
            })
        );
    }
}

/// A changed point changes the clause and package identity, and the accessor
/// follows the rekeyed node rather than the clause kind or shared anchor.
#[trace("TC-462", "FR-058-AC-9")]
#[test]
fn tc_462_point_change_rekeys_node_and_package_without_losing_accessor_value() {
    for (kind, initialized, handled, renamed) in [
        (
            "invariant",
            pointed_invariant(json!({"kind": "initialization", "name": "startup"})),
            pointed_invariant(json!({"kind": "handler", "name": "tick"})),
            pointed_invariant(json!({"kind": "handler", "name": "tock"})),
        ),
        (
            "constraint",
            pointed_constraint(json!({"kind": "initialization", "name": "startup"})),
            pointed_constraint(json!({"kind": "handler", "name": "tick"})),
            pointed_constraint(json!({"kind": "handler", "name": "tock"})),
        ),
    ] {
        let sources = [initialized, handled, renamed];
        let positions = std::array::from_fn::<_, 3, _>(|index| sources[index].clause(kind));
        let bodies = std::array::from_fn::<_, 3, _>(|index| {
            &sources[index].value["semantic_graph"]["nodes"][positions[index]]["body"]
        });
        assert_eq!(bodies[0]["arguments"], bodies[1]["arguments"]);
        assert_eq!(bodies[1]["arguments"], bodies[2]["arguments"]);
        let ids = std::array::from_fn::<_, 3, _>(|index| {
            sources[index].value["semantic_graph"]["nodes"][positions[index]]["node_id"].clone()
        });
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(
            sources[0].value["package_id"],
            sources[1].value["package_id"]
        );
        assert_ne!(
            sources[1].value["package_id"],
            sources[2].value["package_id"]
        );
        for (source, position, expected) in [
            (
                &sources[0],
                positions[0],
                ExecutionPoint::Initialization {
                    name: ExecutionPointName::new("startup").unwrap(),
                },
            ),
            (
                &sources[1],
                positions[1],
                ExecutionPoint::Handler {
                    name: ExecutionPointName::new("tick").unwrap(),
                },
            ),
            (
                &sources[2],
                positions[2],
                ExecutionPoint::Handler {
                    name: ExecutionPointName::new("tock").unwrap(),
                },
            ),
        ] {
            let admitted = match read(&source.value) {
                CheckedPackageV2ReadResult::Admitted(package) => package,
                other => panic!("pointed {kind} refused: {other:?}"),
            };
            let clause = &admitted.graph().nodes[position];
            let claim = clause.occurrences.first().expect("claim");
            assert_eq!(
                admitted.clause_execution_point(&clause.node_id, (&clause.node_id, claim)),
                Ok(expected)
            );
        }
    }
}

/// A source-declared relation over two executions of the selected Account operation.
fn step_relation_package() -> Value {
    let mut package = with_postcondition(StatePackage::new(), None).value;
    let self_position = nodes(&package)
        .iter()
        .position(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "self"
        })
        .expect("self binder");
    let mut second_self = package["semantic_graph"]["nodes"][self_position].clone();
    second_self["node_id"] = node_id(&"ef".repeat(32));
    second_self["body"]["members"][1]["value"]["value"] = json!("1");
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_self);
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    let model = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "model_root")
        .expect("selected model root")["node_id"]
        .clone();
    let anchor = node_id(&digest(
        &package,
        position(&package, "state", "operation_anchor"),
    ));
    let first_self = nodes(&package)[self_position]["node_id"].clone();
    let second_self = nodes(&package)
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "self"
                && node["body"]["members"][1]["value"]["value"] == "1"
        })
        .expect("second binder")["node_id"]
        .clone();
    let clause_id = |kind: &str| {
        nodes(&package)
            .iter()
            .find(|node| {
                node["semantic_form"] == "state_clause"
                    && node["body"]["operation"]["member"]["clause"] == kind
            })
            .expect("clause")["node_id"]
            .clone()
    };
    let invariant = clause_id("invariant");
    let pre = clause_id("precondition");
    let post = clause_id("postcondition");
    let equality_body = json!({
        "term": "application", "operator": "binary",
        "operation": {"identity": "quire.op.reference.eq", "laws": [],
            "mode": null, "member": null, "leaves": []},
        "result_type": node_id(&boolean()),
        "arguments": [
            {"term": "reference", "target": first_self},
            {"term": "reference", "target": second_self},
        ],
    });
    let mut equality_deps = vec![first_self.clone(), second_self.clone()];
    equality_deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&"ab".repeat(32)),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "expression", "semantic_form": "binary",
            "semantic_type": node_id(&boolean()), "dependencies": equality_deps,
            "occurrences": [{"role": "expression", "ordinal": 0}],
            "body": equality_body,
        }));
    settle(&mut package);
    let expression = nodes(&package)
        .iter()
        .find(|node| {
            node["node_tag"] == "expression"
                && node["semantic_form"] == "binary"
                && node["body"]["operation"]["identity"] == "quire.op.reference.eq"
        })
        .expect("relation equality")["node_id"]
        .clone();
    let ref_id = |id: Value| json!({"term": "reference", "target": id});
    let agg = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
    let execution = |variable: &str, self_id: Value| {
        binding(
            variable,
            agg(vec![
                binding("alias", text("V")),
                binding("operation", ref_id(anchor.clone())),
                binding("parameters", agg(vec![ref_id(self_id)])),
                binding("preconditions", agg(vec![ref_id(pre.clone())])),
                binding("postconditions", agg(vec![ref_id(post.clone())])),
                binding("implementation", agg(vec![])),
            ]),
        )
    };
    let body = agg(vec![
        binding("subjects", agg(vec![binding("V", ref_id(model.clone()))])),
        binding(
            "executions",
            agg(vec![
                execution("x", first_self.clone()),
                execution("y", second_self.clone()),
            ]),
        ),
        binding(
            "invariants",
            agg(vec![binding("V", agg(vec![ref_id(invariant.clone())]))]),
        ),
        binding("body", ref_id(expression.clone())),
    ]);
    let mut dependencies = vec![
        model,
        anchor,
        first_self,
        second_self,
        pre,
        post,
        invariant,
        expression,
    ];
    dependencies.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
    dependencies.dedup();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&"cd".repeat(32)),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "claim", "semantic_form": "step_relation",
            "semantic_type": node_id(&boolean()), "dependencies": dependencies,
            "declaration": {"qualified_name": ["Det"]},
            "owner": source_owner(&fixture_source()),
            "occurrences": [{"role": "declaration", "ordinal": 0}, {"role": "claim", "ordinal": 0}],
            "body": body,
        }));
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    package
}

/// Trace: FR-350-AC-1, FR-350-AC-2, FR-350-AC-3
#[trace("FR-350-AC-1", "FR-350-AC-2", "FR-350-AC-3")]
#[test]
fn step_relation_reader_retains_two_executions_and_roundtrips() {
    let package = step_relation_package();
    let swapped = rekey_step_relation(package.clone(), |node| {
        node["body"]["members"][1]["value"]["members"]
            .as_array_mut()
            .expect("executions")
            .swap(0, 1);
    });
    let original = position(&package, "claim", "step_relation");
    let swapped_at = position(&swapped, "claim", "step_relation");
    assert_ne!(
        package["semantic_graph"]["nodes"][original]["node_id"],
        swapped["semantic_graph"]["nodes"][swapped_at]["node_id"]
    );
    assert_ne!(package["package_id"], swapped["package_id"]);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("step relation must admit: {other:?}"),
    };
    let relation = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "step_relation")
        .expect("relation");
    let body = relation.step_relation_body().expect("typed relation");
    assert_eq!(body.subjects.len(), 1);
    assert_eq!(body.executions.len(), 2);
    assert_eq!(body.executions[0].variable.as_ref(), "x");
    assert_eq!(body.executions[1].variable.as_ref(), "y");
    assert_ne!(body.executions[0].parameters, body.executions[1].parameters);
    assert!(!body.requests_code_claim());
    let preimage = admitted.identity_preimage();
    let borrowed = CheckedPackageIdentityPreimageRefV2 {
        version: &preimage.version,
        edition: &preimage.edition,
        profile_selections: &preimage.profile_selections,
        definition_selections: &preimage.definition_selections,
        model_selections: &preimage.model_selections,
        required_features: &preimage.required_features,
        dependency_selections: &preimage.dependency_selections,
        identity_projection: &admitted.graph().nodes,
    };
    let bytes = encode_checked_package_v2(
        &borrowed,
        admitted.package_id(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        Limits::new(u64::MAX),
    )
    .expect("canonical writer");
    match CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Admitted(reloaded) => {
            let again = reloaded
                .graph()
                .nodes
                .iter()
                .find(|node| node.semantic_form.as_ref() == "step_relation")
                .expect("relation");
            assert_eq!(again.step_relation_body(), Some(body));
            assert_eq!(again.node_id, relation.node_id);
            assert_eq!(reloaded.package_id(), admitted.package_id());
        }
        other => panic!("written package must read back: {other:?}"),
    }
}

fn rekey_step_relation(mut package: Value, edit: impl FnOnce(&mut Value)) -> Value {
    fn targets(term: &Value, found: &mut std::collections::BTreeSet<String>) {
        if term["term"] == "reference" {
            if let Some(digest) = term["target"]["digest"].as_str() {
                found.insert(digest.to_owned());
            }
        }
        match term {
            Value::Array(values) => {
                for value in values {
                    targets(value, found);
                }
            }
            Value::Object(values) => {
                for value in values.values() {
                    targets(value, found);
                }
            }
            _ => {}
        }
    }
    let pos = position(&package, "claim", "step_relation");
    edit(&mut package["semantic_graph"]["nodes"][pos]);
    let mut refs = std::collections::BTreeSet::new();
    targets(&package["semantic_graph"]["nodes"][pos]["body"], &mut refs);
    package["semantic_graph"]["nodes"][pos]["dependencies"] = json!(refs
        .into_iter()
        .map(|digest| node_id(&digest))
        .collect::<Vec<_>>());
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    package
}

/// Trace: FR-350-AC-4, FR-350-AC-5
#[trace("FR-350-AC-4", "FR-350-AC-5")]
#[test]
fn step_relation_reader_refuses_layout_alias_targets_and_premises() {
    let base = step_relation_package();
    let pos = position(&base, "claim", "step_relation");
    let prefix = format!("/semantic_graph/nodes/{pos}/body/members");
    let malformed = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][1]["value"]["members"][0]["value"]["members"]
            .as_array_mut()
            .expect("execution bindings")
            .remove(5);
    });
    expect(
        "missing implementation",
        &malformed,
        Code::InvalidSemanticGraph,
        None,
        &format!("{prefix}/1/value/members/0/value/members/5"),
        None,
    );
    let unknown = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][1]["value"]["members"][0]["value"]["members"][0]["value"]
            ["value"] = json!("W");
    });
    expect(
        "unknown alias",
        &unknown,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        &format!("{prefix}/1/value/members/0/value/members/0"),
        None,
    );
    let object = nodes(&base)
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
        .expect("object")["node_id"]
        .clone();
    let wrong_operation = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][1]["value"]["members"][0]["value"]["members"][1]["value"]
            ["target"] = object;
    });
    expect(
        "wrong operation",
        &wrong_operation,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("{prefix}/1/value/members/0/value/members/1/value/target"),
        None,
    );
    let no_precondition = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][1]["value"]["members"][0]["value"]["members"][3]["value"]
            ["members"] = json!([]);
    });
    expect(
        "missing precondition",
        &no_precondition,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("{prefix}/1/value/members/0/value/members/3/value"),
        None,
    );
    let no_invariant = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][2]["value"]["members"][0]["value"]["members"] = json!([]);
    });
    expect(
        "missing invariant",
        &no_invariant,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("{prefix}/2/value/members/0/value"),
        None,
    );
    let mut integer_package = base.clone();
    let mut integer_literal = nodes(&integer_package)
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "literal")
        .expect("literal")
        .clone();
    integer_literal["node_id"] = node_id(&"f1".repeat(32));
    integer_literal["semantic_type"] = node_id(&family_key("a3a3"));
    integer_literal["dependencies"] = json!([node_id(&family_key("a3a3"))]);
    integer_package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(integer_literal);
    mint_ungrouped_structural_keys(&mut integer_package);
    settle(&mut integer_package);
    let integer_literal = nodes(&integer_package)
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "literal"
                && node["semantic_type"] == node_id(&family_key("a3a3"))
        })
        .expect("integer literal")["node_id"]
        .clone();
    let mut integer_expression = nodes(&integer_package)
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference expression")
        .clone();
    integer_expression["node_id"] = node_id(&"f2".repeat(32));
    integer_expression["semantic_type"] = node_id(&family_key("a3a3"));
    integer_expression["body"] = json!({"term": "reference", "target": integer_literal});
    integer_expression["dependencies"] = json!([node_id(&family_key("a3a3")), integer_literal]);
    integer_package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(integer_expression);
    mint_ungrouped_structural_keys(&mut integer_package);
    settle(&mut integer_package);
    let integer_expression = nodes(&integer_package)
        .iter()
        .find(|node| {
            node["node_tag"] == "expression"
                && node["semantic_form"] == "reference"
                && node["semantic_type"] == node_id(&family_key("a3a3"))
        })
        .expect("integer expression")["node_id"]
        .clone();
    let nonboolean = rekey_step_relation(integer_package, |node| {
        node["body"]["members"][3]["value"]["target"] = integer_expression;
    });
    expect(
        "integer body",
        &nonboolean,
        Code::IllTyped,
        Some(Cause::NonBooleanRoot),
        &format!("{prefix}/3/value/target"),
        None,
    );
    let first = base["semantic_graph"]["nodes"][pos]["body"]["members"][1]["value"]["members"][0]
        ["value"]["members"][2]["value"]["members"][0]["target"]
        .clone();
    let reused = rekey_step_relation(base.clone(), |node| {
        node["body"]["members"][1]["value"]["members"][1]["value"]["members"][2]["value"]
            ["members"][0]["target"] = first;
    });
    expect(
        "reused binder",
        &reused,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        &format!("{prefix}/1/value/members/1/value/members/2/value"),
        None,
    );
    let mut stale = base;
    stale["semantic_graph"]["nodes"][pos]["body"]["members"][1]["value"]["members"][1]["value"]
        ["members"][2]["value"]["members"][0]["target"] = stale["semantic_graph"]["nodes"][pos]
        ["body"]["members"][1]["value"]["members"][0]["value"]["members"][2]["value"]["members"][0]
        ["target"]
        .clone();
    refresh_identity(&mut stale);
    assert_eq!(
        refused(&stale).code,
        Code::InvalidSemanticGraph,
        "stale key precedes binder join"
    );
}

fn bound_step_relation_package() -> Value {
    let mut package = step_relation_package();
    let object = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
        .expect("object")["node_id"]
        .clone();
    let body = json!({
        "term": "abstraction_relation",
        "objects": [{"type": object, "rust_type": ["domain", "Account"],
            "fields": [{"name": "balance", "rust_field": "balance"}]}],
        "populations": [],
        "frames": [{"context": object, "operation": "deposit",
            "function": ["domain", "deposit"], "receiver": "self", "parameters": []}],
    });
    let key = sha256_hex(&canonical(&json!({
        "version": "quire.abstraction-relation-node/v1", "body": body,
    })));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&key), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "correspondence", "semantic_form": "abstraction_relation",
            "semantic_type": node_id(&key), "dependencies": [object],
            "occurrences": [{"role": "declaration", "ordinal": 0}], "body": body,
        }));
    settle(&mut package);
    rekey_step_relation(package, |node| {
        for entry in node["body"]["members"][1]["value"]["members"]
            .as_array_mut()
            .expect("executions")
        {
            entry["value"]["members"][5]["value"] = reference(&key);
        }
    })
}

/// Trace: FR-350-AC-2, FR-350-AC-4
#[trace("FR-350-AC-2", "FR-350-AC-4")]
#[test]
fn step_relation_bound_implementation_requests_code_claim_and_checks_frame() {
    let package = bound_step_relation_package();
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("bound relation must admit: {other:?}"),
    };
    let body = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "step_relation")
        .expect("relation")
        .step_relation_body()
        .expect("typed body");
    assert!(body.requests_code_claim());
    assert_eq!(
        body.executions[0].implementation,
        body.executions[1].implementation
    );
    let relation = position(&package, "claim", "step_relation");
    let mut unbound = package.clone();
    let abstraction = position(&package, "correspondence", "abstraction_relation");
    unbound["semantic_graph"]["nodes"][abstraction]["body"]["frames"] = json!([]);
    let new_body = unbound["semantic_graph"]["nodes"][abstraction]["body"].clone();
    let new_key = sha256_hex(&canonical(
        &json!({"version": "quire.abstraction-relation-node/v1", "body": new_body}),
    ));
    unbound["semantic_graph"]["nodes"][abstraction]["semantic_type"] = node_id(&new_key);
    rename_node(
        &mut unbound,
        package["semantic_graph"]["nodes"][abstraction]["node_id"]["digest"]
            .as_str()
            .expect("key"),
        &new_key,
    );
    unbound["semantic_graph"]["nodes"][abstraction]["dependencies"] = json!([]);
    mint_ungrouped_structural_keys(&mut unbound);
    settle(&mut unbound);
    expect("frame missing", &unbound, Code::InvalidModelBinding, Some(Cause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{relation}/body/members/1/value/members/0/value/members/5/value/target"), None);
}
