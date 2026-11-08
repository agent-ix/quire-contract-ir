// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The budget-charged strict V2 read: entries, deltas, the read-storage
//! counter's API and the envelope stages' exact charging (TC-472).

use crate::support::checked_package::{
    all_families_read_work, canonical, evidence_for, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, read_checked_package_with_budget,
    CheckedPackageBudgetedDispatchResult, CheckedPackageEvidence, CheckedPackageLimit,
    CheckedPackagePendingOutcome, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult, NativeJoinBudget,
    NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource, PackageReadResource,
    PackageReadStage,
};
use serde_json::{json, Value};

fn bytes() -> Vec<u8> {
    canonical(&v2_all_families())
}

fn read_with(budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    let value = v2_all_families();
    CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        budget,
    )
}

fn envelope_locus(stage: PackageReadStage, form: &'static str) -> NativeJoinLocus {
    NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(stage, 0),
        form,
    }
}

/// The budgeted read admits what the unbudgeted read admits, with the work
/// delta the unbudgeted work boundary names and no logical storage.
#[trace("TC-472", "FR-038-AC-495", "FR-038-AC-496", "FR-038-AC-508")]
#[test]
fn budgeted_read_matches_the_unbudgeted_read_and_records_its_deltas() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let CheckedPackageV2ReadResult::Admitted(plain) =
        CheckedPackageV2::read(&bytes(), CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the fixture admits");
    };
    let mut budget = NativeJoinBudget::unlimited();
    let CheckedPackageV2BudgetedReadResult::Admitted(budgeted) = read_with(&mut budget) else {
        panic!("the fixture admits under an unlimited budget");
    };
    assert_eq!(*plain, *budgeted);
    let delta = budget
        .last_success_delta()
        .expect("admission records a delta");
    assert_eq!(delta.work, all_families_read_work());
    assert_eq!(delta.storage, 0);
    assert!(budget.last_success_read_storage_delta().expect("delta") > 0);
    assert_eq!(budget.first_unpaid_read_storage(), None);
}

/// One work unit below the requirement stops at the same value the unbudgeted
/// read reports, as a budget denial.
#[trace("TC-472", "FR-038-AC-495", "FR-038-AC-510")]
#[test]
fn one_below_the_work_requirement_is_a_work_budget_incomplete_matching_the_plain_read() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let work = all_families_read_work();
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work - 1;
    let CheckedPackageV2ReadResult::Incomplete(plain) =
        CheckedPackageV2::read(&bytes(), limits, &evidence)
    else {
        panic!("one below the requirement is incomplete");
    };
    assert_eq!(plain.limit_kind, CheckedPackageLimit::Work);
    let mut budget = NativeJoinBudget::new(work - 1, 0).with_read_storage_limit(u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(denied) = read_with(&mut budget)
    else {
        panic!("one below the requirement is a budget denial");
    };
    assert_eq!(denied.denied.resource, PackageReadResource::Work);
    assert_eq!(denied.limit, work - 1);
    assert_eq!(denied.consumed, plain.consumed);
    assert_eq!(denied.path, plain.path);
    assert_eq!(budget.last_success_delta(), None);
    assert_eq!(budget.last_success_read_storage_delta(), None);
}

/// The read's reservations are exact: the measured requirement admits, one
/// byte below is denied at the last request, at its locus (the retained
/// tables built over the declaration index, the last stage to request heap), with the
/// bytes asked.
#[trace("TC-472", "FR-038-AC-497", "FR-038-AC-498", "FR-358-AC-13")]
#[test]
fn read_storage_one_below_the_requirement_is_denied_at_the_last_request() {
    let mut measure = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_with(&mut measure),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let required = measure.read_storage_consumed();
    let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
    assert!(matches!(
        read_with(&mut exact),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(exact.read_storage_consumed(), required);

    let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
    let result = read_with(&mut short);
    let asked = short
        .first_unpaid_read_storage()
        .expect("a denial is recorded");
    assert!(asked > 0);
    assert_eq!(short.first_unpaid(), None);
    // The retained tables are the last requests the read makes.
    assert_eq!(
        short.first_unpaid_locus(),
        Some(NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0),
            form: "declarations",
        })
    );
    assert_eq!(short.read_storage_limit(), required - 1);
    assert_eq!(short.consumed().storage, 0);
    // The denied request belongs to a graph node's stage, so the stop is
    // incomplete at the node's body, which the budget can still name.
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) = result else {
        panic!("a denial at a node is incomplete at its body: {result:?}");
    };
    assert_eq!(incomplete.denied.resource, PackageReadResource::ReadStorage);
    assert_eq!(incomplete.denied.amount, asked);
    assert_eq!(incomplete.limit, required - 1);
    assert_eq!(
        incomplete.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/0/body")
    );
    assert!(short.read_storage_consumed() < required);
}

