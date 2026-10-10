// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-114 through FR-038-AC-117, at the package: the flat wire (an
//! application stands only at a body root, and a body is drawn from the
//! five-stratum grammar of merged QSpec FR-322 "Body grammar"), the order of
//! its refusals ahead of every identity check, and the absence of a depth
//! limit and of any recursion that follows the package's size.
//! FR-038-AC-118 is `tests/conformance_qspec`, which runs QSpec's own
//! mutations through `make conformance-qspec`.
//!
//! Every case builds its package in-repo from this crate's public vocabulary;
//! every expectation is written out.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, json_depth, mint_ungrouped_structural_keys, node_id,
    pointer, refresh_identity, settle, sha256_hex, v2_all_families, v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageEvidence, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};

use CheckedPackageRefusalCode as Code;

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn admitted(package: &Value) {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, read {other:?}"),
    }
}

/// A refusal's code, cause and pointer text.
fn outcome(refusal: &CheckedPackageRefusal) -> (Code, Option<CheckedPackageRefusalCause>, String) {
    (
        refusal.code,
        refusal.cause,
        refusal
            .path
            .as_ref()
            .map_or_else(|| "<none>".to_owned(), |path| path.as_str().to_owned()),
    )
}

/// `malformed_wire` at `path`, with no cause.
fn malformed(path: &str) -> (Code, Option<CheckedPackageRefusalCause>, String) {
    (Code::MalformedWire, None, path.to_owned())
}

/// `ill_typed`/`operator-ineligible` at `path`.
fn ineligible(path: &str) -> (Code, Option<CheckedPackageRefusalCause>, String) {
    (
        Code::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        path.to_owned(),
    )
}

/// A literal: a Leaf.
fn leaf() -> Value {
    json!({
        "term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "integer", "value": 1,
    })
}

fn binding(value: Value) -> Value {
    json!({"term": "binding", "name": "b", "value": value})
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}

/// An application of `operator` naming `identity`. Its own closed shape is
/// complete; what it stands in is what the cases vary.
fn application(operator: &str, identity: &str) -> Value {
    json!({
        "term": "application",
        "operator": operator,
        "operation": {
            "identity": identity, "laws": [], "mode": null, "member": null, "leaves": [],
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [],
    })
}

/// The five operator classes of FR-038-AC-114 other than `case`, each with an
/// identity it names.
const CLASSES: [(&str, &str); 5] = [
    ("call", "quire.op.function.call"),
    ("state_clause", "quire.op.state.clause"),
    ("temporal", "quire.op.temporal.clause"),
    ("temporal_formula", "quire.op.temporal.holds"),
    ("temporal_fairness", "quire.op.temporal.fair"),
];

fn position_of(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("a {tag}/{form} node"))
}

/// The fixture's function call node: its body is an application.
fn call_position(package: &Value) -> usize {
    position_of(package, "function", "pure_function")
}

/// The fixture's `value`/`literal` node, whose body is an empty aggregate.
fn host_position(package: &Value) -> usize {
    position_of(package, "value", "literal")
}

/// The fixture with `edit` applied to the node at `position`, every identity
/// derived again: what is refused is the body, not a stale key.
fn settled(position: impl Fn(&Value) -> usize, edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut package = v2_all_families();
    let position = position(&package);
    edit(&mut package["semantic_graph"]["nodes"][position]);
    settle(&mut package);
    mint_ungrouped_structural_keys(&mut package);
    settle(&mut package);
    (package, position)
}

/// The fixture's call node with `second` as its second argument.
fn call_with_second_argument(second: Value) -> (Value, usize) {
    settled(call_position, |node| {
        node["body"]["arguments"]
            .as_array_mut()
            .expect("arguments")
            .push(second);
    })
}

