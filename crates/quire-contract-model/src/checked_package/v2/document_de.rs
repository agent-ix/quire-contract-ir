// SPDX-License-Identifier: AGPL-3.0-or-later
//! A `serde` deserializer over a shared `quire_canonical::Document`, charging
//! the budget for the heap each decoded string and sequence requests.
//!
//! This is the closed wire's typed decode: the reader's document is decoded in
//! place, with no `serde_json` parser, `Value` or `RawValue`. A JSON value the
//! wire carries opaquely (a node body, a diagnostic detail) is handed out as
//! a [`CheckedJson`] handle into the same document.
//!
//! Implements: FR-038 "Budget-charged strict read", stage `WireDecode`.

use super::checked_json::{CheckedJson, CHECKED_JSON_TOKEN};
use super::native_join_budget::{NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource};
use super::PackageReadStage;
use crate::checked_package::common::{pointer_from_steps, Step};
use crate::checked_package::shared::CheckedNodeId;
use crate::checked_package::shared::JsonPointer;
use quire_canonical::{Document, Items, Members, Node, NodeRef};
use serde::de::value::Error;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// The name under which the wire's node-id member is decoded whole.
pub(super) const NODE_ID_TOKEN: &str = "$quire::NodeId";

thread_local! {
    static NODE_ID: RefCell<Option<CheckedNodeId>> = const { RefCell::new(None) };
}

/// The id the deserializer just decoded for the `Deserialize` call it is
/// answering on this thread.
pub(super) fn take_node_id() -> Option<CheckedNodeId> {
    NODE_ID.with(|slot| slot.borrow_mut().take())
}

/// The name under which a boxed value announces its allocation.
const BOX_TOKEN: &str = "$quire::Box";

/// The name under which an exact vector announces its allocation.
const SEQUENCE_TOKEN: &str = "$quire::Sequence";

thread_local! {
    static BOX_BYTES: Cell<usize> = const { Cell::new(0) };
    static ELEMENT_BYTES: Cell<usize> = const { Cell::new(0) };
}

/// A `Vec` reserved once, at the exact element count the document holds.
///
/// The document deserializer reports the array's length, so the vector is
/// allocated at that capacity before any element is decoded and never grows.
/// Any other deserializer gets the same decode through the iterated elements.
pub(in crate::checked_package) fn exact_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    struct Exact<T>(std::marker::PhantomData<T>);
    impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for Exact<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<T>, A::Error> {
            let mut elements = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(element) = seq.next_element()? {
                elements.push(element);
            }
            Ok(elements)
        }

        // The document deserializer charges the whole vector, then hands the
        // sequence on; any other deserializer passes straight through.
        fn visit_newtype_struct<D2: de::Deserializer<'de>>(
            self,
            inner: D2,
        ) -> Result<Vec<T>, D2::Error> {
            inner.deserialize_seq(self)
        }
    }
    ELEMENT_BYTES.with(|bytes| bytes.set(std::mem::size_of::<T>()));
    deserializer.deserialize_newtype_struct(SEQUENCE_TOKEN, Exact(std::marker::PhantomData))
}

/// The graph's operand rows: an exact vector of exact rows, so neither level
/// grows by doubling.
pub(in crate::checked_package) fn exact_rows<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<u64>>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Row(Vec<u64>);
    impl<'de> serde::Deserialize<'de> for Row {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            exact_vec(deserializer).map(Self)
        }
    }
    // `Row` and `Vec<u64>` have one layout, so collecting back into rows
    // reuses the vector's allocation.
    exact_vec::<D, Row>(deserializer).map(|rows| rows.into_iter().map(|row| row.0).collect())
}

/// [`exact_vec`] for an optional member: absent stays `None`.
pub(in crate::checked_package) fn exact_optional_vec<'de, D, T>(
    deserializer: D,
) -> Result<Option<Vec<T>>, D::Error>
where
    D: de::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    struct Wrapped<T>(Vec<T>);
    impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Wrapped<T> {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            exact_vec(deserializer).map(Self)
        }
    }
    Option::<Wrapped<T>>::deserialize(deserializer).map(|rows| rows.map(|rows| rows.0))
}