/// A zero read-storage limit is denied at the first allocation of the shared
/// reader. The outcome owns no diagnostic, so its diagnostic costs nothing and
/// the outcome is `BudgetIncomplete` (FR-038-AC-510 as amended); an input over
/// the byte limit is still `incomplete` for `bytes`.
#[trace("TC-472", "FR-038-AC-510", "FR-358-AC-14")]
#[test]
fn zero_read_storage_is_denied_at_the_first_request_and_bytes_precede_it() {
    let mut zero = NativeJoinBudget::new(u64::MAX, u64::MAX);
    assert_eq!(zero.read_storage_limit(), 0);
    // No allocation is owned by the outcome, so its diagnostic costs nothing.
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(denied) = read_with(&mut zero) else {
        panic!("an envelope denial with no pointer owns no diagnostic");
    };
    assert_eq!(denied.limit, 0);
    assert_eq!(denied.path, None);
    assert_eq!(
        zero.first_unpaid_locus(),
        Some(envelope_locus(PackageReadStage::Envelope, "package_bytes"))
    );
    assert_eq!(zero.read_storage_consumed(), 0);

    let value = v2_all_families();
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = 8;
    let mut budget = NativeJoinBudget::new(u64::MAX, 0);
    let CheckedPackageV2BudgetedReadResult::Incomplete(bytes) = CheckedPackageV2::read_with_budget(
        &canonical(&value),
        limits,
        &evidence_for(&value),
        &mut budget,
    ) else {
        panic!("over the byte limit is incomplete");
    };
    assert_eq!(bytes.limit_kind, CheckedPackageLimit::Bytes);
    assert_eq!(bytes.path, None);
    assert_eq!(budget.first_unpaid_read_storage(), None);
}

/// The read-storage counter's API: default zero limit, builder, unlimited
/// saturation, and the logical counter untouched by a read.
#[trace("TC-472", "FR-358-AC-14")]
#[test]
fn read_storage_counter_api_and_dispatch() {
    let fresh = NativeJoinBudget::new(5, 6);
    assert_eq!(fresh.read_storage_limit(), 0);
    assert_eq!(fresh.read_storage_consumed(), 0);
    assert_eq!(fresh.last_success_read_storage_delta(), None);
    assert_eq!(fresh.with_read_storage_limit(9).read_storage_limit(), 9);

    let mut unlimited = NativeJoinBudget::unlimited();
    let locus = envelope_locus(PackageReadStage::Graph, "x");
    for _ in 0..2 {
        assert!(unlimited
            .accounted_charge(NativeJoinChargeKind::Work, u128::from(u64::MAX), locus)
            .is_ok());
    }
    assert_eq!(unlimited.consumed().work, u64::MAX);
    assert_eq!(unlimited.first_unpaid(), None);

    let value = v2_all_families();
    let mut budget = NativeJoinBudget::new(u64::MAX, 7).with_read_storage_limit(u64::MAX);
    let CheckedPackageBudgetedDispatchResult::AdmittedV2(_) = read_checked_package_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        &mut budget,
    ) else {
        panic!("the dispatch admits");
    };
    assert_eq!(budget.consumed().storage, 0);
    assert_eq!(budget.storage_limit(), 7);
}

