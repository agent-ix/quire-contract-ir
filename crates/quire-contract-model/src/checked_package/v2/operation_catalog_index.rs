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
#[derive(Clone)]
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

/// Catalog admission stopped by its bytes or by the caller's budget.
pub(super) enum CatalogStop {
    Unavailable(CatalogReadError),
    Budget(super::ValidationFailure),
}

/// One sequence decoded at its exact final capacity.
struct Exact<T>(Vec<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Exact<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::document_de::exact_vec(deserializer).map(Self)
    }
}

type DecodeError = serde::de::value::Error;

fn decode<'d, T: Deserialize<'d>>(
    context: &DecodeContext<'d>,
    node: quire_canonical::NodeRef<'d>,
) -> Result<T, DecodeError> {
    T::deserialize(context.value_at(node))
}

fn field<'d>(
    root: quire_canonical::NodeRef<'d>,
    name: &str,
) -> Result<quire_canonical::NodeRef<'d>, DecodeError> {
    use serde::de::Error;
    let Node::Object(mut members) = root.node() else {
        return Err(DecodeError::custom("catalog is not an object"));
    };
    members
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .ok_or_else(|| DecodeError::custom("missing catalog member"))
}

trait CatalogKey: Sized {
    fn read(name: &str, context: &DecodeContext<'_>) -> Result<Self, DecodeError>;
}

impl CatalogKey for Box<str> {
    fn read(name: &str, context: &DecodeContext<'_>) -> Result<Self, DecodeError> {
        use serde::de::Error;
        if !context.charge_bytes(name.len()) {
            return Err(DecodeError::custom("catalog key reservation denied"));
        }
        Ok(name.into())
    }
}

macro_rules! catalog_key {
    ($type:ty) => {
        impl CatalogKey for $type {
            fn read(name: &str, _: &DecodeContext<'_>) -> Result<Self, DecodeError> {
                use serde::de::Error;
                Self::from_wire(name).ok_or_else(|| DecodeError::custom("unknown catalog key"))
            }
        }
    };
}
catalog_key!(LawRole);
catalog_key!(OperationModeKind);

fn table<'d, K: CatalogKey + Ord, V: Deserialize<'d>>(
    context: &DecodeContext<'d>,
    node: quire_canonical::NodeRef<'d>,
) -> Result<Table<K, Vec<V>>, DecodeError> {
    use serde::de::Error;
    let Node::Object(members) = node.node() else {
        return Err(DecodeError::custom("catalog table is not an object"));
    };
    let count = members.len();
    if !context.charge_bytes(count.saturating_mul(std::mem::size_of::<(K, Vec<V>)>())) {
        return Err(DecodeError::custom("catalog table reservation denied"));
    }
    let mut rows = Vec::with_capacity(count);
    for (name, node) in members {
        let key = K::read(name, context)?;
        let values: Exact<V> = decode(context, node)?;
        rows.push((key, values.0));
    }
    rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    Ok(rows)
}

fn decode_catalog<'d>(
    context: &DecodeContext<'d>,
    root: quire_canonical::NodeRef<'d>,
) -> Result<OperationCatalogWire, DecodeError> {
    use serde::de::Error;
    const FIELDS: &[&str] = &[
        "version",
        "families",
        "groups",
        "law_roles",
        "profile_law_roles",
        "modes",
        "type_pinned_modes",
        "member_kinds",
        "constraint_kinds",
        "result_forms",
        "leaf_sources",
        "operations",
    ];
    let Node::Object(members) = root.node() else {
        return Err(DecodeError::custom("catalog is not an object"));
    };
    if members.len() != FIELDS.len() || members.into_iter().any(|(name, _)| !FIELDS.contains(&name))
    {
        return Err(DecodeError::custom("unknown or missing catalog member"));
    }
    Ok(OperationCatalogWire {
        version: decode(context, field(root, "version")?)?,
        families: decode::<Exact<Box<str>>>(context, field(root, "families")?)?.0,
        groups: table(context, field(root, "groups")?)?,
        law_roles: table(context, field(root, "law_roles")?)?,
        profile_law_roles: decode::<Exact<Box<str>>>(context, field(root, "profile_law_roles")?)?.0,
        modes: table(context, field(root, "modes")?)?,
        type_pinned_modes: table(context, field(root, "type_pinned_modes")?)?,
        member_kinds: decode::<Exact<Box<str>>>(context, field(root, "member_kinds")?)?.0,
        constraint_kinds: decode::<Exact<Box<str>>>(context, field(root, "constraint_kinds")?)?.0,
        result_forms: decode::<Exact<Box<str>>>(context, field(root, "result_forms")?)?.0,
        leaf_sources: decode::<Exact<Box<str>>>(context, field(root, "leaf_sources")?)?.0,
        operations: decode::<Exact<OperationCatalogEntry>>(context, field(root, "operations")?)?.0,
    })
}