/// FR-038-AC-114: an application of any class other than `case`, as an element
/// of another application's arguments, as a member of an aggregate and as the
/// value of a binding, refuses `malformed_wire` at the nested application.
///
/// Tracing: TC-048, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-114")]
#[test]
fn tc_048_an_application_nested_in_any_position_refuses_malformed_wire_at_it() {
    for (operator, identity) in CLASSES {
        let nested = || application(operator, identity);
        let positions: [(&str, Value, &str); 3] = [
            ("an argument element", nested(), "arguments/1"),
            (
                "a member of an aggregate argument",
                aggregate(vec![nested()]),
                "arguments/1/members/0",
            ),
            ("a binding value", binding(nested()), "arguments/1/value"),
        ];
        for (position, second, below) in positions {
            let (package, call) = call_with_second_argument(second);
            let refusal = refused(&package);
            assert_eq!(
                outcome(&refusal),
                malformed(&format!("/semantic_graph/nodes/{call}/body/{below}")),
                "{operator} as {position}"
            );
            assert_eq!(
                refusal.locus, None,
                "{operator} as {position}: no node is at fault"
            );
        }
        // A member of the aggregate that is a node body root.
        let (package, host) = settled(host_position, |node| {
            node["body"] = aggregate(vec![nested()]);
        });
        assert_eq!(
            outcome(&refused(&package)),
            malformed(&format!("/semantic_graph/nodes/{host}/body/members/0")),
            "{operator} as a member of a body aggregate"
        );
    }
}

/// FR-038-AC-114: the shapes the strata do not admit refuse `malformed_wire` at
/// the value outside them: an aggregate inside a Group's members, an aggregate
/// as the value of a Group's binding, a binding whose value is a binding, a
/// binding inside a Tuple's members, an aggregate inside the Group that a
/// Tuple's member is, and a binding as a body root.
///
/// Tracing: TC-048, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-114")]
#[test]
fn tc_048_a_body_outside_the_strata_refuses_malformed_wire_at_the_value() {
    let cases: [(&str, Value, &str); 6] = [
        (
            "an aggregate inside a Group's members",
            aggregate(vec![aggregate(vec![aggregate(vec![leaf()])])]),
            "/members/0/members/0",
        ),
        (
            "a binding whose value is an aggregate where only a Leaf stands",
            aggregate(vec![aggregate(vec![binding(aggregate(vec![leaf()]))])]),
            "/members/0/members/0/value",
        ),
        (
            "a binding whose value is a binding",
            aggregate(vec![binding(binding(leaf()))]),
            "/members/0/value",
        ),
        (
            "a binding inside a Tuple's members",
            aggregate(vec![binding(aggregate(vec![
                aggregate(vec![leaf()]),
                binding(leaf()),
            ]))]),
            "/members/0/value/members/1",
        ),
        (
            "an aggregate inside the Group that a Tuple's member is",
            aggregate(vec![binding(aggregate(vec![aggregate(vec![aggregate(
                vec![leaf()],
            )])]))]),
            "/members/0/value/members/0/members/0",
        ),
        ("a binding as a body root", binding(leaf()), ""),
    ];
    for (name, body, below) in cases {
        let (package, host) = settled(host_position, |node| node["body"] = body);
        assert_eq!(
            outcome(&refused(&package)),
            malformed(&format!("/semantic_graph/nodes/{host}/body{below}")),
            "{name}"
        );
    }
}

/// FR-038-AC-114: no application stands in a `state`/`frame` body, a closed
/// shape of its own: one in an entry refuses `malformed_wire` at the
/// application, a `case` one `ill_typed`/`operator-ineligible` at its operator.
///
/// Tracing: TC-048, FR-038-AC-114, FR-038-AC-115
#[trace("TC-048", "FR-038-AC-114", "FR-038-AC-115")]
#[test]
fn tc_048_an_application_in_a_frame_body_refuses_at_the_application() {
    let frame = |entry: Value| {
        settled(
            |package| position_of(package, "state", "frame"),
            |node| node["body"]["creates"] = json!([entry]),
        )
    };
    let (package, node) = frame(application("call", "quire.op.function.call"));
    assert_eq!(
        outcome(&refused(&package)),
        malformed(&format!("/semantic_graph/nodes/{node}/body/creates/0"))
    );
    let (package, node) = frame(application("case", "quire.op.control.case"));
    assert_eq!(
        outcome(&refused(&package)),
        ineligible(&format!(
            "/semantic_graph/nodes/{node}/body/creates/0/operator"
        ))
    );
}

