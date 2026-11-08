// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX
//! Passive test instrumentation for actual caller-budget requests.
use super::NativeJoinLocus;
use std::cell::Cell;

/// Resource of one actual caller-budget request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeBudgetResource {
    /// Validation work.
    Work,
    /// Logical storage outside the strict reader's heap accounting.
    LogicalStorage,
    /// Heap storage of the strict read.
    ReadStorage,
}

/// An actual request, observed without owning or replacing its budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeBudgetEvent {
    /// Requested resource.
    pub resource: NativeBudgetResource,
    /// Full attempted request.
    pub amount: u128,
    /// Admitted total before this request.
    pub consumed_before: u64,
    /// Whether the caller's actual budget admitted it.
    pub admitted: bool,
    /// Original active caller locus.
    pub locus: Option<NativeJoinLocus>,
    /// Explicit allocation-site classification, when provided by its owner.
    pub allocation_class: Option<&'static str>,
}

thread_local! {
    static OBSERVER: Cell<Option<fn(NativeBudgetEvent)>> = const { Cell::new(None) };
    static CLASS: Cell<Option<&'static str>> = const { Cell::new(None) };
}

struct Restore<T: Copy + 'static> {
    slot: &'static std::thread::LocalKey<Cell<Option<T>>>,
    previous: Option<T>,
}
impl<T: Copy> Drop for Restore<T> {
    fn drop(&mut self) {
        self.slot.set(self.previous);
    }
}

/// Observe requests during `read`; nested scopes and unwinding restore the
/// previous observer. Callbacks must not allocate or change reader state.
pub fn with_native_budget_observer<R>(
    observer: fn(NativeBudgetEvent),
    read: impl FnOnce() -> R,
) -> R {
    let _restore = Restore {
        slot: &OBSERVER,
        previous: OBSERVER.replace(Some(observer)),
    };
    read()
}

/// Attach a passive, explicitly owned allocation class to scoped requests.
pub fn with_native_allocation_class<R>(class: &'static str, read: impl FnOnce() -> R) -> R {
    let _restore = Restore {
        slot: &CLASS,
        previous: CLASS.replace(Some(class)),
    };
    read()
}

pub(super) fn emit(mut event: NativeBudgetEvent) {
    event.allocation_class = CLASS.get();
    if let Some(observer) = OBSERVER.get() {
        observer(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    thread_local! {
        static FIRST: Cell<usize> = const { Cell::new(0) };
        static SECOND: Cell<usize> = const { Cell::new(0) };
        static LAST_CLASS: Cell<Option<&'static str>> = const { Cell::new(None) };
    }
    fn first(event: NativeBudgetEvent) {
        FIRST.set(FIRST.get() + 1);
        LAST_CLASS.set(event.allocation_class);
    }
    fn second(_: NativeBudgetEvent) {
        SECOND.set(SECOND.get() + 1);
    }
    fn request() {
        emit(NativeBudgetEvent {
            resource: NativeBudgetResource::Work,
            amount: 1,
            consumed_before: 0,
            admitted: true,
            locus: None,
            allocation_class: None,
        });
    }
    #[test]
    #[trace("TC-472", "FR-038-AC-511")]
    fn observer_and_class_scopes_restore_after_nesting_and_unwind() {
        FIRST.set(0);
        SECOND.set(0);
        with_native_budget_observer(first, || {
            with_native_allocation_class("outer", || {
                request();
                let stopped = std::panic::catch_unwind(|| {
                    with_native_budget_observer(second, || {
                        with_native_allocation_class("inner", || {
                            request();
                            panic!("scope control");
                        });
                    });
                });
                assert!(stopped.is_err());
                request();
                assert_eq!(LAST_CLASS.get(), Some("outer"));
            });
            request();
            assert_eq!(LAST_CLASS.get(), None);
        });
        request();
        assert_eq!(FIRST.get(), 3);
        assert_eq!(SECOND.get(), 1);
        assert!(OBSERVER.get().is_none());
        assert!(CLASS.get().is_none());
    }
}
