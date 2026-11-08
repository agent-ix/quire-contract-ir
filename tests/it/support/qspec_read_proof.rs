// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Allocation proof at the original driver's public read boundary. The driver
//! retains its input construction, limits, result type and semantic oracle.

#[cfg(feature = "test-support")]
#[path = "native_budget_trace.rs"]
mod native_budget_trace;
#[cfg(feature = "test-support")]
pub(crate) use native_budget_trace::checked as observe_native;
#[path = "read_observation.rs"]
pub(crate) mod observation;
use observation::{accounted, drop_without_request};
pub(crate) use observation::{measure_at, Counter, Observation};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult as Budgeted, CheckedPackageV2ReadResult as Plain,
    PackageReadResource,
};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy)]
struct ReadProvider {
    symbol: &'static str,
    classes: &'static [&'static str],
}
thread_local! {
    // Proof-provider metadata only, never an actual budget or allocation meter.
    static READ_PROVIDER: std::cell::Cell<Option<ReadProvider>> = const { std::cell::Cell::new(None) };
}
struct ProofContext {
    mode: String,
    test_name: String,
    location: &'static std::panic::Location<'static>,
    provider: Option<ReadProvider>,
}

struct RestoreProvider(Option<ReadProvider>);
impl Drop for RestoreProvider {
    fn drop(&mut self) {
        READ_PROVIDER.with(|provider| provider.set(self.0));
    }
}

/// The original fixture-producing test explicitly designates its growth
/// provider; actual charge events, not data sizes, decide which classes ran.
pub(crate) fn with_designated_read_provider<R>(
    symbol: &'static str,
    classes: &'static [&'static str],
    read: impl FnOnce() -> R,
) -> R {
    assert!(!classes.is_empty());
    assert!(classes
        .iter()
        .all(|class| matches!(*class, "walker_growth" | "exact_reservation")));
    let previous =
        READ_PROVIDER.with(|provider| provider.replace(Some(ReadProvider { symbol, classes })));
    let restore = RestoreProvider(previous);
    let result = read();
    drop(restore);
    result
}

fn provider_json(
    provider: ReadProvider,
    events: &[serde_json::Value],
    test: &str,
    location: &'static std::panic::Location<'static>,
) -> serde_json::Value {
    let requested: serde_json::Map<String, serde_json::Value> = provider
        .classes
        .iter()
        .map(|class| ((*class).to_owned(), (*class).into()))
        .collect();
    let reached: serde_json::Map<String, serde_json::Value> = requested
        .iter()
        .filter(|(class, _)| {
            events.iter().any(|event| {
                event["resource"] == "ReadStorage"
                    && event["admitted"] == true
                    && event["allocation_class"] == class.as_str()
            })
        })
        .map(|(class, role)| (class.clone(), role.clone()))
        .collect();
    serde_json::json!({"provider":provider.symbol,"test":test,"driver":location.file(),"line":location.line(),
                      "requested_classes":requested,"classes":reached})
}

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
        observe_native::<DriverCounter>(bytes, evidence, limits, work, storage);
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
        left.budget, right.budget,
        "entire repeated caller-budget state"
    );
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
fn budgeted_kind(result: &Budgeted) -> &'static str {
    match result {
        Budgeted::Admitted(_) => "admitted",
        Budgeted::Refused(_) => "refused",
        Budgeted::Incomplete(_) => "incomplete",
        Budgeted::BudgetIncomplete(_) => "budget-incomplete",
        Budgeted::Exhausted(_) => "exhausted",
        Budgeted::CatalogUnavailable(_) => "catalog-unavailable",
    }
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

