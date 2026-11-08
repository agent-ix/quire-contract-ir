// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Allocation proof at the original driver's public read boundary. The driver
//! retains its input construction, limits, result type and semantic oracle.

#[path = "read_observation.rs"]
mod observation;
use observation::{accounted, drop_without_request, measure_at, Counter, Observation};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult as Budgeted, CheckedPackageV2ReadResult as Plain,
    PackageReadResource,
};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

struct DriverCounter;
impl Counter for DriverCounter {
    fn measure(read: impl FnOnce()) -> u64 {
        crate::measure_qspec_read_requests(read)
    }
}

fn observe(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    work: u64,
    storage: u64,
) -> Observation {
    let observation = measure_at::<DriverCounter>(bytes, evidence, limits, work, storage);
    accounted("original driver read", &observation);
    observation
}

fn same_observation(left: &Observation, right: &Observation) {
    assert_eq!(
        left.result, right.result,
        "complete repeated budgeted outcome"
    );
    assert_eq!(
        left.requested, right.requested,
        "repeated allocation requests"
    );
    assert_eq!(
        left.budget.consumed(),
        right.budget.consumed(),
        "repeated work/logical storage"
    );
    assert_eq!(
        left.budget.read_storage_consumed(),
        right.budget.read_storage_consumed()
    );
    assert_eq!(left.budget.first_unpaid(), right.budget.first_unpaid());
    assert_eq!(
        left.budget.first_unpaid_read_storage(),
        right.budget.first_unpaid_read_storage()
    );
    assert_eq!(
        left.budget.first_unpaid_locus(),
        right.budget.first_unpaid_locus()
    );
}

fn same_public_result(plain: &Plain, measured: &Observation, limits: CheckedPackageReadLimits) {
    match (plain, &measured.result) {
        (Plain::Admitted(a), Budgeted::Admitted(b)) => assert_eq!(a, b),
        (Plain::Refused(a), Budgeted::Refused(b)) => assert_eq!(a, b),
        (Plain::Incomplete(a), Budgeted::Incomplete(b)) => assert_eq!(a, b),
        (Plain::CatalogUnavailable(a), Budgeted::CatalogUnavailable(b)) => assert_eq!(a, b),
        (Plain::Incomplete(a), Budgeted::BudgetIncomplete(b)) => {
            assert_eq!(
                b.denied.resource,
                PackageReadResource::Work,
                "unbounded read storage cannot stop the ordinary entry"
            );
            assert_eq!(a.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(a.limit, limits.work);
            assert_eq!(a.consumed, b.consumed);
            assert_eq!(a.path, b.path);
        }
        (a, b) => panic!("original and measured public reads differ: {a:?} / {b:?}"),
    }
}

fn full_storage_boundaries(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    reference: &Observation,
) -> usize {
    let total = reference.budget.read_storage_consumed();
    let mut before = 0_u64;
    let mut boundaries = 0;
    while before < total {
        let stopped = observe(bytes, limits, evidence, limits.work, before);
        let asked = stopped
            .budget
            .first_unpaid_read_storage()
            .expect("next actual reservation");
        let locus = stopped
            .budget
            .first_unpaid_locus()
            .expect("first unpaid locus");
        assert_eq!(
            stopped.requested, before,
            "no request after an unpaid reservation"
        );
        let next = before
            .checked_add(u64::try_from(asked).expect("bounded reservation"))
            .expect("bounded read total");
        assert!(
            next > before && next <= total,
            "actual reservation stays within reference total"
        );
        let short = observe(bytes, limits, evidence, limits.work, next - 1);
        assert_eq!(short.budget.first_unpaid_read_storage(), Some(asked));
        assert_eq!(short.budget.first_unpaid_locus(), Some(locus));
        let repeated = observe(bytes, limits, evidence, limits.work, next - 1);
        same_observation(&short, &repeated);
        for result in [stopped, short, repeated] {
            drop_without_request::<DriverCounter>(result);
        }
        before = next;
        boundaries += 1;
    }
    boundaries
}

fn full_work_boundaries(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    reference: &Observation,
) -> usize {
    let total = reference.budget.consumed().work;
    let mut before = 0_u64;
    let mut boundaries = 0;
    while before < total {
        let stopped = observe(bytes, limits, evidence, before, u64::MAX);
        let denied = stopped
            .budget
            .first_unpaid()
            .expect("next actual work charge");
        assert_eq!(
            denied.kind,
            quire_contract_model::NativeJoinChargeKind::Work
        );
        assert_eq!(stopped.budget.consumed().work, before);
        let next = before
            .checked_add(u64::try_from(denied.amount).expect("bounded work charge"))
            .expect("bounded work total");
        assert!(next > before && next <= total);
        let short = observe(bytes, limits, evidence, next - 1, u64::MAX);
        assert_eq!(short.budget.first_unpaid(), Some(denied));
        assert_eq!(
            short.budget.first_unpaid_locus(),
            stopped.budget.first_unpaid_locus()
        );
        let repeated = observe(bytes, limits, evidence, next - 1, u64::MAX);
        same_observation(&short, &repeated);
        for result in [stopped, short, repeated] {
            drop_without_request::<DriverCounter>(result);
        }
        before = next;
        boundaries += 1;
    }
    boundaries
}

/// Run proof probes serially on scoped workers so an enclosing original
/// allocation oracle observes exactly its one original read, without probe,
/// logging, thread startup or cross-thread-drop instrumentation overhead.
#[track_caller]
pub(crate) fn read_and_account(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> Plain {
    let location = std::panic::Location::caller();
    let (mode, test_name) = crate::without_qspec_instrumentation(|| {
        let mode = std::env::var("IR1404_READ_PROOF_MODE").unwrap_or_else(|_| "sample".into());
        assert!(
            matches!(mode.as_str(), "sample" | "full"),
            "unknown read-proof mode {mode}"
        );
        (
            mode,
            std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_owned(),
        )
    });
    // The first public read is measured on a fresh worker. Only worker
    // startup/join is outside the caller's enclosing allocation measurement;
    // the worker's own thread-local allocator counter remains fully armed.
    let reference = crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            scope
                .spawn(|| observe(bytes, limits, evidence, limits.work, u64::MAX))
                .join()
                .expect("first measured driver read")
        })
    });
    let plain = CheckedPackageV2::read(bytes, limits, evidence);
    crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            let plain = &plain;
            scope
                .spawn(move || {
                    finish_proof(
                        bytes, limits, evidence, reference, plain, mode, test_name, location,
                    )
                })
                .join()
                .expect("driver accounting proof")
        })
    });
    plain
}

