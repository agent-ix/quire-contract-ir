// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-96 through AC-105 and AC-108, at the package: the temporal step
//! of the reader order (QSpec FR-370) over temporal clauses, formulas and
//! fairness nodes.
//!
//! Every package is built here from this crate's own vocabulary, by editing
//! the in-repo all-families fixture (a clause over `eventually {0, 3}` over
//! `holds`) and re-deriving its application keys, dependency joins and
//! identity with `settle`; nothing of QSpec's fixtures is copied in (the
//! fixtures are read by `make conformance-qspec`, FR-038-AC-107).

use crate::support::checked_package::{
    canonical, evidence_for, family_key, fixture_source, mint_ungrouped_structural_keys,
    model_owner, node_id, owned_structural_node, settle, sha256_hex, source_owner, structural_key,
    typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeTag, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

const ORDERS: &str = "acme/orders";
const ORDERS_VERSION: &str = "1.0.0";
/// A domain package identity the lock never selects.
const OTHER_PACKAGE: &str = "acme/other";
const ORDER_NODE: &str = "ix://acme/orders/Order";
const SUB_NODE: &str = "ix://acme/orders/Sub";
const LEFT_NODE: &str = "ix://acme/orders/Left";
const RIGHT_NODE: &str = "ix://acme/orders/Right";
const BOTH_NODE: &str = "ix://acme/orders/Both";
const NATIVE_INTEGER: &str = "ix://quire/native/Integer";

fn slot(type_ref: &str) -> Value {
    json!({"typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}})
}

fn domain_type(node: &str, supertypes: &[&str], operations: Value) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": ORDERS, "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": [], "operations": operations,
    })
}

/// `Order { scaled(Integer): Integer; reset() }`, `Sub: Order`, and
/// `Both: Left, Right` where `Left` and `Right` each declare `act`.
fn orders_document() -> Value {
    let act = |owner: &str| json!([{"identity": format!("{owner}/act"), "params": []}]);
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": ORDERS, "version": ORDERS_VERSION},
        "constructs": [{
            "kind": {"module": ORDERS, "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [
            domain_type(ORDER_NODE, &[], json!([
                {"identity": format!("{ORDER_NODE}/scaled"), "params": [slot(NATIVE_INTEGER)],
                 "returns": slot(NATIVE_INTEGER)},
                {"identity": format!("{ORDER_NODE}/reset"), "params": []},
            ])),
            domain_type(SUB_NODE, &[ORDER_NODE], json!([])),
            domain_type(LEFT_NODE, &[], act(LEFT_NODE)),
            domain_type(RIGHT_NODE, &[], act(RIGHT_NODE)),
            domain_type(BOTH_NODE, &[LEFT_NODE, RIGHT_NODE], json!([])),
        ],
    })
}

/// The model declaration node key of `node` owned by the domain package
/// `identity`: the SHA-256 of its structural preimage.
fn model_key_in(identity: &str, node: &str) -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null, "declaration": null,
        "recursion": null, "body": {"term": "aggregate", "members": []},
        "owner": {"kind": "model", "identity": identity, "node": node},
    })))
}

fn model_key(node: &str) -> String {
    model_key_in(ORDERS, node)
}

/// Reads `package` with [`orders_document`] supplied under its digest.
fn read(package: &Value) -> CheckedPackageV2ReadResult {
    read_limited(package, CheckedPackageReadLimits::bounded())
}

fn read_limited(package: &Value, limits: CheckedPackageReadLimits) -> CheckedPackageV2ReadResult {
    let document = orders_document();
    let mut evidence = evidence_for(package);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document));
    CheckedPackageV2::read(&canonical(package), limits, &evidence)
}