/// Two reads, each affordable alone, composed on one budget below their sum:
/// the first admits and the second stops at its first unpaid charge.
#[trace("TC-472", "FR-038-AC-496")]
#[test]
fn composed_reads_stop_the_second_at_its_first_unpaid_charge() {
    let mut measure = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_with(&mut measure),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let one = measure.read_storage_consumed();
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(one + one / 2);
    assert!(matches!(
        read_with(&mut budget),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(budget.read_storage_consumed(), one);
    let second = read_with(&mut budget);
    assert!(!matches!(
        second,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(budget.last_success_read_storage_delta(), None);
    assert!(budget.first_unpaid_read_storage().is_some());
}

fn small_package() -> Value {
    let id = json!({"package":"complete-native", "path":["scalar_type","boolean"], "ordinal":0});
    let source = json!({"authority":"test","identity":"complete-native"});
    json!({
        "contract_version":"quire.checked-package/v2", "name":"complete-native",
        "lock":{"sources":[source],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
            "profile_selections":[],"definition_selections":[],"required_features":[],"dependency_selections":[]},
        "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2","nodes":[{
            "node_id":id,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"scalar_type","semantic_form":"boolean",
            "semantic_type":id,"dependencies":[],"occurrences":[{"role":"type","ordinal":0}],"body":{"term":"aggregate","members":[]}
        }],"operand_rows":[[]]},
        "source_map":[{"node_id":id,"role":"type","ordinal":0,"regions":[{"source":source,"start":0,"end":1}]}],
        "capability_report":[],"diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]}
    })
}

fn read_bytes(bytes: &[u8], budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    CheckedPackageV2::read_with_budget(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
        budget,
    )
}

fn storage_budget(limit: u64) -> NativeJoinBudget {
    NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit)
}

/// Every reservation of the envelope and the canonical comparison is exact:
/// with the limit one byte short of any request boundary, the read is denied
/// at that request, the consumption is the previous boundary and the amount
/// asked is the difference, so each reader list class is the denied class at
/// the boundary where it is asked.
#[trace("TC-472", "FR-038-AC-497")]
#[test]
fn each_envelope_and_canonical_request_is_denied_exactly_one_byte_below() {
    let bytes = serde_json::to_vec(&small_package()).unwrap();
    let mut measure = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_bytes(&bytes, &mut measure),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let required = measure.read_storage_consumed();
    assert!(required > 0);
    let mut previous: Option<(u64, u64, PackageReadStage)> = None;
    let mut boundaries = std::collections::BTreeSet::new();
    let mut envelope_end = 0;
    for limit in 0..required {
        let mut budget = storage_budget(limit);
        let result = read_bytes(&bytes, &mut budget);
        assert!(
            !matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)),
            "limit {limit}"
        );
        let asked = u64::try_from(
            budget
                .first_unpaid_read_storage()
                .unwrap_or_else(|| panic!("limit {limit}: a denial is recorded")),
        )
        .unwrap();
        let consumed = budget.read_storage_consumed();
        assert!(
            consumed <= limit && consumed + asked > limit,
            "limit {limit}"
        );
        let stage = match budget.first_unpaid_locus().expect("denial locus").source {
            NativeJoinLocusSource::PackageRead(stage, 0) => stage,
            other => panic!("limit {limit}: {other:?}"),
        };
        if let Some((last_consumed, last_end, last_stage)) = previous {
            if consumed == last_consumed {
                assert_eq!(consumed + asked, last_end, "limit {limit}: same request");
            } else {
                assert_eq!(consumed, last_end, "limit {limit}: next request follows");
            }
            assert!(
                stage >= last_stage,
                "limit {limit}: stages are ordered: {last_stage:?} then {stage:?}"
            );
        }
        if stage == PackageReadStage::Envelope {
            envelope_end = consumed + asked;
        }
        boundaries.insert(consumed + asked);
        previous = Some((consumed, consumed + asked, stage));
    }
    assert_eq!(previous.map(|(_, end, _)| end), Some(required));
    assert!(envelope_end > 0 && envelope_end < required);
    // The envelope's last package-bytes request: one below its end is denied
    // there, at the envelope's locus.
    let mut budget = storage_budget(envelope_end - 1);
    let _ = read_bytes(&bytes, &mut budget);
    assert_eq!(
        budget.first_unpaid_locus(),
        Some(envelope_locus(PackageReadStage::Envelope, "package_bytes"))
    );
    assert_eq!(
        u128::from(budget.read_storage_consumed())
            + budget.first_unpaid_read_storage().expect("denial"),
        u128::from(envelope_end)
    );
    assert!(boundaries.len() >= 6, "{boundaries:?}");
}