/// An explicit `null` is a value and an absent member stays a schema refusal;
/// a present value is boxed after the box's allocation is charged.
pub(in crate::checked_package) fn charged_nullable_box<'de, D, T>(
    deserializer: D,
) -> Result<Option<Box<T>>, D::Error>
where
    D: de::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    struct Charged<T>(Box<T>);
    impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Charged<T> {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Held<T>(std::marker::PhantomData<T>);
            impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for Held<T> {
                type Value = Charged<T>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a boxed value")
                }

                fn visit_newtype_struct<D2: de::Deserializer<'de>>(
                    self,
                    inner: D2,
                ) -> Result<Charged<T>, D2::Error> {
                    T::deserialize(inner).map(|value| Charged(Box::new(value)))
                }
            }
            BOX_BYTES.with(|bytes| bytes.set(std::mem::size_of::<T>()));
            deserializer.deserialize_newtype_struct(BOX_TOKEN, Held(std::marker::PhantomData))
        }
    }
    Option::<Charged<T>>::deserialize(deserializer).map(|boxed| boxed.map(|boxed| boxed.0))
}

/// How many path segments the decode tracks for its locus. The closed wire is
/// shallower than this; a deeper position keeps its enclosing locus.
const TRACKED_DEPTH: usize = 12;

#[derive(Clone, Copy)]
enum Segment<'d> {
    Key(&'d str),
    Index(usize),
}

/// The wire paths (arrays and members, without indices) whose decoded
/// containers name their static `form` in the stage table.
const FORMS: &[(&[&str], &str)] = &[
    (&["name"], "name"),
    (&["contract_version"], "contract_version"),
    (&["lock", "sources"], "lock.sources"),
    (
        &["lock", "dependency_selections"],
        "lock.dependency_selections",
    ),
    (
        &["lock", "definition_selections"],
        "lock.definition_selections",
    ),
    (&["lock", "profile_selections"], "lock.profile_selections"),
    (&["lock", "required_features"], "lock.required_features"),
    (
        &["lock", "refinement_internals"],
        "lock.refinement_internals",
    ),
    (&["semantic_graph", "nodes"], "semantic_graph.nodes"),
    (
        &["semantic_graph", "operand_rows"],
        "semantic_graph.operand_rows",
    ),
    (&["source_map"], "source_map"),
    (&["capability_report"], "capability_report"),
    (&["diagnostics", "entries"], "diagnostics.entries"),
    (&["refinement_step_rows"], "refinement_step_rows"),
    (&["checked_term_data"], "checked_term_data"),
    (&["temporal_bindings"], "temporal_bindings"),
    (&["temporal_clause_names"], "temporal_clause_names"),
    (&["schedulability_claims"], "schedulability_claims"),
    (&["operation_effects"], "operation_effects"),
];