fn admitted(case: &str, package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// The refusal's code, cause and pointer text.
type Outcome = (Code, Option<Cause>, Option<String>);

fn refusal_of(case: &str, package: &Value) -> (Outcome, Option<String>) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => (
            (
                refusal.code,
                refusal.cause,
                refusal.path.map(|path| path.as_str().to_owned()),
            ),
            refusal.locus.map(|locus| locus.digest.to_string()),
        ),
        other => panic!("{case}: expected a refusal, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause, pointer and locus, the locus being the key
/// of the node the pointer is on (the default of FR-038's "Path and locus"
/// table); a refusal with another locus is [`expect_located`].
fn expect(case: &str, package: &Value, code: Code, cause: Cause, path: &str) {
    let position: usize = path
        .strip_prefix("/semantic_graph/nodes/")
        .and_then(|rest| rest.split('/').next())
        .and_then(|node| node.parse().ok())
        .unwrap_or_else(|| panic!("{case}: {path} is on no node"));
    expect_located(
        case,
        package,
        (code, cause, path),
        &digest(package, position),
    );
}

/// Asserts the refusal's code, cause, pointer and locus (the node key).
fn expect_located(case: &str, package: &Value, outcome: (Code, Cause, &str), locus: &str) {
    let (code, cause, path) = outcome;
    assert_eq!(
        refusal_of(case, package),
        (
            (code, Some(cause), Some(path.to_owned())),
            Some(locus.to_owned())
        ),
        "{case}"
    );
}

// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

fn find(package: &Value, test: impl Fn(&Value) -> bool) -> usize {
    nodes(package)
        .iter()
        .position(test)
        .expect("the fixture carries the node")
}

fn find_form(package: &Value, tag: &str, form: &str) -> usize {
    find(package, |node| {
        node["node_tag"] == tag && node["semantic_form"] == form
    })
}

fn find_identity(package: &Value, identity: &str) -> usize {
    find(package, |node| {
        node["body"]["operation"]["identity"] == identity
    })
}

fn digest(package: &Value, position: usize) -> String {
    nodes(package)[position]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned()
}

fn refer(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn aaaa() -> String {
    family_key("aaaa")
}

static NEXT: AtomicU64 = AtomicU64::new(1);

/// A key no node of the fixture has; `settle` replaces it for an application
/// node, and a node with another body keeps it.
fn fresh() -> String {
    format!(
        "{:064x}",
        0xf000_0000_0000_0000_u64 + NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

/// A node keyed `key` whose body is `body`.
fn node_with_key(key: &str, tag: &str, form: &str, body: Value) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": node_id(&aaaa()),
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

fn application(operator: &str, identity: &str, member: Value, arguments: Vec<Value>) -> Value {
    json!({
        "term": "application", "operator": operator,
        "operation": {"identity": identity, "laws": [], "mode": null,
                      "member": member, "leaves": []},
        "result_type": node_id(&aaaa()), "arguments": arguments,
    })
}

fn temporal_identity(short: &str) -> String {
    format!("quire.op.temporal.{short}")
}

/// A `temporal_formula` application of `quire.op.temporal.<short>`.
fn formula_body(short: &str, member: Value, arguments: Vec<Value>) -> Value {
    application(
        "temporal_formula",
        &temporal_identity(short),
        member,
        arguments,
    )
}

/// A `temporal`/`formula` node applying `quire.op.temporal.<short>`.
fn formula(short: &str, member: Value, arguments: Vec<Value>) -> Value {
    node_with_key(
        &fresh(),
        "temporal",
        "formula",
        formula_body(short, member, arguments),
    )
}

/// Appends `node`, returning its position.
fn push(package: &mut Value, node: Value) -> usize {
    let list = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    list.push(node);
    list.len() - 1
}

fn interval(lower: Value, upper: Value) -> Value {
    json!({"kind": "temporal_interval", "interval": {"lower": lower, "upper": upper}})
}

fn closed(lower: &str, upper: &str) -> Value {
    interval(json!(lower), json!(upper))
}

fn unbounded() -> Value {
    json!({"kind": "temporal_interval", "interval": null})
}

fn lower_bounded(lower: &str) -> Value {
    interval(json!(lower), Value::Null)
}

const BOUNDED: &str = "quire.temporal.event-position.false-extension/v1";
const INFINITE: &str = "quire.temporal.infinite-trace/v1";
const TIMED: &str = "quire.temporal.timed/v1";
const FIVE_PROFILES: [&str; 5] = [
    BOUNDED,
    "quire.temporal.fixed-sample.false-extension/v1",
    "quire.temporal.timestamped-event.finite-window/v1",
    INFINITE,
    TIMED,
];

/// Selects `identity` as the temporal profile, in the lock and in every law of
/// the `temporal_profile` role.
fn select_profile(package: &mut Value, identity: &str) {
    for selection in package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("selections")
    {
        if selection["role"] == "temporal_profile" {
            selection["definition"]["identity"] = json!(identity);
        }
    }
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        // `get_mut`, not indexing: indexing a body without an `operation`
        // would insert one.
        let laws = node
            .get_mut("body")
            .and_then(|body| body.get_mut("operation"))
            .and_then(|operation| operation.get_mut("laws"))
            .and_then(Value::as_array_mut);
        if let Some(laws) = laws {
            for law in laws {
                if law["role"] == "temporal_profile" {
                    law["definition"]["identity"] = json!(identity);
                }
            }
        }
    }
}

/// The fixture, its clause's formula argument naming the application of
/// `quire.op.temporal.<short>` over `arguments` (default: the fixture's
/// `holds` node, `arity` times) with `member`, under `profile`. The position
/// of that root formula node.
fn with_root(
    profile: &str,
    short: &str,
    member: Value,
    arguments: Option<Vec<Value>>,
    arity: usize,
) -> (Value, usize) {
    let mut package = v2_all_families();
    select_profile(&mut package, profile);
    let root = find_identity(&package, &temporal_identity("eventually"));
    let holds = digest(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    let arguments = arguments.unwrap_or_else(|| vec![refer(&holds); arity]);
    package["semantic_graph"]["nodes"][root]["body"] = formula_body(short, member, arguments);
    settle(&mut package);
    (package, root)
}

fn clause(package: &Value) -> usize {
    find_form(package, "temporal", "temporal_clause")
}

fn at_node(position: usize, tail: &str) -> String {
    format!("/semantic_graph/nodes/{position}{tail}")
}

/// The domain package, its model declaration nodes and the lock's selection of
/// it, added to `package`.
fn with_model(package: &mut Value) {
    let document = orders_document();
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .insert(
            0,
            json!({
                "identity": ORDERS,
                "digest_domain": "sha256-jcs", "digest": sha256_hex(&canonical(&document)),
            }),
        );
    for declared in [ORDER_NODE, SUB_NODE, LEFT_NODE, RIGHT_NODE, BOTH_NODE] {
        let key = model_key(declared);
        let mut node = node_with_key(
            &key,
            "model",
            "object_type",
            json!({"term": "aggregate", "members": []}),
        );
        node["semantic_type"] = node_id(&key);
        node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
        push(
            package,
            owned_structural_node(node, model_owner(ORDERS, declared)),
        );
    }
}

fn fairness_member(declaration: &str, name: &str) -> Value {
    let mut closed_object = serde_json::Map::new();
    closed_object.insert("kind".to_owned(), json!("fairness"));
    closed_object.insert("fairness_kind".to_owned(), json!("weak"));
    closed_object.insert("granularity".to_owned(), json!("whole"));
    closed_object.insert("declaration".to_owned(), node_id(declaration));
    closed_object.insert("name".to_owned(), json!(name));
    Value::Object(closed_object)
}

/// A `temporal`/`fairness` node applying `quire.op.temporal.fair` with `member`.
fn fairness(member: Value) -> Value {
    node_with_key(
        &fresh(),
        "temporal",
        "fairness",
        application(
            "temporal_fairness",
            &temporal_identity("fair"),
            member,
            vec![],
        ),
    )
}

/// Makes the clause's fairness argument name `fairness_nodes` (positions), in
/// order.
fn with_fairness_argument(package: &mut Value, fairness_nodes: &[usize]) {
    let members: Vec<Value> = fairness_nodes
        .iter()
        .map(|position| refer(&digest(package, *position)))
        .collect();
    let clause = clause(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": members});
}

/// The model-enabled fixture with one fairness node over `member`, under
/// `profile`. The position of the fairness node.
fn with_fairness(profile: &str, member: Value) -> (Value, usize) {
    let mut package = v2_all_families();
    select_profile(&mut package, profile);
    with_model(&mut package);
    let fair = push(&mut package, fairness(member));
    with_fairness_argument(&mut package, &[fair]);
    settle(&mut package);
    (package, fair)
}

fn profile_supporting(tags: &[CheckedNodeTag]) -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: tags.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

// ---------------------------------------------------------------------------
// AC-96
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-96
#[trace("TC-048", "FR-038-AC-96")]
#[test]
fn tc_048_every_temporal_identity_admits_at_its_node_and_lowers_as_data() {
    let mut package = v2_all_families();
    select_profile(&mut package, BOUNDED);
    let holds = digest(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    let eventually = find_identity(&package, &temporal_identity("eventually"));
    let mut tree = digest(&package, eventually);
    let two = |short: &str, left: &str, right: &str| {
        formula(short, Value::Null, vec![refer(left), refer(right)])
    };
    let mut leaves = Vec::new();
    for short in ["true", "false"] {
        leaves.push(push(&mut package, formula(short, Value::Null, vec![])));
    }
    let truth = digest(&package, leaves[0]);
    let falsehood = digest(&package, leaves[1]);
    leaves.push(push(
        &mut package,
        formula("not", Value::Null, vec![refer(&holds)]),
    ));
    leaves.push(push(&mut package, two("or", &truth, &falsehood)));
    leaves.push(push(&mut package, two("implies", &truth, &falsehood)));
    for short in ["always", "once", "historically"] {
        leaves.push(push(
            &mut package,
            formula(short, closed("0", "3"), vec![refer(&holds)]),
        ));
    }
    for short in ["until", "release", "since", "triggered"] {
        leaves.push(push(
            &mut package,
            formula(short, closed("1", "2"), vec![refer(&holds), refer(&truth)]),
        ));
    }
    // `and` over the chain, so every formula node is an operand of one.
    for leaf in leaves {
        let operand = digest(&package, leaf);
        let node = push(&mut package, two("and", &tree, &operand));
        tree = digest(&package, node);
    }
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][5] = refer(&tree);
    settle(&mut package);

    let reader = admitted("fifteen identities", &package);
    let identities: std::collections::BTreeSet<&str> = nodes(&package)
        .iter()
        .filter(|node| node["semantic_form"] == "formula")
        .filter_map(|node| node["body"]["operation"]["identity"].as_str())
        .collect();
    assert_eq!(identities.len(), 15, "{identities:?}");

    // The clause lowers as data under a profile that supports `temporal`, and
    // returns `unsupported` naming it under one that does not.
    let clause_key = typed_node_id(&digest(&package, clause_at));
    let temporal_only = profile_supporting(&[
        CheckedNodeTag::ScalarType,
        CheckedNodeTag::BoundedDomain,
        CheckedNodeTag::CompositeType,
        CheckedNodeTag::Value,
        CheckedNodeTag::Temporal,
    ]);
    let lowered = reader.lower(std::slice::from_ref(&clause_key), &temporal_only);
    assert!(
        matches!(lowered.records[0], CompleteLoweringRecordV2::Lowered { .. }),
        "{:?}",
        lowered.records[0]
    );
    let mut without = temporal_only.clone();
    without.supported_tags.remove(&CheckedNodeTag::Temporal);
    let refused = reader.lower(std::slice::from_ref(&clause_key), &without);
    assert!(
        matches!(
            refused.records[0],
            CompleteLoweringRecordV2::Unsupported {
                node_tag: CheckedNodeTag::Temporal,
                ..
            }
        ),
        "{:?}",
        refused.records[0]
    );
}

/// The fairness node admits as the body root of a `temporal`/`fairness` node
/// referenced from the fairness argument of a clause selecting the
/// infinite-trace profile; a bounded profile admits no fairness argument.
///
/// Tracing: TC-048, FR-038-AC-96
#[trace("TC-048", "FR-038-AC-96")]
#[test]
fn tc_048_a_fairness_node_admits_under_the_infinite_trace_profile() {
    let (package, fair) =
        with_fairness(INFINITE, fairness_member(&model_key(ORDER_NODE), "scaled"));
    admitted("infinite-trace", &package);
    let (bounded, _) = with_fairness(BOUNDED, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let clause_at = clause(&bounded);
    expect(
        "bounded",
        &bounded,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body"),
    );
    assert_eq!(nodes(&package)[fair]["node_tag"], "temporal");
}

// ---------------------------------------------------------------------------
// AC-97
// ---------------------------------------------------------------------------

const INTERVAL_OPERATORS: [(&str, usize); 8] = [
    ("eventually", 1),
    ("always", 1),
    ("once", 1),
    ("historically", 1),
    ("until", 2),
    ("release", 2),
    ("since", 2),
    ("triggered", 2),
];

/// Tracing: TC-048, FR-038-AC-97
#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_an_interval_member_admits_and_refuses_on_each_interval_operator() {
    let admits = [
        ("{0, 3}", closed("0", "3")),
        ("{9, 10}", closed("9", "10")),
        ("{2, null}", lower_bounded("2")),
        ("null interval", unbounded()),
    ];
    // Negative bounds are refused in strict wire validation
    // (`flat_wire::check_interval_bounds`), under every profile.
    let negative = [
        ("{-1, 3}", closed("-1", "3"), "lower"),
        ("{0, -2}", closed("0", "-2"), "upper"),
        ("{-5, -2}", closed("-5", "-2"), "lower"),
    ];
    let reversed = [
        ("{3, 0}", closed("3", "0")),
        ("{10, 9}", closed("10", "9")),
        (
            "{2^64+1, 2^64}",
            closed("18446744073709551617", "18446744073709551616"),
        ),
    ];
    let wrong_shape = vec![
        ("a fairness member", fairness_member(&aaaa(), "Step")),
        (
            "a member holding a third member",
            json!({"kind": "temporal_interval", "extra": 1,
                   "interval": {"lower": "0", "upper": "3"}}),
        ),
        (
            "an interval holding a third member",
            json!({"kind": "temporal_interval",
                   "interval": {"lower": "0", "upper": "3", "extra": 1}}),
        ),
    ];
    // A bound outside the non-negative integer pattern, malformed as well as
    // negative, is `invalid-value` at that bound in the early stage (merged
    // QSpec FR-370: strict wire validation, before the temporal step), first
    // in member order; a JSON integer is no string.
    let mut outside_pattern = vec![
        ("an integer lower", interval(json!(0), json!("3")), "lower"),
        ("an integer upper", interval(json!("0"), json!(3)), "upper"),
    ];
    for bad in ["1.5", "01", "+1", "", "3x"] {
        outside_pattern.push(("a lower outside the pattern", closed(bad, "9"), "lower"));
        outside_pattern.push(("an upper outside the pattern", closed("0", bad), "upper"));
    }
    for (short, arity) in INTERVAL_OPERATORS {
        for (case, member) in &admits {
            let (package, _) = with_root(INFINITE, short, member.clone(), None, arity);
            admitted(&format!("{short} {case}"), &package);
        }
        for profile in [BOUNDED, INFINITE] {
            for (case, member, bound) in &negative {
                let (package, root) = with_root(profile, short, member.clone(), None, arity);
                expect(
                    &format!("{short} {case} under {profile}"),
                    &package,
                    Code::InvalidPackage,
                    Cause::InvalidValue,
                    &at_node(root, &format!("/body/operation/member/interval/{bound}")),
                );
            }
        }
        for (case, member) in &reversed {
            let (package, root) = with_root(BOUNDED, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, "/body"),
            );
        }
        for (case, member) in &wrong_shape {
            let (package, root) = with_root(INFINITE, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case} {member}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body/operation/member"),
            );
        }
        for (case, member, bound) in &outside_pattern {
            let (package, root) = with_root(INFINITE, short, member.clone(), None, arity);
            expect(
                &format!("{short} {case} {member}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
        // A `null` member on an interval-capable operator is refused at the
        // application, not at `operation.member` (merged QSpec FR-370 text).
        let (package, root) = with_root(INFINITE, short, Value::Null, None, arity);
        expect(
            &format!("{short} a null member"),
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(root, "/body"),
        );
    }
}

/// A member on `holds`, `not` and the clause, none of which catalogues one.
///
/// Tracing: TC-048, FR-038-AC-97
#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_a_member_on_holds_not_and_the_clause_is_a_member_mismatch() {
    let member = closed("0", "3");
    let mut holds = v2_all_families();
    let position = find_identity(&holds, &temporal_identity("holds"));
    holds["semantic_graph"]["nodes"][position]["body"]["operation"]["member"] = member.clone();
    settle(&mut holds);
    expect(
        "holds",
        &holds,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(position, "/body/operation/member"),
    );

    let (not, root) = with_root(BOUNDED, "not", member.clone(), None, 1);
    expect(
        "not",
        &not,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(root, "/body/operation/member"),
    );

    let mut clause_member = v2_all_families();
    let clause_at = clause(&clause_member);
    clause_member["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["member"] = member;
    settle(&mut clause_member);
    expect(
        "clause",
        &clause_member,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body/operation/member"),
    );
}

// ---------------------------------------------------------------------------
// AC-98
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-98
#[trace("TC-048", "FR-038-AC-98")]
#[test]
fn tc_048_the_fairness_member_admits_when_closed_and_refuses_otherwise() {
    let order = model_key(ORDER_NODE);
    for (kind, granularity) in [("weak", "whole"), ("strong", "each")] {
        let mut member = fairness_member(&order, "scaled");
        member["fairness_kind"] = json!(kind);
        member["granularity"] = json!(granularity);
        let (package, _) = with_fairness(INFINITE, member);
        admitted(&format!("{kind}/{granularity}"), &package);
    }
    let mut medium = fairness_member(&order, "scaled");
    medium["fairness_kind"] = json!("medium");
    let mut part = fairness_member(&order, "scaled");
    part["granularity"] = json!("part");
    let mut nameless = fairness_member(&order, "scaled");
    nameless.as_object_mut().expect("member").remove("name");
    let mut extra = fairness_member(&order, "scaled");
    extra["extra"] = json!(1);
    let mut interval_kind = fairness_member(&order, "scaled");
    interval_kind["kind"] = json!("temporal_interval");
    for (case, member) in [
        ("medium", medium),
        ("part", part),
        ("no name", nameless),
        ("an extra member", extra),
        ("null", Value::Null),
        ("a temporal_interval kind", interval_kind),
        ("an interval member", closed("0", "3")),
    ] {
        let (package, fair) = with_fairness(INFINITE, member);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(fair, "/body/operation/member"),
        );
    }
}

// ---------------------------------------------------------------------------
// AC-100
// ---------------------------------------------------------------------------

/// The refusal `ill_typed`/`operator-ineligible` at the node.
fn misplaced(case: &str, package: &Value, position: usize) {
    expect(
        case,
        package,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &at_node(position, ""),
    );
}

/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_placed_application_in_another_form_refuses_at_its_node() {
    let holds_body = |package: &Value| {
        let holds = digest(package, find_identity(package, &temporal_identity("holds")));
        formula_body("holds", Value::Null, vec![refer(&holds)])
    };
    let fair_body = application(
        "temporal_fairness",
        &temporal_identity("fair"),
        fairness_member(&aaaa(), "Step"),
        vec![],
    );
    let cases: [(&str, &str, &str, Option<Value>); 3] = [
        (
            "a formula application in a function node",
            "function",
            "pure_function",
            None,
        ),
        (
            "a fair application in a formula node",
            "temporal",
            "formula",
            Some(fair_body.clone()),
        ),
        (
            "a formula application in a fairness node",
            "temporal",
            "fairness",
            None,
        ),
    ];
    for (case, tag, form, body) in cases {
        let mut package = v2_all_families();
        let body = body.unwrap_or_else(|| holds_body(&package));
        let position = push(&mut package, node_with_key(&fresh(), tag, form, body));
        settle(&mut package);
        misplaced(case, &package, position);
    }
}

/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_formula_fairness_or_clause_node_needs_its_own_application() {
    let literal = json!({"term": "literal", "type": node_id(&aaaa()),
                         "value_kind": "boolean", "value": true});
    let other_class = application("call", "quire.op.function.call", Value::Null, vec![]);
    for (tag, form) in [
        ("temporal", "formula"),
        ("temporal", "fairness"),
        ("temporal", "temporal_clause"),
    ] {
        for (case, body) in [
            (
                "an empty aggregate",
                json!({"term": "aggregate", "members": []}),
            ),
            ("a literal", literal.clone()),
            ("an application of another class", other_class.clone()),
        ] {
            let mut package = v2_all_families();
            let position = push(&mut package, node_with_key(&fresh(), tag, form, body));
            mint_ungrouped_structural_keys(&mut package);
            settle(&mut package);
            misplaced(&format!("{form} with {case}"), &package, position);
        }
    }
}

/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_reference_to_a_formula_or_fairness_node_refuses_where_it_may_not_stand() {
    // A reference to the formula node from a function node's body.
    let mut package = v2_all_families();
    let formula_at = find_identity(&package, &temporal_identity("eventually"));
    let formula_key = digest(&package, formula_at);
    let mut function = node_with_key(&fresh(), "function", "pure_function", refer(&formula_key));
    function["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    function["declaration"] = json!({"qualified_name": ["Example", "FormulaUser"]});
    let position = push(
        &mut package,
        owned_structural_node(function, source_owner(&fixture_source())),
    );
    settle(&mut package);
    misplaced("from a function body", &package, position);

    // From a clause's fairness argument.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    let formula_key = digest(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][4] =
        json!({"term": "aggregate", "members": [refer(&formula_key)]});
    settle(&mut package);
    misplaced("from the fairness argument", &package, clause_at);

    // From a `case` application's argument.
    let mut package = v2_all_families();
    let formula_key = digest(
        &package,
        find_identity(&package, &temporal_identity("eventually")),
    );
    let case = push(
        &mut package,
        node_with_key(
            &fresh(),
            "expression",
            "case",
            application(
                "case",
                "quire.op.control.case",
                Value::Null,
                vec![refer(&formula_key)],
            ),
        ),
    );
    settle(&mut package);
    misplaced("from a case argument", &package, case);

    // A reference to a fairness node from a clause's formula argument and
    // from a formula operand.
    let mut package = v2_all_families();
    let fair = push(&mut package, fairness(fairness_member(&aaaa(), "Step")));
    let fair_key = digest(&package, fair);
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][5] = refer(&fair_key);
    settle(&mut package);
    misplaced("fairness from the formula argument", &package, clause_at);

    let mut package = v2_all_families();
    let fair = push(&mut package, fairness(fairness_member(&aaaa(), "Step")));
    let fair_key = digest(&package, fair);
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([refer(&fair_key)]);
    settle(&mut package);
    misplaced("fairness from a formula operand", &package, root);
}

/// A `case` application stands only at the body root of an `expression`/`case`
/// node, and an `expression`/`case` node has no other body.
///
/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_case_application_stands_only_at_the_root_of_a_case_node() {
    let case_body = || application("case", "quire.op.control.case", Value::Null, vec![]);
    let mut package = v2_all_families();
    let position = push(
        &mut package,
        node_with_key(&fresh(), "function", "pure_function", case_body()),
    );
    settle(&mut package);
    misplaced(
        "a case application as the root of a function node",
        &package,
        position,
    );

    let mut package = v2_all_families();
    let call = find(&package, |node| {
        node["node_tag"] == "function" && node["body"]["term"] == "application"
    });
    package["semantic_graph"]["nodes"][call]["body"]["arguments"]
        .as_array_mut()
        .expect("arguments")
        .push(case_body());
    settle(&mut package);
    // A nested `case` is refused in strict wire validation at its own
    // `operator`, with the holder as locus; the fixture's call holds one argument already.
    expect(
        "a nested case application",
        &package,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &at_node(call, "/body/arguments/1/operator"),
    );

    // An `expression` node whose form contradicts its root's operator class is
    // `invalid_semantic_graph` at the node's body: a `case` node with another
    // body, and an `expression` node of another form whose root is a `case`
    // application.
    for (case, form, body) in [
        (
            "a case node with another body",
            "case",
            json!({"term": "aggregate", "members": []}),
        ),
        (
            "a call node with a case application as its root",
            "call",
            case_body(),
        ),
    ] {
        let mut package = v2_all_families();
        let position = push(
            &mut package,
            node_with_key(&fresh(), "expression", form, body),
        );
        mint_ungrouped_structural_keys(&mut package);
        settle(&mut package);
        let locus = digest(&package, position);
        // `invalid_semantic_graph` carries no cause.
        assert_eq!(
            refusal_of(case, &package),
            (
                (
                    Code::InvalidSemanticGraph,
                    None,
                    Some(at_node(position, "/body"))
                ),
                Some(locus)
            ),
            "{case}"
        );
    }

    // `case` placement is the operation step's: a temporal-step defect is
    // reported ahead of it whatever the digest order.
    for key in [LOWEST, HIGHEST] {
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        let case_position = push(
            &mut package,
            node_with_key(&fresh(), "function", "pure_function", case_body()),
        );
        settle(&mut package);
        assert_eq!(
            digest(&package, placement) < digest(&package, case_position),
            key == LOWEST,
            "the valid structural key must exercise this digest order",
        );
        misplaced(
            &format!("a temporal placement defect beside a case placement defect, {key}"),
            &package,
            placement,
        );
    }
}

// ---------------------------------------------------------------------------
// AC-170
// ---------------------------------------------------------------------------

/// Exhaustive words of the reader's typed refusal variants.
fn code_word(code: Code) -> &'static str {
    match code {
        Code::UnknownContractVersion => "unknown_contract_version",
        Code::MalformedWire => "malformed_wire",
        Code::DuplicateMember => "duplicate_member",
        Code::UnknownMember => "unknown_member",
        Code::NoncanonicalWire => "noncanonical_wire",
        Code::StaleDependency => "stale_dependency",
        Code::DigestDomainMismatch => "digest_domain_mismatch",
        Code::UnknownRequiredCapability => "unknown_required_capability",
        Code::InvalidSemanticGraph => "invalid_semantic_graph",
        Code::InvalidSourceMap => "invalid_source_map",
        Code::UnsupportedNodeTag => "unsupported_node_tag",
        Code::MissingDeclaration => "missing_declaration",
        Code::InvalidModelBinding => "invalid_model_binding",
        Code::InvalidPackage => "invalid_package",
        Code::IllTyped => "ill_typed",
        Code::AmbiguousDeclaration => "ambiguous_declaration",
        Code::MissingImport => "missing_import",
        Code::UnknownProfile => "unknown_profile",
        Code::UnsupportedConstruct => "unsupported_construct",
    }
}

fn cause_word(cause: Cause) -> &'static str {
    match cause {
        Cause::MissingName => "missing-name",
        Cause::MalformedDeclaration => "malformed-declaration",
        Cause::StaleNodeKey => "stale-node-key",
        Cause::UnknownOperation => "unknown-operation",
        Cause::OperationClassMismatch => "operation-class-mismatch",
        Cause::OperationLawMissing => "operation-law-missing",
        Cause::OperationLawMismatch => "operation-law-mismatch",
        Cause::OperationLawUnselected => "operation-law-unselected",
        Cause::OperationModeMismatch => "operation-mode-mismatch",
        Cause::OperationModeTypeMismatch => "operation-mode-type-mismatch",
        Cause::OperationMemberMismatch => "operation-member-mismatch",
        Cause::OperatorIneligible => "operator-ineligible",
        Cause::DeclarationNominalMismatch => "declaration-nominal-mismatch",
        Cause::AmbiguousName => "ambiguous-name",
        Cause::MissingSelection => "missing-selection",
        Cause::DigestDomainMismatch => "digest-domain-mismatch",
        Cause::ByteDigestMismatch => "byte-digest-mismatch",
        Cause::WrongModelSelection => "wrong-model-selection",
        Cause::ConflictingBinding => "conflicting-binding",
        Cause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        Cause::ConflictingDefinition => "conflicting-definition",
        Cause::InvalidValue => "invalid-value",
        Cause::DuplicateMember => "duplicate-member",
        Cause::TypeMismatch => "type-mismatch",
        Cause::UnsupportedSelection => "unsupported-selection",
        Cause::WrongSelectionRole => "wrong-selection-role",
        Cause::InexactInteger => "inexact-integer",
        Cause::InexactNumber => "inexact-number",
        Cause::ExpressionForm => "expression-form",
    }
}

/// Trace: FR-038-AC-170
#[trace("TC-048", "FR-038-AC-170")]
#[test]
fn tc_048_relationship_refusal_words_are_typed_and_diagnostics_stay_closed() {
    // The exhaustive mappings keep the public refusal variants explicit.
    assert_eq!(
        code_word(Code::UnsupportedConstruct),
        "unsupported_construct"
    );
    assert_eq!(cause_word(Cause::ExpressionForm), "expression-form");
    assert_eq!(code_word(Code::UnknownProfile), "unknown_profile");
    assert_eq!(cause_word(Cause::DuplicateMember), "duplicate-member");
    assert_eq!(cause_word(Cause::TypeMismatch), "type-mismatch");
    assert_eq!(
        cause_word(Cause::UnsupportedSelection),
        "unsupported-selection"
    );
    assert_eq!(
        cause_word(Cause::WrongSelectionRole),
        "wrong-selection-role"
    );
    // A diagnostics entry has its own closed cause vocabulary.
    let mut package = v2_all_families();
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "expression-form",
        "details": [], "loci": [],
    }]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

/// A `details` term that references a `temporal`/`formula` or `temporal`/`fairness`
/// node is refused `ill_typed`/`operator-ineligible` at the entry
/// (`/diagnostics/entries/{e}/details/{d}`); the entry is no node, so the refusal
/// carries no node key.
///
/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_details_reference_to_a_formula_or_fairness_node_refuses_at_the_entry() {
    let formula_key = digest(
        &v2_all_families(),
        find_identity(&v2_all_families(), &temporal_identity("eventually")),
    );
    let mut with_fairness_node = v2_all_families();
    let fair = push(
        &mut with_fairness_node,
        fairness(fairness_member(&aaaa(), "Step")),
    );
    settle(&mut with_fairness_node);
    let fair_key = digest(&with_fairness_node, fair);
    for (case, mut package, target) in [
        ("a formula node", v2_all_families(), formula_key),
        ("a fairness node", with_fairness_node, fair_key),
    ] {
        package["diagnostics"]["entries"] = json!([{
            "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
            "details": [{"term": "aggregate", "members": [refer(&target)]}], "loci": [],
        }]);
        // The refusal is at the entry's detail, wherever the reference stands in it.
        assert_eq!(
            refusal_of(case, &package),
            (
                (
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    Some("/diagnostics/entries/0/details/0".to_owned())
                ),
                None
            ),
            "{case}"
        );
    }
}

/// A diagnostics entry whose `code` is `unsupported_construct` reads, as QSpec's
/// schema allows: the diagnostics wire vocabulary keeps the word.
///
/// Tracing: TC-048, FR-038-AC-101
#[trace("TC-048", "FR-038-AC-101")]
#[test]
fn tc_048_a_diagnostics_entry_may_carry_the_unsupported_construct_code() {
    let mut package = v2_all_families();
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "unsupported_construct", "cause_tag": "invalid-value",
        "details": [], "loci": [],
    }]);
    settle(&mut package);
    let reader = admitted("unsupported_construct entry", &package);
    assert_eq!(
        serde_json::to_value(reader.diagnostics().entries[0].code).expect("code"),
        "unsupported_construct"
    );
}

