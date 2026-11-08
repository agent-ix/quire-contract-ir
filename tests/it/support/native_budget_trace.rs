// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Passive charge observations. Buffer allocation occurs outside measured reads.

use super::{measure_at, Counter, Observation, ReadViewScope};
use quire_contract_model::{
    with_native_budget_observer, CheckedPackageEvidence, CheckedPackageReadLimits,
    NativeBudgetEvent,
};
use std::cell::RefCell;

#[derive(Default)]
enum Recorder {
    #[default]
    Idle,
    Count(usize),
    Record {
        events: Vec<NativeBudgetEvent>,
        overflow: bool,
    },
}
thread_local! {
    static RECORDER: RefCell<Recorder> = const { RefCell::new(Recorder::Idle) };
}
fn event(event: NativeBudgetEvent) {
    RECORDER.with(|state| match &mut *state.borrow_mut() {
        Recorder::Idle => panic!("observer called outside recording scope"),
        Recorder::Count(count) => *count = count.checked_add(1).expect("event count overflow"),
        Recorder::Record { events, overflow } => {
            if events.len() == events.capacity() {
                *overflow = true;
            } else {
                events.push(event);
            }
        }
    });
}
struct Restore(Option<Recorder>);
impl Drop for Restore {
    fn drop(&mut self) {
        RECORDER.with(|state| *state.borrow_mut() = self.0.take().expect("restore once"));
    }
}
fn scoped<T>(recorder: Recorder, read: impl FnOnce() -> T) -> (T, Recorder) {
    let previous = RECORDER.with(|state| state.replace(recorder));
    let restore = Restore(Some(previous));
    let result = with_native_budget_observer(event, read);
    let recorded = RECORDER.with(|state| state.replace(Recorder::Idle));
    drop(restore);
    (result, recorded)
}
fn unchanged(off: &Observation, on: &Observation, views: ReadViewScope) {
    views.budgeted(
        &off.result,
        &on.result,
        "observer preserves typed outcome under the explicit comparison scope",
    );
    assert_eq!(
        off.budget, on.budget,
        "observer preserves entire caller budget"
    );
    assert_eq!(
        off.requested, on.requested,
        "observer itself allocates nothing during read"
    );
}

/// The first read is observer-OFF, before either observed read. This diagnostic
/// passes through the checked measurement boundary on all three reads. A real
/// accounting defect fails before this consumer can provide normative evidence.
/// This explicit observer API retains its full public-view oracle; ordinary
/// accounting selects Core through checked_with_scope instead.
pub(crate) fn checked<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    work: u64,
    storage: u64,
) -> (Observation, Vec<NativeBudgetEvent>) {
    trace(
        || measure_at::<C>(bytes, evidence, limits, work, storage),
        ReadViewScope::Full,
    )
}

pub(crate) fn checked_with_scope<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    work: u64,
    storage: u64,
    views: ReadViewScope,
) -> (Observation, Vec<NativeBudgetEvent>) {
    match views {
        ReadViewScope::Core => trace(
            || measure_at::<C>(bytes, evidence, limits, work, storage),
            ReadViewScope::Core,
        ),
        ReadViewScope::Full => super::observe_native::<C>(bytes, evidence, limits, work, storage),
    }
}

pub(crate) fn checked_seeded(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    seed: &quire_contract_model::NativeJoinBudget,
) -> (Observation, Vec<NativeBudgetEvent>) {
    trace(
        || super::measure_seeded(bytes, evidence, limits, seed.read_proof_snapshot()),
        ReadViewScope::Full,
    )
}

pub(crate) fn checked_seeded_with_scope(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    seed: &quire_contract_model::NativeJoinBudget,
    views: ReadViewScope,
) -> (Observation, Vec<NativeBudgetEvent>) {
    match views {
        ReadViewScope::Core => trace(
            || super::measure_seeded(bytes, evidence, limits, seed.read_proof_snapshot()),
            ReadViewScope::Core,
        ),
        ReadViewScope::Full => checked_seeded(bytes, evidence, limits, seed),
    }
}

fn trace(
    read: impl Fn() -> Observation,
    views: ReadViewScope,
) -> (Observation, Vec<NativeBudgetEvent>) {
    let off = read();
    let (counted, count) = scoped(Recorder::Count(0), &read);
    unchanged(&off, &counted, views);
    let Recorder::Count(count) = count else {
        unreachable!("count recorder")
    };
    // Capacity is allocated before the measured record read. The callback never
    // reserves, formats, logs, or reallocates; an unexpected extra event fails.
    let events = Vec::with_capacity(count);
    let (recorded, trace) = scoped(
        Recorder::Record {
            events,
            overflow: false,
        },
        read,
    );
    unchanged(&off, &recorded, views);
    let Recorder::Record { events, overflow } = trace else {
        unreachable!("event recorder")
    };
    assert!(
        !overflow,
        "event recorder overflowed its preallocated buffer"
    );
    assert_eq!(
        events.len(),
        count,
        "count and record reads emit identical event counts"
    );
    // Returned outcome destruction is outside every measured read.
    drop(counted);
    drop(recorded);
    (off, events)
}
