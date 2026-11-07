// SPDX-License-Identifier: AGPL-3.0-or-later
//! Charged heap requests of the graph stage: the read site a request is
//! charged at, exact-capacity containers, sorted lookup tables, and the
//! reservations the walker and the exact-number library ask through.
//!
//! Implements: FR-038 "Budget-charged strict read" (stage `Graph`), FR-358.
//!
//! Every request is charged to the caller budget's read-storage counter, at
//! `PackageRead(Graph, index)`, before it is made. A meter that holds its own
//! limit (lowering, unit tests) makes the same requests uncharged.

use super::budgeted_read::{BudgetStop, PackageReadResource};
use super::native_join_budget::{NativeJoinLocus, NativeJoinLocusSource, PackageReadStage};
use super::{WorkMeter, WorkSource};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{CheckedNodeId, JsonPointer};
use std::cmp::Ordering;

/// The value of the package being validated when a request is made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SiteValue {
    /// Not attributable to one package value: a denied request carries no
    /// pointer.
    Package,
    /// The graph node at this position.
    Node(usize),
    /// One row of a sidecar member of the package.
    Row(&'static str, usize),
}

/// Where the graph stage's next request is charged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReadSite {
    pub(super) index: usize,
    pub(super) form: &'static str,
    pub(super) value: SiteValue,
}

impl ReadSite {
    fn locus(self, form: &'static str) -> NativeJoinLocus {
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, self.index),
            form,
        }
    }

    fn pointer(self) -> Option<JsonPointer> {
        match self.value {
            SiteValue::Package => None,
            SiteValue::Node(position) => Some(node_pointer(position)),
            SiteValue::Row(member, row) => Some(JsonPointer::root().key(member).index(row)),
        }
    }
}

impl<'b> WorkMeter<'b> {
    /// Charges the requests that follow to graph node `position`, whose wire
    /// `node_tag` is `form`.
    pub(super) fn at_node(&mut self, position: usize, form: &'static str) {
        self.site = Some(ReadSite {
            index: position,
            form,
            value: SiteValue::Node(position),
        });
    }

    /// Charges the requests that follow to row `row` of the sidecar `member`.
    pub(super) fn at_row(&mut self, member: &'static str, row: usize) {
        self.site = Some(ReadSite {
            index: row,
            form: member,
            value: SiteValue::Row(member, row),
        });
    }

    /// The site now charged.
    pub(super) fn site(&self) -> Option<ReadSite> {
        self.site
    }

    /// Restores a site returned by [`Self::site`].
    pub(super) fn restore_site(&mut self, site: Option<ReadSite>) {
        self.site = site;
    }

    /// Admits `bytes` of heap at the current site before it is requested, with
    /// the site's own form or `form` when the request belongs to a library the
    /// check calls.
    fn reserve_as(
        &mut self,
        bytes: u128,
        form: Option<&'static str>,
    ) -> Result<(), ValidationFailure> {
        let WorkSource::Budget(budget) = &mut self.source else {
            return Ok(());
        };
        let site = self.site.unwrap_or(ReadSite {
            index: 0,
            form: "graph",
            value: SiteValue::Package,
        });
        let previous = budget.active_locus();
        budget.set_active_locus(Some(site.locus(form.unwrap_or(site.form))));
        let admitted = budget.charge_read_storage(bytes);
        budget.set_active_locus(previous);
        if admitted {
            Ok(())
        } else {
            Err(ValidationFailure::Budget(BudgetStop {
                resource: PackageReadResource::ReadStorage,
                amount: bytes,
                // A request made for a library's own heap belongs to no one
                // value.
                path: if form.is_some() { None } else { site.pointer() },
            }))
        }
    }

    /// Admits `bytes` of heap before the check requests it.
    pub(super) fn reserve(&mut self, bytes: usize) -> Result<(), ValidationFailure> {
        if bytes == 0 {
            return Ok(());
        }
        self.reserve_as(bytes as u128, None)
    }

    /// Admits `count` items of `T` before the check requests a block for them.
    pub(super) fn reserve_items<T>(&mut self, count: usize) -> Result<(), ValidationFailure> {
        let bytes = (count as u128).saturating_mul(std::mem::size_of::<T>() as u128);
        if bytes == 0 {
            return Ok(());
        }
        self.reserve_as(bytes, None)
    }

    /// An empty vector with room for exactly `capacity` items, charged first.
    pub(super) fn vec<T>(&mut self, capacity: usize) -> Result<Vec<T>, ValidationFailure> {
        self.reserve_items::<T>(capacity)?;
        Ok(Vec::with_capacity(capacity))
    }

