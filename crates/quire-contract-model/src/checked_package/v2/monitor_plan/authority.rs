// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Same-emission order and progress authority bound to the selected monitor
//! closure (FR-361).

use super::selected_type_closure::SelectedMonitorTypeClosureErrorCode;
use super::selected_type_closure::{native_error, semantic_error, SelectedMonitorTypeClosureError};
use super::{CheckedMonitorClock, CheckedMonitorClockRead, CheckedMonitorPlanRef};
use crate::checked_package::v2::native_join_budget::{read_charge, read_node_work};
use crate::checked_package::v2::native_join_storage as storage;
use crate::checked_package::v2::timed_model::{admitted_time_source, AdmittedTimeSource};
use crate::checked_package::v2::{
    AdmissionBinding, CheckedNodeId, CheckedNodePathSegment, CheckedOccurrence, CheckedOccurrenceRole, CheckedRational,
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};
use num_bigint::BigUint;

const FORM: &str = "selected_authority";

/// The admitted time source QSL selected for the monitored claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedMonitorTimeSource {
    /// A dense time source.
    Dense,
    /// A tick time source with its exact checked period.
    Tick {
        /// The exact period of the checked time source.
        period: CheckedRational,
    },
}

/// The component of an authority row that failed its construction check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedMonitorAuthorityComponent {
    /// The occurrence is not a claim occurrence.
    Occurrence,
    /// The admitted order value is empty.
    OrderValue,
    /// The admitted progress association value is empty.
    ProgressAssociation,
}

/// The first component an authority row refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected monitor authority row refused: {component:?}")]
pub struct SelectedMonitorAuthorityRowError {
    /// The first refused component.
    pub component: SelectedMonitorAuthorityComponent,
}

/// Stable cause of a refused authority binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedMonitorAuthorityErrorCode {
    /// The plan's package does not retain the row's admission capability.
    ForeignEmission,
    /// The row's clause is not the plan's admitted clause.
    ClauseMismatch,
    /// The row's occurrence is not an admitted claim occurrence of the clause.
    OccurrenceMismatch,
    /// The occurrence's admitted time source differs from the row's.
    TimeSourceMismatch,
}

/// The same-emission authority QSL admitted for one monitor root, as IR checks it.
///
/// The order and progress association values are QSL's and the observation
/// owner's; the row borrows them and IR never decodes or re-encodes them.
///
/// ```compile_fail
/// use quire_contract_model::SelectedMonitorAuthorityRow;
/// fn fabricate(row: &SelectedMonitorAuthorityRow<'_>) {
///     let _ = row.order_value;
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SelectedMonitorAuthorityRow<'a> {
    admission: AdmissionBinding,
    clause: CheckedNodeId,
    occurrence: CheckedOccurrence,
    time_source: SelectedMonitorTimeSource,
    order_value: &'a [u8],
    progress_association: &'a [u8],
}

impl<'a> SelectedMonitorAuthorityRow<'a> {
    /// Build a row from QSL's `AdmittedPlanAuthority` components, unchanged.
    ///
    /// Refuses, in this order, a non-claim occurrence, an empty order value
    /// and an empty progress association value.
    pub fn try_new(
        admission: AdmissionBinding,
        clause: CheckedNodeId,
        occurrence: CheckedOccurrence,
        time_source: SelectedMonitorTimeSource,
        order_value: &'a [u8],
        progress_association: &'a [u8],
    ) -> Result<Self, SelectedMonitorAuthorityRowError> {
        let component = if occurrence.role != CheckedOccurrenceRole::Claim {
            Some(SelectedMonitorAuthorityComponent::Occurrence)
        } else if order_value.is_empty() {
            Some(SelectedMonitorAuthorityComponent::OrderValue)
        } else if progress_association.is_empty() {
            Some(SelectedMonitorAuthorityComponent::ProgressAssociation)
        } else {
            None
        };
        match component {
            Some(component) => Err(SelectedMonitorAuthorityRowError { component }),
            None => Ok(Self {
                admission,
                clause,
                occurrence,
                time_source,
                order_value,
                progress_association,
            }),
        }
    }
}

/// The result of IR's structural correspondence checks over the admitted facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMonitorAuthorityReceipt<'a> {
    clause: &'a CheckedNodeId,
    occurrence: CheckedOccurrence,
    clock: CheckedMonitorClockRead<'a>,
    order_value: &'a [u8],
    progress_association: &'a [u8],
}

