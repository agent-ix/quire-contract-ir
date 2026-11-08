// SPDX-License-Identifier: AGPL-3.0-or-later
//! The process-static index of the operation catalog: the catalog bytes read
//! once, with the shared reader and the document deserializer, into tables
//! sorted by key and searched by bisection. The strict read consults the
//! tables through [`super::operation_catalog`]'s accessors, and the first read
//! builds them.

use super::document_de::DecodeContext;
use super::native_join_budget::NativeJoinBudget;
use super::operation_catalog::{CatalogReadError, OperationCatalogEntry};
use super::{CheckedArtifactRef, LawRole, OperationModeKind};
use quire_canonical::Node;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;

/// A table of `(key, value)` rows sorted by key.
type Table<K, V> = Vec<(K, V)>;

/// Reads a JSON object as rows sorted by key.
fn sorted_rows<'de, D, K, V>(deserializer: D) -> Result<Table<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    struct Rows<K, V>(PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>> Visitor<'de> for Rows<K, V> {
        type Value = Table<K, V>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut rows: Table<K, V> = Vec::with_capacity(map.size_hint().unwrap_or(0));
            while let Some(row) = map.next_entry()? {
                rows.push(row);
            }
            rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            Ok(rows)
        }
    }
    deserializer.deserialize_map(Rows(PhantomData))
}

/// The row of `table` for `key`, found by bisection.
fn row<'a, K: Borrow<Q>, V, Q: Ord + ?Sized>(table: &'a [(K, V)], key: &Q) -> Option<&'a V> {
    let at = table
        .binary_search_by(|(candidate, _)| candidate.borrow().cmp(key))
        .ok()?;
    table.get(at).map(|(_, value)| value)
}

/// The full top-level `operation-catalog.json` shape. `deny_unknown_fields`
/// requires every published key to be declared here even though
/// [`OperationCatalog`] (built from this once, in [`operation_catalog`])
/// keeps only the tables this reader's checks actually consult; the
/// `#[allow(dead_code)]` fields below are parsed for that fidelity and then
/// dropped, never read again.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationCatalogWire {
    #[allow(dead_code)]
    pub(super) version: Box<str>,
    #[allow(dead_code)]
    pub(super) families: Vec<Box<str>>,
    #[serde(deserialize_with = "sorted_rows")]
    pub(super) groups: Table<Box<str>, Vec<Box<str>>>,
    /// Each entry is exactly `{authority, identity}` (QSpec `DefinitionRef`).
    #[serde(deserialize_with = "sorted_rows")]
    pub(super) law_roles: Table<LawRole, Vec<CheckedArtifactRef>>,
    #[allow(dead_code)]
    pub(super) profile_law_roles: Vec<Box<str>>,
    #[serde(deserialize_with = "sorted_rows")]
    pub(super) modes: Table<Box<str>, Vec<Box<str>>>,
    #[serde(deserialize_with = "sorted_rows")]
    pub(super) type_pinned_modes: Table<OperationModeKind, Vec<Box<str>>>,
    #[allow(dead_code)]
    pub(super) member_kinds: Vec<Box<str>>,
    #[allow(dead_code)]
    pub(super) constraint_kinds: Vec<Box<str>>,
    #[allow(dead_code)]
    pub(super) result_forms: Vec<Box<str>>,
    #[allow(dead_code)]
    pub(super) leaf_sources: Vec<Box<str>>,
    pub(super) operations: Vec<OperationCatalogEntry>,
}

/// The parsed, indexed `quire.checked-operation-catalog/v1`.
pub(super) struct OperationCatalog {
    /// The entries sorted by identity.
    operations: Vec<OperationCatalogEntry>,
    groups: Table<Box<str>, Vec<Box<str>>>,
    law_roles: Table<LawRole, Vec<CheckedArtifactRef>>,
    modes: Table<Box<str>, Vec<Box<str>>>,
    type_pinned_modes: Table<OperationModeKind, Vec<Box<str>>>,
}