    /// Appends `item`, growing a full vector to double its capacity (at
    /// least four) after charging that whole new block.
    pub(super) fn push<T>(&mut self, items: &mut Vec<T>, item: T) -> Result<(), ValidationFailure> {
        self.room(items, 1)?;
        items.push(item);
        Ok(())
    }

    /// Makes room in `items` for `additional` more, growing to the larger of
    /// the needed length and double the capacity (at least four), charging
    /// the whole new block before it is requested.
    pub(super) fn room<T>(
        &mut self,
        items: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), ValidationFailure> {
        let needed = items.len().saturating_add(additional);
        if needed <= items.capacity() {
            return Ok(());
        }
        let target = needed.max(items.capacity().saturating_mul(2)).max(4);
        self.reserve_items::<T>(target)?;
        items.reserve_exact(target - items.len());
        Ok(())
    }

    /// A vector of `count` copies of `value`, charged first.
    pub(super) fn filled<T: Clone>(
        &mut self,
        count: usize,
        value: T,
    ) -> Result<Vec<T>, ValidationFailure> {
        let mut items = self.vec(count)?;
        items.resize(count, value);
        Ok(items)
    }

    /// A string with room for exactly `capacity` bytes, charged first.
    pub(super) fn string(&mut self, capacity: usize) -> Result<String, ValidationFailure> {
        self.reserve(capacity)?;
        Ok(String::with_capacity(capacity))
    }

    /// An owned copy of `text` at its exact length, charged first.
    pub(super) fn boxed_str(&mut self, text: &str) -> Result<Box<str>, ValidationFailure> {
        self.reserve(text.len())?;
        Ok(Box::from(text))
    }

    /// A copy of `items` at its exact length, charged first.
    pub(super) fn copy_slice<T: Clone>(
        &mut self,
        items: &[T],
    ) -> Result<Vec<T>, ValidationFailure> {
        let mut copy = self.vec(items.len())?;
        copy.extend_from_slice(items);
        Ok(copy)
    }

    /// The reservation a term walk asks before each growth of its lists.
    pub(super) fn walk_reservation<'m>(&'m mut self) -> MeterReservation<'m, 'b> {
        MeterReservation::new(self.reborrow_for_reservation(), Some("walk"))
    }

    /// The reservation a quire-exact operation asks before each limb block.
    pub(super) fn exact_reservation<'m>(&'m mut self) -> MeterReservation<'m, 'b> {
        MeterReservation::new(self.reborrow_for_reservation(), Some("exact_number"))
    }

    /// Runs `walker` over `root` on a walk whose list growths are charged to
    /// the meter, form `"walk"`.
    pub(super) fn walk<W: quire_walk::Walk + ?Sized>(
        &mut self,
        walker: &mut W,
        root: W::Node,
    ) -> Result<std::ops::ControlFlow<W::Stop>, ValidationFailure> {
        let mut reservation = self.walk_reservation();
        match quire_walk::walk_reserved(walker, root, &mut reservation) {
            Ok(flow) => Ok(flow),
            Err(refusal) => Err(reservation.into_denial().unwrap_or_else(|| {
                let requested = match refusal {
                    quire_walk::Refusal::Denied { requested }
                    | quire_walk::Refusal::Allocation { requested } => requested,
                };
                ValidationFailure::Budget(BudgetStop {
                    resource: PackageReadResource::ReadStorage,
                    amount: requested as u128,
                    path: None,
                })
            })),
        }
    }

    fn reborrow_for_reservation(&mut self) -> &mut Self {
        self
    }
}

/// A reservation over the meter's read storage, remembering the denial it
/// returned so the check can end its read with it.
pub(super) struct MeterReservation<'m, 'b> {
    meter: &'m mut WorkMeter<'b>,
    form: Option<&'static str>,
    denial: Option<ValidationFailure>,
}

impl<'m, 'b> MeterReservation<'m, 'b> {
    fn new(meter: &'m mut WorkMeter<'b>, form: Option<&'static str>) -> Self {
        Self {
            meter,
            form,
            denial: None,
        }
    }

    /// The denial this reservation returned, if any.
    pub(super) fn into_denial(self) -> Option<ValidationFailure> {
        self.denial
    }

    fn ask(&mut self, bytes: usize) -> Result<(), ()> {
        match self.meter.reserve_as(bytes as u128, self.form) {
            Ok(()) => Ok(()),
            Err(denial) => {
                self.denial = Some(denial);
                Err(())
            }
        }
    }
}

impl quire_walk::Reservation for MeterReservation<'_, '_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_walk::Denied> {
        self.ask(bytes).map_err(|()| quire_walk::Denied)
    }
}

impl quire_exact::Reservation for MeterReservation<'_, '_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_exact::ReservationDenied> {
        self.ask(bytes).map_err(|()| quire_exact::ReservationDenied)
    }
}