// ---------------------------------------------------------------------------
// AC-102
// ---------------------------------------------------------------------------

/// Valid derived-key formula nodes with opposite digest order, each retaining
/// the same temporal placement defect. The salt lives only in a closed body.
static MISPLACED_FORMULAS: LazyLock<(Value, Value)> = LazyLock::new(|| {
    let mut candidates = (0..128)
        .map(|salt| {
            let body = json!({"term": "aggregate", "members": [{
                "term": "binding", "name": format!("salt{salt}"),
                "value": {"term": "literal", "type": node_id(&aaaa()),
                    "value_kind": "boolean", "value": true},
            }]});
            let key = structural_key("temporal", "formula", Some(&aaaa()), &body);
            (
                key.clone(),
                node_with_key(&key, "temporal", "formula", body),
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    (
        candidates.first().expect("candidate").1.clone(),
        candidates.last().expect("candidate").1.clone(),
    )
});

/// A formula node with a placement defect and a valid low or high key.
fn misplaced_formula(key: &str) -> Value {
    if key == LOWEST {
        MISPLACED_FORMULAS.0.clone()
    } else {
        assert_eq!(key, HIGHEST, "placement fixture key selection");
        MISPLACED_FORMULAS.1.clone()
    }
}

/// A function node whose application has an unknown identity: an operation
/// defect, keyed by its own preimage.
fn unknown_identity_call() -> Value {
    node_with_key(
        &fresh(),
        "function",
        "pure_function",
        application(
            "call",
            "quire.op.function.not-catalogued",
            Value::Null,
            vec![],
        ),
    )
}

const LOWEST: &str = "0000000000000000000000000000000000000000000000000000000000000001";
const HIGHEST: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_placement_defect_is_reported_ahead_of_every_other_defect() {
    for key in [LOWEST, HIGHEST] {
        // Beside an operation defect, whatever the digest order.
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        let operation = push(&mut package, unknown_identity_call());
        settle(&mut package);
        assert_eq!(
            digest(&package, placement) < digest(&package, operation),
            key == LOWEST,
            "the valid structural key must exercise this digest order",
        );
        misplaced(
            &format!("beside an operation defect, {key}"),
            &package,
            placement,
        );

        // Beside a clause defect (an unknown profile), whatever the digest order.
        let mut package = v2_all_families();
        let placement = push(&mut package, misplaced_formula(key));
        select_profile(&mut package, "quire.fixture.temporal-profile/v1");
        settle(&mut package);
        misplaced(
            &format!("beside a clause defect, {key}"),
            &package,
            placement,
        );
    }
    // Two placement defects: the lower digest.
    let mut package = v2_all_families();
    let low = push(&mut package, misplaced_formula(LOWEST));
    let high = push(&mut package, misplaced_formula(HIGHEST));
    settle(&mut package);
    assert!(digest(&package, low) < digest(&package, high));
    misplaced("two placement defects", &package, low);
}

/// A `function` node whose application has an unknown identity, and one whose
/// `operator` differs from its entry's class (`quire.op.control.case` under
/// `unary`), each salted so its digest can be searched.
fn operation_defects(salt: u32) -> [(&'static str, Value); 2] {
    let salted = |operator: &str, identity: &str| {
        node_with_key(
            &fresh(),
            "function",
            "pure_function",
            application(
                operator,
                identity,
                Value::Null,
                vec![json!({"term": "literal", "type": node_id(&aaaa()),
                            "value_kind": "text", "value": format!("s{salt}")})],
            ),
        )
    };
    [
        (
            "an unknown identity",
            salted("call", "quire.op.function.not-catalogued"),
        ),
        ("a class mismatch", salted("unary", "quire.op.control.case")),
    ]
}

/// The AC's own placement defect, a `temporal_formula` application as the body
/// root of a `function` node, beside each operation defect whose digest is
/// lower and higher than its own, in both orders.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_formula_application_in_a_function_node_precedes_operation_defects() {
    for which in 0..2 {
        for placement_is_lower in [true, false] {
            let mut found = false;
            for salt in 0..256 {
                let mut package = v2_all_families();
                let placement = push(
                    &mut package,
                    node_with_key(
                        &fresh(),
                        "function",
                        "pure_function",
                        formula_body(
                            "true",
                            Value::Null,
                            vec![json!({"term": "literal", "type": node_id(&aaaa()),
                                        "value_kind": "text", "value": format!("p{salt}")})],
                        ),
                    ),
                );
                let [unknown, mismatch] = operation_defects(salt);
                let (case, node) = if which == 0 { unknown } else { mismatch };
                let operation = push(&mut package, node);
                settle(&mut package);
                if (digest(&package, placement) < digest(&package, operation)) != placement_is_lower
                {
                    continue;
                }
                found = true;
                misplaced(
                    &format!("{case}, placement lower: {placement_is_lower}"),
                    &package,
                    placement,
                );
                break;
            }
            assert!(found, "a salt orders the two nodes as the case needs");
        }
    }
}

/// Two defects of one clause, the later stage's defect reported only once the
/// earlier one is cured: `over` before fairness resolution, and fairness
/// resolution before profile fit (the bounded profile also refuses the
/// non-empty fairness argument and the `null` interval, both fit defects).
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_within_one_clause_the_stages_run_in_order() {
    let build = |profile: &str, over_defect: bool| {
        let (mut package, _) =
            with_fairness(profile, fairness_member(&model_key(ORDER_NODE), "missing"));
        let root = find_identity(&package, &temporal_identity("eventually"));
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
        let clause_at = clause(&package);
        if over_defect {
            package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
        }
        settle(&mut package);
        (package, clause_at)
    };
    let (package, clause_at) = build(INFINITE, true);
    expect_located(
        "over before fairness resolution",
        &package,
        (
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &aaaa(),
    );
    let (package, _) = build(BOUNDED, false);
    let fair = find_form(&package, "temporal", "fairness");
    expect_located(
        "fairness resolution before profile fit",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingName,
            &at_node(fair, "/body/operation/member/name"),
        ),
        &model_key(ORDER_NODE),
    );
}

/// The interval bounds are the last stage: `{3, 0}` under a bounded profile in
/// a clause whose profile fit also fails refuses the fit defect first, in both
/// operand orders of the two operators.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_profile_fit_precedes_bounds_whichever_operator_comes_first() {
    for fit_first in [true, false] {
        let mut package = v2_all_families();
        select_profile(&mut package, BOUNDED);
        let holds = digest(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let fit = push(
            &mut package,
            formula("always", unbounded(), vec![refer(&holds)]),
        );
        let bounds = push(
            &mut package,
            formula("once", closed("3", "0"), vec![refer(&holds)]),
        );
        let (first, second) = if fit_first {
            (fit, bounds)
        } else {
            (bounds, fit)
        };
        let operands = vec![
            refer(&digest(&package, first)),
            refer(&digest(&package, second)),
        ];
        let root = find_identity(&package, &temporal_identity("eventually"));
        package["semantic_graph"]["nodes"][root]["body"] =
            formula_body("and", Value::Null, operands);
        settle(&mut package);
        expect(
            &format!("fit first: {fit_first}"),
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(fit, "/body"),
        );
    }
}

/// A formula node no clause reaches is checked for `lower > upper` too, in the
/// sweep after every clause (an IR reading, AC-97); `{0, 3}` there admits.
///
/// Tracing: TC-048, FR-038-AC-97
#[trace("TC-048", "FR-038-AC-97")]
#[test]
fn tc_048_an_unreached_formula_node_with_lower_above_upper_is_refused() {
    for (member, refused) in [(closed("0", "3"), false), (closed("3", "0"), true)] {
        let mut package = v2_all_families();
        let holds = digest(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let unreached = push(&mut package, formula("once", member, vec![refer(&holds)]));
        settle(&mut package);
        if refused {
            expect(
                "an unreached {3, 0}",
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(unreached, "/body"),
            );
        } else {
            admitted("an unreached {0, 3}", &package);
        }
    }
}

/// A second clause named `name`, over its own formula tree (`eventually` with
/// `member` over a `holds` of its own) and the fixture's parameter, with the
/// `over` defect of naming a `scalar_type` when `over_defect`. Its position.
fn second_clause(package: &mut Value, name: &str, member: Value, over_defect: bool) -> usize {
    let first = clause(package);
    let mut holds_body =
        nodes(package)[find_identity(package, &temporal_identity("holds"))]["body"].clone();
    // A different operand, so the two `holds` nodes are not one node.
    holds_body["arguments"][0]["value"] = json!(false);
    let holds = push(
        package,
        node_with_key(&fresh(), "temporal", "formula", holds_body),
    );
    let operand = refer(&digest(package, holds));
    let root = push(package, formula("eventually", member, vec![operand]));
    let mut body = nodes(package)[first]["body"].clone();
    body["arguments"][1]["value"] = json!(name);
    body["arguments"][5] = refer(&digest(package, root));
    if over_defect {
        body["arguments"][0] = refer(&aaaa());
    }
    push(
        package,
        node_with_key(&fresh(), "temporal", "temporal_clause", body),
    )
}

/// Two clauses: the lower-digest clause's later-stage defect (profile fit) is
/// reported ahead of the higher-digest clause's earlier-stage defect (`over`),
/// in both digest orders; found by varying the second clause's name.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_lower_digest_clause_is_taken_through_every_stage_before_the_next() {
    for fit_clause_is_lower in [true, false] {
        let mut found = false;
        for salt in 0..64 {
            // The fixture's own clause carries the fit defect (a null interval
            // under the bounded profile); the added one the `over` defect.
            let mut package = v2_all_families();
            select_profile(&mut package, BOUNDED);
            let root = find_identity(&package, &temporal_identity("eventually"));
            package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
            let added = second_clause(
                &mut package,
                &format!("other{salt}"),
                closed("0", "3"),
                true,
            );
            // Both names vary, so the two digests are free of each other.
            let own = clause(&package);
            package["semantic_graph"]["nodes"][own]["body"]["arguments"][1]["value"] =
                json!(format!("own{salt}"));
            settle(&mut package);
            let own = clause(&package);
            if (digest(&package, own) < digest(&package, added)) != fit_clause_is_lower {
                continue;
            }
            found = true;
            if fit_clause_is_lower {
                // The lower-digest clause's later-stage defect comes first.
                expect(
                    "the fit defect of the lower clause",
                    &package,
                    Code::InvalidPackage,
                    Cause::OperationMemberMismatch,
                    &at_node(root, "/body"),
                );
            } else {
                expect_located(
                    "the over defect of the lower clause",
                    &package,
                    (
                        Code::InvalidModelBinding,
                        Cause::MalformedDeclaration,
                        &at_node(added, "/body/arguments/0"),
                    ),
                    &aaaa(),
                );
            }
            break;
        }
        assert!(found, "a salt orders the two clauses as the case needs");
    }
}

/// A member whose shape the operation step refuses is skipped by the temporal
/// step, whatever the profile: `{1.5, 0}` and a wrong-kind member refuse
/// `operation-member-mismatch`, never `invalid-value`.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_the_temporal_step_skips_a_member_the_operation_step_refuses() {
    for profile in [BOUNDED, INFINITE] {
        // A wrong-kind member is skipped by the temporal step and refused at
        // `operation.member`; a `null` member at the application; a bound outside
        // the pattern never reaches the step, being `invalid-value` at the bound
        // in strict wire validation (`flat_wire::check_interval_bounds`), never
        // `operation-member-mismatch`.
        for (case, member, cause, tail) in [
            (
                "a fairness member",
                fairness_member(&aaaa(), "Step"),
                Cause::OperationMemberMismatch,
                "/body/operation/member",
            ),
            ("null", Value::Null, Cause::OperationMemberMismatch, "/body"),
            (
                "{1.5, 0}",
                closed("1.5", "0"),
                Cause::InvalidValue,
                "/body/operation/member/interval/lower",
            ),
        ] {
            let (package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                cause,
                &at_node(root, tail),
            );
        }
    }
}

/// A negative bound is refused in strict wire validation
/// (`flat_wire::check_interval_bounds`): before placement
/// and every temporal step, so also beside a placement defect at a lower-digest
/// node and under a profile whose fit it would also fail.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_a_negative_bound_is_refused_before_any_temporal_step() {
    for profile in [BOUNDED, INFINITE] {
        for (case, member, bound) in [
            ("{0, -2}", closed("0", "-2"), "upper"),
            ("{-1, -3}", closed("-1", "-3"), "lower"),
            ("{-1, null}", lower_bounded("-1"), "lower"),
        ] {
            let (mut package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
            // Beside a placement defect at a lower-digest node.
            push(&mut package, misplaced_formula(LOWEST));
            settle(&mut package);
            expect(
                &format!("{case} beside a placement defect"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
    }
}

/// `lower > upper` is the bounds stage, after profile fit.
///
/// Tracing: TC-048, FR-038-AC-102
#[trace("TC-048", "FR-038-AC-102")]
#[test]
fn tc_048_lower_above_upper_is_refused_at_the_application_after_profile_fit() {
    let (package, root) = with_root(BOUNDED, "eventually", closed("3", "0"), None, 1);
    expect(
        "lower above upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body"),
    );
    // With the profile fit also failing (a non-empty fairness argument under a
    // bounded profile), the fit defect is reported first.
    let (mut package, _) =
        with_fairness(BOUNDED, fairness_member(&model_key(ORDER_NODE), "scaled"));
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = closed("3", "0");
    settle(&mut package);
    let clause_at = clause(&package);
    expect(
        "fit before bounds",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(clause_at, "/body"),
    );
}

// ---------------------------------------------------------------------------
// AC-103
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-103
#[trace("TC-048", "FR-038-AC-103", "FR-038-AC-155")]
#[test]
fn tc_048_a_clause_over_must_be_a_declared_parameter() {
    admitted("a parameter dependency", &v2_all_families());
    // A declared dependency that is a `scalar_type` node.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
    settle(&mut package);
    // The path is on the argument and the locus is the target's key.
    expect_located(
        "a scalar_type over",
        &package,
        (
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &aaaa(),
    );
    // A target that is no node: no declared dependency, and no target key to
    // be the locus, so the clause's key is.
    let mut package = v2_all_families();
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&"c1".repeat(32));
    settle(&mut package);
    expect_located(
        "an absent over",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingName,
            &at_node(clause_at, "/body/arguments/0"),
        ),
        &digest(&package, clause_at),
    );
}

/// Tracing: TC-048, FR-038-AC-103
#[trace("TC-048", "FR-038-AC-103")]
#[test]
fn tc_048_a_fairness_member_resolves_on_its_declaring_model_node() {
    let order = model_key(ORDER_NODE);
    // Own operation, and an operation the node only inherits.
    for (case, declaration, name) in [
        ("an own operation", model_key(ORDER_NODE), "scaled"),
        ("an inherited operation", model_key(SUB_NODE), "scaled"),
    ] {
        let (package, _) = with_fairness(INFINITE, fairness_member(&declaration, name));
        admitted(case, &package);
    }
    // Each row of FR-038's "Path and locus" table: a `name` refusal is on
    // `.../member/name` with the `declaration` target as locus; a refusal of the
    // `declaration` is on `.../member/declaration`, with the fairness node as
    // locus unless the target is what is malformed.
    struct Row {
        case: &'static str,
        declaration: String,
        name: &'static str,
        code: Code,
        cause: Cause,
        member: &'static str,
        locus_is_target: bool,
    }
    let rows = [
        Row {
            case: "a missing name",
            declaration: order.clone(),
            name: "nope",
            code: Code::MissingDeclaration,
            cause: Cause::MissingName,
            member: "name",
            locus_is_target: true,
        },
        Row {
            // A target that names no node is no `model`/`object_type` node
            // (merged FR-370 step 1), located at the key as named.
            case: "an absent declaration",
            declaration: "c2".repeat(32),
            name: "scaled",
            code: Code::InvalidModelBinding,
            cause: Cause::MalformedDeclaration,
            member: "declaration",
            locus_is_target: true,
        },
        Row {
            case: "two matches",
            declaration: model_key(BOTH_NODE),
            name: "act",
            code: Code::AmbiguousDeclaration,
            cause: Cause::AmbiguousName,
            member: "name",
            locus_is_target: true,
        },
    ];
    for row in rows {
        let (package, fair) = with_fairness(INFINITE, fairness_member(&row.declaration, row.name));
        let locus = if row.locus_is_target {
            row.declaration.clone()
        } else {
            digest(&package, fair)
        };
        expect_located(
            row.case,
            &package,
            (
                row.code,
                row.cause,
                &at_node(fair, &format!("/body/operation/member/{}", row.member)),
            ),
            &locus,
        );
    }
    // An unreachable node with an unselected owner is now refused before
    // the fairness member reaches its declaration target (FR-038-AC-155).
    let unselected = model_key_in(OTHER_PACKAGE, ORDER_NODE);
    let (mut package, _) = with_fairness(INFINITE, fairness_member(&unselected, "scaled"));
    let mut node = node_with_key(
        &unselected,
        "model",
        "object_type",
        json!({"term": "aggregate", "members": []}),
    );
    node["semantic_type"] = node_id(&unselected);
    node["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
    let position = push(
        &mut package,
        owned_structural_node(node, model_owner(OTHER_PACKAGE, ORDER_NODE)),
    );
    settle(&mut package);
    expect_located(
        "an unrecovered owner",
        &package,
        (
            Code::MissingDeclaration,
            Cause::MissingSelection,
            &at_node(position, "/node_id"),
        ),
        &unselected,
    );
    // A declaration that is no `model`/`object_type` declaration node is
    // `malformed-declaration` at the member's `declaration` target: a
    // `scalar_type` and a `model`/`value_type` node.
    let value_type = fresh();
    for (case, declaration) in [
        ("a scalar_type declaration", aaaa()),
        ("a model/value_type declaration", value_type.clone()),
    ] {
        let (mut package, fair) = with_fairness(INFINITE, fairness_member(&declaration, "scaled"));
        push(
            &mut package,
            node_with_key(
                &value_type,
                "model",
                "value_type",
                json!({"term": "aggregate", "members": []}),
            ),
        );
        settle(&mut package);
        // The path sits on the referencing member and the locus is the target
        // (FR-342 step 1): the declaration's own key.
        expect_located(
            case,
            &package,
            (
                Code::InvalidModelBinding,
                Cause::MalformedDeclaration,
                &at_node(fair, "/body/operation/member/declaration"),
            ),
            &declaration,
        );
    }
}

// ---------------------------------------------------------------------------
// AC-104
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-104
#[trace("TC-048", "FR-038-AC-104")]
#[test]
fn tc_048_the_profile_fit_admits_each_interval_shape_per_profile() {
    let shapes = [
        ("closed", closed("0", "3")),
        ("lower-bounded", lower_bounded("2")),
        ("null", unbounded()),
    ];
    for (profile, bounded, finite_upper) in [
        (BOUNDED, true, false),
        (
            "quire.temporal.fixed-sample.false-extension/v1",
            true,
            false,
        ),
        (
            "quire.temporal.timestamped-event.finite-window/v1",
            true,
            false,
        ),
        (INFINITE, false, false),
        (TIMED, false, true),
    ] {
        for (case, member) in &shapes {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            let refuses = match *case {
                // Under `timed/v1` only `null` admits: an integer-form interval
                // is not the timed form (merged FR-370-AC-3).
                "closed" => finite_upper,
                "lower-bounded" => bounded || finite_upper,
                _ => bounded,
            };
            if refuses {
                expect(
                    &format!("{case} under {profile}"),
                    &package,
                    Code::InvalidPackage,
                    Cause::OperationMemberMismatch,
                    &at_node(root, "/body"),
                );
            } else {
                admitted(&format!("{case} under {profile}"), &package);
            }
        }
        // A non-empty fairness argument: refused only by the bounded profiles,
        // at the clause's application; an empty one admits.
        let (mut package, _) =
            with_fairness(profile, fairness_member(&model_key(ORDER_NODE), "scaled"));
        if finite_upper {
            // The timed profile admits a `null` interval, which the fairness
            // case then holds.
            let root = find_identity(&package, &temporal_identity("eventually"));
            package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
            settle(&mut package);
        }
        if bounded {
            expect(
                &format!("fairness under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(clause(&package), "/body"),
            );
        } else {
            admitted(&format!("fairness under {profile}"), &package);
        }
    }
}

/// The profile fit reads only the formula nodes reachable from the clause: a
/// second clause whose tree shares no node with the first is fitted against its
/// own profile law and tree, not the first's.
///
/// Tracing: TC-048, FR-038-AC-104
#[trace("TC-048", "FR-038-AC-104")]
#[test]
fn tc_048_the_profile_fit_reads_only_the_formula_nodes_reachable_from_the_clause() {
    let mut package = v2_all_families();
    select_profile(&mut package, BOUNDED);
    // An unreachable formula node with a null interval fits no clause.
    let holds = digest(
        &package,
        find_identity(&package, &temporal_identity("holds")),
    );
    push(
        &mut package,
        formula("always", unbounded(), vec![refer(&holds)]),
    );
    second_clause(&mut package, "second", closed("1", "2"), false);
    settle(&mut package);
    admitted("an unreachable null interval and a second tree", &package);
}

// ---------------------------------------------------------------------------
// AC-105
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-105
#[trace("TC-048", "FR-038-AC-105")]
#[test]
fn tc_048_the_temporal_operands_are_checked_at_the_operation_step() {
    let boolean = family_key("a6a6");
    let holds =
        |package: &Value| digest(package, find_identity(package, &temporal_identity("holds")));
    let ineligible = |case: &str, package: &Value, root: usize, tail: &str| {
        expect(
            case,
            package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &at_node(root, tail),
        );
    };
    // `holds` over a reference to a formula node refuses; over a Boolean-family
    // reference it admits.
    let base = v2_all_families();
    let formula_key = holds(&base);
    let (package, root) = with_root(
        BOUNDED,
        "holds",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("holds over a formula", &package, root, "/body/arguments/0");
    let (package, _) = with_root(
        BOUNDED,
        "holds",
        Value::Null,
        Some(vec![refer(&boolean)]),
        0,
    );
    admitted("holds over a Boolean reference", &package);
    // Wrong operand counts.
    let (package, root) = with_root(
        BOUNDED,
        "until",
        closed("0", "3"),
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("until with one argument", &package, root, "/body/arguments");
    let (package, root) = with_root(
        BOUNDED,
        "not",
        Value::Null,
        Some(vec![refer(&formula_key), refer(&formula_key)]),
        0,
    );
    ineligible("not with two", &package, root, "/body/arguments");
    let (package, root) = with_root(
        BOUNDED,
        "true",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    ineligible("true with one", &package, root, "/body/arguments");
    // `and` over a Boolean-family reference refuses; `not` over a formula
    // reference admits.
    let (package, root) = with_root(
        BOUNDED,
        "and",
        Value::Null,
        Some(vec![refer(&boolean), refer(&formula_key)]),
        0,
    );
    ineligible("and over a Boolean", &package, root, "/body/arguments/0");
    let (package, _) = with_root(
        BOUNDED,
        "not",
        Value::Null,
        Some(vec![refer(&formula_key)]),
        0,
    );
    admitted("not over a formula", &package);
}

// ---------------------------------------------------------------------------
// AC-108
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_each_of_the_five_fr_250_profiles_admits_as_the_clause_profile() {
    for profile in FIVE_PROFILES {
        let mut package = v2_all_families();
        select_profile(&mut package, profile);
        if profile == TIMED {
            // Under `timed/v1` only a `null` interval admits (merged FR-370-AC-3):
            // the fixture's integer-form `{0, 3}` does not (AC-104).
            let root = find_identity(&package, &temporal_identity("eventually"));
            package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
        }
        settle(&mut package);
        admitted(profile, &package);
    }
}

/// The fixture with the clause's own law naming `identity` (the lock still
/// selects the fixture's profile), and `lock_row` added to the lock's profile
/// selections when given. The package and the clause's position.
fn clause_law(identity: &str, lock_row: Option<(&str, &str)>) -> (Value, usize) {
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!(identity);
    if let Some((role, definition)) = lock_row {
        package["lock"]["profile_selections"]
            .as_array_mut()
            .expect("selections")
            .push(json!({"role": role,
                         "definition": {"authority": "agent-ix", "identity": definition}}));
    }
    settle(&mut package);
    (package, clause_at)
}

/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_an_identity_outside_the_five_is_an_unknown_profile() {
    let unknown = |case: &str, identity: &str, row: Option<(&str, &str)>, cause: Cause| {
        let (package, clause_at) = clause_law(identity, row);
        expect(
            case,
            &package,
            Code::UnknownProfile,
            cause,
            &at_node(clause_at, "/body/operation/laws/0/definition"),
        );
    };
    unknown(
        "the fixture profile",
        "quire.fixture.temporal-profile/v1",
        None,
        Cause::UnsupportedSelection,
    );
    unknown("an empty identity", "", None, Cause::UnsupportedSelection);
    unknown(
        "a differently spelled identity",
        "quire.temporal.infinite-trace/v2",
        None,
        Cause::UnsupportedSelection,
    );
    // The same identities, selected by the package's own lock under another
    // role, are the wrong role; selected under none, unsupported.
    unknown(
        "composed, selected as a binding contract",
        "quire.package.composed/v1",
        Some(("binding_contract", "quire.package.composed/v1")),
        Cause::WrongSelectionRole,
    );
    unknown(
        "complete, selected as a protocol profile",
        "quire.protocol.complete/v1",
        Some(("protocol_profile", "quire.protocol.complete/v1")),
        Cause::WrongSelectionRole,
    );
    unknown(
        "composed, selected nowhere else",
        "quire.package.composed/v1",
        None,
        Cause::UnsupportedSelection,
    );
    unknown(
        "complete, selected nowhere else",
        "quire.protocol.complete/v1",
        None,
        Cause::UnsupportedSelection,
    );
}

/// The cause rule matches the same `{authority, identity}`: a law naming the
/// lock's own `edition` definition is the wrong role, and the same identity
/// under another authority, in the edition row or in a profile row, is not.
///
/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_the_cause_matches_the_edition_row_and_the_authority_of_a_definition() {
    let edition = |package: &Value| {
        package["lock"]["edition"]["definition"]["identity"]
            .as_str()
            .expect("the edition identity")
            .to_owned()
    };
    let at_law = |clause_at: usize| at_node(clause_at, "/body/operation/laws/0/definition");

    // The law names the edition definition: the wrong role.
    let (package, clause_at) = clause_law(&edition(&v2_all_families()), None);
    expect(
        "the edition definition",
        &package,
        Code::UnknownProfile,
        Cause::WrongSelectionRole,
        &at_law(clause_at),
    );

    // The edition row carries the identity under another authority: not the same
    // definition, so not the wrong role.
    let (mut package, clause_at) = clause_law(&edition(&v2_all_families()), None);
    package["lock"]["edition"]["definition"]["authority"] = json!("other");
    settle(&mut package);
    expect(
        "the edition identity under another authority",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_law(clause_at),
    );

    // A profile row of another role carries the identity under another authority.
    let (mut package, clause_at) = clause_law("quire.package.composed/v1", None);
    package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("selections")
        .push(json!({"role": "binding_contract",
                     "definition": {"authority": "other",
                                    "identity": "quire.package.composed/v1"}}));
    settle(&mut package);
    expect(
        "a profile row under another authority",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_law(clause_at),
    );
}

/// An unknown profile is read first among a clause's checks and is never read
/// as bounded: a `null` interval under it refuses `unknown_profile`, ahead of
/// the clause's own `over` defect.
///
/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_an_unknown_profile_is_checked_first_and_is_not_read_as_bounded() {
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!("quire.fixture.temporal-profile/v1");
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0] = refer(&aaaa());
    settle(&mut package);
    expect(
        "unknown profile with a null interval and an over defect",
        &package,
        Code::UnknownProfile,
        Cause::UnsupportedSelection,
        &at_node(clause_at, "/body/operation/laws/0/definition"),
    );
}

/// A clause whose `laws` is not exactly one `temporal_profile` law skips the
/// profile check and its profile fit: its law defect refuses at the operation
/// step, at the first law past the catalogued one (`operation-law-mismatch`,
/// `laws/1`) or at the list (`operation-law-missing`).
///
/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108")]
#[test]
fn tc_048_a_clause_without_exactly_one_profile_law_skips_the_profile_check() {
    let laws_at =
        |clause_at: usize, tail: &str| at_node(clause_at, &format!("/body/operation/laws{tail}"));

    // An unknown first law and the fixture's own law: not `unknown_profile`.
    let mut package = v2_all_families();
    let clause_at = clause(&package);
    let own = package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"][0].clone();
    let mut unknown = own.clone();
    unknown["definition"]["identity"] = json!("quire.fixture.temporal-profile/v1");
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] =
        json!([unknown, own.clone()]);
    settle(&mut package);
    expect(
        "an unknown first law and a second law",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &laws_at(clause_at, "/1"),
    );

    // A known profile twice and a `null` interval, which the bounded profile
    // would refuse as a profile fit: not `operation-member-mismatch`.
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] =
        json!([own.clone(), own]);
    settle(&mut package);
    expect(
        "the bounded profile twice and a null interval",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &laws_at(clause_at, "/1"),
    );

    // No law at all, under a `null` interval the lock's bounded profile would
    // refuse as a profile fit: the lawless clause has no profile to fit.
    let mut package = v2_all_families();
    let root = find_identity(&package, &temporal_identity("eventually"));
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = unbounded();
    let clause_at = clause(&package);
    package["semantic_graph"]["nodes"][clause_at]["body"]["operation"]["laws"] = json!([]);
    settle(&mut package);
    expect(
        "no law",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMissing,
        &laws_at(clause_at, ""),
    );
}

/// Ahead of any defect of a higher-digest clause: the lower-digest clause is
/// reported, whether it holds the unknown profile or an `over` defect.
///
/// Tracing: TC-048, FR-038-AC-108
#[trace("TC-048", "FR-038-AC-108", "FR-038-AC-102")]
#[test]
fn tc_048_the_lower_digest_clause_is_read_first_whatever_its_defect() {
    for own_is_unknown in [true, false] {
        for lower_is_own in [true, false] {
            let mut found = false;
            for salt in 0..64 {
                let mut package = v2_all_families();
                // The added clause holds the `over` defect when the fixture's
                // own clause holds the unknown profile, and the reverse.
                let added = second_clause(
                    &mut package,
                    &format!("other{salt}"),
                    closed("0", "3"),
                    own_is_unknown,
                );
                // Both names vary, so the two digests are free of each other.
                let own = clause(&package);
                package["semantic_graph"]["nodes"][own]["body"]["arguments"][1]["value"] =
                    json!(format!("own{salt}"));
                if own_is_unknown {
                    // The fixture's clause alone selects an unknown profile.
                    let own = clause(&package);
                    package["semantic_graph"]["nodes"][own]["body"]["operation"]["laws"][0]
                        ["definition"]["identity"] = json!("quire.fixture.temporal-profile/v1");
                } else {
                    let own = clause(&package);
                    package["semantic_graph"]["nodes"][added]["body"]["operation"]["laws"][0]
                        ["definition"]["identity"] = json!("quire.fixture.temporal-profile/v1");
                    package["semantic_graph"]["nodes"][own]["body"]["arguments"][0] =
                        refer(&aaaa());
                }
                settle(&mut package);
                let own = clause(&package);
                if (digest(&package, own) < digest(&package, added)) != lower_is_own {
                    continue;
                }
                found = true;
                let (lower, lower_is_unknown) = if lower_is_own {
                    (own, own_is_unknown)
                } else {
                    (added, !own_is_unknown)
                };
                let expected = if lower_is_unknown {
                    (
                        Code::UnknownProfile,
                        Cause::UnsupportedSelection,
                        "/body/operation/laws/0/definition",
                    )
                } else {
                    (
                        Code::InvalidModelBinding,
                        Cause::MalformedDeclaration,
                        "/body/arguments/0",
                    )
                };
                // `unknown_profile` has the clause's key as locus; a malformed
                // `over` has its target's, a `scalar_type`.
                let locus = if lower_is_unknown {
                    digest(&package, lower)
                } else {
                    aaaa()
                };
                expect_located(
                    &format!("own unknown {own_is_unknown}, own lower {lower_is_own}"),
                    &package,
                    (expected.0, expected.1, &at_node(lower, expected.2)),
                    &locus,
                );
                break;
            }
            assert!(
                found,
                "a salt orders the two clauses as the case needs: {own_is_unknown} {lower_is_own}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-119 through AC-122: the timed interval form
// ---------------------------------------------------------------------------

fn rational(numerator: &str, denominator: &str) -> Value {
    json!({"numerator": numerator, "denominator": denominator})
}

/// The whole number `n` as a rational.
fn whole(n: &str) -> Value {
    rational(n, "1")
}

/// A timed-form `temporal_interval` member.
fn timed(lower: Value, upper: Value, lower_end: &str, upper_end: &str) -> Value {
    json!({"kind": "temporal_interval", "interval": {
        "lower": lower, "upper": upper, "lower_end": lower_end, "upper_end": upper_end}})
}

/// A timed-form member with both ends closed.
fn timed_closed(lower: Value, upper: Value) -> Value {
    timed(lower, upper, "closed", "closed")
}

/// Asserts an `invalid_semantic_graph` refusal (it names no cause) at `path`,
/// located at the key of the node the pointer is on.
fn expect_graph(case: &str, package: &Value, path: &str) {
    let position: usize = path
        .strip_prefix("/semantic_graph/nodes/")
        .and_then(|rest| rest.split('/').next())
        .and_then(|node| node.parse().ok())
        .unwrap_or_else(|| panic!("{case}: {path} is on no node"));
    assert_eq!(
        refusal_of(case, package),
        (
            (Code::InvalidSemanticGraph, None, Some(path.to_owned())),
            Some(digest(package, position))
        ),
        "{case}"
    );
}

/// Tracing: TC-048, FR-038-AC-119
#[trace("TC-048", "FR-038-AC-119")]
#[test]
fn tc_048_the_timed_form_admits_under_the_timed_profile_on_each_interval_operator() {
    let end_pairs = [
        ("closed", "closed"),
        ("closed", "open"),
        ("open", "closed"),
        ("open", "open"),
    ];
    let mut admits = Vec::new();
    for (lower_end, upper_end) in end_pairs {
        admits.push((
            format!("[0, 3] with ends {lower_end}/{upper_end}"),
            timed(whole("0"), whole("3"), lower_end, upper_end),
        ));
    }
    admits.push((
        "lower 1/2".to_owned(),
        timed_closed(rational("1", "2"), whole("3")),
    ));
    admits.push((
        "a punctual [3, 3]".to_owned(),
        timed_closed(whole("3"), whole("3")),
    ));
    admits.push(("a null interval".to_owned(), unbounded()));
    for (short, arity) in INTERVAL_OPERATORS {
        for (case, member) in &admits {
            let (package, _) = with_root(TIMED, short, member.clone(), None, arity);
            admitted(&format!("{short} {case}"), &package);
        }
    }
}

/// Tracing: TC-048, FR-038-AC-120
#[trace("TC-048", "FR-038-AC-120")]
#[test]
fn tc_048_a_timed_bound_outside_the_rational_pattern_is_invalid_value_at_the_bound() {
    let cases = [
        (
            "lower numerator -1",
            timed_closed(rational("-1", "1"), whole("3")),
            "lower",
        ),
        (
            "lower numerator 01",
            timed_closed(rational("01", "1"), whole("3")),
            "lower",
        ),
        (
            "upper denominator 0",
            timed_closed(whole("0"), rational("3", "0")),
            "upper",
        ),
        (
            "upper denominator -2",
            timed_closed(whole("0"), rational("3", "-2")),
            "upper",
        ),
        (
            "lower a JSON integer",
            timed_closed(json!(0), whole("3")),
            "lower",
        ),
        (
            "lower the string 3",
            timed_closed(json!("3"), whole("3")),
            "lower",
        ),
        (
            "upper null with four members",
            timed_closed(whole("0"), Value::Null),
            "upper",
        ),
        (
            "lower numerator -1 and upper denominator 0",
            timed_closed(rational("-1", "1"), rational("3", "0")),
            "lower",
        ),
        (
            "a rational holding a third member",
            timed_closed(
                json!({"numerator": "0", "denominator": "1", "extra": 1}),
                whole("3"),
            ),
            "lower",
        ),
    ];
    for profile in FIVE_PROFILES {
        for (case, member, bound) in &cases {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, &format!("/body/operation/member/interval/{bound}")),
            );
        }
    }
    // On every interval operator, not only `eventually`.
    for (short, arity) in INTERVAL_OPERATORS {
        let member = timed_closed(rational("-1", "1"), whole("3"));
        let (package, root) = with_root(TIMED, short, member, None, arity);
        expect(
            short,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, "/body/operation/member/interval/lower"),
        );
    }
}

/// A pattern failure in any node is refused ahead of a non-reduced bound in any
/// node, whichever comes first in node order.
///
/// Tracing: TC-048, FR-038-AC-120
#[trace("TC-048", "FR-038-AC-120")]
#[test]
fn tc_048_a_pattern_failure_in_a_later_node_precedes_a_non_reduced_bound() {
    let non_reduced = timed_closed(rational("2", "4"), whole("3"));
    let pattern_failure = timed_closed(whole("0"), rational("3", "0"));
    for reduced_node_first in [true, false] {
        let (first, second) = if reduced_node_first {
            (non_reduced.clone(), pattern_failure.clone())
        } else {
            (pattern_failure.clone(), non_reduced.clone())
        };
        let (mut package, _) = with_root(INFINITE, "eventually", first, None, 1);
        let holds = digest(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let added = push(&mut package, formula("once", second, vec![refer(&holds)]));
        settle(&mut package);
        // The arrangement is by node position, first and later node.
        let root = find_identity(&package, &temporal_identity("eventually"));
        assert!(root < added, "the pushed node is the later one");
        let (at, bound) = if reduced_node_first {
            (added, "upper")
        } else {
            (root, "upper")
        };
        expect(
            &format!("non-reduced node first: {reduced_node_first}"),
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(at, &format!("/body/operation/member/interval/{bound}")),
        );
    }
}

/// The smallest work limit at which `package` is decided rather than
/// `incomplete`.
fn smallest_deciding_work(package: &Value) -> u64 {
    let incomplete = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        matches!(
            read_limited(package, limits),
            CheckedPackageV2ReadResult::Incomplete(_)
        )
    };
    let (mut low, mut high) = (0_u64, CheckedPackageReadLimits::bounded().work);
    assert!(incomplete(low), "a zero work limit is incomplete");
    assert!(!incomplete(high), "the bounded limit decides the package");
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        if incomplete(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    high
}

/// `package` read with `work`, which must be `incomplete` naming the work limit.
fn incomplete_work_path(package: &Value, work: u64) -> (u64, Option<String>) {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    match read_limited(package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, work);
            (
                incomplete.consumed,
                incomplete.path.map(|path| path.as_str().to_owned()),
            )
        }
        other => panic!("expected incomplete at work {work}, read {other:?}"),
    }
}

/// Tracing: TC-048, FR-038-AC-121
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_a_bound_not_in_lowest_terms_is_refused_at_the_bound_before_the_temporal_step() {
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("2", "4"), whole("3")),
        None,
        1,
    );
    expect_graph(
        "a non-reduced lower",
        &package,
        &at_node(root, "/body/operation/member/interval/lower"),
    );
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(whole("0"), rational("2", "4")),
        None,
        1,
    );
    expect_graph(
        "a non-reduced upper alone",
        &package,
        &at_node(root, "/body/operation/member/interval/upper"),
    );
    // A pattern failure at the other bound is refused first, `invalid-value`.
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("2", "4"), rational("-1", "1")),
        None,
        1,
    );
    expect(
        "non-reduced lower, negative upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body/operation/member/interval/upper"),
    );
    let (package, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(rational("-1", "1"), rational("2", "4")),
        None,
        1,
    );
    expect(
        "negative lower, non-reduced upper",
        &package,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &at_node(root, "/body/operation/member/interval/lower"),
    );

    // Beside a profile-fit defect (an integer-form interval under `timed/v1`)
    // in another node, the non-reduced bound is refused: the graph check is
    // ahead of the temporal step.
    let beside_fit_defect = |member: Value, fit_upper: u32| {
        let mut package = v2_all_families();
        select_profile(&mut package, TIMED);
        let holds = digest(
            &package,
            find_identity(&package, &temporal_identity("holds")),
        );
        let fit = push(
            &mut package,
            formula(
                "always",
                closed("0", &fit_upper.to_string()),
                vec![refer(&holds)],
            ),
        );
        let root = find_identity(&package, &temporal_identity("eventually"));
        let operand = vec![refer(&digest(&package, fit))];
        package["semantic_graph"]["nodes"][root]["body"] =
            formula_body("eventually", member, operand);
        settle(&mut package);
        (package, root, fit)
    };
    // Whichever of the two nodes has the lower digest: a multiple of 2/4 as the
    // bound and the fit node's upper bound move the two digests until the
    // arrangement holds, and the test asserts the arrangement it is about.
    for fit_is_lower in [true, false] {
        let mut found = false;
        for (multiple, fit_upper) in (1..16_u32).flat_map(|m| (1..16_u32).map(move |u| (m, u))) {
            let bound = rational(&(2 * multiple).to_string(), &(4 * multiple).to_string());
            let (package, root, fit) =
                beside_fit_defect(timed_closed(bound, whole("3")), fit_upper);
            if (digest(&package, fit) < digest(&package, root)) != fit_is_lower {
                continue;
            }
            found = true;
            expect_graph(
                &format!(
                    "a non-reduced bound beside a profile-fit defect, fit lower: {fit_is_lower}"
                ),
                &package,
                &at_node(root, "/body/operation/member/interval/lower"),
            );
            break;
        }
        assert!(found, "a multiple orders the two nodes as the case needs");
    }
    let (package, _, fit) = beside_fit_defect(timed_closed(whole("1"), whole("3")), 3);
    expect(
        "the same fit defect with reduced bounds",
        &package,
        Code::InvalidPackage,
        Cause::OperationMemberMismatch,
        &at_node(fit, "/body"),
    );
}

