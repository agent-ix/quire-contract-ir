// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A borrowed, allocation-free view of one value of a selected model
//! document, read straight from the shared reader's arena.
//!
//! The view answers the questions the declaration reader asks of a value
//! (member by name, string, boolean, integer, list, members) and owns
//! nothing, so reading a declaration builds no `serde_json::Value`.

use quire_canonical::{Document, Items, Members, Node, NodeRef};

/// One value of a document.
#[derive(Clone, Copy)]
pub(in crate::checked_package::v2) struct View<'d>(NodeRef<'d>);

impl<'d> View<'d> {
    /// The document's top-level value.
    pub(in crate::checked_package::v2) fn root(document: &'d Document) -> Self {
        Self(document.root())
    }

    /// Member `name` when this is an object that has one.
    pub(in crate::checked_package::v2) fn get(self, name: &str) -> Option<View<'d>> {
        self.0.get(name).map(Self)
    }

    pub(in crate::checked_package::v2) fn as_str(self) -> Option<&'d str> {
        match self.0.node() {
            Node::String(text) => Some(text),
            _ => None,
        }
    }

    pub(in crate::checked_package::v2) fn as_bool(self) -> Option<bool> {
        match self.0.node() {
            Node::Bool(flag) => Some(flag),
            _ => None,
        }
    }

    /// The integer the number's text spells, when it spells one that fits.
    pub(in crate::checked_package::v2) fn as_u64(self) -> Option<u64> {
        match self.0.node() {
            Node::Number(number) => number.text().parse::<u64>().ok(),
            _ => None,
        }
    }

    /// The integer the number's text spells, when it spells one that fits.
    pub(in crate::checked_package::v2) fn as_i64(self) -> Option<i64> {
        match self.0.node() {
            Node::Number(number) => number.text().parse::<i64>().ok(),
            _ => None,
        }
    }

    pub(in crate::checked_package::v2) fn is_null(self) -> bool {
        matches!(self.0.node(), Node::Null)
    }

    pub(in crate::checked_package::v2) fn is_object(self) -> bool {
        matches!(self.0.node(), Node::Object(_))
    }

    /// The elements when this is an array.
    pub(in crate::checked_package::v2) fn as_array(self) -> Option<List<'d>> {
        match self.0.node() {
            Node::Array(items) => Some(List(Some(items))),
            _ => None,
        }
    }

    /// Whether this is an array with no element.
    pub(in crate::checked_package::v2) fn is_empty_array(self) -> bool {
        self.as_array().is_some_and(|items| items.is_empty())
    }

    /// The members, in document order, when this is an object.
    pub(in crate::checked_package::v2) fn as_object(self) -> Option<Object<'d>> {
        match self.0.node() {
            Node::Object(members) => Some(Object(members)),
            _ => None,
        }
    }

    /// The raw node, for walks that match on every kind.
    pub(in crate::checked_package::v2) fn node(self) -> Node<'d> {
        self.0.node()
    }
}

/// The elements of an array, or none for an absent member.
#[derive(Clone)]
pub(in crate::checked_package::v2) struct List<'d>(Option<Items<'d>>);

impl<'d> List<'d> {
    /// The list of an absent member.
    pub(in crate::checked_package::v2) const fn empty() -> Self {
        Self(None)
    }

    pub(in crate::checked_package::v2) fn len(&self) -> usize {
        self.0.as_ref().map_or(0, ExactSizeIterator::len)
    }

    pub(in crate::checked_package::v2) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(in crate::checked_package::v2) fn iter(&self) -> ListIter<'d> {
        ListIter(self.0.clone())
    }
}

impl<'d> IntoIterator for List<'d> {
    type Item = View<'d>;
    type IntoIter = ListIter<'d>;

    fn into_iter(self) -> ListIter<'d> {
        ListIter(self.0)
    }
}

/// The elements of a list, in order.
#[derive(Clone)]
pub(in crate::checked_package::v2) struct ListIter<'d>(Option<Items<'d>>);

impl<'d> Iterator for ListIter<'d> {
    type Item = View<'d>;

    fn next(&mut self) -> Option<View<'d>> {
        self.0.as_mut()?.next().map(View)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.as_ref().map_or((0, Some(0)), Iterator::size_hint)
    }
}

impl ExactSizeIterator for ListIter<'_> {}

impl Default for List<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

/// The members of an object, in document order.
#[derive(Clone)]
pub(in crate::checked_package::v2) struct Object<'d>(Members<'d>);

impl<'d> Object<'d> {
    pub(in crate::checked_package::v2) fn len(&self) -> usize {
        self.0.len()
    }

    pub(in crate::checked_package::v2) fn keys(&self) -> impl Iterator<Item = &'d str> {
        self.0.clone().map(|(name, _)| name)
    }

    pub(in crate::checked_package::v2) fn values(&self) -> impl Iterator<Item = View<'d>> {
        self.0.clone().map(|(_, value)| View(value))
    }

    pub(in crate::checked_package::v2) fn get(&self, name: &str) -> Option<View<'d>> {
        self.0
            .clone()
            .find(|(member, _)| *member == name)
            .map(|(_, value)| View(value))
    }
}