/// FR-038-AC-114: the strata admit what they name: a Group as a member of a
/// body aggregate, and a Tuple, a Group and a Leaf as the value of a Member's
/// binding; and the same meaning with each composite subterm as its own node
/// reached by `reference` is admitted.
///
/// Tracing: TC-048, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-114")]
#[test]
fn tc_048_a_body_inside_the_strata_is_admitted() {
    let bodies = [
        (
            "a Group member",
            aggregate(vec![aggregate(vec![binding(leaf())])]),
        ),
        (
            "a Tuple as a Member's binding value",
            aggregate(vec![binding(aggregate(vec![
                aggregate(vec![binding(leaf())]),
                leaf(),
            ]))]),
        ),
        (
            "a Group as a Member's binding value",
            aggregate(vec![binding(aggregate(vec![binding(leaf()), leaf()]))]),
        ),
        (
            "a Leaf as a Member's binding value",
            aggregate(vec![binding(leaf())]),
        ),
        // The deepest body the strata allow: an aggregate, a Member binding,
        // a Tuple, a Group, a binding and a Leaf.
        (
            "the deepest body",
            aggregate(vec![binding(aggregate(vec![aggregate(vec![binding(
                leaf(),
            )])]))]),
        ),
    ];
    let mut deepest = 0;
    for (name, body) in bodies {
        let (package, _) = settled(host_position, |node| node["body"] = body);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{name}"
        );
        deepest = deepest.max(json_depth(&package));
    }
    // No in-grammar package nests past sixteen JSON levels, whatever its size
    // (the five strata and the package's own members, in the unit
    // `json_depth` counts them in).
    assert!(deepest > json_depth(&v2_all_families()) && deepest <= 16);
    // Each composite subterm as its own node, reached by `reference`: the
    // fixture's own call node names its argument by `reference`.
    admitted(&v2_all_families());
}

/// FR-038-AC-114: the same meaning written with each composite subterm as its own
/// node, reached by `reference`, is not refused `malformed_wire`: for each nested
/// application in each position and each body outside the strata above, the
/// flattened form (the subterm's place held by a `reference` to a node that holds
/// it) is admitted.
///
/// Tracing: TC-048, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-114")]
#[test]
fn tc_048_the_flattened_form_of_each_refused_body_is_admitted() {
    let node = || json!({"term": "reference", "target": node_id(&family_key("aaaa"))});
    // The nested application of each position, as its own node.
    for (position, second) in [
        ("an argument element", node()),
        ("a member of an aggregate argument", aggregate(vec![node()])),
        ("a binding value", binding(node())),
    ] {
        let (package, _) = call_with_second_argument(second);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{position}"
        );
    }
    // Each stratum shape of the test above, its composite subterm a node.
    for (name, body) in [
        (
            "an aggregate inside a Group's members",
            aggregate(vec![aggregate(vec![node()])]),
        ),
        (
            "a binding whose value is an aggregate where only a Leaf stands",
            aggregate(vec![aggregate(vec![binding(node())])]),
        ),
        (
            "a binding whose value is a binding",
            aggregate(vec![binding(node())]),
        ),
        (
            "a binding inside a Tuple's members",
            aggregate(vec![binding(aggregate(vec![
                aggregate(vec![node()]),
                node(),
            ]))]),
        ),
        ("a binding as a body root", aggregate(vec![binding(leaf())])),
    ] {
        let (package, _) = settled(host_position, |holder| holder["body"] = body);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{name}"
        );
    }
}

/// A `details` term in `package`, as the one detail of the one entry.
fn with_detail(detail: Value) -> Value {
    let mut package = v2_all_families();
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
        "details": [detail], "loci": [],
    }]);
    package
}

