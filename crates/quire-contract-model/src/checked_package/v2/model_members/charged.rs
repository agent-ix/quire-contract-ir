// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Exact-capacity containers whose every allocation is charged to the
//! caller's budget before it is made (FR-038 "Budget-charged strict read").
//!
//! Lookup tables are sorted slices searched by binary search; the read builds
//! no `BTreeMap`, `BTreeSet` or `HashMap`. A table is created with the
//! capacity its builder computed from the document, and grows (by doubling,
//! charged at the full new capacity) only if the builder under-counted.

use super::Budget;
use crate::checked_package::shared::{CheckedNodeId, CheckedNodePathSegment};
use crate::checked_package::v2::ValidationFailure;
use std::borrow::Borrow;

/// The bytes `Arc::new` requests for a `T`: two counters and the value, in
/// the layout `ArcInner` has (`repr(C)`).
pub(in crate::checked_package::v2) const fn arc_bytes<T>() -> usize {
    let align = if std::mem::align_of::<T>() > std::mem::align_of::<usize>() {
        std::mem::align_of::<T>()
    } else {
        std::mem::align_of::<usize>()
    };
    let header = 2 * std::mem::size_of::<usize>();
    let data = header.div_ceil(std::mem::align_of::<T>()) * std::mem::align_of::<T>();
    (data + std::mem::size_of::<T>()).div_ceil(align) * align
}

/// A map as a sorted slice of entries, searched by binary search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package::v2) struct SortedMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> Default for SortedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SortedMap<K, V> {
    pub(in crate::checked_package::v2) const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// An empty map with room for `capacity` entries, requested first.
    pub(in crate::checked_package::v2) fn with_capacity(
        budget: &mut Budget<'_, '_>,
        capacity: usize,
        form: &'static str,
    ) -> Result<Self, ValidationFailure> {
        Ok(Self {
            entries: budget.vec(capacity, form)?,
        })
    }

    /// A map over entries already in ascending key order with no repeat.
    pub(in crate::checked_package::v2) fn from_sorted(entries: Vec<(K, V)>) -> Self {
        Self { entries }
    }

    /// The entries in key order.
    pub(in crate::checked_package::v2) fn as_slice(&self) -> &[(K, V)] {
        &self.entries
    }

    pub(in crate::checked_package::v2) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(in crate::checked_package::v2) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(in crate::checked_package::v2) fn iter(&self) -> MapIter<'_, K, V> {
        fn pair<K, V>(entry: &(K, V)) -> (&K, &V) {
            (&entry.0, &entry.1)
        }
        self.entries.iter().map(pair as fn(&(K, V)) -> (&K, &V))
    }

    pub(in crate::checked_package::v2) fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.entries.iter().map(|(key, _)| key)
    }

    pub(in crate::checked_package::v2) fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }
}

/// The entries of a [`SortedMap`] in key order.
pub(in crate::checked_package::v2) type MapIter<'a, K, V> =
    std::iter::Map<std::slice::Iter<'a, (K, V)>, fn(&'a (K, V)) -> (&'a K, &'a V)>;

impl<K: Ord, V> SortedMap<K, V> {
    fn search<Q>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries
            .binary_search_by(|(candidate, _)| candidate.borrow().cmp(key))
    }

    pub(in crate::checked_package::v2) fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let at = self.search(key).ok()?;
        self.entries.get(at).map(|(_, value)| value)
    }

    pub(in crate::checked_package::v2) fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let at = self.search(key).ok()?;
        self.entries.get(at).map(|(key, value)| (key, value))
    }

    pub(in crate::checked_package::v2) fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let at = self.search(key).ok()?;
        self.entries.get_mut(at).map(|(_, value)| value)
    }

    pub(in crate::checked_package::v2) fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(key).is_ok()
    }

    /// Sets `key` to `value` in key order and returns the value it replaced.
    /// A full table grows first, with the whole new capacity requested.
    pub(in crate::checked_package::v2) fn insert(
        &mut self,
        budget: &mut Budget<'_, '_>,
        key: K,
        value: V,
        form: &'static str,
    ) -> Result<Option<V>, ValidationFailure> {
        match self.search(&key) {
            Ok(at) => Ok(self
                .entries
                .get_mut(at)
                .map(|entry| std::mem::replace(&mut entry.1, value))),
            Err(at) => {
                grow(&mut self.entries, budget, form)?;
                self.entries.insert(at, (key, value));
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
impl<K: Ord, V> SortedMap<K, V> {
    /// Fixture insertion that charges nothing.
    pub(in crate::checked_package::v2) fn insert_unmetered(&mut self, key: K, value: V) {
        match self.search(&key) {
            Ok(at) => self.entries[at].1 = value,
            Err(at) => self.entries.insert(at, (key, value)),
        }
    }
}

#[cfg(test)]
impl<K: Ord + Borrow<Q>, Q: Ord + ?Sized, V> std::ops::Index<&Q> for SortedMap<K, V> {
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("key present")
    }
}

#[cfg(test)]
impl<K: Ord> SortedSet<K> {
    /// Fixture insertion that charges nothing.
    pub(in crate::checked_package::v2) fn insert_unmetered(&mut self, item: K) {
        if let Err(at) = self.items.binary_search(&item) {
            self.items.insert(at, item);
        }
    }
}

/// A set as a sorted slice, searched by binary search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package::v2) struct SortedSet<K> {
    items: Vec<K>,
}

impl<K> Default for SortedSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> SortedSet<K> {
    pub(in crate::checked_package::v2) const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// A set over items the caller has already sorted, without repeats.
    pub(in crate::checked_package::v2) fn from_sorted(items: Vec<K>) -> Self {
        Self { items }
    }

    pub(in crate::checked_package::v2) fn as_slice(&self) -> &[K] {
        &self.items
    }

    pub(in crate::checked_package::v2) fn len(&self) -> usize {
        self.items.len()
    }

    /// An empty set with room for `capacity` items, requested first.
    pub(in crate::checked_package::v2) fn with_capacity(
        budget: &mut Budget<'_, '_>,
        capacity: usize,
        form: &'static str,
    ) -> Result<Self, ValidationFailure> {
        Ok(Self {
            items: budget.vec(capacity, form)?,
        })
    }

    pub(in crate::checked_package::v2) fn iter(&self) -> std::slice::Iter<'_, K> {
        self.items.iter()
    }
}