/// The pointer of a duplicate-name refusal and the lexeme of a number outside
/// the double range are charged before the refusal is returned.
#[trace("TC-472", "FR-038-AC-497")]
#[test]
fn duplicate_name_and_out_of_range_number_diagnostics_are_charged_before_return() {
    for (bytes, code) in [
        (
            br#"{"a":1,"a":2}"#.as_slice(),
            quire_contract_model::CheckedPackageRefusalCode::DuplicateMember,
        ),
        (
            br#"{"a":1e999}"#.as_slice(),
            quire_contract_model::CheckedPackageRefusalCode::MalformedWire,
        ),
    ] {
        let mut measure = NativeJoinBudget::unlimited();
        let CheckedPackageV2BudgetedReadResult::Refused(refusal) = read_bytes(bytes, &mut measure)
        else {
            panic!("{code:?} refuses");
        };
        assert_eq!(refusal.code, code);
        let required = measure.read_storage_consumed();
        let mut exact = storage_budget(required);
        assert!(matches!(
            read_bytes(bytes, &mut exact),
            CheckedPackageV2BudgetedReadResult::Refused(_)
        ));
        assert_eq!(exact.read_storage_consumed(), required);
        let mut short = storage_budget(required - 1);
        assert!(
            !matches!(
                read_bytes(bytes, &mut short),
                CheckedPackageV2BudgetedReadResult::Refused(_)
            ),
            "{code:?}: one byte below is a budget denial"
        );
        assert!(short.first_unpaid_read_storage().is_some());
    }
}

/// The catalog gate charges nothing: two reads of the same bytes on fresh
/// budgets record equal work and read-storage deltas, whichever is the
/// process's first.
#[trace("TC-472", "FR-038-AC-515")]
#[test]
fn the_catalog_gate_charges_nothing_to_any_read() {
    assert!(initialize_operation_catalog().is_ok());
    let mut first = NativeJoinBudget::unlimited();
    let mut later = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_with(&mut first),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert!(matches!(
        read_with(&mut later),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(first.last_success_delta(), later.last_success_delta());
    assert_eq!(
        first.last_success_read_storage_delta(),
        later.last_success_read_storage_delta()
    );
    assert_eq!(first.consumed(), later.consumed());
}

/// A refusal's owned pointer is a diagnostic the read pays for. Exactly paid,
/// the refusal is returned; one byte short, the read is `Exhausted` with the
/// request that was denied, its bound, locus and pending refusal.
#[trace("TC-472", "FR-038-AC-510")]
#[test]
fn an_unaffordable_refusal_diagnostic_is_exhausted_with_the_pending_refusal() {
    let mut package = small_package();
    package["unknown_member"] = json!(1);
    let bytes = serde_json::to_vec(&package).unwrap();
    let mut measure = NativeJoinBudget::unlimited();
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) = read_bytes(&bytes, &mut measure)
    else {
        panic!("an unknown member refuses");
    };
    let pointer = refusal.loci[0]
        .path
        .as_ref()
        .expect("a pointer")
        .as_str()
        .len() as u64;
    assert!(pointer > 0);
    let paid = measure.read_storage_consumed() - pointer;
    let mut exact = storage_budget(paid + pointer);
    assert!(matches!(
        read_bytes(&bytes, &mut exact),
        CheckedPackageV2BudgetedReadResult::Refused(_)
    ));
    let mut short = storage_budget(paid + pointer - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = read_bytes(&bytes, &mut short)
    else {
        panic!("an unaffordable refusal diagnostic is exhausted");
    };
    assert_eq!(exhausted.denied.resource, PackageReadResource::ReadStorage);
    assert_eq!(exhausted.denied.amount, u128::from(pointer));
    assert_eq!(exhausted.configured_bound, paid + pointer - 1);
    assert_eq!(
        exhausted.locus,
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::WireDecode, 0),
            form: "diagnostic"
        }
    );
    assert_eq!(
        exhausted.pending,
        Some(CheckedPackagePendingOutcome::Refused(refusal.code))
    );
}