/// What a decode shares: the retained document, the budget and the position.
pub(in crate::checked_package) struct DecodeContext<'d> {
    document: &'d Arc<Document>,
    budget: RefCell<NativeJoinBudget>,
    denied: Cell<bool>,
    denied_at: RefCell<Option<JsonPointer>>,
    charging: Cell<bool>,
    path: RefCell<([Option<Segment<'d>>; TRACKED_DEPTH], usize)>,
    /// A locus every charge is made at, for a decode of one term below the
    /// wire's root rather than of the wire itself.
    fixed_locus: Option<NativeJoinLocus>,
    /// The operand row of the node whose body is decoded: an operand slot
    /// `{"operand": k}` in a node ID's path reads as `{"node": row[k]}`, the
    /// form the reader's resolved wire gives it.
    operand_row: Option<&'d [u64]>,
}

impl<'d> DecodeContext<'d> {
    /// A decode over `document`, charging the budget it takes from `budget`
    /// (left as an empty placeholder until [`Self::finish`] hands it back).
    pub(in crate::checked_package) fn new(
        document: &'d Arc<Document>,
        budget: &mut NativeJoinBudget,
    ) -> Self {
        Self {
            document,
            budget: RefCell::new(std::mem::replace(budget, NativeJoinBudget::new(0, 0))),
            denied: Cell::new(false),
            denied_at: RefCell::new(None),
            charging: Cell::new(true),
            path: RefCell::new(([None; TRACKED_DEPTH], 0)),
            fixed_locus: None,
            operand_row: None,
        }
    }

    /// Reads operand slots through `row` (see the field).
    pub(in crate::checked_package) fn with_operand_row(mut self, row: &'d [u64]) -> Self {
        self.operand_row = Some(row);
        self
    }

    /// [`Self::new`] for a decode whose every charge is made at `locus`.
    pub(in crate::checked_package) fn at_locus(
        document: &'d Arc<Document>,
        budget: &mut NativeJoinBudget,
        locus: NativeJoinLocus,
    ) -> Self {
        Self {
            fixed_locus: Some(locus),
            ..Self::new(document, budget)
        }
    }

    /// The deserializer for the document value `node`.
    pub(in crate::checked_package) fn value_at(&self, node: NodeRef<'d>) -> ValueDe<'_, 'd> {
        ValueDe {
            node,
            context: self,
            slots: false,
        }
    }

    /// Charges `bytes` of heap the caller is about to request, at the
    /// decode's locus.
    pub(in crate::checked_package) fn charge_bytes(&self, bytes: usize) -> bool {
        self.charge(bytes).is_ok()
    }

    /// Returns the budget, with every admitted charge, to its owner.
    pub(in crate::checked_package) fn finish(self) -> NativeJoinBudget {
        self.budget.into_inner()
    }

    /// Stops charging, for a second pass that only locates a fault.
    pub(in crate::checked_package) fn pause_charging(&self) {
        self.charging.set(false);
    }

    /// The pointer of the value being decoded when the budget refused.
    pub(in crate::checked_package) fn denied_at(&self) -> Option<JsonPointer> {
        self.denied_at.borrow().clone()
    }

    /// Whether the budget refused a charge during the decode.
    pub(in crate::checked_package) fn denied(&self) -> bool {
        self.denied.get()
    }

    /// The deserializer for the document's root.
    pub(in crate::checked_package) fn root(&self) -> ValueDe<'_, 'd> {
        ValueDe {
            node: self.document.root(),
            context: self,
            prepaid: false,
            slots: false,
        }
    }

    fn push(&self, segment: Segment<'d>) {
        let mut path = self.path.borrow_mut();
        let depth = path.1;
        if let Some(slot) = path.0.get_mut(depth) {
            *slot = Some(segment);
        }
        path.1 = depth.saturating_add(1);
    }

    fn pop(&self) {
        let mut path = self.path.borrow_mut();
        path.1 = path.1.saturating_sub(1);
    }

    /// The `WireDecode` locus of the value being decoded: the position in the
    /// innermost array holding it, and the dotted wire path of that array.
    fn locus(&self) -> NativeJoinLocus {
        if let Some(fixed) = self.fixed_locus {
            return fixed;
        }
        let path = self.path.borrow();
        let tracked = path.0.iter().take(path.1).flatten();
        let mut index = 0;
        let mut keys: [&str; TRACKED_DEPTH] = [""; TRACKED_DEPTH];
        let mut count = 0;
        for segment in tracked {
            match segment {
                Segment::Key(key) => {
                    if let Some(slot) = keys.get_mut(count) {
                        *slot = key;
                        count += 1;
                    }
                }
                Segment::Index(position) => index = *position,
            }
        }
        let wanted = keys.get(..count).unwrap_or_default();
        let form = FORMS
            .iter()
            .find(|(path, _)| *path == wanted)
            .map_or("wire", |(_, form)| *form);
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::WireDecode, index),
            form,
        }
    }

    /// The RFC 6901 pointer of the value being decoded.
    fn pointer(&self) -> JsonPointer {
        let path = self.path.borrow();
        let steps = path
            .0
            .iter()
            .take(path.1)
            .flatten()
            .map(|segment| match segment {
                Segment::Key(key) => Step::Key(key),
                Segment::Index(index) => Step::Index(*index),
            });
        pointer_from_steps(steps)
    }

    /// Charges `bytes` of heap the decode is about to request.
    fn charge(&self, bytes: usize) -> Result<(), Error> {
        if bytes == 0 || !self.charging.get() {
            return Ok(());
        }
        let locus = self.locus();
        let mut budget = self.budget.borrow_mut();
        budget.set_active_locus(Some(locus));
        let admitted = budget.charge_read_storage(bytes as u128);
        budget.set_active_locus(None);
        if admitted {
            Ok(())
        } else {
            self.denied.set(true);
            *self.denied_at.borrow_mut() = Some(self.pointer());
            Err(de::Error::custom(
                "the read-storage budget refused a request",
            ))
        }
    }
}