impl OperationCatalog {
    #[cfg(test)]
    pub(super) fn known(&self, identity: &str) -> Option<super::KnownOperation> {
        self.entry(identity).and_then(|entry| entry.known)
    }
    /// Whether `value` is one of the catalog's closed values for mode `kind`.
    pub(super) fn mode_value_admitted(&self, kind: OperationModeKind, value: &str) -> bool {
        row(&self.modes, kind.as_wire())
            .is_some_and(|values| values.iter().any(|candidate| &**candidate == value))
    }

    /// Every catalogued entry, so a test reads a set of entries from the
    /// catalog this build depends on rather than listing identities.
    #[cfg(test)]
    pub(super) fn entries(&self) -> impl Iterator<Item = &OperationCatalogEntry> {
        self.operations.iter()
    }

    /// The catalogued entry for an `operation.identity`, if it names one.
    pub(super) fn entry(&self, identity: &str) -> Option<&OperationCatalogEntry> {
        let at = self
            .operations
            .binary_search_by(|entry| (*entry.identity).cmp(identity))
            .ok()?;
        self.operations.get(at)
    }

    /// Every catalogued definition of a value law role (e.g.
    /// `integer_division`), if `role` names one.
    pub(super) fn law_role_definitions(&self, role: LawRole) -> Option<&[CheckedArtifactRef]> {
        row(&self.law_roles, &role).map(Vec::as_slice)
    }

    /// Whether `actual` is `expected` itself or a member of the group
    /// `expected` names.
    pub(super) fn family_fits(&self, actual: &str, expected: &str) -> bool {
        actual == expected
            || row(&self.groups, expected)
                .is_some_and(|members| members.iter().any(|member| **member == *actual))
    }

    /// Whether a mode `kind` (`rounding`, `text_profile`) is one whose value
    /// an operand or leaf's own type authoritatively pins.
    pub(super) fn is_type_pinned_mode(&self, kind: OperationModeKind) -> bool {
        row(&self.type_pinned_modes, &kind).is_some()
    }
}

/// Reads and indexes catalog bytes. Bytes the reader cannot read, such as a
/// `law_roles` entry that is not exactly `{authority, identity}`, return an
/// error naming the entry; nothing here panics on supplied bytes.
pub(super) fn read_catalog(bytes: &str) -> Result<OperationCatalog, CatalogReadError> {
    let document =
        quire_canonical::read(bytes.as_bytes(), u64::MAX).map_err(|error| CatalogReadError {
            path: ".".to_owned(),
            message: error.to_string(),
        })?;
    let document = Arc::new(document);
    let mut budget = NativeJoinBudget::unlimited();
    let context = DecodeContext::new(&document, &mut budget);
    let wire: OperationCatalogWire =
        serde_path_to_error::deserialize(context.root()).map_err(|error| {
            // A closed vocabulary's decode error does not echo the word (the
            // package reader classifies by message), so the catalog read names
            // the offending word from the bytes it was given.
            let mut at = Some(document.root());
            for segment in error.path() {
                at = at.and_then(|current| match (segment, current.node()) {
                    (serde_path_to_error::Segment::Seq { index }, Node::Array(mut items)) => {
                        items.nth(*index)
                    }
                    (serde_path_to_error::Segment::Map { key }, Node::Object(mut members)) => {
                        members
                            .find(|(name, _)| *name == key.as_str())
                            .map(|(_, member)| member)
                    }
                    _ => None,
                });
            }
            let word = at
                .and_then(|node| match node.node() {
                    Node::String(text) => Some(format!(" (word `{text}`)")),
                    _ => None,
                })
                .unwrap_or_default();
            CatalogReadError {
                path: error.path().to_string(),
                message: format!("{}{word}", error.inner()),
            }
        })?;
    Ok(index_catalog(wire))
}

fn index_catalog(wire: OperationCatalogWire) -> OperationCatalog {
    let mut operations = wire.operations;
    for entry in &mut operations {
        entry.known = super::KnownOperation::from_wire(&entry.identity);
    }
    operations.sort_unstable_by(|left, right| left.identity.cmp(&right.identity));
    OperationCatalog {
        operations,
        groups: wire.groups,
        law_roles: wire.law_roles,
        modes: wire.modes,
        type_pinned_modes: wire.type_pinned_modes,
    }
}