fn finish_proof(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    reference: Observation,
    plain: &Plain,
    mode: String,
    test_name: String,
    location: &'static std::panic::Location<'static>,
) {
    same_public_result(plain, &reference, limits);
    let storage = reference.budget.read_storage_consumed();
    let work = if reference.budget.first_unpaid().is_some() {
        limits.work
    } else {
        reference.budget.consumed().work
    };
    for (work, storage) in [
        (limits.work, u64::MAX),
        (limits.work, storage),
        (work, storage),
    ] {
        let repeated = observe(bytes, limits, evidence, work, storage);
        same_observation(&reference, &repeated);
        drop_without_request::<DriverCounter>(repeated);
    }
    if storage > 0 {
        let short = observe(bytes, limits, evidence, limits.work, storage - 1);
        assert!(short.budget.first_unpaid_read_storage().is_some());
        assert!(!matches!(short.result, Budgeted::Admitted(_)));
        let repeated = observe(bytes, limits, evidence, limits.work, storage - 1);
        same_observation(&short, &repeated);
        drop_without_request::<DriverCounter>(short);
        drop_without_request::<DriverCounter>(repeated);
    }
    if work > 0 {
        let short = observe(bytes, limits, evidence, work - 1, u64::MAX);
        assert!(
            short.budget.first_unpaid().is_some(),
            "one below the read's work requirement stops"
        );
        let repeated = observe(bytes, limits, evidence, work - 1, u64::MAX);
        same_observation(&short, &repeated);
        drop_without_request::<DriverCounter>(short);
        drop_without_request::<DriverCounter>(repeated);
    }
    let (storage_boundaries, work_boundaries) = if mode == "full" {
        (
            full_storage_boundaries(bytes, limits, evidence, &reference),
            full_work_boundaries(bytes, limits, evidence, &reference),
        )
    } else {
        (0, 0)
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let kind = match &plain {
        Plain::Admitted(_) => "admitted",
        Plain::Refused(_) => "refused",
        Plain::Incomplete(_) => "incomplete",
        Plain::CatalogUnavailable(_) => "catalog-unavailable",
    };
    let log = std::env::var_os("IR1404_READ_PROOF_LOG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("ir1404-read-sites-{}.jsonl", std::process::id()))
        });
    let row = serde_json::json!({"driver":location.file(),"line":location.line(),
        "test":test_name,"sequence":sequence,"mode":mode,"outcome":kind,
        "input_bytes":bytes.len(),"work_limit":limits.work,"work":reference.budget.consumed().work,
        "read_storage":storage,"storage_boundaries":storage_boundaries,"work_boundaries":work_boundaries});
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .expect("temporary read proof log");
    writeln!(output, "{row}").expect("temporary read proof record");
    drop_without_request::<DriverCounter>(reference);
}

