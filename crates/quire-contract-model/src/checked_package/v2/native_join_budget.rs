// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-owned accounting shared by native selected-identity joins.

/// The resource whose next native join charge could not be admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeJoinChargeKind {
    /// An inspection or validation step.
    Work,
    /// Logical retained or temporary owned storage.
    Storage,
}

/// Work and storage admitted by a native join operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NativeJoinCharge {
    /// Admitted work units.
    pub work: u64,
    /// Admitted logical storage units.
    pub storage: u64,
}

/// The first charge refused by the most recent native join operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeJoinDeniedCharge {
    /// Resource that exhausted its finite allowance.
    pub kind: NativeJoinChargeKind,
    /// Full attempted amount before an allocation or inspection.
    pub amount: u128,
}

/// One finite work and logical-storage allowance shared across native joins.
///
/// Admitted charges are cumulative, including charges made before a refused
/// operation. A denied charge is recorded but does not increment consumption.
/// Implements: FR-358.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeJoinBudget {
    work_limit: u64,
    storage_limit: u64,
    consumed: NativeJoinCharge,
    last_success_delta: Option<NativeJoinCharge>,
    first_unpaid: Option<NativeJoinDeniedCharge>,
}

impl NativeJoinBudget {
    /// Construct a caller-owned finite allowance.
    pub fn new(work_limit: u64, storage_limit: u64) -> Self {
        Self {
            work_limit,
            storage_limit,
            consumed: NativeJoinCharge::default(),
            last_success_delta: None,
            first_unpaid: None,
        }
    }

    /// Cumulative admitted work and storage, including a refused call's work.
    pub fn consumed(&self) -> NativeJoinCharge {
        self.consumed
    }

    /// The most recent complete operation's exact work and storage charge.
    pub fn last_success_delta(&self) -> Option<NativeJoinCharge> {
        self.last_success_delta
    }

    /// The first unpaid charge in the most recent refused operation.
    pub fn first_unpaid(&self) -> Option<NativeJoinDeniedCharge> {
        self.first_unpaid
    }

    pub(super) fn begin(&mut self) -> NativeJoinCharge {
        self.last_success_delta = None;
        self.first_unpaid = None;
        self.consumed
    }

    pub(super) fn finish(&mut self, before: NativeJoinCharge) {
        self.last_success_delta = Some(NativeJoinCharge {
            work: self.consumed.work - before.work,
            storage: self.consumed.storage - before.storage,
        });
    }

    pub(super) fn charge(&mut self, kind: NativeJoinChargeKind, amount: u128) -> bool {
        let (consumed, limit) = match kind {
            NativeJoinChargeKind::Work => (&mut self.consumed.work, self.work_limit),
            NativeJoinChargeKind::Storage => (&mut self.consumed.storage, self.storage_limit),
        };
        let next = u128::from(*consumed).checked_add(amount);
        if let Some(next) = next.filter(|next| *next <= u128::from(limit)) {
            *consumed = u64::try_from(next).expect("bounded native charge fits u64");
            true
        } else {
            self.first_unpaid
                .get_or_insert(NativeJoinDeniedCharge { kind, amount });
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeJoinBudget, NativeJoinCharge, NativeJoinChargeKind, NativeJoinDeniedCharge};
    use ix_trace_rs::trace;

    #[trace("FR-358-AC-4")]
    #[test]
    fn overflowing_native_charge_does_not_wrap_or_erase_prior_work() {
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let before = budget.begin();
        assert!(budget.charge(NativeJoinChargeKind::Work, 1));
        budget.finish(before);
        assert_eq!(
            budget.last_success_delta(),
            Some(NativeJoinCharge {
                work: 1,
                storage: 0
            })
        );

        budget.begin();
        assert!(!budget.charge(NativeJoinChargeKind::Work, u128::MAX));
        assert_eq!(
            budget.consumed(),
            NativeJoinCharge {
                work: 1,
                storage: 0
            }
        );
        assert_eq!(budget.last_success_delta(), None);
        assert_eq!(
            budget.first_unpaid(),
            Some(NativeJoinDeniedCharge {
                kind: NativeJoinChargeKind::Work,
                amount: u128::MAX,
            })
        );
        assert!(!budget.charge(NativeJoinChargeKind::Storage, u128::MAX));
        assert_eq!(
            budget.first_unpaid().unwrap().kind,
            NativeJoinChargeKind::Work
        );
    }
}
