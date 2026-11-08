// SPDX-License-Identifier: AGPL-3.0-or-later
//! The process-static index of the operation catalog: the catalog bytes parsed
//! once into ordered tables. The catalog is not package data and is not read
//! while a package is read; the V2 read path reaches it only through
//! [`super::operation_catalog`]'s accessors.

use super::operation_catalog::{CatalogReadError, OperationCatalogEntry};
use super::{CheckedArtifactRef, LawRole, OperationModeKind};
use serde::Deserialize;
use std::collections::BTreeMap;

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
    pub(super) groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    /// Each entry is exactly `{authority, identity}` (QSpec `DefinitionRef`).
    pub(super) law_roles: BTreeMap<LawRole, Vec<CheckedArtifactRef>>,
    #[allow(dead_code)]
    pub(super) profile_law_roles: Vec<Box<str>>,
    pub(super) modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    pub(super) type_pinned_modes: BTreeMap<OperationModeKind, Vec<Box<str>>>,
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
    operations: BTreeMap<Box<str>, OperationCatalogEntry>,
    groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    law_roles: BTreeMap<LawRole, Vec<CheckedArtifactRef>>,
    modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    type_pinned_modes: BTreeMap<OperationModeKind, Vec<Box<str>>>,
}

impl OperationCatalog {
    #[cfg(test)]
    pub(super) fn known(&self, identity: &str) -> Option<super::KnownOperation> {
        self.entry(identity).and_then(|entry| entry.known)
    }
    /// Whether `value` is one of the catalog's closed values for mode `kind`.
    pub(super) fn mode_value_admitted(&self, kind: OperationModeKind, value: &str) -> bool {
        self.modes
            .get(kind.as_wire())
            .is_some_and(|values| values.iter().any(|candidate| &**candidate == value))
    }

    /// Every catalogued entry, so a test reads a set of entries from the
    /// catalog this build depends on rather than listing identities.
    #[cfg(test)]
    pub(super) fn entries(&self) -> impl Iterator<Item = &OperationCatalogEntry> {
        self.operations.values()
    }

    /// The catalogued entry for an `operation.identity`, if it names one.
    pub(super) fn entry(&self, identity: &str) -> Option<&OperationCatalogEntry> {
        self.operations.get(identity)
    }

    /// Every catalogued definition of a value law role (e.g.
    /// `integer_division`), if `role` names one.
    pub(super) fn law_role_definitions(&self, role: LawRole) -> Option<&[CheckedArtifactRef]> {
        self.law_roles.get(&role).map(Vec::as_slice)
    }

    /// Whether `actual` is `expected` itself or a member of the group
    /// `expected` names.
    pub(super) fn family_fits(&self, actual: &str, expected: &str) -> bool {
        actual == expected
            || self
                .groups
                .get(expected)
                .is_some_and(|members| members.iter().any(|member| **member == *actual))
    }

    /// Whether a mode `kind` (`rounding`, `text_profile`) is one whose value
    /// an operand or leaf's own type authoritatively pins.
    pub(super) fn is_type_pinned_mode(&self, kind: OperationModeKind) -> bool {
        self.type_pinned_modes.contains_key(&kind)
    }
}

/// Reads and indexes catalog bytes. Bytes the reader cannot read, such as a
/// `law_roles` entry that is not exactly `{authority, identity}`, return an
/// error naming the entry; nothing here panics on supplied bytes.
pub(super) fn read_catalog(bytes: &str) -> Result<OperationCatalog, CatalogReadError> {
    let value: serde_json::Value =
        serde_json::from_str(bytes).map_err(|error| CatalogReadError {
            path: ".".to_owned(),
            message: error.to_string(),
        })?;
    let wire: OperationCatalogWire = serde_path_to_error::deserialize(&value).map_err(|error| {
        // A closed vocabulary's decode error does not echo the word (the
        // package reader classifies by message), so the catalog read names
        // the offending word from the bytes it was given.
        let mut at = Some(&value);
        for segment in error.path() {
            at = at.and_then(|current| match segment {
                serde_path_to_error::Segment::Seq { index } => current.get(*index),
                serde_path_to_error::Segment::Map { key } => current.get(key.as_str()),
                serde_path_to_error::Segment::Enum { .. }
                | serde_path_to_error::Segment::Unknown => None,
            });
        }
        let word = at
            .and_then(serde_json::Value::as_str)
            .map(|word| format!(" (word `{word}`)"))
            .unwrap_or_default();
        CatalogReadError {
            path: error.path().to_string(),
            message: format!("{}{word}", error.inner()),
        }
    })?;
    Ok(index_catalog(wire))
}

fn index_catalog(wire: OperationCatalogWire) -> OperationCatalog {
    let operations = wire
        .operations
        .into_iter()
        .map(|mut entry| {
            entry.known = super::KnownOperation::from_wire(&entry.identity);
            (entry.identity.clone(), entry)
        })
        .collect();
    OperationCatalog {
        operations,
        groups: wire.groups,
        law_roles: wire.law_roles,
        modes: wire.modes,
        type_pinned_modes: wire.type_pinned_modes,
    }
}
