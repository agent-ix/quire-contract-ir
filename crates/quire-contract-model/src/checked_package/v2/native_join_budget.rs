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

/// Stable caller context identifying a native runtime charge without owning a node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeJoinLocusSource {
    /// Allocation belonging to the package-wide projection itself.
    Package,
    /// Flattened selected monitor seed in the caller plan.
    MonitorSeed(usize),
    /// Reached reader input row before graph identity resolution.
    ReachedRow(usize),
    /// Index into the admitted package's semantic graph.
    GraphNode(usize),
    /// Index into the admitted model declaration iterator.
    SelectedDeclaration(usize),
    /// Index into the caller's selected Exact declaration rows.
    IdentityDeclaration(usize),
    /// Index into the runtime caller's tuple rows.
    TupleRow(usize),
    /// Index into the runtime caller's source-unit rows.
    SourceUnitRow(usize),
}

/// Fixed-size first-denial metadata, resolved using the original caller context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeJoinLocus {
    /// Stable input location; no node or key is cloned.
    pub source: NativeJoinLocusSource,
    /// Static native operation or form whose charge was denied.
    pub form: &'static str,
}

/// One finite work and logical-storage allowance shared across native joins.
///
/// Admitted charges are cumulative, including charges made before a refused
/// operation. A denied charge is recorded but does not increment consumption.
/// Implements: FR-358.
#[derive(Debug, Eq, PartialEq)]
pub struct NativeJoinBudget {
    work_limit: u64,
    storage_limit: u64,
    consumed: NativeJoinCharge,
    last_success_delta: Option<NativeJoinCharge>,
    first_unpaid: Option<NativeJoinDeniedCharge>,
    exact_diagnostics: bool,
    active_locus: Option<NativeJoinLocus>,
    denied_locus: Option<NativeJoinLocus>,
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
            exact_diagnostics: false,
            active_locus: None,
            denied_locus: None,
        }
    }

    /// Original configured work limit, unchanged by admitted or denied charges.
    #[must_use]
    pub fn work_limit(&self) -> u64 {
        self.work_limit
    }

    /// Original configured storage limit, unchanged by admitted or denied charges.
    #[must_use]
    pub fn storage_limit(&self) -> u64 {
        self.storage_limit
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

    /// The first unpaid runtime charge's non-owning caller location.
    pub fn first_unpaid_locus(&self) -> Option<NativeJoinLocus> {
        self.denied_locus
    }

    /// Start a caller-owned accounting phase on this same finite allowance.
    ///
    /// This clears the previous operation's success delta and denial metadata,
    /// but preserves all admitted charges and the original configured limits.
    /// Call before charging selection or adapter work, then pass this budget to
    /// the native read without constructing another allowance.
    pub fn resume(&mut self) {
        self.begin();
        self.active_locus = None;
    }

    /// Admit caller work or logical storage before performing it or allocating.
    ///
    /// The charge shares this budget's finite counters with subsequent native
    /// reads. On denial, the first unpaid kind, attempted amount, and supplied
    /// non-owning locus remain inspectable until the next `resume` or native
    /// operation begins. A denied charge does not consume allowance; later
    /// caller charges in the same phase return the first denial without work.
    pub fn accounted_charge(
        &mut self,
        kind: NativeJoinChargeKind,
        amount: u128,
        locus: NativeJoinLocus,
    ) -> Result<(), NativeJoinDeniedCharge> {
        if let Some(first_unpaid) = self.first_unpaid {
            return Err(first_unpaid);
        }
        let previous_locus = self.active_locus.replace(locus);
        let admitted = self.charge(kind, amount);
        self.active_locus = previous_locus;
        if admitted {
            Ok(())
        } else {
            Err(self
                .first_unpaid
                .expect("denied charge records first unpaid"))
        }
    }

    pub(super) fn active_locus(&self) -> Option<NativeJoinLocus> {
        self.active_locus
    }
    pub(super) fn set_active_locus(&mut self, locus: Option<NativeJoinLocus>) {
        self.active_locus = locus;
    }
    pub(super) fn set_locus_form(&mut self, form: &'static str) {
        if let Some(locus) = &mut self.active_locus {
            locus.form = form;
        }
    }
    pub(super) fn set_locus_source(&mut self, source: NativeJoinLocusSource) {
        if let Some(locus) = &mut self.active_locus {
            locus.source = source;
        }
    }

    pub(super) fn remaining_work(&self) -> u64 {
        self.work_limit - self.consumed.work
    }

    pub(super) fn exact_diagnostics(&self) -> bool {
        self.exact_diagnostics
    }

    pub(super) fn set_exact_diagnostics(&mut self, enabled: bool) {
        self.exact_diagnostics = enabled;
    }

    pub(super) fn begin(&mut self) -> NativeJoinCharge {
        self.last_success_delta = None;
        self.first_unpaid = None;
        self.denied_locus = None;
        self.consumed
    }

    pub(super) fn finish(&mut self, before: NativeJoinCharge) {
        self.last_success_delta = Some(NativeJoinCharge {
            work: self.consumed.work - before.work,
            storage: self.consumed.storage - before.storage,
        });
    }

    /// Clear a failed outer operation's delta without erasing its first denial.
    pub(super) fn abort(&mut self) {
        self.last_success_delta = None;
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
            if self.first_unpaid.is_none() {
                self.denied_locus = self.active_locus;
            }
            self.first_unpaid
                .get_or_insert(NativeJoinDeniedCharge { kind, amount });
            false
        }
    }
}