impl<'a> SelectedMonitorAuthorityReceipt<'a> {
    /// The admitted clause of the monitor plan.
    pub fn clause(&self) -> &'a CheckedNodeId {
        self.clause
    }
    /// The selected claim occurrence.
    pub fn occurrence(&self) -> &CheckedOccurrence {
        &self.occurrence
    }
    /// The admitted clock read for the selected occurrence.
    pub fn clock(&self) -> &CheckedMonitorClockRead<'a> {
        &self.clock
    }
    /// QSL's admitted order value, as the row borrowed it.
    pub fn order_value(&self) -> &'a [u8] {
        self.order_value
    }
    /// The observation owner's admitted progress association value, as the row borrowed it.
    pub fn progress_association(&self) -> &'a [u8] {
        self.progress_association
    }
}

fn locus(source: NativeJoinLocusSource, budget: &mut NativeJoinBudget) {
    budget.set_active_locus(Some(NativeJoinLocus { source, form: FORM }));
}

fn work(budget: &mut NativeJoinBudget) -> Result<(), SelectedMonitorTypeClosureError> {
    read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))
}

fn refusal(
    code: SelectedMonitorAuthorityErrorCode,
    at: &CheckedNodeId,
    occurrence: Option<&CheckedOccurrence>,
    budget: &mut NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    // The cloned outer ID owns its package, path and Arc handles; the Box
    // and optional fixed-size occurrence are additional owned containers.
    if read_node_work(budget, at).is_err() {
        return native_error(budget);
    }
    let mut floor = storage::node(at).saturating_add(1 + u128::from(occurrence.is_some()));
    for segment in &at.path {
        if let CheckedNodePathSegment::Dependency { package, .. } = segment {
            if read_charge(budget, NativeJoinChargeKind::Work, package.len() as u128).is_err() {
                return native_error(budget);
            }
            floor = floor.saturating_add(1 + package.len() as u128);
        }
    }
    if read_charge(budget, NativeJoinChargeKind::Storage, floor).is_err() {
        return native_error(budget);
    }
    let mut error = semantic_error(
        SelectedMonitorTypeClosureErrorCode::Authority(code),
        Some(Box::new(at.clone())),
        budget,
    );
    error.occurrence = occurrence.cloned();
    error
}

/// `numerator/denominator` equals the admitted spelling, `n` or `n/d`, exactly.
fn same_period(
    admitted: &str,
    row: &CheckedRational,
    budget: &mut NativeJoinBudget,
) -> Result<bool, SelectedMonitorTypeClosureError> {
    // Length reads do not scan the input. Include the implicit denominator
    // before admitting every separator, digit and arithmetic inspection.
    let width = (admitted.len() as u128)
        .saturating_add(row.numerator.len() as u128)
        .saturating_add(row.denominator.len() as u128)
        .saturating_add(1);
    read_charge(
        budget,
        NativeJoinChargeKind::Work,
        width.saturating_mul(width).saturating_add(width),
    )
    .map_err(|_| native_error(budget))?;
    let (numerator, denominator) = admitted.split_once('/').unwrap_or((admitted, "1"));
    let components = [numerator, denominator, &row.numerator, &row.denominator];
    let digits = components.iter().fold(0u128, |total, text| {
        total.saturating_add(text.len() as u128)
    });
    if components
        .iter()
        .any(|text| text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Ok(false);
    }
    read_charge(
        budget,
        NativeJoinChargeKind::Storage,
        digits.saturating_mul(4).saturating_add(6),
    )
    .map_err(|_| native_error(budget))?;
    let parse = |text: &str| text.parse::<BigUint>().ok();
    let (Some(an), Some(ad), Some(rn), Some(rd)) = (
        parse(numerator),
        parse(denominator),
        parse(&row.numerator),
        parse(&row.denominator),
    ) else {
        return Ok(false);
    };
    Ok(ad != BigUint::from(0u8) && rd != BigUint::from(0u8) && an * &rd == rn * ad)
}

/// Admit every component of a structural identity before the shared
/// comparator inspects it. Borrowed child handles use a prepaid work stack.
fn identity_work(
    node: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    let mut pending = Vec::new();
    let mut current = Some(node);
    while let Some(node) = current {
        read_node_work(budget, node).map_err(|_| native_error(budget))?;
        for segment in &node.path {
            match segment {
                CheckedNodePathSegment::Node(child) => {
                    read_charge(budget, NativeJoinChargeKind::Storage, if pending.capacity() == 0 { 2 } else { 1 })
                        .map_err(|_| native_error(budget))?;
                    pending.push(child.as_ref());
                }
                CheckedNodePathSegment::Dependency { package, .. } => {
                    read_charge(budget, NativeJoinChargeKind::Work, package.len() as u128)
                        .map_err(|_| native_error(budget))?;
                }
                CheckedNodePathSegment::Name(_) | CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Operand(_) => {}
            }
        }
        current = pending.pop();
    }
    Ok(())
}