/// One document value as a `serde` deserializer.
pub(in crate::checked_package) struct ValueDe<'c, 'd> {
    node: NodeRef<'d>,
    context: &'c DecodeContext<'d>,
    /// The enclosing call already charged this sequence's elements.
    prepaid: bool,
    /// Whether this value is the `path` of a node ID, or a segment of one, in
    /// a decode that resolves operand slots through a row.
    slots: bool,
}

impl<'c, 'd> ValueDe<'c, 'd> {
    fn child(&self, node: NodeRef<'d>) -> ValueDe<'c, 'd> {
        self.child_with(node, true)
    }

    fn child_with(&self, node: NodeRef<'d>, inherit_slots: bool) -> ValueDe<'c, 'd> {
        ValueDe {
            node,
            context: self.context,
            prepaid: false,
            slots: self.slots && inherit_slots,
        }
    }
}

impl<'de, 'c> de::Deserializer<'de> for ValueDe<'c, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.node.node() {
            Node::Null => visitor.visit_unit(),
            Node::Bool(flag) => visitor.visit_bool(flag),
            Node::Number(number) => {
                let text = number.text();
                if let Ok(value) = text.parse::<u64>() {
                    visitor.visit_u64(value)
                } else if let Ok(value) = text.parse::<i64>() {
                    visitor.visit_i64(value)
                } else {
                    visitor.visit_f64(number.value())
                }
            }
            Node::String(text) => visitor.visit_borrowed_str(text),
            Node::Array(items) => {
                let count = items.len();
                visitor.visit_seq(Sequence {
                    items,
                    count,
                    position: 0,
                    charged: self.prepaid,
                    parent: self,
                })
            }
            Node::Object(members) => {
                if let (true, Some(row)) = (self.slots, self.context.operand_row) {
                    let mut only = members.clone();
                    if let (Some(("operand", slot)), None) = (only.next(), only.next()) {
                        let entry = match slot.node() {
                            Node::Number(number) => number
                                .text()
                                .parse::<u64>()
                                .ok()
                                .and_then(|slot| usize::try_from(slot).ok())
                                .and_then(|slot| row.get(slot)),
                            _ => None,
                        };
                        if let Some(&entry) = entry {
                            return visitor.visit_map(ResolvedSlot {
                                entry: Some(entry),
                                keyed: false,
                            });
                        }
                    }
                }
                let path_slots = self.context.operand_row.is_some()
                    && members.len() == 3
                    && ["package", "ordinal"]
                        .iter()
                        .all(|name| members.clone().any(|(member, _)| member == *name));
                visitor.visit_map(Mapping {
                    members,
                    value: None,
                    parent: self,
                    path_slots,
                    name: "",
                })
            }
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.node.node() {
            Node::String(text) => {
                self.context.charge(text.len())?;
                visitor.visit_borrowed_str(text)
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.node.node() {
            Node::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if name == CHECKED_JSON_TOKEN {
            // The handle is a clone of the retained document's `Arc`, which
            // allocates nothing.
            CheckedJson::offer(CheckedJson::new(
                Arc::clone(self.context.document),
                self.node.id(),
            ));
            return visitor.visit_unit();
        }
        if name == SEQUENCE_TOKEN {
            if let Node::Array(items) = self.node.node() {
                let bytes = ELEMENT_BYTES.with(Cell::get).saturating_mul(items.len());
                self.context.charge(bytes)?;
            }
            return visitor.visit_newtype_struct(ValueDe {
                prepaid: true,
                ..self
            });
        }
        if name == BOX_TOKEN {
            let bytes = BOX_BYTES.with(Cell::get);
            self.context.charge(bytes)?;
            return visitor.visit_newtype_struct(self);
        }
        if name == NODE_ID_TOKEN {
            let decoded = CheckedNodeId::deserialize(ValueDe {
                node: self.node,
                context: self.context,
                prepaid: false,
                slots: false,
            })?;
            NODE_ID.with(|slot| *slot.borrow_mut() = Some(decoded));
            return visitor.visit_unit();
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.node.node() {
            Node::String(text) => visitor.visit_enum(text.into_deserializer()),
            Node::Object(mut members) => {
                let Some((variant, value)) = members.next() else {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Map,
                        &"an object with exactly one member",
                    ));
                };
                if members.next().is_some() {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Map,
                        &"an object with exactly one member",
                    ));
                }
                visitor.visit_enum(Variant {
                    name: variant,
                    value: self.child(value),
                })
            }
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("a non-enum value"),
                &"a string or a one-member object",
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}

struct Sequence<'c, 'd> {
    items: Items<'d>,
    count: usize,
    position: usize,
    charged: bool,
    parent: ValueDe<'c, 'd>,
}

impl<'de, 'c> SeqAccess<'de> for Sequence<'c, 'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if !self.charged {
            self.charged = true;
            // The sequence's container is reserved at its exact final length
            // before its first element is decoded.
            let bytes = self.count.saturating_mul(std::mem::size_of::<T::Value>());
            self.parent.context.charge(bytes)?;
        }
        let Some(item) = self.items.next() else {
            return Ok(None);
        };
        let context = self.parent.context;
        context.push(Segment::Index(self.position));
        self.position += 1;
        let decoded = seed.deserialize(self.parent.child(item));
        context.pop();
        decoded.map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count.saturating_sub(self.position))
    }
}