/// A work denial records the stage it happened in; when its pointer diagnostic
/// is unaffordable the read is `Exhausted` at that `PackageRead` locus, with no
/// pending refusal.
#[trace("TC-472", "FR-038-AC-510")]
#[test]
fn an_unaffordable_work_denial_diagnostic_is_exhausted_at_its_package_read_locus() {
    let work = all_families_read_work();
    let mut measure = NativeJoinBudget::new(work - 1, 0).with_read_storage_limit(u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(denied) = read_with(&mut measure)
    else {
        panic!("one below the work requirement is a budget denial");
    };
    let pointer = denied.path.as_ref().expect("a pointer").as_str().len() as u64;
    assert!(pointer > 0);
    let before_diagnostic = measure.read_storage_consumed() - pointer;
    let mut starved = NativeJoinBudget::new(work - 1, 0).with_read_storage_limit(before_diagnostic);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = read_with(&mut starved) else {
        panic!("an unaffordable work-denial diagnostic is exhausted");
    };
    assert_eq!(exhausted.denied.resource, PackageReadResource::Work);
    assert_eq!(exhausted.configured_bound, work - 1);
    assert!(
        matches!(
            exhausted.locus.source,
            NativeJoinLocusSource::PackageRead(stage, _) if stage > PackageReadStage::Canonical
        ),
        "{:?}",
        exhausted.locus
    );
    assert_eq!(exhausted.pending, None);
}

/// serde_json reads an object whose first member is its private number name as
/// a number, so that spelling would decode to the same package as the plain
/// number: the strict reader refuses it as noncanonical.
#[trace("TC-472", "FR-038-AC-509")]
#[test]
fn serde_json_number_token_spelling_is_noncanonical_where_the_number_admits() {
    // `source_map[0].regions[0].start` is a number the package admits.
    let read_with_start = |start: Value| {
        let mut package = small_package();
        package["source_map"][0]["regions"][0]["start"] = start;
        let mut budget = NativeJoinBudget::unlimited();
        read_bytes(&serde_json::to_vec(&package).unwrap(), &mut budget)
    };
    let plain = read_with_start(json!(0));
    assert!(
        matches!(plain, CheckedPackageV2BudgetedReadResult::Admitted(_)),
        "{plain:?}"
    );
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) =
        read_with_start(json!({"$serde_json::private::Number": "0"}))
    else {
        panic!("the token spelling must not read as a number");
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::NoncanonicalWire
    );
    assert_eq!(refusal.loci[0].path, None);
}

/// The shared reader's first-fault precedence on the V2 envelope: invalid
/// UTF-8 anywhere is first; a repeated member followed in its own object by a
/// truncation is `malformed_wire`; a repeated member in an object closed before
/// a later grammar fault is `duplicate_member`; every byte-stream fault
/// precedes `noncanonical_wire`.
#[trace("TC-472", "FR-038-AC-509")]
#[test]
fn the_shared_reader_orders_first_faults_on_the_v2_envelope() {
    use quire_contract_model::CheckedPackageRefusalCode as Code;
    let refused = |bytes: &[u8]| {
        let mut budget = NativeJoinBudget::unlimited();
        match read_bytes(bytes, &mut budget) {
            CheckedPackageV2BudgetedReadResult::Refused(refusal) => (
                refusal.code,
                refusal.loci[0].path.as_ref().map(|p| p.as_str().to_owned()),
            ),
            other => panic!(
                "{:?} must refuse: {other:?}",
                String::from_utf8_lossy(bytes)
            ),
        }
    };
    // Invalid UTF-8 after a repeated member still refuses malformed_wire first.
    assert_eq!(
        refused(b"{\"a\":1,\"a\":2,\"b\":\"\xff\"}"),
        (Code::MalformedWire, None)
    );
    // A repeated member and then truncation inside the same object.
    assert_eq!(refused(b"{\"a\":1,\"a\":2"), (Code::MalformedWire, None));
    // A repeated member in an object closed before a later fault.
    assert_eq!(
        refused(b"{\"x\":{\"a\":1,\"a\":2},\"y\":tru}"),
        (Code::DuplicateMember, Some("/x/a".to_owned()))
    );
    // Whitespace makes valid JSON non-canonical, after every grammar fault.
    assert_eq!(refused(b"{ }"), (Code::NoncanonicalWire, None));
}