/// Tracing: TC-048, FR-038-AC-121
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_timed_bounds_compare_exactly_and_open_ends_refuse_equal_bounds() {
    let refuses = |case: &str, member: Value| {
        let (package, root) = with_root(TIMED, "eventually", member, None, 1);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, "/body"),
        );
    };
    let admits = |case: &str, member: Value| {
        let (package, _) = with_root(TIMED, "eventually", member, None, 1);
        admitted(case, &package);
    };
    refuses("(3, 3]", timed(whole("3"), whole("3"), "open", "closed"));
    refuses("[3, 3)", timed(whole("3"), whole("3"), "closed", "open"));
    refuses("(3, 3)", timed(whole("3"), whole("3"), "open", "open"));
    refuses("5/2 over 2", timed_closed(rational("5", "2"), whole("2")));
    admits("[3, 3]", timed_closed(whole("3"), whole("3")));
    admits(
        "1/2 to 2/3",
        timed_closed(rational("1", "2"), rational("2", "3")),
    );
    refuses(
        "2/3 to 1/2",
        timed_closed(rational("2", "3"), rational("1", "2")),
    );
    // Open ends do not matter once the bounds differ.
    admits(
        "(1/2, 2/3)",
        timed(rational("1", "2"), rational("2", "3"), "open", "open"),
    );
    refuses(
        "(2/3, 1/2)",
        timed(rational("2", "3"), rational("1", "2"), "open", "open"),
    );
    // Beyond 2^64: a float comparison rounds both numerators to 2^64 and
    // admits the first; a checked parse into 64 bits refuses the second.
    let above = rational("18446744073709551617", "3");
    let below = rational("18446744073709551616", "3");
    refuses(
        "2^64+1 over 2^64, both over 3",
        timed_closed(above.clone(), below.clone()),
    );
    admits("2^64 over 2^64+1, both over 3", timed_closed(below, above));
    // Different denominators cross-multiply: 7/3 against 5/2.
    admits(
        "7/3 to 5/2",
        timed_closed(rational("7", "3"), rational("5", "2")),
    );
    refuses(
        "5/2 to 7/3",
        timed_closed(rational("5", "2"), rational("7", "3")),
    );
}

