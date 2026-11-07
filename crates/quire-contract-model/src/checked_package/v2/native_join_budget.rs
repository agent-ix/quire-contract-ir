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
    /// Row `i` of an external adapter's channel being converted into an IR-owned row.
    AdapterRow(usize),
    /// An external preparer's charges while building the same-emission authority row.
    AuthorityRow,
    /// Selection comparison `k`, by position in the external preparer's comparison order.
    PreparationSelection(usize),
    /// Owned output item `k` an external preparer allocates after a native read.
    PreparationOutput(usize),
    /// Contract IR's own authority-row inspections before the occurrence is resolved.
    MonitorAuthority,
    /// A charge of the strict V2 checked-package read: its stage and the index
    /// FR-038's stage table names for it.
    PackageRead(PackageReadStage, usize),
}

/// The stages of the strict V2 checked-package read, in their order.
///
/// The stage table of FR-038 names each stage's index and static form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageReadStage {
    /// The shared reader's reservations while reading the package bytes.
    Envelope,
    /// The shared writer's reservations while encoding the comparison.
    Canonical,
    /// The typed decode of the closed wire.
    WireDecode,
    /// The header, owner-schema, flat-wire and interval-bound checks.
    WireChecks,
    /// The shared structural node keys and the node-index checks.
    NodeIndex,
    /// The lock checks and the dependency and model admission.
    Lock,
    /// One selected model document's read and declarations.
    ModelDocument,
    /// The semantic graph validators and sidecar row checks.
    Graph,
    /// The source-map checks.
    SourceMap,
    /// The equivalence-occurrence checks.
    Equivalence,
    /// The extension source-region checks.
    Extension,
    /// The capability-report checks.
    Capabilities,
    /// The diagnostic checks.
    Diagnostics,
    /// The operation-effect checks.
    OperationEffects,
    /// The checked-term-data checks.
    CheckedTermData,
    /// The retained structures of the admitted package.
    Retained,
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
    read_storage_limit: u64,
    read_storage_consumed: u64,
    last_read_storage_delta: Option<u64>,
    first_unpaid_read_storage: Option<u128>,
    unlimited: bool,
    exact_diagnostics: bool,
    active_locus: Option<NativeJoinLocus>,
    denied_locus: Option<NativeJoinLocus>,
    /// Whether the diagnostic of the read's first denial was paid for before
    /// it was built: `Some(true)` when admitted, `Some(false)` when denied.
    prepaid_diagnostic: Option<bool>,
    /// The outcome whose owned diagnostic the read's first denial is.
    pending_outcome: Option<super::CheckedPackagePendingOutcome>,
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
            read_storage_limit: 0,
            read_storage_consumed: 0,
            last_read_storage_delta: None,
            first_unpaid_read_storage: None,
            unlimited: false,
            exact_diagnostics: false,
            active_locus: None,
            denied_locus: None,
            prepaid_diagnostic: None,
            pending_outcome: None,
        }
    }

    /// Sets the read-storage limit, in bytes of heap requested by the strict
    /// V2 read, at construction.
    #[must_use]
    pub fn with_read_storage_limit(mut self, bytes: u64) -> Self {
        self.read_storage_limit = bytes;
        self
    }

    /// A budget whose work, logical storage and read-storage allowances are
    /// unlimited: every charge is admitted and each counter saturates at
    /// `u64::MAX` instead of denying.
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            work_limit: u64::MAX,
            storage_limit: u64::MAX,
            read_storage_limit: u64::MAX,
            unlimited: true,
            ..Self::new(u64::MAX, u64::MAX)
        }
    }

    /// Original configured read-storage limit, unchanged by any charge.
    #[must_use]
    pub fn read_storage_limit(&self) -> u64 {
        self.read_storage_limit
    }

    /// Cumulative admitted read storage, in bytes, including a refused read's.
    #[must_use]
    pub fn read_storage_consumed(&self) -> u64 {
        self.read_storage_consumed
    }

    /// The most recent successful operation's read-storage delta, absent after
    /// a refused or exhausted operation.
    #[must_use]
    pub fn last_success_read_storage_delta(&self) -> Option<u64> {
        self.last_read_storage_delta
    }

    /// The full attempted amount of the first read-storage charge refused by
    /// the most recent operation; its locus is `first_unpaid_locus`.
    #[must_use]
    pub fn first_unpaid_read_storage(&self) -> Option<u128> {
        self.first_unpaid_read_storage
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
        self.last_read_storage_delta = None;
        self.first_unpaid = None;
        self.first_unpaid_read_storage = None;
        self.denied_locus = None;
        self.prepaid_diagnostic = None;
        self.pending_outcome = None;
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
        self.last_read_storage_delta = None;
    }

    /// Cumulative read storage, the starting point of a read's delta.
    pub(super) fn read_storage_mark(&self) -> u64 {
        self.read_storage_consumed
    }

    /// Records a read's exact work and read-storage deltas, and a logical
    /// storage delta of zero, from the counters it started at.
    pub(super) fn finish_read(&mut self, before: NativeJoinCharge, read_storage_before: u64) {
        self.finish(before);
        self.last_read_storage_delta = Some(self.read_storage_consumed - read_storage_before);
    }

    /// Admits `bytes` of heap the strict read is about to request, at the
    /// active locus. A denial records the first unpaid read-storage charge
    /// once and increments no counter.
    pub(super) fn charge_read_storage(&mut self, bytes: u128) -> bool {
        if self.unlimited {
            let add = u64::try_from(bytes).unwrap_or(u64::MAX);
            self.read_storage_consumed = self.read_storage_consumed.saturating_add(add);
            return true;
        }
        let next = u128::from(self.read_storage_consumed).checked_add(bytes);
        if let Some(next) = next.filter(|next| *next <= u128::from(self.read_storage_limit)) {
            self.read_storage_consumed =
                u64::try_from(next).expect("bounded read storage fits u64");
            true
        } else {
            if self.first_unpaid_read_storage.is_none() && self.first_unpaid.is_none() {
                self.denied_locus = self.active_locus;
                self.first_unpaid_read_storage = Some(bytes);
            }
            false
        }
    }

    /// Pays, before it is built, for the pointer diagnostic of the first
    /// denial just recorded, at that denial's locus with form `"diagnostic"`.
    /// The outcome's own charge then finds it paid.
    pub(super) fn prepay_diagnostic(&mut self, bytes: u128) -> bool {
        let previous = self.active_locus;
        self.active_locus = self.denied_locus.map(|locus| NativeJoinLocus {
            source: locus.source,
            form: "diagnostic",
        });
        let admitted = self.charge_read_storage(bytes);
        self.active_locus = previous;
        self.prepaid_diagnostic = Some(admitted);
        admitted
    }

    /// The prepayment of the first denial's diagnostic, taken once.
    pub(super) fn take_prepaid_diagnostic(&mut self) -> Option<bool> {
        self.prepaid_diagnostic.take()
    }

    /// Records the outcome whose diagnostic could not be paid for.
    pub(super) fn set_pending(&mut self, pending: Option<super::CheckedPackagePendingOutcome>) {
        self.pending_outcome = pending;
    }

    /// The outcome recorded by [`Self::set_pending`], taken once.
    pub(super) fn take_pending(&mut self) -> Option<super::CheckedPackagePendingOutcome> {
        self.pending_outcome.take()
    }

    /// The first unpaid charge of the current read, either resource, with the
    /// original amount.
    pub(super) fn first_unpaid_resource(&self) -> Option<(super::PackageReadResource, u128)> {
        match (self.first_unpaid, self.first_unpaid_read_storage) {
            (Some(denied), _) => Some((super::PackageReadResource::Work, denied.amount)),
            (None, Some(amount)) => Some((super::PackageReadResource::ReadStorage, amount)),
            (None, None) => None,
        }
    }

    pub(super) fn charge(&mut self, kind: NativeJoinChargeKind, amount: u128) -> bool {
        let (consumed, limit) = match kind {
            NativeJoinChargeKind::Work => (&mut self.consumed.work, self.work_limit),
            NativeJoinChargeKind::Storage => (&mut self.consumed.storage, self.storage_limit),
        };
        if self.unlimited {
            *consumed = consumed.saturating_add(u64::try_from(amount).unwrap_or(u64::MAX));
            return true;
        }
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