struct Mapping<'c, 'd> {
    members: Members<'d>,
    value: Option<NodeRef<'d>>,
    parent: ValueDe<'c, 'd>,
    /// This object is a node ID read through an operand row: its `path`
    /// resolves operand slots.
    path_slots: bool,
    /// The name of the member whose value is next.
    name: &'d str,
}

/// An operand slot `{"operand": k}` read as `{"node": row[k]}`.
struct ResolvedSlot {
    entry: Option<u64>,
    keyed: bool,
}

impl<'de> MapAccess<'de> for ResolvedSlot {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        if self.keyed || self.entry.is_none() {
            return Ok(None);
        }
        self.keyed = true;
        seed.deserialize(KeyDe { name: "node" }).map(Some)
    }

    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, Error> {
        let entry = self
            .entry
            .take()
            .ok_or_else(|| de::Error::custom("a value without a member name"))?;
        seed.deserialize(entry.into_deserializer())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(1)
    }
}

impl<'de, 'c> MapAccess<'de> for Mapping<'c, 'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some((name, value)) = self.members.next() else {
            return Ok(None);
        };
        self.value = Some(value);
        self.name = name;
        self.parent.context.push(Segment::Key(name));
        seed.deserialize(KeyDe { name }).map(Some)
    }

    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| de::Error::custom("a value without a member name"))?;
        let slots = self.path_slots && self.name == "path";
        let decoded = seed.deserialize(ValueDe {
            slots,
            ..self.parent.child_with(value, false)
        });
        self.parent.context.pop();
        decoded
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.members.len())
    }
}