/// FR-038-AC-115: a nested `case` application refuses
/// `ill_typed`/`operator-ineligible` at its own `operator`, and where one term
/// holds several offending constructs the refusal is the first in document
/// pre-order, outermost first.
///
/// Tracing: TC-048, FR-038-AC-115
#[trace("TC-048", "FR-038-AC-115")]
#[test]
fn tc_048_the_first_offending_construct_in_pre_order_is_refused() {
    let case = || application("case", "quire.op.control.case");
    let call = || application("call", "quire.op.function.call");
    let formula = || application("temporal_formula", "quire.op.temporal.holds");

    // A nested `case` is refused at its `operator`, with the holder as locus.
    let (package, node) = call_with_second_argument(case());
    let refusal = refused(&package);
    assert_eq!(
        outcome(&refusal),
        ineligible(&format!(
            "/semantic_graph/nodes/{node}/body/arguments/1/operator"
        ))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][node]["node_id"]
    );

    // A body whose `arguments/0` is a nested `case` and whose `arguments/1`
    // is a nested call refuses at the `case`; with the two swapped, at the
    // call, as `malformed_wire`.
    let (package, node) = settled(call_position, |node| {
        node["body"]["arguments"] = json!([case(), call()]);
    });
    assert_eq!(
        outcome(&refused(&package)),
        ineligible(&format!(
            "/semantic_graph/nodes/{node}/body/arguments/0/operator"
        ))
    );
    let (package, node) = settled(call_position, |node| {
        node["body"]["arguments"] = json!([call(), case()]);
    });
    assert_eq!(
        outcome(&refused(&package)),
        malformed(&format!("/semantic_graph/nodes/{node}/body/arguments/0"))
    );

    // A `details` term that is a `temporal_formula` application holding a
    // nested `case` argument refuses at the outer application's operator.
    let mut outer = formula();
    outer["arguments"] = json!([case()]);
    assert_eq!(
        outcome(&refused(&with_detail(outer))),
        ineligible("/diagnostics/entries/0/details/0/operator")
    );
    // A `details` aggregate of a `temporal_formula` application and then a
    // `case` application refuses at the first member's operator.
    assert_eq!(
        outcome(&refused(&with_detail(aggregate(vec![formula(), case()])))),
        ineligible("/diagnostics/entries/0/details/0/members/0/operator")
    );
    // An application of any other class in a `details` term is outside the
    // grammar: `malformed_wire` at that application, as the root or nested.
    assert_eq!(
        outcome(&refused(&with_detail(call()))),
        malformed("/diagnostics/entries/0/details/0")
    );
    assert_eq!(
        outcome(&refused(&with_detail(aggregate(vec![leaf(), call()])))),
        malformed("/diagnostics/entries/0/details/0/members/1")
    );
    assert_eq!(
        outcome(&refused(&with_detail(application(
            "temporal",
            "quire.op.temporal.clause"
        )))),
        malformed("/diagnostics/entries/0/details/0")
    );
}

/// FR-038-AC-115: a non-`case` application at the body root of a node its class
/// does not place it in refuses `ill_typed`/`operator-ineligible` at that node,
/// and not `malformed_wire`.
///
/// Tracing: TC-048, FR-038-AC-115
#[trace("TC-048", "FR-038-AC-115")]
#[test]
fn tc_048_a_misplaced_body_root_application_refuses_at_the_node() {
    let (package, host) = settled(host_position, |node| {
        node["body"] = application("temporal_formula", "quire.op.temporal.holds");
        node["dependencies"] = json!([]);
    });
    assert_eq!(
        outcome(&refused(&package)),
        ineligible(&format!("/semantic_graph/nodes/{host}"))
    );
}