fn same_clause(
    left: &CheckedNodeId,
    right: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
) -> Result<bool, SelectedMonitorTypeClosureError> {
    identity_work(left, budget)?;
    identity_work(right, budget)?;
    left.cmp_growing(right, &mut |bytes| {
        // The shared seam supplies a whole scratch block's size. Its byte
        // width conservatively bounds all owned continuation handles, plus
        // one container, without duplicating the private frame layout.
        read_charge(budget, NativeJoinChargeKind::Storage, (bytes as u128).saturating_add(1))
            .map_err(|_| native_error(budget))
    }).map(|order| order.is_eq())
}

impl<'a> CheckedMonitorPlanRef<'a> {
    /// Check the row against this plan's admitted facts, stopping at the first failure.
    pub(super) fn bind_authority(
        &self,
        row: &'a SelectedMonitorAuthorityRow<'a>,
        clocks: &[CheckedMonitorClockRead<'a>],
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedMonitorAuthorityReceipt<'a>, SelectedMonitorTypeClosureError> {
        let root = self.node_id();
        locus(NativeJoinLocusSource::MonitorAuthority, budget);
        work(budget)?;
        if !self.package.retains(&row.admission) {
            return Err(refusal(
                SelectedMonitorAuthorityErrorCode::ForeignEmission,
                root,
                None,
                budget,
            ));
        }
        if !same_clause(&row.clause, self.clause(), budget)? {
            return Err(refusal(
                SelectedMonitorAuthorityErrorCode::ClauseMismatch,
                root,
                None,
                budget,
            ));
        }
        let mut selected = None;
        for (index, clock) in clocks.iter().enumerate() {
            work(budget)?;
            if *clock.occurrence() == row.occurrence {
                selected = Some((index, clock));
                break;
            }
        }
        let Some((index, clock)) = selected else {
            return Err(refusal(
                SelectedMonitorAuthorityErrorCode::OccurrenceMismatch,
                self.clause(),
                Some(&row.occurrence),
                budget,
            ));
        };
        locus(NativeJoinLocusSource::MonitorSeed(index), budget);
        let mismatch = |budget: &mut NativeJoinBudget| {
            refusal(
                SelectedMonitorAuthorityErrorCode::TimeSourceMismatch,
                self.clause(),
                Some(&row.occurrence),
                budget,
            )
        };
        work(budget)?;
        let CheckedMonitorClock::ModelTime { source, .. } = clock.clock() else {
            return Err(mismatch(budget));
        };
        work(budget)?;
        let agrees = match (admitted_time_source(self.package, source), &row.time_source) {
            (Some(AdmittedTimeSource::Dense), SelectedMonitorTimeSource::Dense) => true,
            (
                Some(AdmittedTimeSource::Tick { period }),
                SelectedMonitorTimeSource::Tick { period: exact },
            ) => {
                work(budget)?;
                same_period(period, exact, budget)?
            }
            _ => false,
        };
        if !agrees {
            return Err(mismatch(budget));
        }
        read_charge(budget, NativeJoinChargeKind::Storage, 1).map_err(|_| native_error(budget))?;
        Ok(SelectedMonitorAuthorityReceipt {
            clause: self.clause(),
            occurrence: row.occurrence.clone(),
            clock: clock.clone(),
            order_value: row.order_value,
            progress_association: row.progress_association,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn period(numerator: &str, denominator: &str) -> CheckedRational {
        CheckedRational {
            numerator: numerator.into(),
            denominator: denominator.into(),
        }
    }

    #[trace("TC-471", "FR-361-AC-1")]
    #[test]
    fn exact_period_comparison_rejects_invalid_denominators_and_text() {
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        assert!(same_period("2", &period("4", "2"), &mut budget).unwrap());
        assert!(same_period("2/3", &period("4", "6"), &mut budget).unwrap());
        for row in [
            period("0", "0"),
            period("2", "0"),
            period("-2", "1"),
            period("x", "1"),
            period("", "1"),
            period("2", "3"),
        ] {
            assert!(!same_period("2", &row, &mut budget).unwrap(), "{row:?}");
        }
    }

    #[trace("TC-471", "FR-361-AC-4")]
    #[test]
    fn exact_period_comparison_reserves_work_and_scratch_at_the_authority_locus() {
        let row = period(
            "40000000000000000000000000000000000000000000000000",
            "20000000000000000000000000000000000000000000000000",
        );
        let location = NativeJoinLocus {
            source: NativeJoinLocusSource::MonitorSeed(1),
            form: FORM,
        };
        let read = |budget: &mut NativeJoinBudget| {
            budget.set_active_locus(Some(location));
            same_period("2", &row, budget)
        };
        let mut measured = NativeJoinBudget::new(u64::MAX, u64::MAX);
        assert!(read(&mut measured).unwrap());
        let spent = measured.consumed();
        assert!(read(&mut NativeJoinBudget::new(spent.work, spent.storage)).unwrap());
        for (budget, expected) in [
            (
                NativeJoinBudget::new(spent.work - 1, spent.storage),
                SelectedMonitorTypeClosureErrorCode::WorkLimit,
            ),
            (
                NativeJoinBudget::new(spent.work, spent.storage - 1),
                SelectedMonitorTypeClosureErrorCode::StorageLimit,
            ),
        ] {
            let mut budget = budget;
            let error = read(&mut budget).expect_err("one below refuses before arithmetic");
            assert_eq!(error.code, expected);
            assert_eq!(error.denied_locus, Some(location));
            assert!(error.denied.is_some());
        }
    }

    fn identity(text: &str, ordinal: u64) -> CheckedNodeId {
        CheckedNodeId {
            package: "ir-owned-authority-test".into(),
            path: vec![CheckedNodePathSegment::Name(text.into())],
            ordinal,
        }
    }

    #[trace("TC-471", "FR-361-AC-4")]
    #[test]
    fn clause_comparison_prepays_large_equal_prefixes_and_nested_components() {
        let prefix = "prefix".repeat(2048);
        let left = identity(&format!("{prefix}a"), 0);
        let right = identity(&format!("{prefix}b"), 0);
        let mut measured = NativeJoinBudget::new(u64::MAX, u64::MAX);
        locus(NativeJoinLocusSource::MonitorAuthority, &mut measured);
        assert!(!same_clause(&left, &right, &mut measured).unwrap());
        let spent = measured.consumed();
        assert!(spent.work > 2 * prefix.len() as u64);
        assert_eq!(spent.storage, 0, "flat borrowed comparisons own no scratch");
        let mut exact = NativeJoinBudget::new(spent.work, 0);
        assert!(!same_clause(&left, &right, &mut exact).unwrap());
        let mut below = NativeJoinBudget::new(spent.work - 1, 0);
        locus(NativeJoinLocusSource::MonitorAuthority, &mut below);
        let error = same_clause(&left, &right, &mut below).unwrap_err();
        assert_eq!(error.code, SelectedMonitorTypeClosureErrorCode::WorkLimit);
        assert_eq!(error.denied_locus, Some(NativeJoinLocus { source: NativeJoinLocusSource::MonitorAuthority, form: FORM }));
        assert!(error.denied.unwrap().amount > 1);

        let wrap = |child| CheckedNodeId {
            package: "nested".into(),
            path: vec![CheckedNodePathSegment::Node(std::sync::Arc::new(child))],
            ordinal: 0,
        };
        let left = wrap(left);
        let right = wrap(identity(&format!("{prefix}a"), 0));
        let mut measured = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let mut equal = None;
        let full_allocation = allocation_counter::measure(|| equal = Some(same_clause(&left, &right, &mut measured)));
        assert!(equal.unwrap().unwrap());
        let spent = measured.consumed();
        assert!(spent.work > 2 * prefix.len() as u64);
        assert!(spent.storage > 4, "the shared comparator's scratch is prepaid after both identity walks");
        assert!(same_clause(&left, &right, &mut NativeJoinBudget::new(spent.work, spent.storage)).unwrap());
        let mut below = NativeJoinBudget::new(spent.work, spent.storage - 1);
        let mut result = None;
        let allocated = allocation_counter::measure(|| result = Some(same_clause(&left, &right, &mut below)));
        assert_eq!(allocated.count_total + 1, full_allocation.count_total, "the unpaid comparator scratch block must not allocate");
        let error = result.unwrap().unwrap_err();
        assert_eq!(error.code, SelectedMonitorTypeClosureErrorCode::StorageLimit);
        assert!(error.denied.unwrap().amount > 1);
        let dependent = |package: &str, node| CheckedNodeId {
            package: "nested-dependency".into(),
            path: vec![CheckedNodePathSegment::Node(std::sync::Arc::new(CheckedNodeId {
                package: "child".into(),
                path: vec![CheckedNodePathSegment::Dependency { package: package.into(), node }],
                ordinal: 0,
            }))],
            ordinal: 0,
        };
        let selected = dependent("producer", 7);
        for (candidate, expected) in [(dependent("producer", 7), true), (dependent("producer-next", 7), false), (dependent("producer", 8), false)] {
            assert_eq!(same_clause(&selected, &candidate, &mut NativeJoinBudget::new(u64::MAX, u64::MAX)).unwrap(), expected, "nested dependency package and position keep their shared structural semantics");
        }
    }

    #[trace("TC-471", "FR-361-AC-4")]
    #[test]
    fn period_scan_is_prepaid_before_searching_for_a_separator() {
        let digits = "7".repeat(4096);
        let row = period("2", "1");
        for admitted in [digits.clone(), format!("{digits}/1")] {
            let width = admitted.len() as u128 + row.numerator.len() as u128 + row.denominator.len() as u128 + 1;
            let charge = width * width + width;
            for allowance in [0, u64::try_from(charge).unwrap() - 1] {
                let mut budget = NativeJoinBudget::new(allowance, u64::MAX);
                locus(NativeJoinLocusSource::MonitorSeed(0), &mut budget);
                let mut result = None;
                let allocated = allocation_counter::measure(|| result = Some(same_period(&admitted, &row, &mut budget)));
                assert_eq!(allocated.count_total, 0);
                let error = result.unwrap().unwrap_err();
                assert_eq!(error.code, SelectedMonitorTypeClosureErrorCode::WorkLimit);
                assert_eq!(error.work, 0, "the scan reservation is the first inspection");
                assert_eq!(error.denied, Some(crate::NativeJoinDeniedCharge { kind: NativeJoinChargeKind::Work, amount: charge }));
                assert_eq!(error.denied_locus, Some(NativeJoinLocus { source: NativeJoinLocusSource::MonitorSeed(0), form: FORM }));
            }
        }
    }

    #[trace("TC-471", "FR-358-AC-3")]
    #[test]
    fn authority_refusal_prepays_its_complete_owned_diagnostic_or_stays_compact() {
        let mut at = identity(&"diagnostic".repeat(128), 4);
        let dependency = "diagnostic-selected-dependency".repeat(64);
        at.path.push(CheckedNodePathSegment::Dependency { package: dependency.clone().into(), node: 3 });
        let occurrence = CheckedOccurrence { role: CheckedOccurrenceRole::Claim, ordinal: 7 };
        let floor = storage::node(&at) + 2 + 1 + dependency.len() as u128;
        let code = SelectedMonitorAuthorityErrorCode::OccurrenceMismatch;
        let mut measured = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let full = refusal(code, &at, Some(&occurrence), &mut measured);
        assert_eq!(full.code, SelectedMonitorTypeClosureErrorCode::Authority(code));
        assert_eq!(full.at.as_deref(), Some(&at));
        assert_eq!(full.occurrence, Some(occurrence.clone()));
        assert_eq!(measured.consumed().storage as u128, floor);
        let mut exact = NativeJoinBudget::new(measured.consumed().work, u64::try_from(floor).unwrap());
        assert_eq!(refusal(code, &at, Some(&occurrence), &mut exact), full);
        for storage in [0, u64::try_from(floor).unwrap() - 1] {
            let mut budget = NativeJoinBudget::new(u64::MAX, storage);
            locus(NativeJoinLocusSource::MonitorAuthority, &mut budget);
            let mut result = None;
            let allocated = allocation_counter::measure(|| result = Some(refusal(code, &at, Some(&occurrence), &mut budget)));
            assert_eq!(allocated.count_total, 0, "an unaffordable diagnostic owns nothing");
            let compact = result.unwrap();
            assert_eq!(compact.code, SelectedMonitorTypeClosureErrorCode::StorageLimit);
            assert_eq!(compact.at, None);
            assert_eq!(compact.occurrence, None);
            assert_eq!(compact.work, full.work);
            assert_eq!(compact.denied, Some(crate::NativeJoinDeniedCharge { kind: NativeJoinChargeKind::Storage, amount: floor }));
            assert_eq!(compact.denied_locus, Some(NativeJoinLocus { source: NativeJoinLocusSource::MonitorAuthority, form: FORM }));
            assert_eq!(budget.consumed().storage, 0);
        }
    }

}
