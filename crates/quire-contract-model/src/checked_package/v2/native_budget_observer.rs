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
