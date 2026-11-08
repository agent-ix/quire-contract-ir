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

#[derive(Clone, Copy)]
enum CatalogStep<'d> {
    Key(&'d str),
    Index(usize),
}

/// The longest closed path is operations[i].constraints[j].operands[k];
/// one extra slot names an unknown member without inspecting its value.
#[derive(Clone, Copy)]
struct CatalogPath<'d> {
    steps: [Option<CatalogStep<'d>>; 7],
}

impl<'d> CatalogPath<'d> {
    fn root() -> Self {
        Self { steps: [None; 7] }
    }
    fn below(mut self, step: CatalogStep<'d>) -> Self {
        if let Some(slot) = self.steps.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(step);
        }
        self
    }
}

impl std::fmt::Display for CatalogPath<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut empty = true;
        for step in self.steps.iter().flatten() {
            match step {
                CatalogStep::Key(key) => {
                    if !empty {
                        formatter.write_str(".")?;
                    }
                    formatter.write_str(key)?;
                }
                CatalogStep::Index(index) => write!(formatter, "[{index}]")?,
            }
            empty = false;
        }
        if empty {
            formatter.write_str(".")?;
        }
        Ok(())
    }
}

enum CatalogDecodeError<'d> {
    Denied,
    Shape {
        path: CatalogPath<'d>,
        message: &'static str,
        word: Option<&'d str>,
    },
}

struct CatalogMessage<'d> {
    message: &'static str,
    word: Option<&'d str>,
}
impl std::fmt::Display for CatalogMessage<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)?;
        if let Some(word) = self.word {
            write!(formatter, " (word `{word}`)")?;
        }
        Ok(())
    }
}

type CatalogResultOf<'d, T> = Result<T, CatalogDecodeError<'d>>;

/// The typed catalog decode borrows the canonical document and owns only the
/// fields it charges to the caller before requesting each allocation.
struct CatalogDecoder<'b, 'd> {
    budget: &'b mut NativeJoinBudget,
    path: CatalogPath<'d>,
}

impl<'d> CatalogDecoder<'_, 'd> {
    fn invalid(&self, message: &'static str) -> CatalogDecodeError<'d> {
        CatalogDecodeError::Shape {
            path: self.path,
            message,
            word: None,
        }
    }
    fn unknown(&self, word: &'d str) -> CatalogDecodeError<'d> {
        CatalogDecodeError::Shape {
            path: self.path,
            message: "unknown catalog word",
            word: Some(word),
        }
    }
    fn reserve(&mut self, bytes: usize) -> CatalogResultOf<'d, ()> {
        if self.budget.charge_read_storage(bytes as u128) {
            Ok(())
        } else {
            Err(CatalogDecodeError::Denied)
        }
    }
    fn members(
        &self,
        node: quire_canonical::NodeRef<'d>,
    ) -> CatalogResultOf<'d, quire_canonical::Members<'d>> {
        match node.node() {
            Node::Object(members) => Ok(members),
            _ => Err(self.invalid("expected catalog object")),
        }
    }
    fn closed(
        &self,
        node: quire_canonical::NodeRef<'d>,
        required: &[&str],
        optional: &[&str],
    ) -> CatalogResultOf<'d, ()> {
        let members = self.members(node)?;
        for (name, _) in members.clone() {
            if !required.contains(&name) && !optional.contains(&name) {
                return Err(CatalogDecodeError::Shape {
                    path: self.path.below(CatalogStep::Key(name)),
                    message: "unknown catalog member",
                    word: None,
                });
            }
        }
        for name in required {
            if !members.clone().any(|(candidate, _)| candidate == *name) {
                return Err(self.invalid("missing required catalog member"));
            }
        }
        Ok(())
    }
    fn member(
        &self,
        node: quire_canonical::NodeRef<'d>,
        name: &str,
    ) -> CatalogResultOf<'d, quire_canonical::NodeRef<'d>> {
        self.members(node)?
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, value)| value)
            .ok_or_else(|| self.invalid("missing required catalog member"))
    }
    fn field<T: CatalogValue<'d>>(
        &mut self,
        node: quire_canonical::NodeRef<'d>,
        name: &'d str,
    ) -> CatalogResultOf<'d, T> {
        let previous = self.path;
        self.path = self.path.below(CatalogStep::Key(name));
        let result = self
            .member(node, name)
            .and_then(|member| T::read(member, self));
        self.path = previous;
        result
    }
    fn optional<T: CatalogValue<'d>>(
        &mut self,
        node: quire_canonical::NodeRef<'d>,
        name: &'d str,
    ) -> CatalogResultOf<'d, Option<T>> {
        if self.members(node)?.any(|(candidate, _)| candidate == name) {
            self.field(node, name)
        } else {
            Ok(None)
        }
    }
    fn table_field<K: CatalogKey<'d> + Ord, V: CatalogValue<'d>>(
        &mut self,
        node: quire_canonical::NodeRef<'d>,
        name: &'d str,
    ) -> CatalogResultOf<'d, Table<K, Vec<V>>> {
        let previous = self.path;
        self.path = self.path.below(CatalogStep::Key(name));
        let result = self
            .member(node, name)
            .and_then(|member| self.table(member));
        self.path = previous;
        result
    }
    fn table<K: CatalogKey<'d> + Ord, V: CatalogValue<'d>>(
        &mut self,
        node: quire_canonical::NodeRef<'d>,
    ) -> CatalogResultOf<'d, Table<K, Vec<V>>> {
        let members = self.members(node)?;
        let count = members.len();
        self.reserve(count.saturating_mul(std::mem::size_of::<(K, Vec<V>)>()))?;
        let mut rows = Vec::with_capacity(count);
        for (name, value) in members {
            let previous = self.path;
            self.path = self.path.below(CatalogStep::Key(name));
            let key = K::key(name, self)?;
            let value = Vec::<V>::read(value, self)?;
            self.path = previous;
            rows.push((key, value));
        }
        rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Ok(rows)
    }
}