/// FR-038-AC-116: the body grammar is strict wire validation, ahead of the
/// recomputation of `package_id` and of every node key. A package whose node
/// keys, `identity_preimage` and `package_id` are all stale refuses at the
/// nested application, not at an identity; the same package flattened refuses at
/// an identity.
///
/// Tracing: TC-048, FR-038-AC-116
#[trace("TC-048", "FR-038-AC-116")]
#[test]
fn tc_048_the_body_grammar_is_checked_ahead_of_every_identity() {
    // Every identity stale: the body is changed and nothing is derived again.
    let stale = |second: Value| {
        let mut package = v2_all_families();
        let call = call_position(&package);
        package["semantic_graph"]["nodes"][call]["body"]["arguments"]
            .as_array_mut()
            .expect("arguments")
            .push(second);
        package["identity_preimage"]["identity_projection"] = json!([]);
        package["package_id"]["digest"] = json!("0".repeat(64));
        (package, call)
    };
    for (operator, identity) in CLASSES {
        let (package, call) = stale(application(operator, identity));
        assert_eq!(
            outcome(&refused(&package)),
            malformed(&format!("/semantic_graph/nodes/{call}/body/arguments/1")),
            "{operator}"
        );
    }
    let (package, call) = stale(application("case", "quire.op.control.case"));
    assert_eq!(
        outcome(&refused(&package)),
        ineligible(&format!(
            "/semantic_graph/nodes/{call}/body/arguments/1/operator"
        ))
    );

    // The same body flattened (every composite subterm a `reference`), its
    // identities left stale, refuses at an identity and not as a malformed
    // wire: first the package id.
    let mut flattened = v2_all_families();
    let call = call_position(&flattened);
    flattened["package_id"]["digest"] = json!("0".repeat(64));
    assert_eq!(
        outcome(&refused(&flattened)),
        (Code::StaleDependency, None, "/package_id/digest".to_owned())
    );
    // With the package id derived again, the node key.
    let mut flattened = v2_all_families();
    flattened["semantic_graph"]["nodes"][call]["body"]["result_type"] =
        node_id(&family_key("bbbb"));
    refresh_identity(&mut flattened);
    assert_eq!(
        outcome(&refused(&flattened)),
        (
            Code::InvalidPackage,
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            format!("/semantic_graph/nodes/{call}/node_id")
        )
    );
}

/// Runs `work` on a thread whose stack is 256 KiB, far fewer frames than the
/// documents below have levels or nodes, so any recursion that follows the
/// input aborts the process.
fn on_small_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(work)
        .expect("spawn")
        .join()
        .expect("the work ran to completion")
}

/// The read limits of a package of `nodes` nodes, every one sized past what any
/// of its nodes could take, so none decides the outcome.
fn generous_limits(nodes: usize) -> CheckedPackageReadLimits {
    let count = u64::try_from(nodes).expect("count");
    CheckedPackageReadLimits {
        bytes: 1 << 36,
        nodes: count * 2,
        edges: count * 16,
        occurrences: count * 16,
        diagnostics: 1 << 20,
        work: count * 1024,
    }
}

/// FR-038-AC-117: a package holding a chain of 100000 nodes, each referencing
/// the previous, read under limits that decide nothing on a 256 KiB stack, is
/// admitted and lowered from its last node, with no outcome naming a depth, and
/// has the same JSON nesting depth as a package of one level.
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_a_hundred_thousand_node_chain_is_admitted_and_lowered_on_a_small_stack() {
    const LENGTH: usize = 100_000;
    let (one_level, _) = v2_reference_chain(1);
    let (bytes, last) = v2_reference_chain(LENGTH);
    assert_eq!(
        nesting(&bytes),
        nesting(&one_level),
        "a chain of any length has the JSON depth of a chain of one"
    );
    drop(one_level);
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let total = LENGTH
        + base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
    drop(base);
    let last = typed(&last);
    on_small_stack(move || {
        let read = CheckedPackageV2::read(&bytes, generous_limits(total), &evidence);
        let CheckedPackageV2ReadResult::Admitted(package) = read else {
            panic!("the chain admits, read {read:?}");
        };
        drop(bytes);
        assert_eq!(package.graph().nodes.len(), total);
        let profile = CompleteLoweringProfileV2 {
            supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
            require_bounds: false,
            work_limit: u64::MAX / 2,
        };
        let lowered = package.lower(std::slice::from_ref(&last), &profile);
        let [CompleteLoweringRecordV2::Lowered { node }] = lowered.records.as_slice() else {
            panic!("the last node lowers, got {:?}", lowered.records.len());
        };
        // Every node of the chain, the family's nodes it ends at and the
        // fixture's boolean type are in the closure, the last node itself not.
        assert!(node.dependencies.len() > LENGTH);
        assert!(!node.dependencies.contains(&last));
        assert!(lowered.package.canonical_bytes().is_some());
    });
}

