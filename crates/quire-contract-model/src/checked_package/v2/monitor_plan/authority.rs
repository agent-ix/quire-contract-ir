// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Same-emission order and progress authority bound to the selected monitor
//! closure (FR-361).

use super::selected_type_closure::SelectedMonitorTypeClosureErrorCode;
use super::selected_type_closure::{native_error, semantic_error, SelectedMonitorTypeClosureError};
use super::{CheckedMonitorClock, CheckedMonitorClockRead, CheckedMonitorPlanRef};
use crate::checked_package::v2::native_join_budget::read_charge;
use crate::checked_package::v2::timed_model::{admitted_time_source, AdmittedTimeSource};
use crate::checked_package::v2::{
    AdmissionBinding, CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedRational,
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
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
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
    let (numerator, denominator) = admitted.split_once('/').unwrap_or((admitted, "1"));
    let components = [numerator, denominator, &row.numerator, &row.denominator];
    let digits = components.iter().fold(0u128, |total, text| {
        total.saturating_add(text.len() as u128)
    });
    // Decimal parsing and the two schoolbook products are bounded by the
    // square of the total decimal width. Reserve their scratch before parsing.
    read_charge(
        budget,
        NativeJoinChargeKind::Work,
        digits.saturating_mul(digits).saturating_add(digits),
    )
    .map_err(|_| native_error(budget))?;
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
        work(budget)?;
        if row.clause != *self.clause() {
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
        let mismatch = |budget: &NativeJoinBudget| {
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