trait CatalogValue<'d>: Sized {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self>;
}
trait CatalogKey<'d>: Sized {
    fn key(name: &'d str, reader: &mut CatalogDecoder<'_, 'd>) -> CatalogResultOf<'d, Self>;
}

impl<'d> CatalogValue<'d> for Box<str> {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        let Node::String(text) = node.node() else {
            return Err(reader.invalid("expected catalog string"));
        };
        Self::key(text, reader)
    }
}
impl<'d> CatalogKey<'d> for Box<str> {
    fn key(name: &'d str, reader: &mut CatalogDecoder<'_, 'd>) -> CatalogResultOf<'d, Self> {
        reader.reserve(name.len())?;
        Ok(name.into())
    }
}
impl<'d> CatalogValue<'d> for u64 {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        let Node::Number(number) = node.node() else {
            return Err(reader.invalid("expected catalog unsigned integer"));
        };
        number
            .text()
            .parse()
            .ok()
            .or_else(|| {
                number
                    .text()
                    .parse::<i64>()
                    .ok()
                    .and_then(|number| u64::try_from(number).ok())
            })
            .ok_or_else(|| reader.invalid("catalog unsigned integer is out of range"))
    }
}
impl<'d, T: CatalogValue<'d>> CatalogValue<'d> for Vec<T> {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        let Node::Array(items) = node.node() else {
            return Err(reader.invalid("expected catalog array"));
        };
        let count = items.len();
        reader.reserve(count.saturating_mul(std::mem::size_of::<T>()))?;
        let mut values = Vec::with_capacity(count);
        for (index, item) in items.enumerate() {
            let previous = reader.path;
            reader.path = reader.path.below(CatalogStep::Index(index));
            let value = T::read(item, reader)?;
            reader.path = previous;
            values.push(value);
        }
        Ok(values)
    }
}
impl<'d, T: CatalogValue<'d>> CatalogValue<'d> for Option<T> {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        if matches!(node.node(), Node::Null) {
            Ok(None)
        } else {
            T::read(node, reader).map(Some)
        }
    }
}
macro_rules! catalog_word {
    ($($type:ty),* $(,)?) => { $(
        impl<'d> CatalogValue<'d> for $type {
            fn read(node: quire_canonical::NodeRef<'d>, reader: &mut CatalogDecoder<'_, 'd>) -> CatalogResultOf<'d, Self> {
                let Node::String(word) = node.node() else { return Err(reader.invalid("expected closed catalog word")); };
                Self::key(word, reader)
            }
        }
        impl<'d> CatalogKey<'d> for $type {
            fn key(word: &'d str, reader: &mut CatalogDecoder<'_, 'd>) -> CatalogResultOf<'d, Self> {
                Self::from_wire(word).ok_or_else(|| reader.unknown(word))
            }
        }
    )* };
}
catalog_word!(
    LawRole,
    OperationModeKind,
    super::ApplicationOperator,
    super::OperationConstraintKind,
    super::OperationMemberKind
);
macro_rules! catalog_text {
    ($($type:ty),* $(,)?) => { $(
        impl<'d> CatalogValue<'d> for $type {
            fn read(node: quire_canonical::NodeRef<'d>, reader: &mut CatalogDecoder<'_, 'd>) -> CatalogResultOf<'d, Self> {
                Box::<str>::read(node, reader).map(Self::from)
            }
        }
    )* };
}
catalog_text!(
    super::operation_catalog::CatalogResult,
    super::operation_catalog::CatalogFamily,
    super::operation_catalog::CatalogLeafSource
);

impl<'d> CatalogValue<'d> for CheckedArtifactRef {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        reader.closed(node, &["authority", "identity"], &[])?;
        Ok(Self {
            authority: reader.field(node, "authority")?,
            identity: reader.field(node, "identity")?,
        })
    }
}
impl<'d> CatalogValue<'d> for super::operation_catalog::OperationConstraint {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        reader.closed(node, &["kind", "operands"], &["family"])?;
        Ok(Self {
            kind: reader.field(node, "kind")?,
            operands: reader.field(node, "operands")?,
            family: reader.optional(node, "family")?,
        })
    }
}
impl<'d> CatalogValue<'d> for OperationCatalogEntry {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        reader.closed(
            node,
            &[
                "identity",
                "operator",
                "operands",
                "result",
                "laws",
                "constraints",
            ],
            &["rest", "mode", "member", "leaves"],
        )?;
        Ok(Self {
            identity: reader.field(node, "identity")?,
            known: None,
            operator: reader.field(node, "operator")?,
            operands: reader.field(node, "operands")?,
            rest: reader.optional(node, "rest")?,
            result: reader.field(node, "result")?,
            laws: reader.field(node, "laws")?,
            mode: reader.optional(node, "mode")?,
            member: reader.optional(node, "member")?,
            constraints: reader.field(node, "constraints")?,
            leaves: reader.optional(node, "leaves")?,
        })
    }
}
impl<'d> CatalogValue<'d> for OperationCatalogWire {
    fn read(
        node: quire_canonical::NodeRef<'d>,
        reader: &mut CatalogDecoder<'_, 'd>,
    ) -> CatalogResultOf<'d, Self> {
        reader.closed(
            node,
            &[
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
            ],
            &[],
        )?;
        Ok(Self {
            version: reader.field(node, "version")?,
            families: reader.field(node, "families")?,
            groups: reader.table_field(node, "groups")?,
            law_roles: reader.table_field(node, "law_roles")?,
            profile_law_roles: reader.field(node, "profile_law_roles")?,
            modes: reader.table_field(node, "modes")?,
            type_pinned_modes: reader.table_field(node, "type_pinned_modes")?,
            member_kinds: reader.field(node, "member_kinds")?,
            constraint_kinds: reader.field(node, "constraint_kinds")?,
            result_forms: reader.field(node, "result_forms")?,
            leaf_sources: reader.field(node, "leaf_sources")?,
            operations: reader.field(node, "operations")?,
        })
    }
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
    let decoded = {
        let mut reader = CatalogDecoder {
            budget,
            path: CatalogPath::root(),
        };
        OperationCatalogWire::read(document.root(), &mut reader)
    };
    let wire = match decoded {
        Ok(wire) => wire,
        Err(CatalogDecodeError::Denied) => {
            return Err(CatalogStop::Budget(budgeted_read::denial(budget)))
        }
        Err(CatalogDecodeError::Shape {
            path,
            message,
            word,
        }) => {
            return Err(catalog_error_at(
                budget,
                &path,
                &CatalogMessage { message, word },
            ));
        }
    };
    if !budget.charge_read_storage(read_charge::arc_block_bytes::<OperationCatalog>() as u128) {
        return Err(CatalogStop::Budget(budgeted_read::denial(budget)));
    }
    let catalog = super::operation_catalog::CatalogRef(Arc::new(index_catalog(wire)));
    budget.set_active_locus(None);
    Ok(catalog)
}