/// The deepest nesting of arrays and objects in the JSON text `bytes`, counted
/// without building the document: a scan over its brackets that leaves strings
/// alone.
fn nesting(bytes: &[u8]) -> usize {
    let (mut depth, mut deepest, mut in_string, mut escaped) = (0_usize, 0_usize, false, false);
    for byte in bytes {
        if in_string {
            match (escaped, byte) {
                (true, _) => escaped = false,
                (false, b'\\') => escaped = true,
                (false, b'"') => in_string = false,
                (false, _) => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

fn typed(digest: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(digest)).expect("node id")
}

/// FR-038-AC-117: no read limit, and no outcome of a read, names a depth. A
/// constructor of the limits and a match on the limit kinds that list every
/// member and every kind fail to compile if one is added back.
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_no_read_limit_and_no_limit_kind_names_a_depth() {
    let CheckedPackageReadLimits {
        bytes,
        nodes,
        edges,
        occurrences,
        diagnostics,
        work,
    } = CheckedPackageReadLimits::bounded();
    assert!([bytes, nodes, edges, occurrences, diagnostics, work]
        .iter()
        .all(|limit| *limit > 0));
    for kind in [
        CheckedPackageLimit::Bytes,
        CheckedPackageLimit::Nodes,
        CheckedPackageLimit::Edges,
        CheckedPackageLimit::Occurrences,
        CheckedPackageLimit::Diagnostics,
        CheckedPackageLimit::Work,
    ] {
        match kind {
            CheckedPackageLimit::Bytes
            | CheckedPackageLimit::Nodes
            | CheckedPackageLimit::Edges
            | CheckedPackageLimit::Occurrences
            | CheckedPackageLimit::Diagnostics
            | CheckedPackageLimit::Work => {}
        }
    }
}

/// FR-038-AC-117: no source file of the V2 reader holds a depth ceiling, or a
/// stack that grows with the input: the words `MAXIMUM_DEPTH`, any other
/// `MAX_*DEPTH` constant, `stacker`, `serde_stacker` and `on_stack_for` are
/// absent from `crates/quire-contract-model/src/checked_package/`. The crate's
/// v1 modules and its manifest are outside the scan (FR-019, FR-023).
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_the_v2_reader_sources_hold_no_depth_ceiling_or_growing_stack() {
    fn sources(directory: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/quire-contract-model/src/checked_package");
    let mut files = Vec::new();
    sources(&root, &mut files);
    assert!(files.len() > 10, "the scan reads the reader's sources");
    for file in files {
        let text = std::fs::read_to_string(&file).expect("source");
        for word in ["stacker", "serde_stacker", "on_stack_for", "MAXIMUM_DEPTH"] {
            assert!(!text.contains(word), "{} holds {word}", file.display());
        }
        let ceiling = text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .find(|token| token.starts_with("MAX") && token.ends_with("DEPTH"));
        assert_eq!(ceiling, None, "{} holds a depth ceiling", file.display());
    }
}

/// The node body of a fixture with `deep` as its text in place of a marker, the
/// package id derived again over the spliced document: canonical, with every
/// identity current, whatever the body is. Built as text, so the test's own
/// stack never recurses over the nesting.
fn package_with_body_text(deep: &str) -> Vec<u8> {
    let marker = "\u{1}deep";
    let quoted = serde_json::to_string(marker).expect("string");
    let mut package = v2_all_families();
    let host = host_position(&package);
    package["semantic_graph"]["nodes"][host]["body"] = json!(marker);
    refresh_identity(&mut package);
    let splice = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("utf-8")
            .replace(&quoted, deep)
            .into_bytes()
    };
    let digest = sha256_hex(&splice(canonical(&package["identity_preimage"])));
    package["package_id"]["digest"] = json!(digest);
    splice(canonical(&package))
}

/// `levels` aggregates, each holding the next, ending in an empty one.
fn nested_aggregates(levels: usize) -> String {
    format!(
        "{}{{\"members\":[],\"term\":\"aggregate\"}}{}",
        "{\"members\":[".repeat(levels - 1),
        "],\"term\":\"aggregate\"}".repeat(levels - 1)
    )
}

fn read_on_small_stack(
    bytes: Vec<u8>,
    evidence: CheckedPackageEvidence,
) -> CheckedPackageV2ReadResult {
    on_small_stack(move || {
        CheckedPackageV2::read(
            &bytes,
            CheckedPackageReadLimits {
                bytes: 1 << 30,
                ..CheckedPackageReadLimits::bounded()
            },
            &evidence,
        )
    })
}

/// FR-038-AC-117: an otherwise canonical document whose node body nests a term
/// 300 levels deep, past the strict parse's recursion limit of 128, refuses
/// `malformed_wire` with no pointer at the strict parse, ahead of the
/// canonical-bytes check and the body grammar, never `incomplete`, on a
/// 256 KiB stack; the same document nested 20 levels deep refuses
/// `malformed_wire` at the first value outside the body grammar.
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_a_document_nested_past_the_parse_limit_refuses_at_the_parse() {
    let evidence = evidence_for(&v2_all_families());
    let host = host_position(&v2_all_families());

    let deep = package_with_body_text(&nested_aggregates(300));
    let result = read_on_small_stack(deep.clone(), evidence.clone());
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("300 levels refuse, read {result:?}");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.path, None, "the parse names no pointer");

    // The parse comes ahead of the canonical-bytes check: the same document,
    // no longer canonical, is refused as the same malformed wire.
    let mut spaced = b" ".to_vec();
    spaced.extend_from_slice(&deep);
    let result = read_on_small_stack(spaced, evidence.clone());
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("300 levels, not canonical, refuse, read {result:?}");
    };
    assert_eq!((refusal.code, refusal.path), (Code::MalformedWire, None));

    // Twenty levels read, and the first value outside the grammar refuses:
    // the body is an aggregate whose member is a Group, so that Group's own
    // aggregate member is the first value outside it.
    let shallow = package_with_body_text(&nested_aggregates(20));
    let result = read_on_small_stack(shallow, evidence.clone());
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("20 levels refuse, read {result:?}");
    };
    assert_eq!(
        outcome(&refusal),
        malformed(&format!(
            "/semantic_graph/nodes/{host}/body/members/0/members/0"
        ))
    );

    // The window between the grammar and the parse limit: a body of 43 to 61
    // aggregates is 91 to 126 JSON levels deep, within the parse's limit of 128
    // levels and far past the grammar. It refuses at the same value as the
    // 20-level one, and never aborts the read on a 256 KiB stack, in a debug
    // build too: the closed decode takes the bodies out before it reads the
    // document (`intake`).
    for levels in [41, 42, 43, 45, 50, 55, 58, 61] {
        let window = package_with_body_text(&nested_aggregates(levels));
        let result = read_on_small_stack(window, evidence.clone());
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{levels} levels refuse, read {result:?}");
        };
        assert_eq!(
            outcome(&refusal),
            malformed(&format!(
                "/semantic_graph/nodes/{host}/body/members/0/members/0"
            )),
            "{levels} levels"
        );
    }
    // The first depth past the parse's limit refuses at the parse.
    let past = package_with_body_text(&nested_aggregates(62));
    let result = read_on_small_stack(past, evidence);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("62 levels refuse, read {result:?}");
    };
    assert_eq!((refusal.code, refusal.path), (Code::MalformedWire, None));
}