#[cfg(feature = "test-support")]
fn assert_native_denial(
    observation: &Observation,
    event: quire_contract_model::NativeBudgetEvent,
    bound: u64,
) -> serde_json::Value {
    use quire_contract_model::NativeBudgetResource as Resource;
    let resource = match event.resource {
        Resource::Work => {
            let unpaid = observation
                .budget
                .first_unpaid()
                .expect("first unpaid work");
            assert_eq!(
                unpaid.kind,
                quire_contract_model::NativeJoinChargeKind::Work
            );
            assert_eq!(unpaid.amount, event.amount);
            assert!(observation.budget.first_unpaid_read_storage().is_none());
            PackageReadResource::Work
        }
        Resource::ReadStorage => {
            assert_eq!(
                observation.budget.first_unpaid_read_storage(),
                Some(event.amount)
            );
            assert!(observation.budget.first_unpaid().is_none());
            PackageReadResource::ReadStorage
        }
        Resource::LogicalStorage => panic!("reader consumed logical storage"),
    };
    assert_eq!(observation.budget.first_unpaid_locus(), event.locus);
    let outcome = match &observation.result {
        Budgeted::BudgetIncomplete(value) => {
            assert_eq!(value.denied.resource, resource);
            assert_eq!(value.denied.amount, event.amount);
            assert_eq!(value.limit, bound);
            assert_eq!(
                value.consumed,
                event
                    .consumed_before
                    .saturating_add(u64::try_from(event.amount).expect("admitted request fits"))
            );
            "budget-incomplete"
        }
        Budgeted::Exhausted(value) => {
            assert_eq!(value.denied.resource, resource);
            assert_eq!(value.denied.amount, event.amount);
            assert_eq!(value.configured_bound, bound);
            assert_eq!(Some(value.locus), event.locus);
            "exhausted"
        }
        _ => panic!("a denied native charge must return a typed budget outcome"),
    };
    serde_json::json!({"outcome":outcome,"resource":format!("{resource:?}"),"amount":event.amount,
        "configured_bound":bound,"locus":native_event_json(&event)["locus"],"entire_repeat_budget_equal":true})
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
) -> (
    Vec<serde_json::Value>,
    Vec<serde_json::Value>,
    serde_json::Value,
) {
    use quire_contract_model::{NativeBudgetResource as Resource, NativeJoinLocusSource};
    use std::collections::{BTreeMap, BTreeSet};
    let unlimited_seed =
        seed.map_or_else(quire_contract_model::NativeJoinBudget::unlimited, |seed| {
            seed.read_proof_snapshot()
                .read_proof_rebound(u64::MAX, u64::MAX, true)
        });
    let prior_work = unlimited_seed.consumed().work;
    let prior_read = unlimited_seed.read_storage_consumed();
    let (complete, events) =
        native_budget_trace::checked_seeded(bytes, evidence, limits, &unlimited_seed);
    assert!(
        complete.budget.first_unpaid().is_none()
            && complete.budget.first_unpaid_read_storage().is_none(),
        "unlimited native reference has no native denial"
    );
    assert!(
        !matches!(
            &complete.result,
            Budgeted::BudgetIncomplete(_) | Budgeted::Exhausted(_)
        ),
        "full reference must complete beyond original native limits"
    );
    if reference.budget.first_unpaid().is_none()
        && reference.budget.first_unpaid_read_storage().is_none()
        && !matches!(
            &reference.result,
            Budgeted::BudgetIncomplete(_) | Budgeted::Exhausted(_)
        )
    {
        assert_eq!(
            complete.result, reference.result,
            "sufficient original context preserves full outcome"
        );
    }
    let (repeated, repeated_events) =
        native_budget_trace::checked_seeded(bytes, evidence, limits, &unlimited_seed);
    same_observation(&complete, &repeated);
    assert_eq!(
        complete.budget, repeated.budget,
        "entire unlimited reference repeats"
    );
    assert_eq!(
        events, repeated_events,
        "complete unlimited event stream repeats"
    );
    drop_without_request::<DriverCounter>(repeated);
    let work = complete.budget.consumed().work;
    let storage = complete.budget.read_storage_consumed();
    let exact_seed = unlimited_seed
        .read_proof_snapshot()
        .read_proof_rebound(work, storage, false);
    let (exact, exact_events) =
        native_budget_trace::checked_seeded(bytes, evidence, limits, &exact_seed);
    assert_eq!(
        complete.result, exact.result,
        "exact complete requirement preserves full outcome"
    );
    assert_eq!(
        complete.requested, exact.requested,
        "exact complete requirement preserves requests"
    );
    assert_eq!(
        complete
            .budget
            .read_proof_snapshot()
            .read_proof_rebound(work, storage, false),
        exact.budget,
        "exact complete requirement preserves all remaining budget state"
    );
    assert_eq!(
        events, exact_events,
        "exact complete requirement preserves every native event"
    );
    drop_without_request::<DriverCounter>(exact);
    let above_work = work
        .checked_add(1)
        .expect("above-work bound is representable");
    let above_storage = storage
        .checked_add(1)
        .expect("above-storage bound is representable");
    let above_seed =
        unlimited_seed
            .read_proof_snapshot()
            .read_proof_rebound(above_work, above_storage, false);
    let (above, above_events) =
        native_budget_trace::checked_seeded(bytes, evidence, limits, &above_seed);
    assert_eq!(
        complete.result, above.result,
        "above complete requirement preserves full outcome"
    );
    assert_eq!(complete.requested, above.requested);
    assert_eq!(events, above_events);
    assert_eq!(
        complete
            .budget
            .read_proof_snapshot()
            .read_proof_rebound(above_work, above_storage, false),
        above.budget
    );
    drop_without_request::<DriverCounter>(above);
    let refusal_code = match &complete.result {
        Budgeted::Refused(value) => Some(format!("{:?}", value.code)),
        _ => None,
    };
    let complete_reference = serde_json::json!({"native_budget":"unlimited","outcome":budgeted_kind(&complete.result),
        "prior_work":prior_work,"prior_read_storage":prior_read,"work":work,"read_storage":storage,
        "requested":complete.requested,"logical_storage_delta":0,"exact_work_limit":work,"exact_read_storage_limit":storage,
        "exact_outcome_equal":true,"exact_counts_equal":true,"complete_repeat_equal":true,
        "above_work_limit":above_work,"above_read_storage_limit":above_storage,"above_outcome_equal":true,"above_counts_equal":true,
        "above_budget_equal":true,"refusal_code":refusal_code,
        "original_context_outcome":budgeted_kind(&reference.result),
        "original_context_work_limit":reference.budget.work_limit(),
        "original_context_read_storage_limit":reference.budget.read_storage_limit()});
    drop_without_request::<DriverCounter>(complete);
    let mut stage_last = BTreeMap::new();
    let mut class_ends = BTreeMap::new();
    let mut selected = BTreeSet::new();
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
            // None stays unknown. Independent canonical class denial evidence
            // supplies class-specific proof; amounts never manufacture labels.
            if event
                .allocation_class
                .is_some_and(|class| exhaustive_classes.contains(&class))
            {
                selected.insert(index);
            }
        }
        stage_last.insert((resource, stage), index);
        let key = (resource, stage, locus.form, event.allocation_class);
        class_ends
            .entry(key)
            .and_modify(|ends: &mut (usize, usize)| ends.1 = index)
            .or_insert((index, index));
    }
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
                Resource::Work => (bound, u64::MAX),
                Resource::ReadStorage => (u64::MAX, bound),
                Resource::LogicalStorage => unreachable!("logical storage rejected"),
            };
            let rebound = unlimited_seed
                .read_proof_snapshot()
                .read_proof_rebound(work, storage, false);
            native_budget_trace::checked_seeded(bytes, evidence, limits, &rebound)
        };
        let (exact, exact_events) = probe(bound);
        assert_eq!(
            exact_events.get(..=index),
            events.get(..=index),
            "exact bound admits the same entire event prefix"
        );
        let above_bound = bound
            .checked_add(1)
            .expect("above event bound is representable");
        let (above, above_events) = probe(above_bound);
        assert_eq!(
            above_events.get(..=index),
            events.get(..=index),
            "above bound admits the same entire event prefix"
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
        let denial = assert_native_denial(&short, event, bound - 1);
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
        object.insert("above_bound".into(), above_bound.into());
        object.insert("above_event_admitted".into(), true.into());
        object.insert("denial".into(), denial);
        for key in [
            "exact_event_admitted",
            "one_below_event_denied",
            "full_repeat_equal",
            "accounting_equal",
        ] {
            object.insert(key.into(), true.into());
        }
        results.push(row);
        for observation in [exact, above, short, repeated] {
            drop_without_request::<DriverCounter>(observation);
        }
    }
    (
        events.iter().map(native_event_json).collect(),
        results,
        complete_reference,
    )
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
    let provider = READ_PROVIDER.with(std::cell::Cell::get);
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
                        bytes,
                        limits,
                        evidence,
                        reference,
                        plain,
                        ProofContext {
                            mode,
                            test_name,
                            location,
                            provider,
                        },
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
    context: ProofContext,
) {
    let ProofContext {
        mode,
        test_name,
        location,
        provider,
    } = context;
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
    let (native_events, boundary_results, complete_reference) = if mode == "full" {
        // Original QSpec calls use stage/class endpoints. Designated adverse
        // providers must explicitly request all intermediate growth events.
        full_native_boundaries(
            bytes,
            limits,
            evidence,
            &reference,
            provider.map_or(&[], |provider| provider.classes),
            None,
        )
    } else {
        (Vec::new(), Vec::new(), serde_json::Value::Null)
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
    let row = if let Some(provider) = provider {
        let mut row = row;
        #[cfg(feature = "test-support")]
        let events = &native_events;
        #[cfg(not(feature = "test-support"))]
        let events = &[];
        row["designated_growth_case"] = provider_json(provider, events, &test_name, location);
        row
    } else {
        row
    };
    #[cfg(feature = "test-support")]
    let row = {
        let mut row = row;
        if mode == "full" {
            row["proof_schema"] = "native-budget-events/v1".into();
            row["native_budget_events"] = native_events.into();
            row["boundary_results"] = boundary_results.into();
            row["complete_reference"] = complete_reference;
            row["original_context_outcome"] = kind.into();
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
    let provider = READ_PROVIDER.with(std::cell::Cell::get);
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
    let (native_events, boundary_results, complete_reference) =
        crate::without_qspec_instrumentation(|| {
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
                                provider.map_or(&[], |provider| provider.classes),
                                Some(&seed),
                            )
                        } else {
                            (Vec::new(), Vec::new(), serde_json::Value::Null)
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
        let row = if let Some(provider) = provider {
            let mut row = row;
            #[cfg(feature = "test-support")]
            let events = &native_events;
            #[cfg(not(feature = "test-support"))]
            let events = &[];
            let test = std::env::var("IR1404_ORIGINAL_TEST_NAME").unwrap_or_else(|_| {
                std::thread::current()
                    .name()
                    .unwrap_or("unnamed")
                    .to_owned()
            });
            row["designated_growth_case"] = provider_json(provider, events, &test, location);
            row
        } else {
            row
        };
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
            row["complete_reference"] = complete_reference;
            row["original_context_outcome"] =
                budgeted_kind(result.as_ref().expect("original caller outcome")).into();
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

    #[cfg(feature = "test-support")]
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn full_reference_rebounds_finite_caller_limits_and_checks_typed_denials() {
        let mut seed = NativeJoinBudget::new(3, 2).with_read_storage_limit(u64::MAX);
        let locus = quire_contract_model::NativeJoinLocus {
            source: quire_contract_model::NativeJoinLocusSource::Package,
            form: "prior_caller_charge",
        };
        seed.accounted_charge(NativeJoinChargeKind::Work, 3, locus)
            .expect("prior work");
        seed.accounted_charge(NativeJoinChargeKind::Storage, 2, locus)
            .expect("prior logical storage");
        let evidence = CheckedPackageEvidence::new();
        let limits = CheckedPackageReadLimits::bounded();
        let reference =
            super::measure_seeded(b"null", &evidence, limits, seed.read_proof_snapshot());
        assert!(matches!(
            reference.result,
            quire_contract_model::CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
                | quire_contract_model::CheckedPackageV2BudgetedReadResult::Exhausted(_)
        ));
        let (events, results, complete) =
            super::full_native_boundaries(b"null", limits, &evidence, &reference, &[], Some(&seed));
        assert_eq!(complete["prior_work"], 3);
        assert_eq!(complete["prior_read_storage"], 0);
        assert_eq!(complete["outcome"], "refused");
        assert!(complete["work"].as_u64().expect("work") > 3);
        assert!(!events.is_empty());
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .all(|row| row["denial"]["entire_repeat_budget_equal"] == true));
        assert_eq!(
            seed.consumed().work,
            3,
            "the actual caller snapshot remains unchanged"
        );
        assert_eq!(seed.consumed().storage, 2);
        super::drop_without_request::<DriverCounter>(reference);
    }

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

#[cfg(test)]
mod provider_scope_tests {
    use super::{with_designated_read_provider, READ_PROVIDER};

    #[test]
    fn provider_metadata_restores_after_nested_scope_and_panic() {
        assert!(READ_PROVIDER.with(std::cell::Cell::get).is_none());
        with_designated_read_provider("outer-provider", &["walker_growth"], || {
            let outer = READ_PROVIDER
                .with(std::cell::Cell::get)
                .expect("outer metadata");
            assert_eq!(outer.symbol, "outer-provider");
            let panic = std::panic::catch_unwind(|| {
                with_designated_read_provider("inner-provider", &["exact_reservation"], || {
                    let inner = READ_PROVIDER
                        .with(std::cell::Cell::get)
                        .expect("inner metadata");
                    assert_eq!(inner.classes, &["exact_reservation"]);
                    panic!("exercise metadata unwind restoration");
                });
            });
            assert!(panic.is_err());
            let restored = READ_PROVIDER
                .with(std::cell::Cell::get)
                .expect("restored metadata");
            assert_eq!(restored.symbol, outer.symbol);
            assert_eq!(restored.classes, outer.classes);
        });
        assert!(READ_PROVIDER.with(std::cell::Cell::get).is_none());
    }
}
