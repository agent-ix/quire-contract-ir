// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The graph stage's read-storage charging: each family validator's heap
//! requests are charged to the caller's budget at `PackageRead(Graph, i)`
//! before they are made (TC-472).

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2BudgetedReadResult, NativeJoinBudget,
    NativeJoinLocusSource, PackageReadStage,
};

/// One heap request of the read, in the order it was made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub stage: PackageReadStage,
    pub index: usize,
    pub form: &'static str,
    pub amount: u128,
    /// Read storage admitted before this request.
    pub before: u64,
}

/// How a fixture is read under a budget.
pub type Reader<'a> = &'a dyn Fn(&mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult;

fn all_families(budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    let value = v2_all_families();
    crate::qspec_read_proof::read_budget_and_account(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        budget,
    )
}

fn read_at(read: Reader<'_>, limit: u64) -> (CheckedPackageV2BudgetedReadResult, NativeJoinBudget) {
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    let result = read(&mut budget);
    (result, budget)
}

/// The request a read with `limit` bytes of read storage stops at, if any.
pub fn denial_at(read: Reader<'_>, limit: u64) -> Option<Request> {
    let (result, budget) = read_at(read, limit);
    if matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)) {
        return None;
    }
    let locus = budget.first_unpaid_locus().expect("a denial has a locus");
    let NativeJoinLocusSource::PackageRead(stage, index) = locus.source else {
        panic!("a read denial is at a package read locus: {locus:?}");
    };
    Some(Request {
        stage,
        index,
        form: locus.form,
        amount: budget
            .first_unpaid_read_storage()
            .expect("read storage denial"),
        before: limit,
    })
}

/// Every request the read makes while it is in `stage`, discovered from
/// actual first-unpaid prefixes in runtime order. Stage labels may decrease
/// or reappear: selected-document readers and retained tables interleave them.
/// Only the supplied caller budget is queried, so fixture preparation using
/// a different budget cannot contaminate the discovered requests.
pub fn stage_requests(read: Reader<'_>, stage: PackageReadStage) -> Vec<Request> {
    let total = {
        let (result, budget) = read_at(read, u64::MAX);
        assert!(
            matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)),
            "{result:?}"
        );
        budget.read_storage_consumed()
    };
    let mut limit = 0_u64;
    let mut found = Vec::new();
    while let Some(mut request) = denial_at(read, limit) {
        request.before = limit;
        let next = limit
            .checked_add(u64::try_from(request.amount).expect("amount fits"))
            .expect("request boundary fits");
        assert!(
            next > limit && next <= total,
            "request boundary advances within the measured read"
        );
        limit = next;
        if request.stage == stage {
            found.push(request);
        }
    }
    assert_eq!(limit, total, "every charged request boundary is visited");
    found
}

/// Each request of `requests` is charged before it is made: the allowance
/// exactly covering it gets past it, and one byte short is denied at it with
/// its locus and amount.
pub fn assert_exact_and_one_short(read: Reader<'_>, requests: &[Request]) {
    for (position, request) in requests.iter().enumerate() {
        let exact = request.before + u64::try_from(request.amount).unwrap();
        let short = denial_at(read, exact - 1).expect("one byte short is denied");
        assert_eq!(
            (short.stage, short.index, short.form, short.amount),
            (request.stage, request.index, request.form, request.amount),
            "request {position}"
        );
        if let Some(next) = denial_at(read, exact) {
            assert!(
                next.stage != request.stage
                    || requests.get(position + 1).is_some_and(|following| {
                        (
                            following.stage,
                            following.index,
                            following.form,
                            following.amount,
                        ) == (next.stage, next.index, next.form, next.amount)
                    }),
                "request {position} got past at its allowance"
            );
        }
    }
}

/// The graph-stage requests of the all-families fixture.
fn graph_requests() -> Vec<Request> {
    stage_requests(&all_families, PackageReadStage::Graph)
}

/// Every graph-stage request is charged before it is made (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn each_graph_request_is_denied_one_byte_short_and_admitted_at_its_exact_allowance() {
    let requests = graph_requests();
    assert!(!requests.is_empty(), "the graph stage makes requests");
    assert_exact_and_one_short(&all_families, &requests);
}

/// The graph index, the per-node tables and the state step's walk and anchor
/// table are each charged at their own node and form.
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn the_graph_index_state_tables_and_walks_are_charged_at_graph_loci() {
    let requests = graph_requests();
    let at = |form: &str| {
        requests
            .iter()
            .filter(|request| request.form == form)
            .count()
    };
    assert!(at("walk") > 0, "a term walk grows its lists on the budget");
    assert!(
        at("state") > 0,
        "the state step builds its tables on the budget"
    );
    assert!(
        at("temporal") > 0,
        "the temporal step charges its clause tables at its nodes"
    );
    assert!(
        at("protocol") > 0,
        "the protocol step charges its tables at its nodes"
    );
    let first = requests.first().expect("requests");
    assert_eq!(first.index, 0, "the index table is charged at node 0");
    assert!(requests
        .iter()
        .all(|request| request.stage == PackageReadStage::Graph));
}