/// The work limit covers the lowest-terms check and the exact bounds check: a
/// limit the work goes past is `incomplete` naming `work` at the node's body,
/// and not a refusal of the interval.
///
/// Tracing: TC-048, FR-038-AC-121
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_timed_bound_checks_are_charged_to_the_work_limit() {
    let (bounds, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(
            rational("18446744073709551617", "3"),
            rational("18446744073709551616", "3"),
        ),
        None,
        1,
    );
    let body = at_node(root, "/body");
    let needed = smallest_deciding_work(&bounds);
    let mut exact = CheckedPackageReadLimits::bounded();
    exact.work = needed;
    expect(
        "the exact work decides the refusal",
        &bounds,
        Code::InvalidPackage,
        Cause::InvalidValue,
        &body,
    );
    assert!(matches!(
        read_limited(&bounds, exact),
        CheckedPackageV2ReadResult::Refused(_)
    ));
    // One under, the cross-multiplication of the bounds is what the limit stops.
    assert_eq!(
        incomplete_work_path(&bounds, needed - 1),
        (needed, Some(body))
    );
    // The bounds cost more than a pair of small ones of the same shape.
    let (small, _) = with_root(
        TIMED,
        "eventually",
        timed_closed(whole("0"), whole("1")),
        None,
        1,
    );
    assert!(needed > smallest_deciding_work(&small));

    // The GCD of a non-reduced bound is charged at the body too.
    let (non_reduced, root) = with_root(
        TIMED,
        "eventually",
        timed_closed(
            rational("55340232221128654851", "3"),
            whole("99999999999999999999999"),
        ),
        None,
        1,
    );
    let needed = smallest_deciding_work(&non_reduced);
    let mut exact = CheckedPackageReadLimits::bounded();
    exact.work = needed;
    let CheckedPackageV2ReadResult::Refused(refusal) = read_limited(&non_reduced, exact) else {
        panic!("the exact work decides the refusal");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str().to_owned()),
        Some(at_node(root, "/body/operation/member/interval/lower"))
    );
    assert_eq!(
        incomplete_work_path(&non_reduced, needed - 1),
        (needed, Some(at_node(root, "/body")))
    );
}