/// FR-038-AC-117: an identity-projection body, which the body grammar does not
/// check, can nest as deep as the strict parse reads. A stale one, 61 aggregates
/// (126 JSON levels) and one at the parse's limit of 127 levels, is compared with
/// its node's body and refused `stale_dependency` at the first member that
/// differs, on a 256 KiB debug stack, with no abort.
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_a_deep_stale_projection_body_is_refused_on_a_small_stack() {
    let evidence = evidence_for(&v2_all_families());
    let marker = "\u{1}deep";
    let quoted = serde_json::to_string(marker).expect("string");
    let aggregates = |levels: usize, innermost: &str| {
        format!(
            "{}{{\"members\":{innermost},\"term\":\"aggregate\"}}{}",
            "{\"members\":[".repeat(levels - 1),
            "],\"term\":\"aggregate\"}".repeat(levels - 1)
        )
    };
    for deep in [aggregates(61, "[]"), aggregates(61, "[[]]")] {
        let mut package = v2_all_families();
        let host = host_position(&package);
        package["identity_preimage"]["identity_projection"][host]["body"] = json!(marker);
        let splice = |text: String| text.replace(&quoted, &deep);
        let preimage =
            splice(String::from_utf8(canonical(&package["identity_preimage"])).expect("utf-8"));
        package["package_id"]["digest"] = json!(sha256_hex(preimage.as_bytes()));
        let text = splice(String::from_utf8(canonical(&package)).expect("utf-8"));
        let result = read_on_small_stack(text.into_bytes(), evidence.clone());
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("a stale deep projection refuses, read {result:?}");
        };
        assert_eq!(
            outcome(&refusal),
            (
                Code::StaleDependency,
                None,
                format!("/identity_preimage/identity_projection/{host}/body/members")
            )
        );
    }
}

