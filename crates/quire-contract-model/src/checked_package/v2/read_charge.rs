// SPDX-License-Identifier: AGPL-3.0-or-later
//! Charging helpers of the strict V2 read's wire-check stages: every heap
//! block is charged to the caller's budget, at the stage locus, before it is
//! requested.
//!
//! Implements: FR-038 "Budget-charged strict read".

use super::budgeted_read::denial;
use super::native_join_budget::{
    NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
};
use crate::checked_package::common::ValidationFailure;
use std::sync::Arc;

/// The smallest capacity a charged list grows to, and its growth factor: the
/// policy of `quire-walk` and `quire-canonical`, so one account stays exact.
const MINIMUM_CAPACITY: usize = 4;

/// A charge site: the stage, the index its table names and its static form.
#[derive(Clone, Copy, Debug)]
pub(super) struct Site {
    pub(super) stage: PackageReadStage,
    pub(super) index: usize,
    pub(super) form: &'static str,
}

impl Site {
    pub(super) fn new(stage: PackageReadStage, index: usize, form: &'static str) -> Self {
        Self { stage, index, form }
    }

    /// The same site with another form.
    pub(super) fn form(self, form: &'static str) -> Self {
        Self { form, ..self }
    }

    pub(super) fn locus(self) -> NativeJoinLocus {
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(self.stage, self.index),
            form: self.form,
        }
    }
}

/// Charges `bytes` of read storage at `site`.
pub(super) fn charge(
    budget: &mut NativeJoinBudget,
    site: Site,
    bytes: u128,
) -> Result<(), ValidationFailure> {
    budget.set_active_locus(Some(site.locus()));
    if budget.charge_read_storage(bytes) {
        Ok(())
    } else {
        Err(denial(budget))
    }
}

/// Charges `count` items of `T`.
pub(super) fn charge_items<T>(
    budget: &mut NativeJoinBudget,
    site: Site,
    count: usize,
) -> Result<(), ValidationFailure> {
    charge(
        budget,
        site,
        (count as u128).saturating_mul(std::mem::size_of::<T>() as u128),
    )
}

/// A vector of exactly `count` items of capacity, charged first.
pub(super) fn with_capacity<T>(
    budget: &mut NativeJoinBudget,
    site: Site,
    count: usize,
) -> Result<Vec<T>, ValidationFailure> {
    charge_items::<T>(budget, site, count)?;
    Ok(Vec::with_capacity(count))
}

/// A vector of `count` clones of `item`, charged first.
pub(super) fn filled<T: Clone>(
    budget: &mut NativeJoinBudget,
    site: Site,
    count: usize,
    item: T,
) -> Result<Vec<T>, ValidationFailure> {
    charge_items::<T>(budget, site, count)?;
    Ok(vec![item; count])
}

/// Pushes `item`, charging the full new capacity first when the list grows
/// (doubling from four, reserved exactly).
pub(super) fn push<T>(
    budget: &mut NativeJoinBudget,
    site: Site,
    list: &mut Vec<T>,
    item: T,
) -> Result<(), ValidationFailure> {
    reserve(budget, site, list, 1)?;
    list.push(item);
    Ok(())
}

/// Makes room for `additional` more items, charging the full new capacity
/// first when the list grows.
pub(super) fn reserve<T>(
    budget: &mut NativeJoinBudget,
    site: Site,
    list: &mut Vec<T>,
    additional: usize,
) -> Result<(), ValidationFailure> {
    let needed = list.len().saturating_add(additional);
    let capacity = list.capacity();
    if needed <= capacity {
        return Ok(());
    }
    let target = needed.max(capacity.saturating_mul(2)).max(MINIMUM_CAPACITY);
    charge_items::<T>(budget, site, target)?;
    list.try_reserve_exact(target - list.len())
        .map_err(|_| denial(budget))
}

/// The bytes of the block `Arc::new` requests for a `T`: the strong and weak
/// counts and the value.
pub(super) fn arc_block_bytes<T>() -> usize {
    let align = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
    (2 * std::mem::size_of::<usize>() + std::mem::size_of::<T>()).next_multiple_of(align)
}

/// `Arc::new(value)`, its block charged first.
pub(super) fn new_arc<T>(
    budget: &mut NativeJoinBudget,
    site: Site,
    value: T,
) -> Result<Arc<T>, ValidationFailure> {
    charge(budget, site, arc_block_bytes::<T>() as u128)?;
    Ok(Arc::new(value))
}

/// A boxed copy of `text`, charged first.
pub(super) fn boxed_str(
    budget: &mut NativeJoinBudget,
    site: Site,
    text: &str,
) -> Result<Box<str>, ValidationFailure> {
    charge(budget, site, text.len() as u128)?;
    Ok(text.into())
}

/// Sets no active locus: the stage is over.
pub(super) fn end_stage(budget: &mut NativeJoinBudget) {
    budget.set_active_locus(None);
}

/// A reservation for the walker: its growths charge the budget at `site`, the
/// calling check's stage and index with the form `"walk"`.
pub(super) struct WalkReservation<'b> {
    pub(super) budget: &'b mut NativeJoinBudget,
    pub(super) site: Site,
}

impl<'b> WalkReservation<'b> {
    pub(super) fn new(budget: &'b mut NativeJoinBudget, site: Site) -> Self {
        Self {
            budget,
            site: site.form("walk"),
        }
    }
}

impl quire_walk::Reservation for WalkReservation<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_walk::Denied> {
        self.budget.set_active_locus(Some(self.site.locus()));
        if self.budget.charge_read_storage(bytes as u128) {
            Ok(())
        } else {
            Err(quire_walk::Denied)
        }
    }
}

/// Runs `walker` over `root` on a heap stack whose every growth is charged
/// through `reservation`, and returns the denial the budget recorded when a
/// growth is refused.
pub(super) fn walk<W: quire_walk::Walk + ?Sized>(
    walker: &mut W,
    root: W::Node,
    reservation: &mut WalkReservation<'_>,
) -> Result<std::ops::ControlFlow<W::Stop>, ValidationFailure> {
    quire_walk::walk_reserved(walker, root, reservation).map_err(|_| denial(reservation.budget))
}

/// Runs `walk` with a reservation that charges nothing. Only a walk whose
/// calling stage has not yet threaded the read's budget to it uses this; that
/// stage replaces the call with a [`WalkReservation`] over the budget.
pub(super) fn with_unmetered_walk<R>(walk: impl FnOnce(&mut WalkReservation<'_>) -> R) -> R {
    let mut budget = NativeJoinBudget::unlimited();
    let site = Site::new(PackageReadStage::Graph, 0, "walk");
    walk(&mut WalkReservation::new(&mut budget, site))
}