/// Tracing: TC-048, FR-038-AC-122
#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_the_timed_form_fits_only_the_timed_profile() {
    let member = || timed_closed(whole("0"), whole("3"));
    for profile in FIVE_PROFILES {
        let (package, root) = with_root(profile, "eventually", member(), None, 1);
        if profile == TIMED {
            admitted("the timed form under timed/v1", &package);
        } else {
            expect(
                &format!("the timed form under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body"),
            );
        }
    }
}

/// Tracing: TC-048, FR-038-AC-122
#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_a_timed_form_member_set_defect_is_a_member_mismatch_at_the_member() {
    let interval = |members: Value| json!({"kind": "temporal_interval", "interval": members});
    let mismatches = [
        (
            "lower_end half",
            timed(whole("0"), whole("3"), "half", "closed"),
        ),
        (
            "upper_end half",
            timed(whole("0"), whole("3"), "closed", "half"),
        ),
        (
            "a missing end with integer-string bounds",
            interval(json!({"lower": "0", "upper": "3", "lower_end": "closed"})),
        ),
        (
            "a fifth member with integer-string bounds",
            interval(json!({"lower": "0", "upper": "3", "lower_end": "closed",
                            "upper_end": "closed", "extra": "x"})),
        ),
    ];
    for profile in [INFINITE, TIMED] {
        for (case, member) in &mismatches {
            let (package, root) = with_root(profile, "eventually", member.clone(), None, 1);
            expect(
                &format!("{case} under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::OperationMemberMismatch,
                &at_node(root, "/body/operation/member"),
            );
        }
        // The same member sets with the timed form's rational-object bounds have
        // no form, so the integer pattern refuses the object at `lower`.
        for (case, member) in [
            (
                "a missing end",
                interval(json!({"lower": whole("0"), "upper": whole("3"),
                                "lower_end": "closed"})),
            ),
            (
                "a fifth member",
                interval(json!({"lower": whole("0"), "upper": whole("3"),
                                "lower_end": "closed", "upper_end": "closed", "extra": "x"})),
            ),
        ] {
            let (package, root) = with_root(profile, "eventually", member, None, 1);
            expect(
                &format!("{case} with rational bounds under {profile}"),
                &package,
                Code::InvalidPackage,
                Cause::InvalidValue,
                &at_node(root, "/body/operation/member/interval/lower"),
            );
        }
    }
}