/// The canonical key of an id: bytes whose lexicographic order is the id
/// order. A string is its bytes with `0x00` written `0x00 0xFF` and ended
/// by `0x00 0x01`; a segment is a kind tag (name 1, index 2, operand 3,
/// node 4) and its value, a nested node being its own whole key; a path
/// ends with tag 0; the ordinal comes last.
fn canonical_key(id: &quire_contract_model::CheckedNodeId, out: &mut Vec<u8>) {
    use quire_contract_model::CheckedNodePathSegment as Segment;
    fn text(value: &str, out: &mut Vec<u8>) {
        for byte in value.bytes() {
            out.push(byte);
            if byte == 0 {
                out.push(0xFF);
            }
        }
        out.extend([0, 1]);
    }
    text(&id.package, out);
    for segment in &id.path {
        match segment {
            Segment::Name(name) => {
                out.push(1);
                text(name, out);
            }
            Segment::Index(index) => {
                out.push(2);
                out.extend(index.to_be_bytes());
            }
            Segment::Operand(slot) => {
                out.push(3);
                out.extend(slot.to_be_bytes());
            }
            Segment::Node(node) => {
                out.push(4);
                canonical_key(node, out);
            }
            Segment::Dependency { package, node } => {
                out.push(5);
                text(package, out);
                out.extend(node.to_be_bytes());
            }
        }
    }
    out.push(0);
    out.extend(id.ordinal.to_be_bytes());
}

/// On the structural keys of every node of real packages, the id order is
/// the order of the canonical key's bytes (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn node_id_order_equals_canonical_key_byte_order_on_real_packages() {
    use quire_contract_model::CheckedPackageV2ReadResult;
    let mut ids = Vec::new();
    let wires = [
        v2_all_families(),
        crate::checked_package_v2_map_type::nested_bounded_quantity_wire().0,
    ];
    for value in &wires {
        let result = crate::read_and_account(
            &canonical(value),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(value),
        );
        let CheckedPackageV2ReadResult::Admitted(package) = result else {
            panic!("fixture admits");
        };
        for at in 0..package.graph().nodes.len() {
            ids.push(package.structural_key_at(at).expect("key").clone());
        }
    }
    assert!(ids.len() > 10, "{} ids", ids.len());
    let keys: Vec<Vec<u8>> = ids
        .iter()
        .map(|id| {
            let mut key = Vec::new();
            canonical_key(id, &mut key);
            key
        })
        .collect();
    for (left, left_key) in ids.iter().zip(&keys) {
        for (right, right_key) in ids.iter().zip(&keys) {
            assert_eq!(
                left.cmp(right),
                left_key.cmp(right_key),
                "{left:?} {right:?}"
            );
        }
    }
}

/// A full read of packages whose structural keys nest far past the inline
/// comparison stack performs no uncharged `CheckedNodeId::cmp` spill: the
/// debug guard panics on this thread at the first plain comparison that
/// would allocate (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn deeply_nested_keys_read_without_a_plain_comparison_spill() {
    use quire_contract_model::CheckedPackageV2ReadResult;
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            std::env::remove_var("QUIRE_FORBID_PLAIN_CMP_SPILL_ON");
        }
    }
    let name = std::thread::current()
        .name()
        .expect("test threads are named")
        .to_owned();
    // The guard is armed: two separately read 40-level chains of keys compare
    // past the inline stack, and the plain comparison panics on this thread.
    let (wire, maps, _) = crate::checked_package_v2_map_type::map_chain_wire(40);
    let read_key = || {
        let CheckedPackageV2ReadResult::Admitted(package) = crate::read_and_account(
            &canonical(&wire),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&wire),
        ) else {
            panic!("chain admits");
        };
        package
            .structural_key_at(maps[39])
            .expect("deepest map key")
            .clone()
    };
    let (left, right) = (read_key(), read_key());
    std::env::set_var("QUIRE_FORBID_PLAIN_CMP_SPILL_ON", &name);
    let _armed = Guard;
    let spilled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| left.cmp(&right)));
    assert!(spilled.is_err(), "a deep plain comparison trips the guard");
    for depth in [40_usize, 120] {
        let (wire, maps, _) = crate::checked_package_v2_map_type::map_chain_wire(depth);
        assert_eq!(maps.len(), depth);
        std::env::set_var("QUIRE_FORBID_PLAIN_CMP_SPILL_ON", &name);
        let _guard = Guard;
        let result = crate::read_and_account(
            &canonical(&wire),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&wire),
        );
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "depth {depth} admits: {result:?}"
        );
    }
}
