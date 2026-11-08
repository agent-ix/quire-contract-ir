// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Allocation proof at the original driver's public read boundary. The driver
//! retains its input construction, limits, result type and semantic oracle.

#[cfg(feature = "test-support")]
#[path = "native_budget_trace.rs"]
mod native_budget_trace;
#[path = "read_observation.rs"]
mod observation;
use observation::{accounted, drop_without_request};
pub(crate) use observation::{measure_at, Counter, Observation};
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
    #[cfg(feature = "test-support")]
    let (observation, _events) =
        native_budget_trace::checked::<DriverCounter>(bytes, evidence, limits, work, storage);
    #[cfg(not(feature = "test-support"))]
    let observation = measure_at::<DriverCounter>(bytes, evidence, limits, work, storage);
    accounted("original driver read", &observation);
    observation
}

/// A checked read from an exact caller-state snapshot, not reconstructed totals.
#[cfg(feature = "test-support")]
fn measure_seeded(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    mut budget: quire_contract_model::NativeJoinBudget,
) -> Observation {
    let before = budget.read_storage_consumed();
    let logical_before = budget.consumed().storage;
    let mut result = None;
    let requested = <DriverCounter as Counter>::measure(|| {
        result = Some(CheckedPackageV2::read_with_budget(
            bytes,
            limits,
            evidence,
            &mut budget,
        ));
    });
    assert_eq!(
        budget.consumed().storage,
        logical_before,
        "seeded replay consumes no new logical storage"
    );
    assert_eq!(
        requested,
        budget
            .read_storage_consumed()
            .checked_sub(before)
            .expect("monotone caller consumption"),
        "seeded checked public read pays every actual allocation request"
    );
    Observation {
        result: result.expect("seeded read ran"),
        budget,
        requested,
    }
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

#[cfg(not(feature = "test-support"))]
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

#[cfg(not(feature = "test-support"))]
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

#[cfg(feature = "test-support")]
fn native_event_json(event: &quire_contract_model::NativeBudgetEvent) -> serde_json::Value {
    serde_json::json!({
        "resource": format!("{:?}", event.resource), "amount": event.amount,
        "consumed_before": event.consumed_before, "admitted": event.admitted,
        "allocation_class": event.allocation_class,
        "locus": event.locus.map(|locus| serde_json::json!({
            "source":format!("{:?}",locus.source), "form":locus.form
        }))
    })
}

/// These endpoints are selected from actual events, never inferred from byte
/// amounts or a fixed number of stages. Designated growth cases additionally
/// supply source-grounded exhaustive classes at their original provider site.
#[cfg(feature = "test-support")]
fn full_native_boundaries(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    reference: &Observation,
    exhaustive_classes: &[&str],
    seed: Option<&quire_contract_model::NativeJoinBudget>,
) -> (Vec<serde_json::Value>, Vec<serde_json::Value>) {
    use quire_contract_model::{NativeBudgetResource as Resource, NativeJoinLocusSource};
    use std::collections::{BTreeMap, BTreeSet};
    let (observed, events) = if let Some(seed) = seed {
        native_budget_trace::checked_seeded(bytes, evidence, limits, seed)
    } else {
        native_budget_trace::checked::<DriverCounter>(
            bytes,
            evidence,
            limits,
            limits.work,
            u64::MAX,
        )
    };
    same_observation(reference, &observed);
    assert_eq!(
        reference.budget, observed.budget,
        "entire original baseline state repeats"
    );
    drop_without_request::<DriverCounter>(observed);
    let mut stage_last = BTreeMap::new();
    let mut class_ends = BTreeMap::new();
    let mut selected = BTreeSet::new();
    let mut seen_exhaustive = BTreeSet::new();
    for (index, event) in events.iter().enumerate() {
        if !event.admitted || event.amount == 0 {
            continue;
        }
        let resource = match event.resource {
            Resource::Work => 0,
            Resource::ReadStorage => 1,
            Resource::LogicalStorage => panic!("reader admitted native logical storage"),
        };
        let locus = event.locus.expect("actual native event has reader locus");
        let NativeJoinLocusSource::PackageRead(stage, _) = locus.source else {
            panic!("actual native event is outside strict-reader stage context")
        };
        if resource == 1 {
            let class = event
                .allocation_class
                .expect("full proof needs genuine source-derived reservation class");
            if exhaustive_classes.contains(&class) {
                selected.insert(index);
                seen_exhaustive.insert(class);
            }
        }
        stage_last.insert((resource, stage), index);
        let key = (resource, stage, locus.form, event.allocation_class);
        class_ends
            .entry(key)
            .and_modify(|ends: &mut (usize, usize)| ends.1 = index)
            .or_insert((index, index));
    }
    assert_eq!(
        seen_exhaustive,
        exhaustive_classes.iter().copied().collect::<BTreeSet<_>>(),
        "every designated source class emitted actual reservations"
    );
    selected.extend(stage_last.into_values());
    for (first, last) in class_ends.into_values() {
        selected.insert(first);
        selected.insert(last);
    }
    let mut results = Vec::with_capacity(selected.len());
    for index in selected {
        let event = events[index];
        let bound = event
            .consumed_before
            .checked_add(u64::try_from(event.amount).expect("admitted amount fits u64"))
            .expect("admitted bound fits u64");
        let probe = |bound| {
            let (work, storage) = match event.resource {
                Resource::Work => (
                    bound,
                    seed.map_or(u64::MAX, |seed| seed.read_storage_limit()),
                ),
                Resource::ReadStorage => {
                    (seed.map_or(limits.work, |seed| seed.work_limit()), bound)
                }
                Resource::LogicalStorage => unreachable!("logical storage rejected"),
            };
            if let Some(seed) = seed {
                let rebound = seed
                    .read_proof_snapshot()
                    .read_proof_rebound(work, storage, false);
                native_budget_trace::checked_seeded(bytes, evidence, limits, &rebound)
            } else {
                native_budget_trace::checked::<DriverCounter>(
                    bytes, evidence, limits, work, storage,
                )
            }
        };
        let (exact, exact_events) = probe(bound);
        assert_eq!(
            exact_events.get(..=index),
            events.get(..=index),
            "exact bound admits the same entire event prefix"
        );
        let (short, short_events) = probe(bound - 1);
        assert_eq!(
            short_events.get(..index),
            events.get(..index),
            "one below preserves every earlier native event"
        );
        let mut denied = event;
        denied.admitted = false;
        assert_eq!(
            short_events.get(index),
            Some(&denied),
            "one below denies this actual amount/locus/class"
        );
        let (repeated, repeated_events) = probe(bound - 1);
        same_observation(&short, &repeated);
        assert_eq!(
            short_events, repeated_events,
            "full passive denial trace repeats"
        );
        let mut row = native_event_json(&event);
        let object = row.as_object_mut().expect("event object");
        object.insert("event_index".into(), index.into());
        object.insert("exact_bound".into(), bound.into());
        object.insert("one_below_bound".into(), (bound - 1).into());
        for key in [
            "exact_event_admitted",
            "one_below_event_denied",
            "full_repeat_equal",
            "accounting_equal",
        ] {
            object.insert(key.into(), true.into());
        }
        results.push(row);
        for observation in [exact, short, repeated] {
            drop_without_request::<DriverCounter>(observation);
        }
    }
    (events.iter().map(native_event_json).collect(), results)
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
            std::env::var("IR1404_ORIGINAL_TEST_NAME").unwrap_or_else(|_| {
                std::thread::current()
                    .name()
                    .unwrap_or("unnamed")
                    .to_owned()
            }),
        )
    });
    // The first public read is measured on a fresh worker. Only worker
    // startup/join is outside the caller's enclosing allocation measurement;
    // the worker's own thread-local allocator counter remains fully armed.
    let reference = crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name(format!(
                    "ir1404-first:{}:{} / {}",
                    location.file(),
                    location.line(),
                    test_name
                ))
                .spawn_scoped(scope, || {
                    observe(bytes, limits, evidence, limits.work, u64::MAX)
                })
                .expect("start first measured driver worker")
                .join()
                .expect("first measured driver read")
        })
    });
    let plain = CheckedPackageV2::read(bytes, limits, evidence);
    crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            let plain = &plain;
            std::thread::Builder::new()
                .name(format!(
                    "ir1404-proof:{}:{} / {}",
                    location.file(),
                    location.line(),
                    test_name
                ))
                .spawn_scoped(scope, move || {
                    finish_proof(
                        bytes, limits, evidence, reference, plain, mode, test_name, location,
                    )
                })
                .expect("start driver accounting worker")
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
        if reference.budget.first_unpaid().is_none() {
            assert!(
                short.budget.first_unpaid_read_storage().is_some(),
                "without an earlier work stop, one below storage denies a reservation"
            );
        } else {
            assert!(
                short.budget.first_unpaid_read_storage().is_some()
                    || short.budget.first_unpaid().is_some(),
                "the original finite work cap or the smaller storage cap stops the read"
            );
        }
        assert!(
            short.budget.first_unpaid_locus().is_some(),
            "the actual first unpaid charge retains its locus"
        );
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
    #[cfg(feature = "test-support")]
    let (native_events, boundary_results) = if mode == "full" {
        // Original QSpec calls use stage/class endpoints. Designated adverse
        // providers must explicitly request all intermediate growth events.
        full_native_boundaries(bytes, limits, evidence, &reference, &[], None)
    } else {
        (Vec::new(), Vec::new())
    };
    #[cfg(feature = "test-support")]
    let (storage_boundaries, work_boundaries) = (
        boundary_results
            .iter()
            .filter(|row| row["resource"] == "ReadStorage")
            .count(),
        boundary_results
            .iter()
            .filter(|row| row["resource"] == "Work")
            .count(),
    );
    #[cfg(not(feature = "test-support"))]
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
    let row = serde_json::json!({"driver":location.file(),"line":location.line(),"column":location.column(),
        "test":test_name,"sequence":sequence,"mode":mode,"outcome":kind,
        "input_bytes":bytes.len(),"work_limit":limits.work,"work":reference.budget.consumed().work,
        "read_storage":storage,"storage_boundaries":storage_boundaries,"work_boundaries":work_boundaries});
    #[cfg(feature = "test-support")]
    let row = {
        let mut row = row;
        if mode == "full" {
            row["proof_schema"] = "native-budget-events/v1".into();
            row["native_budget_events"] = native_events.into();
            row["boundary_results"] = boundary_results.into();
        }
        row
    };
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
    #[cfg(feature = "test-support")]
    let mode = crate::without_qspec_instrumentation(|| {
        let mode = std::env::var("IR1404_READ_PROOF_MODE").unwrap_or_else(|_| "sample".into());
        assert!(
            matches!(mode.as_str(), "sample" | "full"),
            "unknown caller read-proof mode"
        );
        mode
    });
    #[cfg(feature = "test-support")]
    let seed = budget.read_proof_snapshot();
    #[cfg(feature = "test-support")]
    let reference = crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name(format!(
                    "ir1404-seeded-first:{}:{}",
                    location.file(),
                    location.line()
                ))
                .spawn_scoped(scope, || {
                    native_budget_trace::checked_seeded(bytes, evidence, limits, &seed).0
                })
                .expect("start exact caller-state measured worker")
                .join()
                .expect("first exact caller-state measured read")
        })
    });
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
    #[cfg(feature = "test-support")]
    let (native_events, boundary_results) = crate::without_qspec_instrumentation(|| {
        std::thread::scope(|scope| {
            let original = result.as_ref().expect("original caller read ran");
            let full = mode == "full";
            let final_budget = &*budget;
            scope
                .spawn(move || {
                    assert_eq!(
                        &reference.result, original,
                        "original caller returns the complete snapshot-replay outcome"
                    );
                    assert_eq!(
                        &reference.budget, final_budget,
                        "original caller retains the complete cumulative snapshot state"
                    );
                    assert_eq!(
                        reference.requested, requested,
                        "original caller requests match exact-state replay"
                    );
                    let (repeated, _events) =
                        native_budget_trace::checked_seeded(bytes, evidence, limits, &seed);
                    same_observation(&reference, &repeated);
                    assert_eq!(
                        reference.budget, repeated.budget,
                        "entire repeated caller state"
                    );
                    drop_without_request::<DriverCounter>(repeated);
                    let evidence_rows = if full {
                        full_native_boundaries(
                            bytes,
                            limits,
                            evidence,
                            &reference,
                            &[],
                            Some(&seed),
                        )
                    } else {
                        (Vec::new(), Vec::new())
                    };
                    drop_without_request::<DriverCounter>(reference);
                    evidence_rows
                })
                .join()
                .expect("exact caller-state repeat proof")
        })
    });
    crate::without_qspec_instrumentation(|| {
        let log = std::env::var_os("IR1404_READ_PROOF_LOG")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!("ir1404-read-sites-{}.jsonl", std::process::id()))
            });
        let row = serde_json::json!({"driver":location.file(),"line":location.line(),"column":location.column(),
            "test":std::env::var("IR1404_ORIGINAL_TEST_NAME").unwrap_or_else(|_| std::thread::current().name().unwrap_or("unnamed").to_owned()),"entry":"caller-budget",
            "input_bytes":bytes.len(),"work_limit":budget.work_limit(),
            "storage_limit":budget.read_storage_limit(),"requested":requested});
        #[cfg(feature = "test-support")]
        let row = {
            let mut row = row;
            row["mode"] = mode.into();
            row["read_storage"] = requested.into();
            row["prior_read_storage"] = before.into();
            row["prior_logical_storage"] = logical_before.into();
            row["snapshot_replay"] = "exact-entire-caller-state".into();
            // Move/destroy every worker-owned evidence allocation only while
            // caller capture is suspended, including sample's empty vectors.
            row["native_budget_events"] = native_events.into();
            row["boundary_results"] = boundary_results.into();
            if row["mode"] == "full" {
                row["proof_schema"] = "native-budget-events/v1".into();
            }
            row
        };
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