/// A member-set defect is the operation step's, after the whole temporal step:
/// a profile-fit defect in a higher-digest clause is reported first.
///
/// Tracing: TC-048, FR-038-AC-122
#[trace("TC-048", "FR-038-AC-122")]
#[test]
fn tc_048_a_member_set_defect_is_reported_after_a_profile_fit_defect_of_a_later_clause() {
    let mut found = false;
    for salt in 0..64 {
        let mut package = v2_all_families();
        select_profile(&mut package, TIMED);
        let root = find_identity(&package, &temporal_identity("eventually"));
        // The fixture's own clause holds the member-set defect, the added
        // clause the fit defect (an integer-form interval under `timed/v1`).
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] =
            timed(whole("0"), whole("3"), "half", "closed");
        let added = second_clause(
            &mut package,
            &format!("other{salt}"),
            closed("0", "3"),
            false,
        );
        let own = clause(&package);
        package["semantic_graph"]["nodes"][own]["body"]["arguments"][1]["value"] =
            json!(format!("own{salt}"));
        settle(&mut package);
        let own = clause(&package);
        if digest(&package, own) > digest(&package, added) {
            continue;
        }
        found = true;
        let added_root = nodes(&package)
            .iter()
            .rposition(|node| {
                node["body"]["operation"]["identity"] == "quire.op.temporal.eventually"
            })
            .expect("the added clause's formula");
        expect(
            "the fit defect of the higher-digest clause",
            &package,
            Code::InvalidPackage,
            Cause::OperationMemberMismatch,
            &at_node(added_root, "/body"),
        );
        break;
    }
    assert!(found, "a salt orders the two clauses as the case needs");
}