/// A member name as a deserializer; a name is an identifier and allocates
/// nothing here.
struct KeyDe<'d> {
    name: &'d str,
}

impl<'de> de::Deserializer<'de> for KeyDe<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_borrowed_str(self.name)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

struct Variant<'c, 'd> {
    name: &'d str,
    value: ValueDe<'c, 'd>,
}

impl<'de, 'c> EnumAccess<'de> for Variant<'c, 'de> {
    type Error = Error;
    type Variant = ValueDe<'c, 'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let name = seed.deserialize(KeyDe { name: self.name })?;
        Ok((name, self.value))
    }
}

impl<'de> VariantAccess<'de> for ValueDe<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.node.node() {
            Node::Null => Ok(()),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("a value"),
                &"a unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_any(self, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_any(self, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::super::intake::decode_borrowed_wire;
    use super::super::{NativeJoinBudget, ValidationFailure};
    use crate::checked_package::shared::{CheckedArtifactRef, CheckedSelection};
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::sync::Arc;

    /// A closed package holding a vector over 1 MiB (70001 operand rows) and a
    /// row over 1 MiB (140000 entries); no validation applies to the decode.
    fn wide_package() -> Vec<u8> {
        let artifact = CheckedArtifactRef {
            authority: "a".into(),
            identity: "i".into(),
        };
        let selection = CheckedSelection {
            role: super::super::CheckedSelectionRole::ProtocolProfile,
            definition: artifact.clone(),
        };
        let mut rows = vec![json!([]); 70_000];
        rows.push(json!(vec![7_u64; 140_000]));
        let package = json!({
            "contract_version": "quire.checked-package/v2",
            "name": "p",
            "lock": {
                "sources": [],
                "edition": selection,
                "profile_selections": [],
                "definition_selections": [],
                "required_features": [],
                "dependency_selections": [],
            },
            "semantic_graph": {
                "graph_version": "quire.checked-semantic-graph/v2",
                "nodes": [],
                "operand_rows": rows,
            },
            "source_map": [],
            "capability_report": [],
            "diagnostics": {"catalog": artifact, "entries": []},
        });
        quire_canonical::to_vec(&package, quire_canonical::Limits::new(u64::MAX))
            .expect("the package encodes")
    }

    fn decode(
        bytes: &[u8],
        document: &Arc<quire_canonical::Document>,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), ValidationFailure> {
        decode_borrowed_wire(bytes, document, budget).map(drop)
    }

    /// The bytes the decode allocates equal the bytes it charged, including a
    /// vector and a row past serde's 1 MiB pre-size cap; the exact limit
    /// admits and one byte short refuses.
    #[trace("TC-472", "FR-038-AC-499", "FR-358-AC-12")]
    #[test]
    fn the_decode_allocates_exactly_what_it_charges() {
        let bytes = wide_package();
        let document = Arc::new(quire_canonical::read(&bytes, u64::MAX).expect("reads"));
        let mut measured = NativeJoinBudget::unlimited();
        let mut outcome = None;
        let info = allocation_counter::measure(|| {
            outcome = Some(decode(&bytes, &document, &mut measured));
        });
        assert!(outcome.expect("decoded").is_ok());
        let charged = measured.read_storage_consumed();
        assert!(charged > 140_000 * 8 + 70_001 * 24, "{charged}");
        assert_eq!(info.bytes_total, charged);

        let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(charged);
        assert!(decode(&bytes, &document, &mut exact).is_ok());
        let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(charged - 1);
        assert!(matches!(
            decode(&bytes, &document, &mut short),
            Err(ValidationFailure::Budget(_))
        ));
    }
}
