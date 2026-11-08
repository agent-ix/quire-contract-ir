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
use std::cell::{RefCell, RefMut};
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
    pub(super) fn reservation(&self, check: Site) -> WalkReservation<'_, 'b> {
        WalkReservation { meter: self, check }
    }
}

/// A reservation for the walker: its growths charge the budget at the calling
/// check's stage and index with the form `"walk"`.
pub(super) struct WalkReservation<'m, 'b> {
    pub(super) meter: &'m Meter<'b>,
    /// The calling check's site: its form names the check's own temporaries.
    pub(super) check: Site,
}

impl quire_walk::Reservation for WalkReservation<'_, '_> {
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
    reservation: &mut WalkReservation<'_, '_>,
) -> Result<std::ops::ControlFlow<W::Stop>, ValidationFailure> {
    let meter = reservation.meter;
    quire_walk::walk_reserved(walker, root, reservation).map_err(|_| denial(&meter.budget()))
}

/// Runs `walk` with a reservation that charges nothing. Only a walk whose
/// calling stage has not yet threaded the read's budget to it uses this; that
/// stage replaces the call with a reservation over the budget.
pub(super) fn with_unmetered_walk<R>(walk: impl FnOnce(&mut WalkReservation<'_, '_>) -> R) -> R {
    let mut budget = NativeJoinBudget::unlimited();
    let meter = Meter::new(&mut budget);
    let site = Site::new(PackageReadStage::Graph, 0, "walk");
    walk(&mut meter.reservation(site))
}

/// The node ID the JSON value `value` holds as a V2 wire ID, or `None` when it
/// is not one: an object of exactly a `package` string, a `path` of names and
/// `{"node": i}` / `{"operand": k}` segments no larger than 2^53 - 1, and an
/// `ordinal`. With `well_formed`, the package and path are also nonempty and
/// the ordinal no larger than 2^53 - 1. The ID's package, path and names are
/// charged at `site` before each is allocated.
pub(super) fn node_id_from_value(
    budget: &mut NativeJoinBudget,
    site: Site,
    value: &serde_json::Value,
    well_formed: bool,
) -> Result<Option<super::CheckedNodeId>, ValidationFailure> {
    use super::CheckedNodePathSegment as Segment;
    const MAXIMUM: u64 = 9_007_199_254_740_991;
    let Some(object) = value.as_object() else {
        return Ok(None);
    };
    let (Some(package), Some(path), Some(ordinal)) = (
        object.get("package").and_then(serde_json::Value::as_str),
        object.get("path").and_then(serde_json::Value::as_array),
        object.get("ordinal").and_then(serde_json::Value::as_u64),
    ) else {
        return Ok(None);
    };
    if object.len() != 3
        || (well_formed && (package.is_empty() || path.is_empty() || ordinal > MAXIMUM))
    {
        return Ok(None);
    }
    // Check every segment before the first allocation.
    for segment in path {
        let valid = match segment {
            serde_json::Value::String(_) => true,
            serde_json::Value::Object(map) if map.len() == 1 => {
                map.iter().next().is_some_and(|(key, index)| {
                    matches!(key.as_str(), "node" | "operand")
                        && index.as_u64().is_some_and(|index| index <= MAXIMUM)
                })
            }
            _ => false,
        };
        if !valid {
            return Ok(None);
        }
    }
    let package = boxed_str(budget, site, package)?;
    let mut segments = with_capacity(budget, site, path.len())?;
    for segment in path {
        segments.push(match segment {
            serde_json::Value::String(name) => Segment::Name(boxed_str(budget, site, name)?),
            serde_json::Value::Object(map) => match map.iter().next() {
                Some((key, index)) if key == "operand" => {
                    Segment::Operand(index.as_u64().unwrap_or_default())
                }
                Some((_, index)) => Segment::Index(index.as_u64().unwrap_or_default()),
                None => return Ok(None),
            },
            _ => return Ok(None),
        });
    }
    Ok(Some(super::CheckedNodeId {
        package,
        path: segments,
        ordinal,
    }))
}

/// The target of a `reference` term, as an owned V2 node ID, or `None` when
/// the term is no reference or its target is not a well-formed ID. The ID's
/// package, path and names are charged at `site` before they are allocated.
pub(super) fn reference_target(
    budget: &mut NativeJoinBudget,
    site: Site,
    term: &serde_json::Value,
) -> Result<Option<super::CheckedNodeId>, ValidationFailure> {
    match super::CheckedTerm::view(term) {
        Some(super::CheckedTermView::Reference { target }) => {
            node_id_from_value(budget, site, target, true)
        }
        _ => Ok(None),
    }
}