/// The bound pattern is strict wire validation, ahead of every identity check:
/// in a package whose node ids and `package_id` are all stale, a bound outside
/// its form's pattern, timed or integer, refuses `invalid-value` at the bound
/// and not at an identity check; the same package with a good bound refuses at
/// the identity check.
///
/// Tracing: TC-048, FR-038-AC-97, FR-038-AC-120
#[trace("TC-048", "FR-038-AC-97", "FR-038-AC-120")]
#[test]
fn tc_048_a_bound_outside_the_pattern_is_refused_ahead_of_every_identity_check() {
    let stale_with = |member: Value| {
        let (mut package, root) = with_root(INFINITE, "eventually", closed("0", "3"), None, 1);
        // Not settled again: the application key, every dependent node id and the
        // `package_id` no longer match the body.
        package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"] = member;
        (package, root)
    };
    // With a good bound the stale identities refuse at the first identity check.
    let (good, good_root) = stale_with(timed_closed(whole("0"), whole("3")));
    let ((code, cause, path), _) = refusal_of("a stale package with a good bound", &good);
    assert_eq!(
        (code, cause, path),
        (
            Code::InvalidPackage,
            Some(Cause::StaleNodeKey),
            Some(at_node(good_root, "/node_id"))
        )
    );
    // The `package_id` alone stale (every node key settled) with a bad bound.
    let stale_package_id = |member: Value| {
        let (mut package, _) = with_root(INFINITE, "eventually", member, None, 1);
        package["package_id"]["digest"] = json!("0".repeat(64));
        package
    };
    let ((code, cause, path), _) = refusal_of(
        "a stale package_id with a good bound",
        &stale_package_id(timed_closed(whole("0"), whole("3"))),
    );
    assert_eq!(
        (code, cause, path.as_deref()),
        (Code::StaleDependency, None, Some("/package_id/digest"))
    );
    for (case, member, bound) in [
        (
            "a timed negative numerator",
            timed_closed(rational("-1", "1"), whole("3")),
            "lower",
        ),
        (
            "a timed zero denominator",
            timed_closed(whole("0"), rational("3", "0")),
            "upper",
        ),
        ("an integer bound", closed("1.5", "0"), "lower"),
        ("a negative integer bound", closed("0", "-2"), "upper"),
    ] {
        let (package, root) = stale_with(member.clone());
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, &format!("/body/operation/member/interval/{bound}")),
        );
        let package = stale_package_id(member);
        let root = find_identity(&package, &temporal_identity("eventually"));
        expect(
            &format!("{case}, a stale package_id alone"),
            &package,
            Code::InvalidPackage,
            Cause::InvalidValue,
            &at_node(root, &format!("/body/operation/member/interval/{bound}")),
        );
    }
}

/// The lowest-terms check reads nodes in ascending `node_id` digest order, not
/// position order: of two non-reduced nodes, the refusal is at the one with the
/// lower digest, whichever is first in the package.
///
/// Tracing: TC-048, FR-038-AC-121
#[trace("TC-048", "FR-038-AC-121")]
#[test]
fn tc_048_the_first_non_reduced_bound_is_the_lower_digest_nodes() {
    for added_is_lower in [true, false] {
        let mut found = false;
        for multiple in 1..64_u32 {
            let (mut package, root) = with_root(
                TIMED,
                "eventually",
                timed_closed(rational("2", "4"), whole("3")),
                None,
                1,
            );
            let holds = digest(
                &package,
                find_identity(&package, &temporal_identity("holds")),
            );
            let second = timed_closed(
                whole("0"),
                rational(&(2 * multiple).to_string(), &(4 * multiple).to_string()),
            );
            let added = push(&mut package, formula("once", second, vec![refer(&holds)]));
            settle(&mut package);
            assert!(root < added, "the pushed node is the later one");
            if (digest(&package, added) < digest(&package, root)) != added_is_lower {
                continue;
            }
            found = true;
            // The arrangement asserted: the refused node is the lower digest one,
            // at a position that is after or before the other.
            let (lower_digest, bound) = if added_is_lower {
                (added, "upper")
            } else {
                (root, "lower")
            };
            expect_graph(
                &format!("added node has the lower digest: {added_is_lower}"),
                &package,
                &at_node(
                    lower_digest,
                    &format!("/body/operation/member/interval/{bound}"),
                ),
            );
            break;
        }
        assert!(found, "a multiple orders the two nodes as the case needs");
    }
}