/// Compact private refusal used before any owning runtime diagnostic is payable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeReadDenied;

pub(super) fn read_charge(
    budget: &mut NativeJoinBudget,
    kind: NativeJoinChargeKind,
    amount: u128,
) -> Result<(), NativeReadDenied> {
    if budget.charge(kind, amount) {
        Ok(())
    } else {
        Err(NativeReadDenied)
    }
}

/// Admit the identity walk before computing its shared footprint or using it as a key.
pub(super) fn read_node_work(
    budget: &mut NativeJoinBudget,
    node: &super::CheckedNodeId,
) -> Result<(), NativeReadDenied> {
    read_charge(budget, NativeJoinChargeKind::Work, 1)?;
    read_charge(
        budget,
        NativeJoinChargeKind::Work,
        node.package.len() as u128,
    )?;
    for segment in &node.path {
        read_charge(budget, NativeJoinChargeKind::Work, 1)?;
        if let super::CheckedNodePathSegment::Name(name) = segment {
            read_charge(budget, NativeJoinChargeKind::Work, name.len() as u128)?;
        }
    }
    Ok(())
}

pub(super) fn read_node_storage(
    budget: &mut NativeJoinBudget,
    node: &super::CheckedNodeId,
) -> Result<(), NativeReadDenied> {
    read_node_work(budget, node)?;
    read_charge(
        budget,
        NativeJoinChargeKind::Storage,
        super::native_join_storage::node(node),
    )
}

#[cfg(test)]
mod tests {
    use super::super::native_join_storage;
    use super::{NativeJoinBudget, NativeJoinCharge, NativeJoinChargeKind, NativeJoinDeniedCharge};
    use ix_trace_rs::trace;
    use quire_exact::{
        CarriedQuantityType, DecimalType, Integer, IntegerInterval, NodeKey, PathSegment,
        QuantityCarrier, RationalDomain, RoundingMode, UnitId, ValueType,
    };

    /// Trace: TC-470, FR-360-AC-3.
    #[trace("TC-470", "FR-360-AC-3")]
    #[test]
    fn first_native_denial_retains_its_original_fixed_locus() {
        use super::{NativeJoinLocus, NativeJoinLocusSource};
        let first = NativeJoinLocus {
            source: NativeJoinLocusSource::TupleRow(1),
            form: "selected_tuple_row",
        };
        let mut budget = NativeJoinBudget::new(0, 0);
        budget.set_active_locus(Some(first));
        assert!(!budget.charge(NativeJoinChargeKind::Storage, 9));
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::SourceUnitRow(2),
            form: "later_row",
        }));
        assert!(!budget.charge(NativeJoinChargeKind::Work, 3));
        budget.abort();
        assert_eq!(budget.first_unpaid_locus(), Some(first));
        assert_eq!(
            budget.first_unpaid(),
            Some(NativeJoinDeniedCharge {
                kind: NativeJoinChargeKind::Storage,
                amount: 9
            })
        );
        assert_eq!(budget.consumed(), NativeJoinCharge::default());
        budget.begin();
        assert_eq!(budget.first_unpaid_locus(), None);
    }

    #[trace("FR-358-AC-2", "FR-358-AC-3")]
    #[test]
    fn carried_quantity_storage_charges_unit_and_each_carrier() {
        let interval = IntegerInterval::new(Integer::from(0_i64), Integer::from(9_i64))
            .expect("nonempty interval");
        let denominator = IntegerInterval::new(Integer::from(1_i64), Integer::from(9_i64))
            .expect("positive denominator interval");
        let decimal = DecimalType::new(
            Integer::from(0_i64),
            Integer::from(9_i64),
            0,
            2,
            RoundingMode::Exact,
        )
        .expect("valid decimal carrier");
        let rational =
            RationalDomain::new(interval.clone(), denominator).expect("valid rational carrier");
        for (carrier, expected_storage) in [
            (QuantityCarrier::Decimal(decimal), 7),
            (
                QuantityCarrier::Integer {
                    domain: interval,
                    rounding: RoundingMode::Exact,
                },
                7,
            ),
            (QuantityCarrier::Rational(rational), 11),
        ] {
            let value = ValueType::carried_quantity(CarriedQuantityType::new(
                UnitId::declared(
                    NodeKey::new("example", vec![PathSegment::Text("unit".into())], 0)
                        .expect("valid unit key"),
                ),
                carrier,
            ));
            let charge = native_join_storage::value_type(&value);
            assert_eq!(charge, expected_storage);

            let mut exact = NativeJoinBudget::new(0, expected_storage as u64);
            assert!(exact.charge(NativeJoinChargeKind::Storage, charge));
            assert_eq!(exact.consumed().storage, expected_storage as u64);

            let mut short = NativeJoinBudget::new(0, expected_storage as u64 - 1);
            assert!(!short.charge(NativeJoinChargeKind::Storage, charge));
            assert_eq!(short.consumed().storage, 0);
            assert_eq!(
                short.first_unpaid(),
                Some(NativeJoinDeniedCharge {
                    kind: NativeJoinChargeKind::Storage,
                    amount: expected_storage,
                })
            );
        }
    }

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
