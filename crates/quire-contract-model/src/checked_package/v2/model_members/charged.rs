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
use crate::checked_package::v2::ValidationFailure;
use std::borrow::Borrow;

/// The bytes `Arc::new` requests for a `T`: two counters and the value, in
/// the layout `ArcInner` has (`repr(C)`).
pub(super) const fn arc_bytes<T>() -> usize {
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
    pub(super) fn with_capacity(
        budget: &mut Budget<'_, '_>,
        capacity: usize,
        form: &'static str,
    ) -> Result<Self, ValidationFailure> {
        Ok(Self {
            entries: budget.vec(capacity, form)?,
        })
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

    pub(in crate::checked_package::v2) fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search(key).is_ok()
    }

    /// Sets `key` to `value` in key order and returns the value it replaced.
    /// A full table grows first, with the whole new capacity requested.
    pub(super) fn insert(
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

    /// An empty set with room for `capacity` items, requested first.
    pub(super) fn with_capacity(
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
    pub(super) fn insert(
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