/// The ordered table of the graph: every node's id and position, sorted by id
/// then position, searched by binary search.
///
/// It replaces the ordered map the graph stage built node by node. Iteration is
/// in id order, as the map's was.
pub(super) struct GraphIndex<'g> {
    entries: Box<[(&'g CheckedNodeId, usize)]>,
}

impl<'g> GraphIndex<'g> {
    /// Builds the table of `ids` (in position order), charging its exact
    /// block first.
    pub(super) fn build(
        ids: impl ExactSizeIterator<Item = &'g CheckedNodeId>,
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let mut entries = meter.vec::<(&CheckedNodeId, usize)>(ids.len())?;
        entries.extend(ids.enumerate().map(|(position, id)| (id, position)));
        entries.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
        Ok(Self {
            entries: entries.into_boxed_slice(),
        })
    }

    /// The position of the first node with this id, as a map lookup.
    pub(super) fn get(&self, id: &CheckedNodeId) -> Option<&usize> {
        let at = self
            .entries
            .partition_point(|(entry, _)| entry.cmp(&id) == Ordering::Less);
        self.entries
            .get(at)
            .filter(|(entry, _)| *entry == id)
            .map(|(_, position)| position)
    }

    pub(super) fn contains_key(&self, id: &CheckedNodeId) -> bool {
        self.get(id).is_some()
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Positions in ascending id order.
    pub(super) fn values(&self) -> impl Iterator<Item = &usize> + '_ {
        self.entries.iter().map(|(_, position)| position)
    }

    /// Ids and positions in ascending id order.
    pub(super) fn iter(&self) -> impl Iterator<Item = (&&'g CheckedNodeId, &usize)> + '_ {
        self.entries.iter().map(|(id, position)| (id, position))
    }

    /// Ids in ascending order.
    pub(super) fn keys(&self) -> impl Iterator<Item = &&'g CheckedNodeId> + '_ {
        self.entries.iter().map(|(id, _)| id)
    }

    /// The position of the first node whose id repeats an earlier node's.
    pub(super) fn first_repeat(&self) -> Option<usize> {
        self.entries
            .windows(2)
            .filter(|pair| pair[0].0 == pair[1].0)
            .map(|pair| pair[1].1)
            .min()
    }
}

#[cfg(test)]
impl<'g> FromIterator<(&'g CheckedNodeId, usize)> for GraphIndex<'g> {
    fn from_iter<I: IntoIterator<Item = (&'g CheckedNodeId, usize)>>(entries: I) -> Self {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
        Self {
            entries: entries.into_boxed_slice(),
        }
    }
}

impl<'a, 'g> IntoIterator for &'a GraphIndex<'g> {
    type Item = (&'a &'g CheckedNodeId, &'a usize);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (&'g CheckedNodeId, usize)>,
        fn(&'a (&'g CheckedNodeId, usize)) -> (&'a &'g CheckedNodeId, &'a usize),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(id, position)| (id, position))
    }
}

/// A sorted exact-capacity table of keyed entries, searched by binary search,
/// that stands for an ordered map the stage would otherwise build.
pub(super) struct SortedTable<K, V> {
    entries: Box<[(K, V)]>,
}

impl<K: Ord, V> SortedTable<K, V> {
    /// Builds the table from `entries` held in an exact-capacity vector the
    /// caller charged; a repeated key keeps its first entry in `entries`
    /// order.
    pub(super) fn from_vec(mut entries: Vec<(K, V)>) -> Self {
        // A stable sort keeps the first of equal keys first.
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries.dedup_by(|later, earlier| later.0 == earlier.0);
        Self {
            entries: entries.into_boxed_slice(),
        }
    }

    pub(super) fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .binary_search_by(|(entry, _)| entry.cmp(key))
            .ok()
            .and_then(|at| self.entries.get(at))
            .map(|(_, value)| value)
    }

    pub(super) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &(K, V)> + '_ {
        self.entries.iter()
    }
}

/// The index of the first key of `keys` equal to an earlier one, found by
/// sorting an exact-capacity table of the keys (charged first).
pub(super) fn first_repeat<K: Ord>(
    keys: impl ExactSizeIterator<Item = K>,
    meter: &mut WorkMeter,
) -> Result<Option<usize>, ValidationFailure> {
    let mut keyed = meter.vec::<(K, usize)>(keys.len())?;
    keyed.extend(keys.enumerate().map(|(position, key)| (key, position)));
    keyed.sort_unstable();
    Ok(keyed
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min())
}

/// The static wire spelling of a node's `node_tag`, for a request's form.
pub(super) fn node_form(tag: &str) -> &'static str {
    super::CheckedNodeTag::from_wire(tag).map_or("node", super::CheckedNodeTag::as_wire)
}