/// Parses a fresh catalog and retains it under the caller's read-storage
/// account. The catalog has no process-global allocation or hidden budget.
pub(super) fn read_catalog_budgeted(
    bytes: &str,
    budget: &mut NativeJoinBudget,
) -> Result<super::operation_catalog::CatalogRef, CatalogStop> {
    use super::budgeted_read::{self, ReadStorageReservation};
    use super::native_join_budget::{NativeJoinLocus, NativeJoinLocusSource, PackageReadStage};
    use super::read_charge;
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(PackageReadStage::Envelope, 0),
        form: "operation_catalog",
    };
    budget.set_active_locus(Some(locus));
    let document = match quire_canonical::read_with_reservation(
        bytes.as_bytes(),
        u64::MAX,
        &mut ReadStorageReservation(budget),
    ) {
        Ok(document) => document,
        Err(quire_canonical::ReadError::ReservationDenied { .. }) => {
            return Err(CatalogStop::Budget(budgeted_read::denial(budget)))
        }
        Err(error) => return Err(catalog_error(budget, &error)),
    };
    if !budget
        .charge_read_storage(read_charge::arc_block_bytes::<quire_canonical::Document>() as u128)
    {
        return Err(CatalogStop::Budget(budgeted_read::denial(budget)));
    }
    let document = Arc::new(document);
    let context = DecodeContext::at_locus(&document, budget, locus);
    let decoded = decode_catalog(&context, document.root());
    let denied = context.denied();
    *budget = context.finish();
    if denied {
        return Err(CatalogStop::Budget(budgeted_read::denial(budget)));
    }
    let wire = decoded.map_err(|error| catalog_error(budget, &error))?;
    if !budget.charge_read_storage(read_charge::arc_block_bytes::<OperationCatalog>() as u128) {
        return Err(CatalogStop::Budget(budgeted_read::denial(budget)));
    }
    let catalog = super::operation_catalog::CatalogRef(Arc::new(index_catalog(wire)));
    budget.set_active_locus(None);
    Ok(catalog)
}

fn catalog_error(budget: &mut NativeJoinBudget, error: &impl std::fmt::Display) -> CatalogStop {
    use std::fmt::Write;
    struct Count(usize);
    impl std::fmt::Write for Count {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.saturating_add(text.len());
            Ok(())
        }
    }
    let mut count = Count(0);
    let _ = write!(count, "{error}");
    if !budget.charge_read_storage(count.0.saturating_add(1) as u128) {
        return CatalogStop::Budget(super::budgeted_read::denial(budget));
    }
    let mut message = String::with_capacity(count.0);
    let _ = write!(message, "{error}");
    CatalogStop::Unavailable(CatalogReadError {
        path: ".".into(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    /// Every catalog read owns its tables and reserves all measured storage,
    /// including the retained shared catalog, before making its requests.
    #[trace("TC-472", "FR-038-AC-511", "FR-038-AC-515")]
    #[test]
    fn owned_catalog_allocates_exactly_its_admitted_storage() {
        fn measure(limit: u64) -> (bool, u64, NativeJoinBudget) {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocations = allocation_counter::measure(|| {
                result = Some(super::super::operation_catalog::read_catalog_budgeted(
                    &mut budget,
                ));
            });
            let admitted = result.expect("catalog was read").is_ok();
            (admitted, allocations.bytes_total, budget)
        }
        let (admitted, allocated, unlimited) = measure(u64::MAX);
        assert!(admitted, "the published catalog is valid");
        let required = unlimited.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required, "every catalog allocation is charged");
        let (admitted, allocated, exact) = measure(required);
        assert!(admitted, "exact catalog allowance admits");
        assert_eq!(allocated, exact.read_storage_consumed());
        let (admitted, allocated, short) = measure(required - 1);
        assert!(
            !admitted,
            "one below the retained catalog allocation denies"
        );
        assert_eq!(allocated, short.read_storage_consumed());
        assert_eq!(
            short.first_unpaid_locus().map(|locus| locus.form),
            Some("operation_catalog")
        );
        let (admitted, allocated, zero) = measure(0);
        assert!(!admitted);
        assert_eq!(allocated, 0, "a denied catalog request allocates nothing");
        assert_eq!(zero.read_storage_consumed(), 0);
    }
}