/// FR-038-AC-117: a `details` term in the window between the grammar and the
/// parse limit refuses at the first value outside the grammar too, on a 256 KiB
/// stack.
///
/// Tracing: TC-048, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-117")]
#[test]
fn tc_048_a_deep_details_term_within_the_parse_limit_refuses_at_the_grammar() {
    let evidence = evidence_for(&v2_all_families());
    let marker = "\u{1}deep";
    let quoted = serde_json::to_string(marker).expect("string");
    let mut package = with_detail(json!(marker));
    refresh_identity(&mut package);
    let digest_of =
        |text: &str| sha256_hex(text.replace(&quoted, &nested_aggregates(50)).as_bytes());
    let preimage = String::from_utf8(canonical(&package["identity_preimage"])).expect("utf-8");
    package["package_id"]["digest"] = json!(digest_of(&preimage));
    let text = String::from_utf8(canonical(&package))
        .expect("utf-8")
        .replace(&quoted, &nested_aggregates(50));
    let result = read_on_small_stack(text.into_bytes(), evidence);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("a deep detail refuses, read {result:?}");
    };
    assert_eq!(
        outcome(&refusal),
        malformed("/diagnostics/entries/0/details/0/members/0")
    );
}

/// FR-038-AC-117: a document that hides its depth under an unknown member is
/// refused by the closed-schema decode, not stopped for its depth, and a syntax
/// error or a duplicate member anywhere in a document nested past the parse
/// limit is refused first.
///
/// Tracing: TC-048, FR-038-AC-3, FR-038-AC-117
#[trace("TC-048", "FR-038-AC-3", "FR-038-AC-117")]
#[test]
fn tc_048_a_syntax_or_duplicate_defect_is_refused_before_the_parse_limit() {
    let evidence = evidence_for(&v2_all_families());
    let nested = |levels: usize| format!("{}{}", "[".repeat(levels), "]".repeat(levels));
    let deep = nested(200);
    on_small_stack(move || {
        let read = |text: &str| {
            CheckedPackageV2::read(
                text.as_bytes(),
                CheckedPackageReadLimits::bounded(),
                &evidence,
            )
        };
        let refusal = |text: &str| match read(text) {
            CheckedPackageV2ReadResult::Refused(refusal) => refusal,
            other => panic!("expected a refusal, read {other:?}"),
        };
        assert_eq!(
            (refusal(&deep).code, refusal(&deep).path),
            (Code::MalformedWire, None)
        );
        let broken = format!("{}x{}", "[".repeat(200), "]".repeat(200));
        assert_eq!(
            (refusal(&broken).code, refusal(&broken).path),
            (Code::MalformedWire, None)
        );
        let duplicate = format!("{{\"a\":{deep},\"a\":1}}");
        assert_eq!(
            (refusal(&duplicate).code, refusal(&duplicate).path),
            (Code::DuplicateMember, Some(pointer("/a")))
        );
    });
}
