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
    #[allow(dead_code)] // part of the charging toolkit the other read groups fold in
    pub(super) fn site(&self) -> Option<ReadSite> {
        self.site
    }

    /// Restores a site returned by [`Self::site`].
    #[allow(dead_code)] // part of the charging toolkit the other read groups fold in
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

    /// Puts `item` into the sorted `items`, growing a full vector under the
    /// meter; `false` when an equal item was already there.
    pub(super) fn insert_sorted<T: Ord>(
        &mut self,
        items: &mut Vec<T>,
        item: T,
    ) -> Result<bool, ValidationFailure> {
        match items.binary_search(&item) {
            Ok(_) => Ok(false),
            Err(at) => {
                self.room(items, 1)?;
                items.insert(at, item);
                Ok(true)
            }
        }
    }

    /// The order of two node ids. A comparison nested past the inline stack
    /// asks for its spill here, so the block is charged before it is made.
    pub(super) fn compare_ids(
        &mut self,
        left: &CheckedNodeId,
        right: &CheckedNodeId,
    ) -> Result<Ordering, ValidationFailure> {
        left.cmp_growing(right, &mut |bytes| self.reserve(bytes))
    }

    /// A copy of `id` (its path list and the name and package text it holds;
    /// nested nodes are shared), charged first.
    pub(super) fn clone_id(
        &mut self,
        id: &CheckedNodeId,
    ) -> Result<CheckedNodeId, ValidationFailure> {
        use crate::checked_package::shared::CheckedNodePathSegment as Segment;
        let mut bytes = id
            .path
            .len()
            .saturating_mul(std::mem::size_of::<Segment>())
            .saturating_add(id.package.len());
        for segment in &id.path {
            if let Segment::Name(name) = segment {
                bytes = bytes.saturating_add(name.len());
            }
        }
        self.reserve(bytes)?;
        Ok(id.clone())
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
    #[allow(dead_code)] // part of the charging toolkit the other read groups fold in
    pub(super) fn boxed_str(&mut self, text: &str) -> Result<Box<str>, ValidationFailure> {
        self.reserve(text.len())?;
        Ok(Box::from(text))
    }

    /// `arguments` rendered into a block of exactly its length, charged
    /// first. The length is counted by rendering into nothing.
    pub(super) fn formatted(
        &mut self,
        arguments: std::fmt::Arguments<'_>,
    ) -> Result<Box<str>, ValidationFailure> {
        use std::fmt::Write;
        struct Count(usize);
        impl Write for Count {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                self.0 = self.0.saturating_add(text.len());
                Ok(())
            }
        }
        let mut count = Count(0);
        let _ = count.write_fmt(arguments);
        let mut text = self.string(count.0)?;
        let _ = text.write_fmt(arguments);
        Ok(text.into_boxed_str())
    }

    /// A copy of `items` at its exact length, charged first.
    #[allow(dead_code)] // part of the charging toolkit the other read groups fold in
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

    /// `text` as an exact integer, or `None` when it is not the canonical
    /// spelling. The limb block a large value needs is charged first.
    pub(super) fn exact_integer(
        &mut self,
        text: &str,
    ) -> Result<Option<quire_exact::Integer>, ValidationFailure> {
        let mut reservation = self.exact_reservation();
        match quire_exact::Integer::parse_reserved(text, &mut reservation) {
            Ok(value) => Ok(Some(value)),
            Err(quire_exact::IntegerParseError::NonCanonical(_)) => Ok(None),
            Err(quire_exact::IntegerParseError::Reserve(error)) => Err(reservation.failure(error)),
        }
    }

    /// `text` as a canonical reduced rational, `n` or `n/d` with `d > 1`
    /// coprime to `n`: its numerator and denominator.
    pub(super) fn canonical_rational(
        &mut self,
        text: &str,
    ) -> Result<Option<(quire_exact::Integer, quire_exact::Integer)>, ValidationFailure> {
        let Some((numerator, denominator)) = text.split_once('/') else {
            return Ok(self
                .exact_integer(text)?
                .map(|numerator| (numerator, quire_exact::Integer::one())));
        };
        let (Some(numerator), Some(denominator)) = (
            self.exact_integer(numerator)?,
            self.exact_integer(denominator)?,
        ) else {
            return Ok(None);
        };
        if denominator <= quire_exact::Integer::one() {
            return Ok(None);
        }
        let mut reservation = self.exact_reservation();
        let divisor = numerator
            .gcd_reserved(&denominator, &mut reservation)
            .map_err(|error| reservation.failure(error))?;
        Ok((divisor == quire_exact::Integer::one()).then_some((numerator, denominator)))
    }

    /// `left + right`, its limb block charged first.
    pub(super) fn exact_add(
        &mut self,
        left: &quire_exact::Integer,
        right: &quire_exact::Integer,
    ) -> Result<quire_exact::Integer, ValidationFailure> {
        let mut reservation = self.exact_reservation();
        left.add_reserved(right, &mut reservation)
            .map_err(|error| reservation.failure(error))
    }

    /// The greatest common divisor, its work blocks charged first.
    pub(super) fn exact_gcd(
        &mut self,
        left: &quire_exact::Integer,
        right: &quire_exact::Integer,
    ) -> Result<quire_exact::Integer, ValidationFailure> {
        let mut reservation = self.exact_reservation();
        left.gcd_reserved(right, &mut reservation)
            .map_err(|error| reservation.failure(error))
    }

    /// The quotient toward zero, or `None` for a zero divisor; its blocks are
    /// charged first.
    pub(super) fn exact_div(
        &mut self,
        dividend: &quire_exact::Integer,
        divisor: &quire_exact::Integer,
    ) -> Result<Option<quire_exact::Integer>, ValidationFailure> {
        let Ok(divisor) = quire_exact::NonZeroInteger::try_from(divisor) else {
            return Ok(None);
        };
        let mut reservation = self.exact_reservation();
        dividend
            .exact_div_reserved(divisor, &mut reservation)
            .map(Some)
            .map_err(|error| reservation.failure(error))
    }

    /// A copy of `value`, the storage it holds charged first.
    pub(super) fn exact_clone(
        &mut self,
        value: &quire_exact::Integer,
    ) -> Result<quire_exact::Integer, ValidationFailure> {
        let bytes = value.copy_footprint().bytes;
        if bytes != 0 {
            let mut reservation = self.exact_reservation();
            quire_exact::Reservation::reserve(&mut reservation, bytes)
                .map_err(|_| reservation.failure(quire_exact::IntegerReserveError::Denied))?;
        }
        Ok(value.clone())
    }

    /// `left * right`, its limb blocks charged first.
    pub(super) fn exact_mul(
        &mut self,
        left: &quire_exact::Integer,
        right: &quire_exact::Integer,
    ) -> Result<quire_exact::Integer, ValidationFailure> {
        let mut reservation = self.exact_reservation();
        left.mul_reserved(right, &mut reservation)
            .map_err(|error| reservation.failure(error))
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

    /// The failure an exact-number operation ends the read with: the denial
    /// this reservation returned, or the allocator's refusal.
    pub(super) fn failure(self, error: quire_exact::IntegerReserveError) -> ValidationFailure {
        self.denial.unwrap_or_else(|| {
            let requested = match error {
                quire_exact::IntegerReserveError::Allocation { requested } => requested,
                quire_exact::IntegerReserveError::Denied => 0,
            };
            ValidationFailure::Budget(BudgetStop {
                resource: PackageReadResource::ReadStorage,
                amount: requested as u128,
                path: None,
            })
        })
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
    /// One stamp per entry: the walk of a type chain that last visited it.
    /// A chain walk asks for no memory of its own; it stamps this table.
    walked: std::cell::RefCell<Vec<u32>>,
    walk: std::cell::Cell<u32>,
}

#[allow(dead_code)] // map-shaped accessors the other read groups fold in
impl<'g> GraphIndex<'g> {
    /// Builds the table of `ids` (in position order), charging its exact
    /// block first.
    pub(super) fn build(
        ids: impl ExactSizeIterator<Item = &'g CheckedNodeId>,
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let mut entries = meter.vec::<(&CheckedNodeId, usize)>(ids.len())?;
        entries.extend(ids.enumerate().map(|(position, id)| (id, position)));
        // A sort cannot return an error: the first refusal is kept and the
        // remaining comparisons answer without work.
        let mut refused = None;
        entries.sort_unstable_by(|left, right| {
            if refused.is_some() {
                return Ordering::Equal;
            }
            match meter.compare_ids(left.0, right.0) {
                Ok(Ordering::Equal) => left.1.cmp(&right.1),
                Ok(order) => order,
                Err(failure) => {
                    refused = Some(failure);
                    Ordering::Equal
                }
            }
        });
        if let Some(failure) = refused {
            return Err(failure);
        }
        let walked = meter.filled(entries.len(), 0_u32)?;
        Ok(Self {
            entries: entries.into_boxed_slice(),
            walked: std::cell::RefCell::new(walked),
            walk: std::cell::Cell::new(0),
        })
    }

    /// Starts a new chain walk: no node counts as visited by it yet.
    pub(super) fn begin_walk(&self) {
        let next = self.walk.get().wrapping_add(1);
        if next == 0 {
            self.walked.borrow_mut().fill(0);
            self.walk.set(1);
        } else {
            self.walk.set(next);
        }
    }

    /// Marks the node with this id visited by the current walk; `false` when
    /// it already was. A node outside the index cannot repeat.
    pub(super) fn first_visit(&self, id: &CheckedNodeId) -> bool {
        let at = self
            .entries
            .partition_point(|(entry, _)| entry.cmp(&id) == Ordering::Less);
        if self.entries.get(at).is_none_or(|(entry, _)| *entry != id) {
            return true;
        }
        let mut walked = self.walked.borrow_mut();
        let Some(stamp) = walked.get_mut(at) else {
            return true;
        };
        let first = *stamp != self.walk.get();
        *stamp = self.walk.get();
        first
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
            walked: std::cell::RefCell::new(vec![0; entries.len()]),
            walk: std::cell::Cell::new(0),
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

#[allow(dead_code)] // map-shaped accessors the other read groups fold in
impl<K: Ord, V> SortedTable<K, V> {
    /// Builds the table from `entries` held in an exact-capacity vector the
    /// caller charged. Of entries with one key, the one `tie` orders first
    /// is kept. The sort is in place (never a stable sort's scratch block).
    pub(super) fn from_vec(mut entries: Vec<(K, V)>, tie: impl Fn(&V, &V) -> Ordering) -> Self {
        entries.sort_unstable_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| tie(&left.1, &right.1))
        });
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

    /// The value of the entry `probe` finds equal, where `probe` orders each
    /// key against the one sought. The sought key need not outlive the table.
    pub(super) fn find(&self, probe: impl Fn(&K) -> Ordering) -> Option<&V> {
        self.entries
            .binary_search_by(|(entry, _)| probe(entry))
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

/// The position of the first of each distinct key of `keys`, as a sorted
/// exact-capacity table (charged first).
pub(super) fn first_positions<K: Ord>(
    keys: impl ExactSizeIterator<Item = K>,
    meter: &mut WorkMeter,
) -> Result<SortedTable<K, usize>, ValidationFailure> {
    let mut entries = meter.vec::<(K, usize)>(keys.len())?;
    entries.extend(keys.enumerate().map(|(position, key)| (key, position)));
    Ok(SortedTable::from_vec(entries, |left, right| {
        left.cmp(right)
    }))
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

/// Visited marks over node positions that a repeated walk reuses: each walk
/// starts a new generation instead of clearing or rebuilding the table.
pub(super) struct Stamps {
    marks: Vec<u32>,
    generation: u32,
}

impl Stamps {
    /// A table for `len` positions, charged first.
    pub(super) fn new(len: usize, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(Self {
            marks: meter.filled(len, 0_u32)?,
            generation: 0,
        })
    }

    /// Starts a walk: no position counts as visited yet.
    pub(super) fn next(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.marks.fill(0);
            self.generation = 1;
        }
    }

    /// Marks `position` visited; `false` when it already was this walk.
    pub(super) fn insert(&mut self, position: usize) -> bool {
        match self.marks.get_mut(position) {
            Some(mark) if *mark == self.generation => false,
            Some(mark) => {
                *mark = self.generation;
                true
            }
            None => true,
        }
    }
}

/// The static wire spelling of a node's `node_tag`, for a request's form.
pub(super) fn node_form(tag: &str) -> &'static str {
    super::CheckedNodeTag::from_wire(tag).map_or("node", super::CheckedNodeTag::as_wire)
}

#[cfg(test)]
mod chain_walk_tests {
    use super::GraphIndex;
    use crate::checked_package::shared::{CheckedNodeId, CheckedNodePathSegment};
    use ix_trace_rs::trace;

    fn id(name: &str) -> CheckedNodeId {
        CheckedNodeId {
            package: "p".into(),
            path: vec![CheckedNodePathSegment::Name(name.into())],
            ordinal: 0,
        }
    }

    /// A type-chain walk finds its first repeat by stamping the index's own
    /// table, and each walk starts clean, so the walk asks for no memory
    /// (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn chain_walks_stamp_the_index_and_each_starts_clean() {
        let (a, b, outside) = (id("a"), id("b"), id("z"));
        let index: GraphIndex<'_> = [(&a, 0), (&b, 1)].into_iter().collect();
        index.begin_walk();
        assert!(index.first_visit(&a));
        assert!(index.first_visit(&b));
        assert!(!index.first_visit(&a));
        assert!(index.first_visit(&outside) && index.first_visit(&outside));
        index.begin_walk();
        assert!(index.first_visit(&a) && !index.first_visit(&a));
        index.walk.set(u32::MAX);
        index.begin_walk();
        assert!(index.first_visit(&b) && !index.first_visit(&b));
    }

    /// A sorted set that grows under the meter refuses a block it cannot
    /// pay for before allocating it, and reports a repeated item (the
    /// replacement for the ordered sets the validators held).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn sorted_inserts_are_charged_before_the_block_grows() {
        use super::super::WorkMeter;
        use crate::NativeJoinBudget;
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            let mut items: Vec<u64> = Vec::new();
            let mut outcome = Vec::new();
            for item in [5_u64, 1, 9, 5, 3, 1, 7] {
                match meter.insert_sorted(&mut items, item) {
                    Ok(first) => outcome.push(first),
                    Err(_) => return (None, items, budget.read_storage_consumed()),
                }
            }
            drop(meter);
            (Some(outcome), items, budget.read_storage_consumed())
        };
        let (outcome, items, spent) = run(u64::MAX);
        assert_eq!(
            outcome.unwrap(),
            [true, true, true, false, true, false, true]
        );
        assert_eq!(items, [1, 3, 5, 7, 9]);
        assert!(spent > 0);
        assert!(run(spent).0.is_some());
        let (refused, partial, _) = run(spent - 1);
        assert!(refused.is_none());
        assert!(partial.len() < 5, "the short run did not complete");
    }
}