#[cfg(test)]
mod tests {
    use super::{Counter, DriverCounter};

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn worker_counts_full_reallocation_while_caller_ignores_only_instrumentation() {
        let mut worker_requested = 0;
        let caller_requested = crate::measure_qspec_read_requests(|| {
            worker_requested = crate::without_qspec_instrumentation(|| {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| {
                            <DriverCounter as Counter>::measure(|| {
                                let mut value = Vec::<u8>::with_capacity(8);
                                value.extend([0; 8]);
                                value.reserve_exact(24);
                                drop(value);
                            })
                        })
                        .join()
                        .expect("worker allocation control")
                })
            });
        });
        assert_eq!(
            worker_requested, 40,
            "initial 8 plus full 32-byte replacement"
        );
        assert_eq!(
            caller_requested, 0,
            "only worker setup/proof work was excluded from caller capture"
        );
    }
}

/// Observe an original caller-budget probe once, without replacing its
/// cumulative budget state or changing the driver's existing boundary sweep.
#[track_caller]
pub(crate) fn read_budget_and_account(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    budget: &mut quire_contract_model::NativeJoinBudget,
) -> Budgeted {
    let location = std::panic::Location::caller();
    let before = budget.read_storage_consumed();
    let logical_before = budget.consumed().storage;
    let mut result = None;
    let requested = <DriverCounter as Counter>::measure(|| {
        result = Some(CheckedPackageV2::read_with_budget(
            bytes, limits, evidence, budget,
        ));
    });
    assert_eq!(
        requested,
        budget.read_storage_consumed() - before,
        "the actual caller-budget read charges every request including its diagnostic"
    );
    assert_eq!(
        budget.consumed().storage,
        logical_before,
        "read storage remains separate from native logical storage"
    );
    crate::without_qspec_instrumentation(|| {
        let log = std::env::var_os("IR1404_READ_PROOF_LOG")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!("ir1404-read-sites-{}.jsonl", std::process::id()))
            });
        let row = serde_json::json!({"driver":location.file(),"line":location.line(),
            "test":std::thread::current().name(),"entry":"caller-budget",
            "input_bytes":bytes.len(),"work_limit":budget.work_limit(),
            "storage_limit":budget.read_storage_limit(),"requested":requested});
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .expect("temporary caller-budget proof log");
        writeln!(output, "{row}").expect("temporary caller-budget proof record");
    });
    result.expect("the original caller-budget read ran once")
}

#[cfg(test)]
mod caller_budget_tests {
    use super::{Counter, DriverCounter};
    use quire_contract_model::{
        CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, NativeJoinBudget,
        NativeJoinChargeKind,
    };

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
    #[test]
    fn caller_budget_observation_preserves_prior_charges_and_outer_capture() {
        let seeded = || {
            let mut budget =
                NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(u64::MAX);
            let locus = quire_contract_model::NativeJoinLocus {
                source: quire_contract_model::NativeJoinLocusSource::Package,
                form: "prior_caller_charge",
            };
            budget
                .accounted_charge(NativeJoinChargeKind::Work, 3, locus)
                .expect("prior work");
            budget
                .accounted_charge(NativeJoinChargeKind::Storage, 2, locus)
                .expect("prior logical storage");
            budget
        };
        let evidence = CheckedPackageEvidence::new();
        let limits = CheckedPackageReadLimits::bounded();
        let mut observed_budget = seeded();
        let mut observed = None;
        let requests = <DriverCounter as Counter>::measure(|| {
            observed = Some(super::read_budget_and_account(
                b"not json",
                limits,
                &evidence,
                &mut observed_budget,
            ));
        });
        let mut original_budget = seeded();
        let mut original = None;
        let expected = <DriverCounter as Counter>::measure(|| {
            original = Some(CheckedPackageV2::read_with_budget(
                b"not json",
                limits,
                &evidence,
                &mut original_budget,
            ));
        });
        assert_eq!(
            observed, original,
            "the original typed outcome is returned unchanged"
        );
        assert_eq!(
            observed_budget, original_budget,
            "the original cumulative budget is unchanged by observation"
        );
        assert_eq!(
            requests, expected,
            "only one actual read reaches the caller's counter"
        );
        assert!(requests > 0, "the real catalogue and parser were observed");
    }
}