impl<K: Ord> SortedSet<K> {
    pub(in crate::checked_package::v2) fn contains<Q>(&self, item: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.items
            .binary_search_by(|candidate| candidate.borrow().cmp(item))
            .is_ok()
    }

    /// Adds `item` in order; `false` when it was already there. A full set
    /// grows first, with the whole new capacity requested.
    pub(in crate::checked_package::v2) fn insert(
        &mut self,
        budget: &mut Budget<'_, '_>,
        item: K,
        form: &'static str,
    ) -> Result<bool, ValidationFailure> {
        match self.items.binary_search(&item) {
            Ok(_) => Ok(false),
            Err(at) => {
                grow(&mut self.items, budget, form)?;
                self.items.insert(at, item);
                Ok(true)
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a SortedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = MapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K> IntoIterator for &'a SortedSet<K> {
    type Item = &'a K;
    type IntoIter = std::slice::Iter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Makes room for one more entry, requesting the whole new capacity first
/// when the list is full.
fn grow<T>(
    list: &mut Vec<T>,
    budget: &mut Budget<'_, '_>,
    form: &'static str,
) -> Result<(), ValidationFailure> {
    if list.len() == list.capacity() {
        let target = list.capacity().saturating_mul(2).max(4);
        budget.request(target.saturating_mul(std::mem::size_of::<T>()), form)?;
        list.reserve_exact(target - list.len());
    }
    Ok(())
}

/// The bytes `Arc::<str>::from` requests for `len` bytes of text: two counters
/// and the text, rounded to the counters' alignment.
pub(in crate::checked_package::v2) const fn arc_str_bytes(len: usize) -> usize {
    (2 * std::mem::size_of::<usize>() + len).next_multiple_of(std::mem::align_of::<usize>())
}

/// The bytes cloning `id` requests: its package text, its path vector at the
/// exact length and each path name. A shared operand node is not copied.
pub(in crate::checked_package::v2) fn node_id_bytes(id: &CheckedNodeId) -> usize {
    id.path.iter().fold(
        id.package.len() + id.path.len() * std::mem::size_of::<CheckedNodePathSegment>(),
        |total, segment| match segment {
            CheckedNodePathSegment::Name(name) => total + name.len(),
            CheckedNodePathSegment::Dependency { package, .. } => total + package.len(),
            _ => total,
        },
    )
}

/// The bytes [`super::declaration_id`] requests for the node `node`.
pub(in crate::checked_package::v2) fn declaration_id_bytes(
    identity: &str,
    form: super::DeclarationForm,
    node: &str,
) -> usize {
    identity.len()
        + 3 * std::mem::size_of::<CheckedNodePathSegment>()
        + form.tag().len()
        + form.form().len()
        + node.len()
}

/// The bytes cloning a selected type reference requests.
pub(in crate::checked_package::v2) fn type_ref_bytes(type_ref: &super::SelectedTypeRef) -> usize {
    match type_ref {
        super::SelectedTypeRef::Native(_) => 0,
        super::SelectedTypeRef::Unit(name) | super::SelectedTypeRef::Declared(name) => name.len(),
        super::SelectedTypeRef::Map(key, value) => key.len() + value.len(),
    }
}

/// The bytes cloning a field declaration requests.
pub(in crate::checked_package::v2) fn field_decl_bytes(field: &super::FieldDecl) -> usize {
    field.identity.len()
        + field.slot.name.as_deref().map_or(0, str::len)
        + field.slot.type_ref.as_ref().map_or(0, type_ref_bytes)
        + field.redefines.as_deref().map_or(0, str::len)
}

/// The bytes cloning a relationship declaration requests.
pub(in crate::checked_package::v2) fn relationship_decl_bytes(
    declaration: &super::RelationshipDecl,
) -> usize {
    let end =
        |end: &super::RelationshipEnd| end.role.as_deref().map_or(0, str::len) + end.type_ref.len();
    end(&declaration.source) + end(&declaration.target)
}

/// The bytes converting `member` to its public form requests: a box for each
/// wrapper and for a carried rational's carrier, and each owned id and text.
pub(in crate::checked_package::v2) fn checked_member_type_bytes(
    member: &super::MemberType,
) -> usize {
    let boxed = std::mem::size_of::<crate::checked_package::v2::CheckedMemberType>();
    let mut total = 0_usize;
    let mut cursor = member;
    loop {
        match cursor {
            super::MemberType::Option(inner) => {
                total += boxed;
                cursor = inner;
            }
            super::MemberType::Collection { element, .. } => {
                total += boxed;
                cursor = element;
            }
            super::MemberType::CarriedRational(_, unit) => {
                return total + boxed + node_id_bytes(unit);
            }
            super::MemberType::TextBounds(bounds) => return total + bounds.profile.len(),
            super::MemberType::Reference(id)
            | super::MemberType::Record(id)
            | super::MemberType::Tuple(id)
            | super::MemberType::Variant(id)
            | super::MemberType::Quantity(id) => return total + node_id_bytes(id),
            _ => return total,
        }
    }
}