fn catalog_error(budget: &mut NativeJoinBudget, error: &impl std::fmt::Display) -> CatalogStop {
    catalog_error_at(budget, &".", error)
}

fn catalog_error_at(
    budget: &mut NativeJoinBudget,
    path: &impl std::fmt::Display,
    error: &impl std::fmt::Display,
) -> CatalogStop {
    use std::fmt::Write;
    struct Count(usize);
    impl std::fmt::Write for Count {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.saturating_add(text.len());
            Ok(())
        }
    }
    let mut message_size = Count(0);
    let mut path_size = Count(0);
    let _ = write!(message_size, "{error}");
    let _ = write!(path_size, "{path}");
    if !budget.charge_read_storage(message_size.0.saturating_add(path_size.0) as u128) {
        return CatalogStop::Budget(super::budgeted_read::denial(budget));
    }
    let mut message = String::with_capacity(message_size.0);
    let mut location = String::with_capacity(path_size.0);
    let _ = write!(message, "{error}");
    let _ = write!(location, "{path}");
    CatalogStop::Unavailable(CatalogReadError {
        path: location,
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
        // Visit request boundaries rather than rerunning every byte allowance.
        let mut allowance = 0;
        while allowance < required {
            let (admitted, allocated, denied) = measure(allowance);
            assert!(!admitted);
            assert_eq!(
                allocated,
                denied.read_storage_consumed(),
                "catalog allowance {allowance}: a denied request allocates nothing"
            );
            let asked = u64::try_from(
                denied
                    .first_unpaid_read_storage()
                    .expect("first denied request"),
            )
            .expect("bounded catalog request");
            let next = denied.read_storage_consumed().saturating_add(asked);
            assert!(next > allowance, "catalog request boundary advances");
            allowance = next;
        }
        assert_eq!(allowance, required);
    }

    /// An unreadable catalog retains its earlier charges and pays for its
    /// diagnostic without allocating a temporary schema-error message.
    #[trace("TC-472", "FR-038-AC-511", "FR-038-AC-515")]
    #[test]
    fn unreadable_catalog_allocations_are_charged_to_the_caller() {
        let original: serde_json::Value =
            serde_json::from_str(super::super::operation_catalog::CATALOG_BYTES)
                .expect("catalog fixture");
        let mut cases = vec![
            ("{".to_owned(), ".".to_owned()),
            ("{}".to_owned(), ".".to_owned()),
        ];
        for (path, value, expected) in [
            (
                "/operations/0/operator",
                serde_json::json!("unknown_operator"),
                "operations[0].operator",
            ),
            (
                "/operations/0/operands",
                serde_json::json!([1]),
                "operations[0].operands[0]",
            ),
            (
                "/operations/0/laws",
                serde_json::json!(["unknown_law"]),
                "operations[0].laws[0]",
            ),
            (
                "/operations/0/member",
                serde_json::json!("unknown_member"),
                "operations[0].member",
            ),
            (
                "/operations/0/mode",
                serde_json::json!(false),
                "operations[0].mode",
            ),
        ] {
            let mut invalid = original.clone();
            *invalid.pointer_mut(path).expect("catalog field") = value;
            cases.push((
                serde_json::to_string(&invalid).expect("fixture encodes"),
                expected.to_owned(),
            ));
        }
        let mut unknown_field = original.clone();
        unknown_field["unexpected"] = serde_json::json!(true);
        cases.push((
            serde_json::to_string(&unknown_field).expect("fixture encodes"),
            "unexpected".to_owned(),
        ));
        let role = original["law_roles"]
            .as_object()
            .expect("law table")
            .iter()
            .find(|(_, values)| values.as_array().is_some_and(|values| !values.is_empty()))
            .map(|(role, _)| role.as_str())
            .expect("catalog has law definitions");
        let mut nested = original.clone();
        nested["law_roles"][role][0]["revision"] = serde_json::json!("unadmitted");
        cases.push((
            serde_json::to_string(&nested).expect("fixture encodes"),
            format!("law_roles.{role}[0].revision"),
        ));
        let constraint = original["operations"]
            .as_array()
            .expect("operations")
            .iter()
            .find_map(|operation| operation["constraints"].as_array()?.first())
            .expect("catalog has a constraint");
        for operand in [serde_json::json!(-1), serde_json::json!("not_an_integer")] {
            let mut invalid = original.clone();
            let mut changed = constraint.clone();
            changed["operands"] = serde_json::json!([operand]);
            invalid["operations"][0]["constraints"] = serde_json::json!([changed]);
            cases.push((
                serde_json::to_string(&invalid).expect("fixture encodes"),
                "operations[0].constraints[0].operands[0]".to_owned(),
            ));
        }
        for (bytes, expected_path) in cases {
            let measure = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut unavailable = None;
                let allocations = allocation_counter::measure(|| {
                    if let Err(CatalogStop::Unavailable(error)) =
                        read_catalog_budgeted(&bytes, &mut budget)
                    {
                        unavailable = Some(error);
                    }
                });
                (unavailable, allocations.bytes_total, budget)
            };
            let (unavailable, allocated, unlimited) = measure(u64::MAX);
            assert!(
                unavailable.is_some(),
                "the corrupted catalog is unavailable"
            );
            assert_eq!(
                unavailable.as_ref().map(|error| error.path()),
                Some(expected_path.as_str())
            );
            let charged = unlimited.read_storage_consumed();
            assert_eq!(
                allocated, charged,
                "every catalog error allocation is charged"
            );
            assert!(
                charged > 0,
                "earlier requests and diagnostic remain charged"
            );
            let (unavailable, allocated, exact) = measure(charged);
            assert!(
                unavailable.is_some(),
                "the exact diagnostic allowance returns the catalog error"
            );
            assert_eq!(allocated, exact.read_storage_consumed());
            let (unavailable, allocated, short) = measure(charged - 1);
            assert!(
                unavailable.is_none(),
                "one byte below the diagnostic allowance returns the budget denial"
            );
            assert_eq!(allocated, short.read_storage_consumed());
            assert_eq!(
                short.first_unpaid_locus().map(|locus| locus.form),
                Some("operation_catalog")
            );
        }
    }

    /// The charged projections retain the same catalog decisions and complete
    /// operation records as the existing reader over the published bytes.
    #[trace("TC-472", "FR-038-AC-512", "FR-038-AC-515")]
    #[test]
    fn owned_catalog_preserves_the_published_typed_catalog() {
        let bytes = super::super::operation_catalog::CATALOG_BYTES;
        let legacy = read_catalog(bytes).expect("existing catalog reads");
        let mut budget = NativeJoinBudget::unlimited();
        let Ok(owned) = read_catalog_budgeted(bytes, &mut budget) else {
            panic!("owned catalog reads");
        };
        assert_eq!(
            format!("{:?}", owned.operations),
            format!("{:?}", legacy.operations)
        );
        assert_eq!(owned.groups, legacy.groups);
        assert_eq!(owned.law_roles, legacy.law_roles);
        assert_eq!(owned.modes, legacy.modes);
        assert_eq!(owned.type_pinned_modes, legacy.type_pinned_modes);
    }
}
