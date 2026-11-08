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
use crate::checked_package::shared::CheckedOperandEntry;
use std::cell::{RefCell, RefMut};
use std::sync::Arc;

/// The smallest capacity a charged list grows to, and its growth factor: the
/// policy of `quire-walk` and `quire-canonical`, so one account stays exact.
const MINIMUM_CAPACITY: usize = 4;

/// A charge site: the stage, the index its table names and its static form.
#[derive(Clone, Copy, Debug)]
pub(in crate::checked_package) struct Site {
    pub(super) stage: PackageReadStage,
    pub(super) index: usize,
    pub(super) form: &'static str,
}

impl Site {
    pub(in crate::checked_package) fn new(
        stage: PackageReadStage,
        index: usize,
        form: &'static str,
    ) -> Self {
        Self { stage, index, form }
    }

    /// The same site with another form.
    pub(in crate::checked_package) fn form(self, form: &'static str) -> Self {
        Self { form, ..self }
    }

    pub(super) fn locus(self) -> NativeJoinLocus {
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(self.stage, self.index),
            form: self.form,
        }
    }
}

/// A charge the sink refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct Denied;

/// Where a read's heap requests are charged: the caller's budget, or a meter
/// with a limit of its own. A request is charged before it is made.
pub(in crate::checked_package) trait ReadStorage {
    /// Admits `bytes` of heap about to be requested at `site`, or refuses.
    fn reserve(&mut self, bytes: u128, site: Site) -> Result<(), Denied>;

    /// The stop that ends the read after this sink refused a request.
    fn denial(&self) -> ValidationFailure;
}

impl ReadStorage for NativeJoinBudget {
    fn reserve(&mut self, bytes: u128, site: Site) -> Result<(), Denied> {
        self.set_active_locus(Some(site.locus()));
        if self.charge_read_storage(bytes) {
            Ok(())
        } else {
            Err(Denied)
        }
    }

    fn denial(&self) -> ValidationFailure {
        denial(self)
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

/// The caller's budget, shared by the parts of one metered walk: the walker's
/// own stacks and the code that walks (a cursor's steps), each charged before
/// it grows.
pub(super) struct Meter<'b> {
    budget: RefCell<&'b mut NativeJoinBudget>,
}

impl<'b> Meter<'b> {
    pub(super) fn new(budget: &'b mut NativeJoinBudget) -> Self {
        Self {
            budget: RefCell::new(budget),
        }
    }

    /// The budget, for the stage's own charges between walks.
    pub(super) fn budget(&self) -> RefMut<'_, &'b mut NativeJoinBudget> {
        self.budget.borrow_mut()
    }

    /// Charges `bytes` at `site`.
    pub(super) fn charge(&self, site: Site, bytes: u128) -> Result<(), ValidationFailure> {
        charge(&mut self.budget.borrow_mut(), site, bytes)
    }

    /// A reservation for a walk of the check at `check`; the walker's own
    /// growths are charged with the form `"walk"`.
    pub(super) fn reservation(&self, check: Site) -> WalkReservation<'_> {
        WalkReservation { meter: self, check }
    }
}

/// What a metered walk charges: the read's budget through a [`Meter`], or any
/// [`ReadStorage`] sink through a [`SinkCell`].
pub(super) trait ChargeAt {
    /// Charges `bytes` at `site`.
    fn charge(&self, site: Site, bytes: u128) -> Result<(), ValidationFailure>;

    /// The stop that ends the read after a refused charge.
    fn denial(&self) -> ValidationFailure;
}

impl ChargeAt for Meter<'_> {
    fn charge(&self, site: Site, bytes: u128) -> Result<(), ValidationFailure> {
        Meter::charge(self, site, bytes)
    }

    fn denial(&self) -> ValidationFailure {
        denial(&self.budget())
    }
}

/// A [`ReadStorage`] sink shared by the parts of one metered walk.
pub(super) struct SinkCell<'s> {
    sink: RefCell<&'s mut dyn ReadStorage>,
}

impl<'s> SinkCell<'s> {
    pub(super) fn new(sink: &'s mut dyn ReadStorage) -> Self {
        Self {
            sink: RefCell::new(sink),
        }
    }

    /// A reservation for a walk of the check at `check`.
    pub(super) fn reservation(&self, check: Site) -> WalkReservation<'_> {
        WalkReservation { meter: self, check }
    }
}

impl ChargeAt for SinkCell<'_> {
    fn charge(&self, site: Site, bytes: u128) -> Result<(), ValidationFailure> {
        let mut sink = self.sink.borrow_mut();
        sink.reserve(bytes, site).map_err(|_| sink.denial())
    }

    fn denial(&self) -> ValidationFailure {
        self.sink.borrow().denial()
    }
}

/// A reservation for the walker: its growths charge the budget at the calling
/// check's stage and index with the form `"walk"`.
pub(super) struct WalkReservation<'m> {
    pub(super) meter: &'m dyn ChargeAt,
    /// The calling check's site: its form names the check's own temporaries.
    pub(super) check: Site,
}

impl quire_walk::Reservation for WalkReservation<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_walk::Denied> {
        self.meter
            .charge(self.check.form("walk"), bytes as u128)
            .map_err(|_| quire_walk::Denied)
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
    let meter = reservation.meter;
    quire_walk::walk_reserved(walker, root, reservation).map_err(|_| meter.denial())
}

/// Runs `walk` with a reservation that charges nothing. To be deleted with
/// its two `refinement.rs` callers when that file moves to the node views.
pub(super) fn with_unmetered_walk<R>(walk: impl FnOnce(&mut WalkReservation<'_>) -> R) -> R {
    let mut budget = NativeJoinBudget::unlimited();
    let meter = Meter::new(&mut budget);
    let site = Site::new(PackageReadStage::Graph, 0, "walk");
    walk(&mut meter.reservation(site))
}

/// An [`Ids`](crate::checked_package::common::Ids) charging `sink` at the
/// given stage, index and form, reading operand slots through `row`.
pub(in crate::checked_package) fn ids<'s, 'r>(
    sink: &'s mut dyn ReadStorage,
    stage: PackageReadStage,
    index: usize,
    form: &'static str,
    row: &'r [CheckedOperandEntry],
) -> crate::checked_package::common::Ids<'s, 'r> {
    crate::checked_package::common::Ids {
        sink,
        site: Site::new(stage, index, form),
        row,
    }
}
