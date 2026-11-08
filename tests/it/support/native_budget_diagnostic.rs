// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! One strict diagnostic read; never a normative measured-probe boundary.
//! Its original accounting assertion remains unconditional. Normative checked
//! observations retain their independent observer-OFF/count/record sequence.

use super::observation::{accounted, measure_at_start, Counter, Observation};
use quire_contract_model::{
    with_native_budget_observer, CheckedPackageEvidence, CheckedPackageReadLimits,
    NativeBudgetEvent, NativeBudgetResource,
};
use std::cell::RefCell;
use std::path::Path;

#[derive(Clone, Copy)]
struct Sample {
    index: usize,
    event: NativeBudgetEvent,
    requested: Option<u64>,
    paid_before: u128,
}
struct Ring {
    samples: Vec<Sample>,
    total: usize,
    next: usize,
    paid: u128,
    baseline: Option<u64>,
    prefix: fn() -> Option<u64>,
    invalid_prefix: bool,
    first_visible: Option<Sample>,
}
thread_local! {
    static DIAGNOSTIC: RefCell<Option<Ring>> = const { RefCell::new(None) };
}
struct Restore(Option<Ring>);
impl Drop for Restore {
    fn drop(&mut self) {
        DIAGNOSTIC.with(|state| *state.borrow_mut() = self.0.take());
    }
}
fn start() {
    DIAGNOSTIC.with(|state| {
        let mut state = state.borrow_mut();
        let ring = state.as_mut().expect("diagnostic scope");
        ring.baseline = (ring.prefix)();
    });
}
fn event(event: NativeBudgetEvent) {
    DIAGNOSTIC.with(|state| {
        let mut state = state.borrow_mut();
        let ring = state.as_mut().expect("diagnostic scope");
        let current = (ring.prefix)();
        let requested = ring.baseline.zip(current).and_then(|(before, now)| {
            let delta = now.checked_sub(before);
            ring.invalid_prefix |= delta.is_none();
            delta
        });
        let sample = Sample {
            index: ring.total,
            event,
            requested,
            paid_before: ring.paid,
        };
        if ring.first_visible.is_none()
            && requested.is_some_and(|bytes| u128::from(bytes) > ring.paid)
        {
            ring.first_visible = Some(sample);
        }
        if ring.samples.len() < ring.samples.capacity() {
            ring.samples.push(sample);
        } else {
            ring.samples[ring.next] = sample;
        }
        ring.next = (ring.next + 1) % ring.samples.capacity();
        ring.total = ring
            .total
            .checked_add(1)
            .expect("diagnostic event count fits");
        if event.resource == NativeBudgetResource::ReadStorage && event.admitted {
            ring.paid = ring
                .paid
                .checked_add(event.amount)
                .expect("finite reader admitted total fits u128");
        }
    });
}
fn sample_json(sample: Sample) -> serde_json::Value {
    serde_json::json!({"event_index":sample.index,"event":super::native_event_json(&sample.event),
        "actual_requested_prefix":sample.requested,"admitted_read_storage_before":super::native_amount_json(sample.paid_before)})
}

/// Diagnostic-only: one actual public read, with the caller's exact input and
/// limits, then the unchanged accounting assertion. It deliberately bypasses
/// the normative measure_at *boundary*, not its accounting obligation: a use
/// in the original corpus contributes an unaccounted production entry.
#[track_caller]
pub(crate) fn diagnostic_native_read<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    work: u64,
    read_storage: u64,
    output: &Path,
    capacity: usize,
) -> Observation {
    assert!(
        (1..=16384).contains(&capacity),
        "bounded diagnostic ring capacity"
    );
    let previous = DIAGNOSTIC.with(|state| {
        state.replace(Some(Ring {
            samples: Vec::with_capacity(capacity),
            total: 0,
            next: 0,
            paid: 0,
            baseline: None,
            prefix: C::requested_prefix,
            invalid_prefix: false,
            first_visible: None,
        }))
    });
    let restore = Restore(previous);
    let observation = with_native_budget_observer(event, || {
        measure_at_start::<C>(bytes, evidence, limits, work, read_storage, start)
    });
    let ring = DIAGNOSTIC
        .with(|state| state.replace(None))
        .expect("diagnostic recorder");
    drop(restore);
    // Every allocation for serialization, I/O, comparison and buffer teardown
    // happens after the counter closed. No diagnostic read is replayed here.
    let start = if ring.total > ring.samples.len() {
        ring.next
    } else {
        0
    };
    let samples: Vec<_> = (0..ring.samples.len())
        .map(|offset| sample_json(ring.samples[(start + offset) % ring.samples.len()]))
        .collect();
    let site = std::panic::Location::caller();
    let receipt = serde_json::json!({"schema":"native-budget-diagnostic/v1",
        "scope":"one diagnostic-only strict measured read; not normative or source attribution",
        "full_corpus":false,"normative_boundary_proof":false,"static_ac513_proven":false,
        "driver":site.file(),"line":site.line(),"column":site.column(),"input_bytes":bytes.len(),
        "work_limit":work,"read_storage_limit":read_storage,
        "outcome":super::budgeted_kind(&observation.result),"requested":observation.requested,
        "admitted_read_storage":observation.budget.read_storage_consumed(),
        "ring_capacity":capacity,"events_seen":ring.total,"events_retained":ring.samples.len(),
        "truncated":ring.total>ring.samples.len(),"dropped_events":ring.total-ring.samples.len(),
        "allocator_prefix_available":ring.baseline.is_some(),"invalid_prefix":ring.invalid_prefix,
        "first_visible_unpaid_boundary":ring.first_visible.map(sample_json),
        "first_unpaid_allocation_source":"unknown; callback locus is observed context, not allocation provenance",
        "events":samples});
    let written = (|| -> std::io::Result<()> {
        let file = std::fs::File::create(output)?;
        serde_json::to_writer(file, &receipt).map_err(std::io::Error::other)
    })();
    // I/O failure cannot replace a real accounting defect's original panic.
    accounted("diagnostic-only strict public read", &observation);
    written.expect("diagnostic receipt could not be preserved");
    observation
}
