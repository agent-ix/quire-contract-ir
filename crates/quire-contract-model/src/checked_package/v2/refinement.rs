//! Closed FR-375 refinement body vocabulary and reader checks.

use super::CheckedTerm;
use crate::checked_package::common::{
    validate_term, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSourceMapEntry, CheckedSourceRef,
    CheckedSourceRegion,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Source region of one canonical refinement step row, outside package identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedRefinementStepRowRegion {
    /// Retained refinement relation node.
    pub node_id: CheckedNodeId,
    /// Zero-based position in its received steps array.
    pub ordinal: u64,
    /// The checked source row's single region.
    pub region: CheckedSourceRegion,
}

impl Encode for CheckedRefinementStepRowRegion {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "node_id", &self.node_id)?;
        super::encode::member(writer, "ordinal", &self.ordinal)?;
        super::encode::member(writer, "region", &self.region)?;
        writer.end_object()
    }
}

fn node_at(position: usize) -> crate::checked_package::shared::JsonPointer {
    crate::checked_package::common::node_pointer(position)
}

/// A refinement target is either a local node or a node in a directly selected package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefinementKey {
    /// A declaration in this graph.
    Local(CheckedNodeId),
    /// A declaration in a directly selected dependency package.
    External(RefinementPackageNodeKey),
}

/// A node of a directly selected dependency package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementPackageNodeKey {
    /// The selected package identity.
    pub package: Box<str>,
    /// The target node identity in that package.
    pub node: CheckedNodeId,
}

/// One abstract and concrete population pair.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementPopulation {
    /// Abstract population.
    pub r#abstract: RefinementKey,
    /// Concrete population.
    pub concrete: CheckedNodeId,
}

/// One mapped abstract field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementField {
    /// The abstract field name.
    pub name: Box<str>,
    /// Its value in the concrete state.
    pub expression: CheckedTerm,
}

/// One abstract object type mapped from a concrete object type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementObject {
    /// Abstract object declaration.
    pub abstract_type: RefinementKey,
    /// Concrete object declaration.
    pub concrete_type: CheckedNodeId,
    /// Mapped fields in name order.
    pub fields: Vec<RefinementField>,
}

/// An argument expression or the special open argument `_`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefinementStepArgument {
    /// An explicitly open argument.
    Open(RefinementOpen),
    /// A closed semantic term.
    Term(Box<CheckedTerm>),
}

/// The exact step-argument-only open term.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct RefinementOpen {
    /// Always `open`.
    pub term: Box<str>,
}

/// The sole spelling admitted for an open step argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RefinementOpenTerm {
    /// An explicitly open argument.
    Open,
}

/// One of the four closed abstract step targets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RefinementStepTarget {
    /// An abstract operation call.
    Operation {
        /// Abstract operation declaration.
        operation: RefinementKey,
        /// Mapped receiver.
        receiver: Box<CheckedTerm>,
        /// Mapped arguments.
        arguments: Vec<RefinementStepArgument>,
    },
    /// An abstract protocol step.
    ProtocolStep {
        /// Alias-free canonical step class.
        class: Box<str>,
        /// Declaration node for declaration-naming classes.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<RefinementKey>,
        /// Mapped arguments.
        arguments: Vec<RefinementStepArgument>,
    },
    /// No abstract state change.
    Stutter,
    /// A deliberately unconstrained abstract step.
    Any,
}

/// One concrete step mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementStep {
    /// Alias-free canonical concrete class.
    pub source_class: Box<str>,
    /// Concrete declaration for declaration-naming classes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node: Option<CheckedNodeId>,
    /// Abstract target.
    pub target: RefinementStepTarget,
}

/// Checked classes of the selected concrete and abstract subjects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementClassCatalog {
    /// Concrete operation, protocol and memory classes.
    pub concrete: Vec<RefinementConcreteClass>,
    /// Abstract operation, protocol and memory classes.
    pub r#abstract: Vec<RefinementAbstractClass>,
}

/// A concrete class with a declaration or a source-default memory step.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RefinementConcreteClass {
    /// A class backed by a local declaration node.
    Declared {
        /// Alias-free canonical spelling.
        class: Box<str>,
        /// Local declaring node.
        node: CheckedNodeId,
        /// Checked binder types in declaration order.
        binder_types: Vec<CheckedNodeId>,
    },
    /// A source-default memory class.
    Memory(RefinementMemoryClass),
}

/// An abstract class with a declaration or a source-default memory step.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RefinementAbstractClass {
    /// A class backed by a selected declaration node.
    Declared {
        /// Alias-free canonical spelling.
        class: Box<str>,
        /// Local or directly selected declaring node.
        node: RefinementKey,
        /// Checked binder types in declaration order.
        binder_types: Vec<RefinementKey>,
    },
    /// A source-default memory class.
    Memory(RefinementMemoryClass),
}

/// A source-default memory class, distinguished by its closed step semantics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "class", deny_unknown_fields)]
pub enum RefinementMemoryClass {
    /// Drain pending writes under the selected source-default model.
    #[serde(rename = "memory drain")]
    Drain {
        /// The source-default model selected by a parallel control.
        memory_model: RefinementMemoryModel,
        /// The sole admitted drain operation.
        drain: RefinementDrain,
    },
    /// Remove the oldest buffered TSO write.
    #[serde(rename = "memory flush")]
    Flush {
        /// The sole source-default model that defines flush.
        memory_model: RefinementTsoModel,
    },
}

/// Closed operation performed by a memory drain step.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "kebab-case")]
pub enum RefinementDrain {
    /// Drain the thread's store buffer until its writes are visible.
    StoreBufferFlush,
}

/// The only source-default model that defines a flush class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum RefinementTsoModel {
    /// Total store order.
    Tso,
}

/// Closed source-default memory model vocabulary.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, FixedShape,
)]
#[serde(rename_all = "snake_case")]
pub enum RefinementMemoryModel {
    /// Sequential consistency.
    Sc,
    /// Total store order.
    Tso,
    /// Release acquire.
    Ra,
}

impl RefinementConcreteClass {
    fn class(&self) -> &str {
        match self {
            Self::Declared { class, .. } => class,
            Self::Memory(memory) => memory.class(),
        }
    }
}

impl RefinementAbstractClass {
    fn class(&self) -> &str {
        match self {
            Self::Declared { class, .. } => class,
            Self::Memory(memory) => memory.class(),
        }
    }
}

impl RefinementMemoryClass {
    fn class(&self) -> &'static str {
        match self {
            Self::Drain { .. } => "memory drain",
            Self::Flush { .. } => "memory flush",
        }
    }

    fn model(&self) -> RefinementMemoryModel {
        match self {
            Self::Drain { memory_model, .. } => *memory_model,
            Self::Flush { .. } => RefinementMemoryModel::Tso,
        }
    }
}

impl RefinementMemoryModel {
    fn as_wire(self) -> &'static str {
        match self {
            Self::Ra => "ra",
            Self::Sc => "sc",
            Self::Tso => "tso",
        }
    }
}

fn concrete_catalog_key(entry: &RefinementConcreteClass) -> (&str, &str) {
    match entry {
        RefinementConcreteClass::Declared { class, .. } => (class, ""),
        RefinementConcreteClass::Memory(memory) => (memory.class(), memory.model().as_wire()),
    }
}

fn abstract_catalog_key(entry: &RefinementAbstractClass) -> (&str, &str) {
    match entry {
        RefinementAbstractClass::Declared { class, .. } => (class, ""),
        RefinementAbstractClass::Memory(memory) => (memory.class(), memory.model().as_wire()),
    }
}

/// A history update after one concrete operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementHistoryUpdate {
    /// Concrete operation declaration.
    pub operation: CheckedNodeId,
    /// New history value.
    pub expression: CheckedTerm,
}

/// One history field introduced by the refinement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementHistory {
    /// Concrete object type.
    pub concrete_type: CheckedNodeId,
    /// New field name.
    pub name: Box<str>,
    /// Bounded history value type.
    #[serde(rename = "type")]
    pub type_id: CheckedNodeId,
    /// Literal or local history-literal reference.
    pub initial: CheckedTerm,
    /// Updates in operation-key order.
    pub updates: Vec<RefinementHistoryUpdate>,
}

/// Fairness strength.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, FixedShape,
)]
#[serde(rename_all = "snake_case")]
pub enum RefinementFairnessKind {
    /// Weak fairness.
    Weak,
    /// Strong fairness.
    Strong,
}

/// Fairness granularity.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, FixedShape,
)]
#[serde(rename_all = "snake_case")]
pub enum RefinementGranularity {
    /// Per-instance fairness.
    Each,
    /// Whole-operation fairness.
    Whole,
}

/// One concrete fairness assumption.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementAssume {
    /// Concrete operation.
    pub operation: CheckedNodeId,
    /// Weak or strong fairness.
    pub kind: RefinementFairnessKind,
    /// Per-instance or whole-operation fairness.
    pub granularity: RefinementGranularity,
}

/// One abstract fairness guarantee.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementEnsure {
    /// Abstract operation.
    pub operation: RefinementKey,
    /// Weak or strong fairness.
    pub kind: RefinementFairnessKind,
    /// Per-instance or whole-operation fairness.
    pub granularity: RefinementGranularity,
}

/// Encode bounded refinement rows through the canonical writer, including
/// nested semantic terms whose shape is not fixed at the Rust type level.
pub(super) fn encode_rows<T: Encode, S: Sink + ?Sized>(
    rows: &[T],
    writer: &mut Writer<'_, S>,
) -> Result<(), Error> {
    writer.begin_array()?;
    for row in rows {
        row.encode_into(writer)?;
    }
    writer.end_array()
}

impl Encode for RefinementKey {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        match self {
            Self::Local(node) => node.encode_into(writer),
            Self::External(external) => external.encode_into(writer),
        }
    }
}

impl Encode for RefinementPackageNodeKey {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "package", &self.package)?;
        super::encode::encoded(writer, "node", &self.node)?;
        writer.end_object()
    }
}

impl Encode for RefinementPopulation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "abstract", &self.r#abstract)?;
        super::encode::encoded(writer, "concrete", &self.concrete)?;
        writer.end_object()
    }
}

impl Encode for RefinementConcreteClass {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        match self {
            Self::Memory(memory) => writer.serialize(memory),
            Self::Declared {
                class,
                node,
                binder_types,
            } => {
                writer.begin_object()?;
                super::encode::member(writer, "class", class)?;
                super::encode::encoded(writer, "node", node)?;
                super::encode::elements(writer, "binder_types", binder_types)?;
                writer.end_object()
            }
        }
    }
}

impl Encode for RefinementAbstractClass {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        match self {
            Self::Memory(memory) => writer.serialize(memory),
            Self::Declared {
                class,
                node,
                binder_types,
            } => {
                writer.begin_object()?;
                super::encode::member(writer, "class", class)?;
                super::encode::encoded(writer, "node", node)?;
                super::encode::elements(writer, "binder_types", binder_types)?;
                writer.end_object()
            }
        }
    }
}

impl Encode for RefinementClassCatalog {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::elements(writer, "concrete", &self.concrete)?;
        super::encode::elements(writer, "abstract", &self.r#abstract)?;
        writer.end_object()
    }
}

impl Encode for RefinementAssume {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "operation", &self.operation)?;
        super::encode::member(writer, "kind", &self.kind)?;
        super::encode::member(writer, "granularity", &self.granularity)?;
        writer.end_object()
    }
}

impl Encode for RefinementEnsure {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "operation", &self.operation)?;
        super::encode::member(writer, "kind", &self.kind)?;
        super::encode::member(writer, "granularity", &self.granularity)?;
        writer.end_object()
    }
}

impl Encode for RefinementField {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("name")?;
        writer.string(&self.name)?;
        writer.name("expression")?;
        self.expression.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for RefinementObject {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("abstract_type")?;
        self.abstract_type.encode_into(writer)?;
        writer.name("concrete_type")?;
        self.concrete_type.encode_into(writer)?;
        writer.name("fields")?;
        encode_rows(&self.fields, writer)?;
        writer.end_object()
    }
}

impl Encode for RefinementStepArgument {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        match self {
            Self::Open(open) => writer.serialize(open),
            Self::Term(term) => term.encode_into(writer),
        }
    }
}

impl Encode for RefinementStepTarget {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("kind")?;
        match self {
            Self::Operation {
                operation,
                receiver,
                arguments,
            } => {
                writer.string("operation")?;
                writer.name("operation")?;
                operation.encode_into(writer)?;
                writer.name("receiver")?;
                receiver.encode_into(writer)?;
                writer.name("arguments")?;
                encode_rows(arguments, writer)?;
            }
            Self::ProtocolStep {
                class,
                node,
                arguments,
            } => {
                writer.string("protocol_step")?;
                writer.name("class")?;
                writer.string(class)?;
                if let Some(node) = node {
                    writer.name("node")?;
                    node.encode_into(writer)?;
                }
                writer.name("arguments")?;
                encode_rows(arguments, writer)?;
            }
            Self::Stutter => writer.string("stutter")?,
            Self::Any => writer.string("any")?,
        }
        writer.end_object()
    }
}

impl Encode for RefinementStep {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("source_class")?;
        writer.string(&self.source_class)?;
        if let Some(node) = &self.source_node {
            writer.name("source_node")?;
            node.encode_into(writer)?;
        }
        writer.name("target")?;
        self.target.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for RefinementHistoryUpdate {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("operation")?;
        self.operation.encode_into(writer)?;
        writer.name("expression")?;
        self.expression.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for RefinementHistory {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("concrete_type")?;
        self.concrete_type.encode_into(writer)?;
        writer.name("name")?;
        writer.string(&self.name)?;
        writer.name("type")?;
        self.type_id.encode_into(writer)?;
        writer.name("initial")?;
        self.initial.encode_into(writer)?;
        writer.name("updates")?;
        encode_rows(&self.updates, writer)?;
        writer.end_object()
    }
}

/// Retarget a draft refinement's typed node references without projecting
/// the admitted body through an untyped JSON encoder.
pub(super) fn replace_node_ids(body: &mut CheckedTerm, old: &CheckedNodeId, new: &CheckedNodeId) {
    fn replace(id: &mut CheckedNodeId, old: &CheckedNodeId, new: &CheckedNodeId) {
        if id == old {
            *id = new.clone();
        }
    }
    fn replace_key(key: &mut RefinementKey, old: &CheckedNodeId, new: &CheckedNodeId) {
        match key {
            RefinementKey::Local(id) => replace(id, old, new),
            RefinementKey::External(external) => replace(&mut external.node, old, new),
        }
    }
    fn replace_arguments(
        arguments: &mut [RefinementStepArgument],
        old: &CheckedNodeId,
        new: &CheckedNodeId,
    ) {
        for argument in arguments {
            if let RefinementStepArgument::Term(term) = argument {
                term.replace_node_id(old, new);
            }
        }
    }
    let CheckedTerm::Refinement {
        abstract_key,
        populations,
        objects,
        steps,
        histories,
        assume,
        ensure,
        protocol,
        class_catalog,
        ..
    } = body
    else {
        return;
    };
    replace_key(abstract_key, old, new);
    for row in populations {
        replace_key(&mut row.r#abstract, old, new);
        replace(&mut row.concrete, old, new);
    }
    for row in objects {
        replace_key(&mut row.abstract_type, old, new);
        replace(&mut row.concrete_type, old, new);
        for field in &mut row.fields {
            field.expression.replace_node_id(old, new);
        }
    }
    for row in steps {
        if let Some(node) = &mut row.source_node {
            replace(node, old, new);
        }
        match &mut row.target {
            RefinementStepTarget::Operation {
                operation,
                receiver,
                arguments,
            } => {
                replace_key(operation, old, new);
                receiver.replace_node_id(old, new);
                replace_arguments(arguments, old, new);
            }
            RefinementStepTarget::ProtocolStep {
                node, arguments, ..
            } => {
                if let Some(node) = node {
                    replace_key(node, old, new);
                }
                replace_arguments(arguments, old, new);
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for row in histories {
        replace(&mut row.concrete_type, old, new);
        replace(&mut row.type_id, old, new);
        row.initial.replace_node_id(old, new);
        for update in &mut row.updates {
            replace(&mut update.operation, old, new);
            update.expression.replace_node_id(old, new);
        }
    }
    for row in assume {
        replace(&mut row.operation, old, new);
    }
    for row in ensure {
        replace_key(&mut row.operation, old, new);
    }
    if let Some(protocol) = protocol {
        replace_key(protocol, old, new);
    }
    if let Some(catalog) = class_catalog {
        for entry in &mut catalog.concrete {
            if let RefinementConcreteClass::Declared {
                node, binder_types, ..
            } = entry
            {
                replace(node, old, new);
                for binder in binder_types {
                    replace(binder, old, new);
                }
            }
        }
        for entry in &mut catalog.r#abstract {
            if let RefinementAbstractClass::Declared {
                node, binder_types, ..
            } = entry
            {
                replace_key(node, old, new);
                for binder in binder_types {
                    replace_key(binder, old, new);
                }
            }
        }
    }
}

fn invalid(at: &Trail<'_>) -> ValidationFailure {
    ValidationFailure::refused(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        at.pointer(),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RefinementClassKind {
    Declared,
    Memory,
}

/// Convert a raw source class spelling before graph admission.
#[ix_trace_rs::string_edge]
fn class_kind(class: &str) -> Option<RefinementClassKind> {
    if class.is_empty() {
        None
    } else if class.starts_with("memory ") {
        Some(RefinementClassKind::Memory)
    } else {
        Some(RefinementClassKind::Declared)
    }
}

#[ix_trace_rs::string_edge]
fn memory_class_kind(class: &str) -> bool {
    matches!(class, "memory drain" | "memory flush")
}

/// Convert the untrusted argument-only special term before row admission.
#[ix_trace_rs::string_edge]
fn open_term_kind(term: &str) -> Option<RefinementOpenTerm> {
    (term == "open").then_some(RefinementOpenTerm::Open)
}

fn member<'a>(
    value: &'a Value,
    name: &str,
    at: &Trail<'_>,
) -> Result<&'a Value, ValidationFailure> {
    value.get(name).ok_or_else(|| invalid(&at.key(name)))
}

fn array<'a>(
    value: &'a Value,
    name: &str,
    at: &Trail<'_>,
) -> Result<&'a [Value], ValidationFailure> {
    member(value, name, at)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(&at.key(name)))
}

fn typed<T: DeserializeOwned>(value: &Value, at: &Trail<'_>) -> Result<T, ValidationFailure> {
    serde_json::from_value(value.clone()).map_err(|_| invalid(at))
}

fn shape(
    value: &Value,
    required: &[&str],
    optional: &[&str],
    at: &Trail<'_>,
) -> Result<(), ValidationFailure> {
    let object = value.as_object().ok_or_else(|| invalid(at))?;
    for name in required {
        if !object.contains_key(*name) {
            return Err(invalid(&at.key(name)));
        }
    }
    if let Some(extra) = object
        .keys()
        .find(|name| !required.contains(&name.as_str()) && !optional.contains(&name.as_str()))
    {
        return Err(invalid(&at.key(extra)));
    }
    Ok(())
}

fn text(value: &Value, at: &Trail<'_>) -> Result<Box<str>, ValidationFailure> {
    value.as_str().map(Into::into).ok_or_else(|| invalid(at))
}

fn node(value: &Value, at: &Trail<'_>) -> Result<CheckedNodeId, ValidationFailure> {
    CheckedNodeId::from_value(value).ok_or_else(|| invalid(at))
}

fn key(value: &Value, at: &Trail<'_>) -> Result<RefinementKey, ValidationFailure> {
    if value.get("node").is_some() {
        shape(value, &["package", "node"], &[], at)?;
        let package = text(member(value, "package", at)?, &at.key("package"))?;
        let target = node(member(value, "node", at)?, &at.key("node"))?;
        Ok(RefinementKey::External(RefinementPackageNodeKey {
            package,
            node: target,
        }))
    } else {
        node(value, at).map(RefinementKey::Local)
    }
}

fn term(value: &Value, at: &Trail<'_>) -> Result<CheckedTerm, ValidationFailure> {
    super::flat_wire::check_refinement_term(value, at)?;
    CheckedTerm::decode_admitted(value).map_err(|_| invalid(at))
}

fn rows<T>(
    value: &Value,
    name: &str,
    at: &Trail<'_>,
    mut parse: impl FnMut(&Value, &Trail<'_>) -> Result<T, ValidationFailure>,
) -> Result<Vec<T>, ValidationFailure> {
    array(value, name, at)?
        .iter()
        .enumerate()
        .map(|(index, row)| parse(row, &at.key(name).index(index)))
        .collect()
}

fn parse_argument(
    value: &Value,
    at: &Trail<'_>,
) -> Result<RefinementStepArgument, ValidationFailure> {
    if value.get("term").and_then(Value::as_str) == Some("open") {
        shape(value, &["term"], &[], at)?;
        Ok(RefinementStepArgument::Open(RefinementOpen {
            term: "open".into(),
        }))
    } else {
        Ok(RefinementStepArgument::Term(Box::new(term(value, at)?)))
    }
}

#[ix_trace_rs::string_edge]
fn parse_target(value: &Value, at: &Trail<'_>) -> Result<RefinementStepTarget, ValidationFailure> {
    let kind = member(value, "kind", at)?
        .as_str()
        .ok_or_else(|| invalid(&at.key("kind")))?;
    match kind {
        "operation" => {
            shape(
                value,
                &["kind", "operation", "receiver", "arguments"],
                &[],
                at,
            )?;
            Ok(RefinementStepTarget::Operation {
                operation: key(member(value, "operation", at)?, &at.key("operation"))?,
                receiver: Box::new(term(member(value, "receiver", at)?, &at.key("receiver"))?),
                arguments: rows(value, "arguments", at, parse_argument)?,
            })
        }
        "protocol_step" => {
            shape(value, &["kind", "class", "arguments"], &["node"], at)?;
            Ok(RefinementStepTarget::ProtocolStep {
                class: text(member(value, "class", at)?, &at.key("class"))?,
                node: value
                    .get("node")
                    .map(|v| key(v, &at.key("node")))
                    .transpose()?,
                arguments: rows(value, "arguments", at, parse_argument)?,
            })
        }
        "stutter" => {
            shape(value, &["kind"], &[], at)?;
            Ok(RefinementStepTarget::Stutter)
        }
        "any" => {
            shape(value, &["kind"], &[], at)?;
            Ok(RefinementStepTarget::Any)
        }
        _ => Err(invalid(&at.key("kind"))),
    }
}

fn parse_concrete_class(
    value: &Value,
    at: &Trail<'_>,
) -> Result<RefinementConcreteClass, ValidationFailure> {
    if value.get("node").is_none() {
        return typed(value, at).map(RefinementConcreteClass::Memory);
    }
    shape(value, &["class", "node", "binder_types"], &[], at)?;
    Ok(RefinementConcreteClass::Declared {
        class: text(member(value, "class", at)?, &at.key("class"))?,
        node: node(member(value, "node", at)?, &at.key("node"))?,
        binder_types: rows(value, "binder_types", at, node)?,
    })
}

fn parse_abstract_class(
    value: &Value,
    at: &Trail<'_>,
) -> Result<RefinementAbstractClass, ValidationFailure> {
    if value.get("node").is_none() {
        return typed(value, at).map(RefinementAbstractClass::Memory);
    }
    shape(value, &["class", "node", "binder_types"], &[], at)?;
    Ok(RefinementAbstractClass::Declared {
        class: text(member(value, "class", at)?, &at.key("class"))?,
        node: key(member(value, "node", at)?, &at.key("node"))?,
        binder_types: rows(value, "binder_types", at, key)?,
    })
}

/// Decode a refinement after the closed wire checks, without passing a
/// structural node path through serde's recursive Value deserializer.
pub(super) fn decode_body(body: &Value, at: &Trail<'_>) -> Result<CheckedTerm, ValidationFailure> {
    check_body_header(body, at)?;
    let populations = rows(body, "populations", at, |value, at| {
        shape(value, &["abstract", "concrete"], &[], at)?;
        Ok(RefinementPopulation {
            r#abstract: key(member(value, "abstract", at)?, &at.key("abstract"))?,
            concrete: node(member(value, "concrete", at)?, &at.key("concrete"))?,
        })
    })?;
    let objects = rows(body, "objects", at, |value, at| {
        shape(
            value,
            &["abstract_type", "concrete_type", "fields"],
            &[],
            at,
        )?;
        Ok(RefinementObject {
            abstract_type: key(
                member(value, "abstract_type", at)?,
                &at.key("abstract_type"),
            )?,
            concrete_type: node(
                member(value, "concrete_type", at)?,
                &at.key("concrete_type"),
            )?,
            fields: rows(value, "fields", at, |value, at| {
                shape(value, &["name", "expression"], &[], at)?;
                Ok(RefinementField {
                    name: text(member(value, "name", at)?, &at.key("name"))?,
                    expression: term(member(value, "expression", at)?, &at.key("expression"))?,
                })
            })?,
        })
    })?;
    let steps = rows(body, "steps", at, |value, at| {
        shape(value, &["source_class", "target"], &["source_node"], at)?;
        Ok(RefinementStep {
            source_class: text(member(value, "source_class", at)?, &at.key("source_class"))?,
            source_node: value
                .get("source_node")
                .map(|v| node(v, &at.key("source_node")))
                .transpose()?,
            target: parse_target(member(value, "target", at)?, &at.key("target"))?,
        })
    })?;
    let histories = rows(body, "histories", at, |value, at| {
        shape(
            value,
            &["concrete_type", "name", "type", "initial", "updates"],
            &[],
            at,
        )?;
        Ok(RefinementHistory {
            concrete_type: node(
                member(value, "concrete_type", at)?,
                &at.key("concrete_type"),
            )?,
            name: text(member(value, "name", at)?, &at.key("name"))?,
            type_id: node(member(value, "type", at)?, &at.key("type"))?,
            initial: term(member(value, "initial", at)?, &at.key("initial"))?,
            updates: rows(value, "updates", at, |value, at| {
                shape(value, &["operation", "expression"], &[], at)?;
                Ok(RefinementHistoryUpdate {
                    operation: node(member(value, "operation", at)?, &at.key("operation"))?,
                    expression: term(member(value, "expression", at)?, &at.key("expression"))?,
                })
            })?,
        })
    })?;
    let assume = rows(body, "assume", at, |value, at| {
        shape(value, &["operation", "kind", "granularity"], &[], at)?;
        Ok(RefinementAssume {
            operation: node(member(value, "operation", at)?, &at.key("operation"))?,
            kind: typed(member(value, "kind", at)?, &at.key("kind"))?,
            granularity: typed(member(value, "granularity", at)?, &at.key("granularity"))?,
        })
    })?;
    let ensure = rows(body, "ensure", at, |value, at| {
        shape(value, &["operation", "kind", "granularity"], &[], at)?;
        Ok(RefinementEnsure {
            operation: key(member(value, "operation", at)?, &at.key("operation"))?,
            kind: typed(member(value, "kind", at)?, &at.key("kind"))?,
            granularity: typed(member(value, "granularity", at)?, &at.key("granularity"))?,
        })
    })?;
    let class_catalog = body
        .get("class_catalog")
        .map(|value| {
            let at = at.key("class_catalog");
            Ok(RefinementClassCatalog {
                concrete: rows(value, "concrete", &at, parse_concrete_class)?,
                r#abstract: rows(value, "abstract", &at, parse_abstract_class)?,
            })
        })
        .transpose()?;
    Ok(CheckedTerm::Refinement {
        abstract_key: Box::new(key(member(body, "abstract", at)?, &at.key("abstract"))?),
        profile: typed(member(body, "profile", at)?, &at.key("profile"))?,
        populations,
        objects,
        steps,
        histories,
        assume,
        ensure,
        protocol: body
            .get("protocol")
            .map(|v| key(v, &at.key("protocol")).map(Box::new))
            .transpose()?,
        abstract_internal: body
            .get("abstract_internal")
            .map(|_| rows(body, "abstract_internal", at, text))
            .transpose()?,
        class_catalog,
    })
}

/// Walk each actual semantic term in FR-375 row order. The special `open`
/// argument is not a semantic term and is excluded here.
fn row_terms(
    body: &Value,
    at: &Trail<'_>,
    mut visit: impl FnMut(&Value, &Trail<'_>) -> Result<(), ValidationFailure>,
) -> Result<(), ValidationFailure> {
    for (object_index, object) in array(body, "objects", at)?.iter().enumerate() {
        let objects_at = at.key("objects");
        let object_at = objects_at.index(object_index);
        for (field_index, field) in array(object, "fields", &object_at)?.iter().enumerate() {
            let fields_at = object_at.key("fields");
            let field_at = fields_at.index(field_index);
            visit(
                member(field, "expression", &field_at)?,
                &field_at.key("expression"),
            )?;
        }
    }
    for (step_index, step) in array(body, "steps", at)?.iter().enumerate() {
        let steps_at = at.key("steps");
        let step_at = steps_at.index(step_index);
        let target_at = step_at.key("target");
        let target = member(step, "target", &step_at)?;
        let kind = member(target, "kind", &target_at)?.as_str();
        if kind == Some("operation") {
            visit(
                member(target, "receiver", &target_at)?,
                &target_at.key("receiver"),
            )?;
        }
        if matches!(kind, Some("operation" | "protocol_step")) {
            for (argument_index, argument) in
                array(target, "arguments", &target_at)?.iter().enumerate()
            {
                if argument.get("term").and_then(Value::as_str) != Some("open") {
                    visit(argument, &target_at.key("arguments").index(argument_index))?;
                }
            }
        }
    }
    for (history_index, history) in array(body, "histories", at)?.iter().enumerate() {
        let histories_at = at.key("histories");
        let history_at = histories_at.index(history_index);
        visit(
            member(history, "initial", &history_at)?,
            &history_at.key("initial"),
        )?;
        for (update_index, update) in array(history, "updates", &history_at)?.iter().enumerate() {
            let updates_at = history_at.key("updates");
            let update_at = updates_at.index(update_index);
            visit(
                member(update, "expression", &update_at)?,
                &update_at.key("expression"),
            )?;
        }
    }
    Ok(())
}

/// Check the complete closed body before graph admission, including the
/// nested flat-term rule that applies to each row expression independently.
pub(super) fn check_wire(body: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    check_body_header(body, at)?;
    if let Some(catalog) = body.get("class_catalog") {
        check_catalog_wire(catalog, &at.key("class_catalog"))?;
    }
    let typed = decode_body(body, at)?;
    let CheckedTerm::Refinement {
        steps,
        histories,
        abstract_internal,
        protocol,
        ..
    } = typed
    else {
        return Err(invalid(at));
    };
    if abstract_internal
        .as_ref()
        .is_some_and(|list| list.is_empty() || protocol.is_none())
    {
        return Err(invalid(&at.key("abstract_internal")));
    }
    for (index, step) in steps.iter().enumerate() {
        let kind = class_kind(&step.source_class);
        if kind.is_none()
            || (kind == Some(RefinementClassKind::Memory)) == step.source_node.is_some()
        {
            return Err(invalid(&at.key("steps").index(index)));
        }
        match &step.target {
            RefinementStepTarget::Operation { arguments, .. } => {
                check_open_arguments(arguments, &at.key("steps").index(index).key("target"))?;
            }
            RefinementStepTarget::ProtocolStep {
                class,
                node,
                arguments,
            } => {
                let kind = class_kind(class);
                if kind.is_none() || (kind == Some(RefinementClassKind::Memory)) == node.is_some() {
                    return Err(invalid(&at.key("steps").index(index).key("target")));
                }
                check_open_arguments(arguments, &at.key("steps").index(index).key("target"))?;
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for (index, history) in histories.iter().enumerate() {
        if !matches!(
            history.initial,
            CheckedTerm::Literal { .. } | CheckedTerm::Reference { .. }
        ) {
            return Err(invalid(&at.key("histories").index(index).key("initial")));
        }
    }
    row_terms(body, at, |term, term_at| {
        super::flat_wire::check_refinement_term(term, term_at)
    })
}

/// Convert the outer raw object shape before typed semantic checks.
#[ix_trace_rs::string_edge]
fn check_body_header(body: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    let Some(object) = body.as_object() else {
        return Err(invalid(at));
    };
    let required = [
        "term",
        "abstract",
        "profile",
        "populations",
        "objects",
        "steps",
        "histories",
        "assume",
        "ensure",
    ];
    if object.get("term").and_then(Value::as_str) != Some("refinement")
        || required.iter().any(|name| !object.contains_key(*name))
        || object.keys().any(|name| {
            !required.contains(&name.as_str())
                && name != "protocol"
                && name != "abstract_internal"
                && name != "class_catalog"
        })
    {
        return Err(invalid(at));
    }
    Ok(())
}

#[ix_trace_rs::string_edge]
fn check_catalog_wire(value: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    let Some(object) = value.as_object() else {
        return Err(invalid(at));
    };
    if object.len() != 2 || !object.contains_key("concrete") || !object.contains_key("abstract") {
        return Err(invalid(at));
    }
    for side in ["concrete", "abstract"] {
        let side_at = at.key(side);
        for (index, entry) in array(value, side, at)?.iter().enumerate() {
            let entry_at = side_at.index(index);
            let Some(entry) = entry.as_object() else {
                return Err(invalid(&entry_at));
            };
            let Some(class) = entry.get("class").and_then(Value::as_str) else {
                return Err(invalid(&entry_at.key("class")));
            };
            let memory = class_kind(class) == Some(RefinementClassKind::Memory);
            if memory {
                let malformed = |at: Trail<'_>| {
                    ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        at.pointer(),
                    )
                };
                if !memory_class_kind(class) {
                    return Err(malformed(entry_at.key("class")));
                }
                let Some(model) = entry.get("memory_model") else {
                    return Err(malformed(entry_at.key("memory_model")));
                };
                let Ok(model) = serde_json::from_value::<RefinementMemoryModel>(model.clone())
                else {
                    return Err(malformed(entry_at.key("memory_model")));
                };
                if class == "memory flush" && model != RefinementMemoryModel::Tso {
                    return Err(malformed(entry_at.key("memory_model")));
                }
                if class == "memory drain"
                    && entry.get("drain").and_then(Value::as_str) != Some("store-buffer-flush")
                {
                    return Err(malformed(entry_at.key("drain")));
                }
                let allowed: &[&str] = if class == "memory drain" {
                    &["class", "memory_model", "drain"]
                } else {
                    &["class", "memory_model"]
                };
                if let Some(extra) = entry.keys().find(|key| !allowed.contains(&key.as_str())) {
                    return Err(malformed(entry_at.key(extra)));
                }
            } else if class.is_empty()
                || entry.len() != 3
                || !entry.contains_key("node")
                || !entry.contains_key("binder_types")
            {
                return Err(invalid(&entry_at));
            }
        }
    }
    Ok(())
}

fn check_open_arguments(
    arguments: &[RefinementStepArgument],
    at: &Trail<'_>,
) -> Result<(), ValidationFailure> {
    for (index, argument) in arguments.iter().enumerate() {
        if let RefinementStepArgument::Open(open) = argument {
            if open_term_kind(&open.term).is_none() {
                return Err(invalid(&at.key("arguments").index(index)));
            }
        }
    }
    Ok(())
}

/// Validate every row term and report its local node references to the graph.
/// The whole body has already passed `check_wire`.
pub(super) fn validate_body(
    body: &Value,
    at: &Trail<'_>,
    visitor: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let mut work = 1_u64;
    row_terms(body, at, |term, term_at| {
        work = work.saturating_add(validate_term(
            term,
            TermGrammar::V2,
            true,
            term_at,
            visitor,
        )?);
        Ok(())
    })?;
    Ok(work)
}

impl RefinementKey {
    fn node_id(&self) -> &CheckedNodeId {
        match self {
            Self::Local(node) => node,
            Self::External(key) => &key.node,
        }
    }

    fn local(&self) -> Option<&CheckedNodeId> {
        match self {
            Self::Local(node) => Some(node),
            Self::External(_) => None,
        }
    }
}

fn order_defect(position: usize, array: &str, index: usize) -> ValidationFailure {
    ValidationFailure::refused(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        node_at(position).key("body").key(array).index(index),
    )
}

/// Check the ordered FR-375 row arrays and their nested arrays in published order.
fn canonical_order(
    body: &CheckedTerm,
    position: usize,
    graph: &RefinementGraph<'_, '_>,
) -> Result<(), ValidationFailure> {
    let CheckedTerm::Refinement {
        populations,
        objects,
        histories,
        assume,
        ensure,
        abstract_internal,
        class_catalog,
        ..
    } = body
    else {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    for (index, pair) in populations.windows(2).enumerate() {
        if graph.resolved_key(&pair[0].r#abstract) >= graph.resolved_key(&pair[1].r#abstract) {
            return Err(order_defect(position, "populations", index + 1));
        }
    }
    for (index, object) in objects.iter().enumerate() {
        for (field_index, pair) in object.fields.windows(2).enumerate() {
            if pair[0].name >= pair[1].name {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position)
                        .key("body")
                        .key("objects")
                        .index(index)
                        .key("fields")
                        .index(field_index + 1),
                ));
            }
        }
    }
    for (index, pair) in objects.windows(2).enumerate() {
        if graph.resolved_key(&pair[0].abstract_type) >= graph.resolved_key(&pair[1].abstract_type)
        {
            return Err(order_defect(position, "objects", index + 1));
        }
    }
    for (index, history) in histories.iter().enumerate() {
        for (update_index, pair) in history.updates.windows(2).enumerate() {
            if pair[0].operation >= pair[1].operation {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position)
                        .key("body")
                        .key("histories")
                        .index(index)
                        .key("updates")
                        .index(update_index + 1),
                ));
            }
        }
    }
    for (index, pair) in histories.windows(2).enumerate() {
        if (
            super::structural_key(&pair[0].concrete_type, graph.index, graph.structural_keys),
            pair[0].name.as_ref(),
        ) >= (
            super::structural_key(&pair[1].concrete_type, graph.index, graph.structural_keys),
            pair[1].name.as_ref(),
        ) {
            return Err(order_defect(position, "histories", index + 1));
        }
    }
    for (index, pair) in assume.windows(2).enumerate() {
        fn kind(kind: RefinementFairnessKind) -> &'static str {
            match kind {
                RefinementFairnessKind::Weak => "weak",
                RefinementFairnessKind::Strong => "strong",
            }
        }
        fn granularity(granularity: RefinementGranularity) -> &'static str {
            match granularity {
                RefinementGranularity::Each => "each",
                RefinementGranularity::Whole => "whole",
            }
        }
        fn key<'a>(
            row: &'a RefinementAssume,
            graph: &'a RefinementGraph<'_, '_>,
        ) -> (&'a CheckedNodeId, &'static str, &'static str) {
            (
                super::structural_key(&row.operation, graph.index, graph.structural_keys),
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        if key(&pair[0], graph) >= key(&pair[1], graph) {
            return Err(order_defect(position, "assume", index + 1));
        }
    }
    for (index, pair) in ensure.windows(2).enumerate() {
        fn kind(kind: RefinementFairnessKind) -> &'static str {
            match kind {
                RefinementFairnessKind::Weak => "weak",
                RefinementFairnessKind::Strong => "strong",
            }
        }
        fn granularity(granularity: RefinementGranularity) -> &'static str {
            match granularity {
                RefinementGranularity::Each => "each",
                RefinementGranularity::Whole => "whole",
            }
        }
        fn key<'a>(
            row: &'a RefinementEnsure,
            graph: &'a RefinementGraph<'_, '_>,
        ) -> (&'a CheckedNodeId, &'static str, &'static str) {
            (
                graph.resolved_key(&row.operation),
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        if key(&pair[0], graph) >= key(&pair[1], graph) {
            return Err(order_defect(position, "ensure", index + 1));
        }
    }
    if let Some(internal) = abstract_internal {
        for (index, pair) in internal.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(order_defect(position, "abstract_internal", index + 1));
            }
        }
    }
    if let Some(catalog) = class_catalog {
        for (index, pair) in catalog.concrete.windows(2).enumerate() {
            if concrete_catalog_key(&pair[0]) >= concrete_catalog_key(&pair[1]) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position)
                        .key("body")
                        .key("class_catalog")
                        .key("concrete")
                        .index(index + 1),
                ));
            }
        }
        for (index, pair) in catalog.r#abstract.windows(2).enumerate() {
            if abstract_catalog_key(&pair[0]) >= abstract_catalog_key(&pair[1]) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position)
                        .key("body")
                        .key("class_catalog")
                        .key("abstract")
                        .index(index + 1),
                ));
            }
        }
    }
    Ok(())
}

pub(super) struct RefinementGraph<'a, 'e> {
    nodes: &'a [super::RawSemanticNodeV2],
    kinds: &'a [super::CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    structural_keys: &'a [std::sync::Arc<CheckedNodeId>],
    dependencies: &'a super::SuppliedDependencies<'e>,
    package_name: &'a str,
}

impl<'a, 'e> RefinementGraph<'a, 'e> {
    pub(super) fn new(
        nodes: &'a [super::RawSemanticNodeV2],
        kinds: &'a [super::CheckedNodeKind],
        terms: &'a [Option<CheckedTerm>],
        index: &'a BTreeMap<&'a CheckedNodeId, usize>,
        structural_keys: &'a [std::sync::Arc<CheckedNodeId>],
        dependencies: &'a super::SuppliedDependencies<'e>,
        package_name: &'a str,
    ) -> Self {
        Self {
            nodes,
            kinds,
            terms,
            index,
            structural_keys,
            dependencies,
            package_name,
        }
    }
}

impl RefinementGraph<'_, '_> {
    fn resolved_key<'a>(&'a self, key: &'a RefinementKey) -> &'a CheckedNodeId {
        match key {
            RefinementKey::Local(id) => super::structural_key(id, self.index, self.structural_keys),
            RefinementKey::External(external) => self
                .dependencies
                .selected_package(&external.package)
                .and_then(|package| package.structural_key(&external.node))
                .unwrap_or(&external.node),
        }
    }

    fn same_key(&self, actual: &RefinementKey, expected: &RefinementKey) -> bool {
        match (actual, expected) {
            (RefinementKey::Local(_), RefinementKey::Local(_)) => {
                self.resolved_key(actual) == self.resolved_key(expected)
            }
            (RefinementKey::External(actual_ref), RefinementKey::External(expected_ref)) => {
                actual_ref.package == expected_ref.package
                    && self.resolved_key(actual) == self.resolved_key(expected)
            }
            _ => false,
        }
    }

    fn local_term(&self, id: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.terms
            .get(*self.index.get(id)?)
            .and_then(Option::as_ref)
    }

    fn local_node(&self, id: &CheckedNodeId) -> Option<&super::RawSemanticNodeV2> {
        self.nodes.get(*self.index.get(id)?)
    }

    fn key_owner_identity(&self, key: &RefinementKey) -> Option<&str> {
        let owner = match key {
            RefinementKey::Local(id) => {
                let owner = self.local_node(id)?.owner_model.as_ref()?;
                self.local_node(owner)?.owner.as_ref()?
            }
            RefinementKey::External(external) => {
                let selected = self.dependencies.selected_package(&external.package)?;
                let owner = selected.node(&external.node)?.owner_model.as_ref()?;
                selected.node(owner)?.owner.as_ref()?
            }
        };
        match owner {
            super::CheckedNodeOwner::Model { identity, .. } => Some(identity),
            _ => None,
        }
    }

    fn key_anchor(&self, key: &RefinementKey) -> Option<(CheckedNodeId, Box<str>)> {
        let term = match key {
            RefinementKey::Local(id) => self.local_term(id)?,
            RefinementKey::External(external) => self
                .dependencies
                .selected_package(&external.package)?
                .term(&external.node)?,
        };
        let anchor = super::structural::typed_anchor_body(term)?;
        Some((anchor.context, Box::from(anchor.operation)))
    }

    fn local_kind(
        &self,
        id: &CheckedNodeId,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<super::CheckedNodeKind, ValidationFailure> {
        let Some(&position) = self.index.get(id) else {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::MissingDeclaration,
                at,
                Some(CheckedPackageRefusalCause::MissingName),
                locus.clone(),
            ));
        };
        self.kinds.get(position).copied().ok_or_else(|| {
            ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, at)
        })
    }

    fn key_kind(
        &self,
        key: &RefinementKey,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<super::CheckedNodeKind, ValidationFailure> {
        match key {
            RefinementKey::Local(id) => self.local_kind(id, at, locus),
            RefinementKey::External(key) => {
                if key.package.as_ref() == self.package_name {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at.key("package"),
                        None,
                        locus.clone(),
                    ));
                }
                let selected = self
                    .dependencies
                    .selected_package(&key.package)
                    .ok_or_else(|| {
                        ValidationFailure::refused_at(
                            CheckedPackageRefusalCode::MissingDeclaration,
                            at.clone().key("package"),
                            Some(CheckedPackageRefusalCause::MissingSelection),
                            locus.clone(),
                        )
                    })?;
                selected.node_kind(&key.node).ok_or_else(|| {
                    ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::MissingDeclaration,
                        at,
                        Some(CheckedPackageRefusalCause::MissingName),
                        locus.clone(),
                    )
                })
            }
        }
    }

    fn require_kind(
        &self,
        key: &RefinementKey,
        expected: super::CheckedNodeKind,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        let actual = self.key_kind(key, at.clone(), locus)?;
        if actual == expected {
            Ok(())
        } else {
            Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                at,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                locus.clone(),
            ))
        }
    }

    fn require_local_kind(
        &self,
        id: &CheckedNodeId,
        expected: super::CheckedNodeKind,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        self.require_kind(&RefinementKey::Local(id.clone()), expected, at, locus)
    }

    fn require_anchor_subject(
        &self,
        key: &RefinementKey,
        subject: &RefinementKey,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<Box<str>, ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner, ModelForm};
        self.require_kind(
            key,
            K::State(super::StateForm::OperationAnchor),
            at.clone(),
            locus,
        )?;
        let Some((context, operation)) = self.key_anchor(key) else {
            return Err(wrong_binding(at, locus));
        };
        // An anchor names its declaring type, while `subject` names the selected model root.
        let (declaring_kind, declaring_identity, selected_identity) = match (key, subject) {
            (RefinementKey::Local(_), RefinementKey::Local(root)) => {
                let kind = self.local_kind(&context, at.clone(), locus)?;
                let declaring = self
                    .local_node(&context)
                    .and_then(|node| match &node.owner {
                        Some(CheckedNodeOwner::Model { identity, .. }) => Some(identity.as_ref()),
                        _ => None,
                    });
                let selected = self.local_node(root).and_then(|node| match &node.owner {
                    Some(CheckedNodeOwner::ModelRoot { identity, .. }) => Some(identity.as_ref()),
                    _ => None,
                });
                (kind, declaring, selected)
            }
            (RefinementKey::External(anchor), RefinementKey::External(root))
                if anchor.package == root.package =>
            {
                let Some(selected) = self.dependencies.selected_package(&anchor.package) else {
                    return Err(wrong_binding(at, locus));
                };
                let Some(kind) = selected.node_kind(&context) else {
                    return Err(wrong_binding(at, locus));
                };
                let declaring = selected.node(&context).and_then(|node| match &node.owner {
                    Some(CheckedNodeOwner::Model { identity, .. }) => Some(identity.as_ref()),
                    _ => None,
                });
                let selected = selected
                    .node(&root.node)
                    .and_then(|node| match &node.owner {
                        Some(CheckedNodeOwner::ModelRoot { identity, .. }) => {
                            Some(identity.as_ref())
                        }
                        _ => None,
                    });
                (kind, declaring, selected)
            }
            _ => return Err(wrong_binding(at, locus)),
        };
        if !matches!(
            declaring_kind,
            K::Model(ModelForm::ObjectType | ModelForm::Process | ModelForm::StateMachine)
        ) || declaring_identity.is_none()
            || declaring_identity != selected_identity
        {
            return Err(wrong_binding(at, locus));
        }
        Ok(operation)
    }

    fn require_concrete_step_anchor(
        &self,
        source: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<Box<str>, ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner, ModelForm};
        self.require_local_kind(
            source,
            K::State(super::StateForm::OperationAnchor),
            at.clone(),
            locus,
        )?;
        let Some((context, operation)) = self.key_anchor(&RefinementKey::Local(source.clone()))
        else {
            return Err(wrong_binding(at, locus));
        };
        let declaring_kind = self
            .index
            .get(&context)
            .and_then(|&index| self.kinds.get(index));
        if !matches!(
            declaring_kind,
            Some(K::Model(
                ModelForm::ObjectType | ModelForm::Process | ModelForm::StateMachine
            ))
        ) {
            return Err(wrong_binding(at, locus));
        }
        let selected_identity = self.local_node(selected_root).and_then(|root| {
            if let Some(CheckedNodeOwner::ModelRoot { identity, .. }) = &root.owner {
                Some(identity.as_ref())
            } else {
                None
            }
        });
        let declaring_identity = self.local_node(&context).and_then(|declaration| {
            if let Some(CheckedNodeOwner::Model { identity, .. }) = &declaration.owner {
                Some(identity.as_ref())
            } else {
                None
            }
        });
        if selected_identity.is_none() || selected_identity != declaring_identity {
            return Err(wrong_binding(at, locus));
        }
        Ok(operation)
    }

    fn require_concrete_owner(
        &self,
        declaration: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        use super::CheckedNodeOwner;
        let selected = self
            .local_node(selected_root)
            .and_then(|node| match &node.owner {
                Some(CheckedNodeOwner::ModelRoot { identity, .. }) => Some(identity),
                _ => None,
            });
        let declared = self
            .local_node(declaration)
            .and_then(|node| match &node.owner {
                Some(CheckedNodeOwner::Model { identity, .. }) => Some(identity),
                _ => None,
            });
        if selected.is_some() && selected == declared {
            Ok(())
        } else {
            Err(wrong_binding(at, locus))
        }
    }

    fn require_concrete_class_source(
        &self,
        source: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: JsonPointer,
        locus: &CheckedNodeId,
        meter: &mut super::WorkMeter,
    ) -> Result<(), ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner};
        let kind = self.local_kind(source, at.clone(), locus)?;
        if kind == K::State(super::StateForm::OperationAnchor) {
            self.require_concrete_step_anchor(source, selected_root, at, locus)?;
            return Ok(());
        }
        if matches!(kind, K::Model(_)) {
            return self.require_concrete_owner(source, selected_root, at, locus);
        }
        let selected_identity = self.local_node(selected_root).and_then(|root| {
            if let Some(CheckedNodeOwner::ModelRoot { identity, .. }) = &root.owner {
                Some(identity.as_ref())
            } else {
                None
            }
        });
        let clause = if kind == K::Protocol(super::ProtocolForm::ProtocolClause) {
            Some(source)
        } else if matches!(kind, K::Protocol(_)) {
            let name = self
                .local_node(source)
                .and_then(|node| node.declaration.as_ref());
            let mut closest = None;
            for &position in self.index.values() {
                meter.charge(1, || at.clone())?;
                let candidate = &self.nodes[position];
                if self.kinds[position] != K::Protocol(super::ProtocolForm::ProtocolClause) {
                    continue;
                }
                let Some(path) = candidate
                    .declaration
                    .as_ref()
                    .map(|decl| &decl.qualified_name)
                else {
                    continue;
                };
                if name.is_some_and(|name| protocol_clause_prefix(&name.qualified_name, path))
                    && closest.is_none_or(|(_, length)| path.len() >= length)
                {
                    closest = Some((&candidate.node_id, path.len()));
                }
            }
            closest.map(|(id, _)| id)
        } else {
            None
        };
        // A malformed clause owner has its own earlier protocol-body refusal.
        // Only compare an otherwise valid concrete class against the selected
        // root here, so that this row check does not mask that owner defect.
        let Some(clause) = clause else {
            return Err(wrong_binding(at, locus));
        };
        let valid_clause_owner = self
            .local_node(clause)
            .and_then(|node| node.owner_model.as_ref());
        if !valid_clause_owner.is_some_and(|id| {
            self.index.get(id).is_some_and(|&position| {
                self.kinds[position] == K::Model(super::ModelForm::ObjectType)
            })
        }) {
            return Ok(());
        }
        let declared_identity = self.key_owner_identity(&RefinementKey::Local(clause.clone()));
        if selected_identity.is_some() && selected_identity == declared_identity {
            Ok(())
        } else {
            Err(wrong_binding(at, locus))
        }
    }
}

fn wrong_binding(at: JsonPointer, locus: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::InvalidModelBinding,
        at,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        locus.clone(),
    )
}

fn missing_class(at: JsonPointer, locus: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::MissingDeclaration,
        at,
        Some(CheckedPackageRefusalCause::MissingName),
        locus.clone(),
    )
}

fn class_node_kind(kind: super::CheckedNodeKind) -> bool {
    matches!(
        kind,
        super::CheckedNodeKind::Model(
            super::ModelForm::ObjectType
                | super::ModelForm::StateMachine
                | super::ModelForm::SystemsInterface
        ) | super::CheckedNodeKind::State(super::StateForm::OperationAnchor)
            | super::CheckedNodeKind::Protocol(_)
    )
}

fn class_type_kind(kind: super::CheckedNodeKind) -> bool {
    matches!(
        kind,
        super::CheckedNodeKind::ScalarType(_)
            | super::CheckedNodeKind::CompositeType(_)
            | super::CheckedNodeKind::BoundedDomain(_)
            | super::CheckedNodeKind::Model(_)
    )
}

fn validate_catalog(
    body: &CheckedTerm,
    position: usize,
    graph: &RefinementGraph<'_, '_>,
    concrete_model: &CheckedNodeId,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    use super::CheckedNodeOwner;
    let CheckedTerm::Refinement {
        protocol,
        class_catalog,
        steps,
        ..
    } = body
    else {
        return Ok(());
    };
    let root = node_at(position).key("body").key("class_catalog");
    let mut requires_catalog = protocol.is_some();
    for step in steps {
        meter.charge(1, || root.clone())?;
        requires_catalog |= step.source_node.is_none()
            || matches!(step.target, RefinementStepTarget::ProtocolStep { .. });
    }
    let selected_identity = graph.local_node(concrete_model).and_then(|model| {
        if let Some(CheckedNodeOwner::ModelRoot { identity, .. }) = &model.owner {
            Some(identity)
        } else {
            None
        }
    });
    for &source_position in graph.index.values() {
        meter.charge(1, || root.clone())?;
        if graph.kinds.get(source_position)
            != Some(&super::CheckedNodeKind::Protocol(
                super::ProtocolForm::ProtocolClause,
            ))
        {
            continue;
        }
        let same_selected_model = graph.nodes[source_position]
            .owner_model
            .as_ref()
            .and_then(|owner_model| graph.index.get(owner_model).copied())
            .filter(|&owner_position| {
                graph.kinds[owner_position]
                    == super::CheckedNodeKind::Model(super::ModelForm::ObjectType)
            })
            .and_then(|owner_position| graph.nodes[owner_position].owner.as_ref())
            .zip(selected_identity)
            .is_some_and(|(declaration_owner, selected)| {
                matches!(declaration_owner,
                    CheckedNodeOwner::Model { identity, .. } if identity == selected)
            });
        if same_selected_model {
            requires_catalog = true;
        }
    }
    if requires_catalog && class_catalog.is_none() {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            root,
        ));
    }
    let Some(catalog) = class_catalog else {
        return Ok(());
    };
    if protocol.is_none() && catalog.concrete.is_empty() && catalog.r#abstract.is_empty() {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            root,
        ));
    }
    for (index, entry) in catalog.concrete.iter().enumerate() {
        let at = root.clone().key("concrete").index(index);
        meter.charge(1, || at.clone())?;
        if let RefinementConcreteClass::Declared {
            node, binder_types, ..
        } = entry
        {
            let node_at = at.clone().key("node");
            if !class_node_kind(graph.local_kind(node, node_at.clone(), locus)?) {
                return Err(wrong_binding(node_at, locus));
            }
            for (binder_index, binder) in binder_types.iter().enumerate() {
                let binder_at = at.clone().key("binder_types").index(binder_index);
                meter.charge(1, || binder_at.clone())?;
                if !class_type_kind(graph.local_kind(binder, binder_at.clone(), locus)?) {
                    return Err(wrong_binding(binder_at, locus));
                }
            }
        }
    }
    for (index, entry) in catalog.r#abstract.iter().enumerate() {
        let at = root.clone().key("abstract").index(index);
        meter.charge(1, || at.clone())?;
        if let RefinementAbstractClass::Declared {
            node, binder_types, ..
        } = entry
        {
            let node_at = at.clone().key("node");
            if !class_node_kind(graph.key_kind(node, node_at.clone(), locus)?) {
                return Err(wrong_binding(node_at, locus));
            }
            for (binder_index, binder) in binder_types.iter().enumerate() {
                let binder_at = at.clone().key("binder_types").index(binder_index);
                meter.charge(1, || binder_at.clone())?;
                if !class_type_kind(graph.key_kind(binder, binder_at.clone(), locus)?) {
                    return Err(wrong_binding(binder_at, locus));
                }
            }
        }
    }
    Ok(())
}

fn validate_rows(
    node: &super::RawSemanticNodeV2,
    body: &CheckedTerm,
    position: usize,
    graph: &RefinementGraph<'_, '_>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    use super::{CheckedNodeKind as K, ModelForm, RelationForm};
    let CheckedTerm::Refinement {
        abstract_key,
        populations,
        objects,
        steps,
        histories,
        assume,
        ensure,
        class_catalog,
        ..
    } = body
    else {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    let root = node_at(position).key("body");
    let locus = &node.node_id;
    for (index, row) in populations.iter().enumerate() {
        let at = root.clone().key("populations").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_kind(
            &row.r#abstract,
            K::Relation(RelationForm::Population),
            at.clone().key("abstract"),
            locus,
        )?;
        graph.require_local_kind(
            &row.concrete,
            K::Relation(RelationForm::Population),
            at.clone().key("concrete"),
            locus,
        )?;
        graph.require_concrete_owner(
            &row.concrete,
            &node.semantic_type,
            at.key("concrete"),
            locus,
        )?;
    }
    for (index, row) in objects.iter().enumerate() {
        let at = root.clone().key("objects").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_kind(
            &row.abstract_type,
            K::Model(ModelForm::ObjectType),
            at.clone().key("abstract_type"),
            locus,
        )?;
        graph.require_local_kind(
            &row.concrete_type,
            K::Model(ModelForm::ObjectType),
            at.clone().key("concrete_type"),
            locus,
        )?;
        graph.require_concrete_owner(
            &row.concrete_type,
            &node.semantic_type,
            at.clone().key("concrete_type"),
            locus,
        )?;
        let known = abstract_fields(&row.abstract_type, owners, graph.dependencies);
        for (field_index, field) in row.fields.iter().enumerate() {
            let field_at = at.clone().key("fields").index(field_index);
            meter.charge(1, || field_at.clone())?;
            if !known
                .as_ref()
                .is_some_and(|names| names.contains(&field.name))
            {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::MissingDeclaration,
                    field_at.key("name"),
                    Some(CheckedPackageRefusalCause::MissingName),
                    locus.clone(),
                ));
            }
        }
    }
    for (index, row) in steps.iter().enumerate() {
        let at = root.clone().key("steps").index(index);
        meter.charge(1, || at.clone())?;
        if let Some(catalog) = class_catalog {
            let Some(class) = catalog
                .concrete
                .iter()
                .find(|entry| entry.class() == row.source_class.as_ref())
            else {
                return Err(missing_class(at.clone().key("source_class"), locus));
            };
            let matches_node = match (class, &row.source_node) {
                (RefinementConcreteClass::Declared { node, .. }, Some(source)) => node == source,
                (RefinementConcreteClass::Memory(_), None) => true,
                _ => false,
            };
            if !matches_node {
                return Err(wrong_binding(at.clone().key("source_node"), locus));
            }
            if let Some(source) = &row.source_node {
                graph.require_concrete_class_source(
                    source,
                    &node.semantic_type,
                    at.clone().key("source_node"),
                    locus,
                    meter,
                )?;
            }
        } else if let Some(source) = &row.source_node {
            let operation = graph.require_concrete_step_anchor(
                source,
                &node.semantic_type,
                at.clone().key("source_node"),
                locus,
            )?;
            if row.source_class.as_ref() != operation.as_ref() {
                return Err(wrong_binding(at.clone().key("source_class"), locus));
            }
        } else {
            return Err(missing_class(at.clone().key("source_class"), locus));
        }
        let target_at = at.key("target");
        match &row.target {
            RefinementStepTarget::Operation { operation, .. } => {
                graph.require_anchor_subject(
                    operation,
                    abstract_key,
                    target_at.key("operation"),
                    locus,
                )?;
            }
            RefinementStepTarget::ProtocolStep {
                class,
                node,
                arguments,
            } => {
                let selected = class_catalog
                    .as_ref()
                    .and_then(|catalog| {
                        catalog
                            .r#abstract
                            .iter()
                            .find(|entry| entry.class() == class.as_ref())
                    })
                    .ok_or_else(|| missing_class(target_at.clone().key("class"), locus))?;
                let (matches_node, binders) = match selected {
                    RefinementAbstractClass::Declared {
                        node: declared,
                        binder_types,
                        ..
                    } => (
                        node.as_ref() == Some(declared),
                        Some(binder_types.as_slice()),
                    ),
                    RefinementAbstractClass::Memory(_) => (node.is_none(), None),
                };
                if !matches_node {
                    return Err(wrong_binding(target_at.clone().key("node"), locus));
                }
                let expected = binders.map_or(0, <[_]>::len);
                if !arguments.is_empty() && arguments.len() != expected {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::IllTyped,
                        target_at.key("arguments"),
                        Some(CheckedPackageRefusalCause::OperatorIneligible),
                        locus.clone(),
                    ));
                }
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for (index, row) in histories.iter().enumerate() {
        let at = root.clone().key("histories").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_local_kind(
            &row.concrete_type,
            K::Model(ModelForm::ObjectType),
            at.clone().key("concrete_type"),
            locus,
        )?;
        graph.require_concrete_owner(
            &row.concrete_type,
            &node.semantic_type,
            at.clone().key("concrete_type"),
            locus,
        )?;
        graph.local_kind(&row.type_id, at.clone().key("type"), locus)?;
        if let CheckedTerm::Reference { target } = &row.initial {
            graph.require_local_kind(
                target,
                K::Value(super::ValueForm::Literal),
                at.clone().key("initial").key("target"),
                locus,
            )?;
        }
        for (update_index, update) in row.updates.iter().enumerate() {
            let update_at = at.clone().key("updates").index(update_index);
            meter.charge(1, || update_at.clone())?;
            graph.require_concrete_step_anchor(
                &update.operation,
                &node.semantic_type,
                update_at.key("operation"),
                locus,
            )?;
        }
    }
    for (index, row) in assume.iter().enumerate() {
        let at = root.clone().key("assume").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_concrete_step_anchor(
            &row.operation,
            &node.semantic_type,
            at.key("operation"),
            locus,
        )?;
    }
    for (index, row) in ensure.iter().enumerate() {
        let at = root.clone().key("ensure").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_anchor_subject(&row.operation, abstract_key, at.key("operation"), locus)?;
    }
    Ok(())
}

fn check_headers(
    node: &super::RawSemanticNodeV2,
    body: &CheckedTerm,
    position: usize,
    graph: &RefinementGraph<'_, '_>,
    lock: &super::CheckedPackageLockV2,
) -> Result<(), ValidationFailure> {
    use super::{CheckedNodeKind as K, ModelForm, ProtocolForm};
    let CheckedTerm::Refinement {
        abstract_key,
        protocol,
        profile,
        ..
    } = body
    else {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    let root = node_at(position);
    let locus = &node.node_id;
    graph.require_local_kind(
        &node.semantic_type,
        K::Model(ModelForm::ModelRoot),
        root.clone().key("semantic_type"),
        locus,
    )?;
    graph.require_kind(
        abstract_key,
        K::Model(ModelForm::ModelRoot),
        root.clone().key("body").key("abstract"),
        locus,
    )?;
    if let Some(protocol) = protocol {
        graph.require_kind(
            protocol,
            K::Protocol(ProtocolForm::ProtocolClause),
            root.clone().key("body").key("protocol"),
            locus,
        )?;
        let root_identity = match abstract_key.as_ref() {
            RefinementKey::Local(id) => graph.local_node(id).and_then(|node| node.owner.as_ref()),
            RefinementKey::External(key) => graph
                .dependencies
                .selected_package(&key.package)
                .and_then(|selected| selected.node(&key.node))
                .and_then(|node| node.owner.as_ref()),
        };
        let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = root_identity else {
            return Err(wrong_binding(
                root.clone().key("body").key("abstract"),
                locus,
            ));
        };
        if graph.key_owner_identity(protocol) != Some(identity.as_ref()) {
            return Err(wrong_binding(
                root.clone().key("body").key("protocol"),
                locus,
            ));
        }
    }
    let selected = lock.profile_selections.iter().any(|selection| {
        selection.role == super::CheckedSelectionRole::TemporalProfile
            && selection.definition == *profile
            && super::TemporalProfile::from_wire(&profile.identity)
                == Some(super::TemporalProfile::InfiniteTrace)
    });
    if !selected {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            root.key("body").key("profile"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            locus.clone(),
        ));
    }
    Ok(())
}

fn abstract_fields(
    key: &RefinementKey,
    owners: &super::model_members::ModelOwners<'_>,
    dependencies: &super::SuppliedDependencies<'_>,
) -> Option<BTreeSet<Box<str>>> {
    match key {
        RefinementKey::Local(id) => owners.refinement_object_fields(id),
        RefinementKey::External(external) => dependencies
            .selected_package(&external.package)?
            .models
            .refinement_object_fields(&external.node),
    }
}

fn proof_eligible(
    body: &CheckedTerm,
    owners: &super::model_members::ModelOwners<'_>,
    dependencies: &super::SuppliedDependencies<'_>,
    meter: &mut super::WorkMeter,
    position: usize,
) -> Result<Option<usize>, ValidationFailure> {
    let CheckedTerm::Refinement {
        objects,
        steps,
        histories,
        ..
    } = body
    else {
        return Ok(None);
    };
    if !histories.is_empty()
        || steps
            .iter()
            .any(|step| matches!(step.target, RefinementStepTarget::Any))
    {
        return Ok(Some(0));
    }
    for (index, object) in objects.iter().enumerate() {
        let Some(fields) = abstract_fields(&object.abstract_type, owners, dependencies) else {
            // A missing or wrong-kind key receives its ordered meaning refusal later.
            return Ok(None);
        };
        meter.charge(u64::try_from(fields.len()).unwrap_or(u64::MAX), || {
            node_at(position).key("body").key("objects").index(index)
        })?;
        let mapped: BTreeSet<&str> = object
            .fields
            .iter()
            .map(|field| field.name.as_ref())
            .collect();
        if fields.iter().any(|field| !mapped.contains(field.as_ref())) {
            return Ok(Some(0));
        }
    }
    Ok(Some(steps.len().saturating_add(1)))
}

/// Check the closed, identity-excluded row-region carrier before occurrences.
pub(super) fn validate_step_rows(
    nodes: &[super::RawSemanticNodeV2],
    kinds: &[super::CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    rows: Option<&[CheckedRefinementStepRowRegion]>,
    locked_sources: &[CheckedSourceRef],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = BTreeMap::new();
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != super::CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
        if let Some(CheckedTerm::Refinement { steps, .. }) = terms[position].as_ref() {
            meter.charge(u64::try_from(steps.len()).unwrap_or(u64::MAX), || {
                node_at(position).key("body").key("steps")
            })?;
            expected.insert(node.node_id.clone(), steps.len());
        }
    }
    let count = expected
        .values()
        .try_fold(0usize, |total, len| total.checked_add(*len))
        .ok_or_else(|| invalid_step_rows(None))?;
    if count == 0 && rows.is_none() {
        return Ok(());
    }
    if count == 0 {
        return Err(invalid_step_rows(None));
    }
    let Some(rows) = rows else {
        return Err(invalid_step_rows(None));
    };
    meter.charge(
        u64::try_from(locked_sources.len()).unwrap_or(u64::MAX),
        || JsonPointer::root().key("lock").key("sources"),
    )?;
    let locked_sources = locked_sources.iter().collect::<BTreeSet<_>>();
    let mut previous: Option<(&CheckedNodeId, u64)> = None;
    for (index, row) in rows.iter().enumerate() {
        meter.charge(1, || {
            JsonPointer::root().key("refinement_step_rows").index(index)
        })?;
        let in_range = expected
            .get(&row.node_id)
            .is_some_and(|len| usize::try_from(row.ordinal).is_ok_and(|ordinal| ordinal < *len));
        let ordered =
            previous.is_none_or(|(node, ordinal)| (node, ordinal) < (&row.node_id, row.ordinal));
        if !in_range || !ordered {
            return Err(invalid_step_rows(Some(index)));
        }
        if !locked_sources.contains(&row.region.source) {
            return Err(invalid_step_row_region(index, true));
        }
        if row.region.start >= row.region.end {
            return Err(invalid_step_row_region(index, false));
        }
        previous = Some((&row.node_id, row.ordinal));
    }
    if rows.len() != count {
        return Err(invalid_step_rows(None));
    }
    Ok(())
}

fn invalid_step_row_region(index: usize, source: bool) -> ValidationFailure {
    let region = JsonPointer::root()
        .key("refinement_step_rows")
        .index(index)
        .key("region");
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        if source { region.key("source") } else { region },
        CheckedPackageRefusalCause::InvalidValue,
    )
}

fn invalid_step_rows(index: Option<usize>) -> ValidationFailure {
    let path = JsonPointer::root().key("refinement_step_rows");
    let path = index.map_or(path.clone(), |index| path.index(index));
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}

/// Check the source-map occurrences against declaration and step-row regions.
pub(super) fn validate_occurrences(
    nodes: &[super::RawSemanticNodeV2],
    kinds: &[super::CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    source_map: &[CheckedSourceMapEntry],
    rows: Option<&[CheckedRefinementStepRowRegion]>,
    owners: &super::model_members::ModelOwners<'_>,
    dependencies: &super::SuppliedDependencies<'_>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut row_regions = BTreeMap::new();
    if let Some(rows) = rows {
        for (index, row) in rows.iter().enumerate() {
            meter.charge(1, || {
                JsonPointer::root().key("refinement_step_rows").index(index)
            })?;
            row_regions.insert((&row.node_id, row.ordinal), &row.region);
        }
    }
    let mut declarations = BTreeMap::new();
    for (index, entry) in source_map.iter().enumerate() {
        meter.charge(1, || JsonPointer::root().key("source_map").index(index))?;
        if entry.role == CheckedOccurrenceRole::Declaration && entry.ordinal == 0 {
            declarations.insert(&entry.node_id, entry.regions.as_slice());
        }
    }
    let mut refinement_positions: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter_map(|(position, _)| {
            (kinds[position] == super::CheckedNodeKind::Relation(super::RelationForm::Refinement))
                .then_some(position)
        })
        .collect();
    refinement_positions
        .sort_unstable_by(|&left, &right| structural_keys[left].cmp(&structural_keys[right]));
    for position in refinement_positions {
        let node = &nodes[position];
        let Some(body) = terms[position].as_ref() else {
            continue;
        };
        let Some(generated_count) = proof_eligible(body, owners, dependencies, meter, position)?
        else {
            continue;
        };
        let declaration = declarations.get(&node.node_id);
        let mut present = BTreeSet::new();
        for (entry_index, entry) in source_map.iter().enumerate() {
            meter.charge(1, || {
                JsonPointer::root().key("source_map").index(entry_index)
            })?;
            if entry.node_id != node.node_id {
                continue;
            }
            let valid = match entry.role {
                CheckedOccurrenceRole::Declaration => entry.ordinal == 0,
                CheckedOccurrenceRole::Generated => {
                    usize::try_from(entry.ordinal).is_ok_and(|ordinal| ordinal < generated_count)
                }
                _ => false,
            };
            if !valid
                || entry.regions.len() != 1
                || (entry.role == CheckedOccurrenceRole::Generated
                    && entry.ordinal == 0
                    && declaration.is_some_and(|regions| entry.regions != *regions))
                || (entry.role == CheckedOccurrenceRole::Generated
                    && entry.ordinal > 0
                    && row_regions
                        .get(&(&node.node_id, entry.ordinal - 1))
                        .is_some_and(|region| entry.regions.first() != Some(*region)))
            {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    JsonPointer::root().key("source_map").index(entry_index),
                    Some(CheckedPackageRefusalCause::InvalidValue),
                    node.node_id.clone(),
                ));
            }
            present.insert((entry.role.clone(), entry.ordinal));
        }
        if !present.contains(&(CheckedOccurrenceRole::Declaration, 0))
            || (0..generated_count).any(|ordinal| {
                u64::try_from(ordinal).map_or(true, |ordinal| {
                    !present.contains(&(CheckedOccurrenceRole::Generated, ordinal))
                })
            })
        {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                JsonPointer::root().key("source_map"),
                Some(CheckedPackageRefusalCause::InvalidValue),
                node.node_id.clone(),
            ));
        }
    }
    Ok(())
}

/// The dedicated refinement body step, in retained node-key order.
pub(super) fn validate_refinements(
    graph: RefinementGraph<'_, '_>,
    lock: &super::CheckedPackageLockV2,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    for &position in graph.index.values() {
        if graph.kinds.get(position)
            != Some(&super::CheckedNodeKind::Relation(
                super::RelationForm::Refinement,
            ))
        {
            continue;
        }
        let node = &graph.nodes[position];
        let body = graph
            .terms
            .get(position)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position).key("body"),
                )
            })?;
        meter.charge(1, || node_at(position))?;
        canonical_order(body, position, &graph)?;
        check_headers(node, body, position, &graph, lock)?;
        validate_catalog(
            body,
            position,
            &graph,
            &node.semantic_type,
            &node.node_id,
            meter,
        )?;
        validate_rows(node, body, position, &graph, owners, meter)?;
    }
    Ok(())
}

/// Join declaration-backed catalog entries to the admitted protocol graph.
/// This runs after every protocol body, so an invalid body keeps its earlier
/// refusal and no class is inferred from an unadmitted protocol declaration.
pub(super) fn validate_catalog_graph(
    graph: RefinementGraph<'_, '_>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    for &position in graph.index.values() {
        if graph.kinds[position]
            != super::CheckedNodeKind::Relation(super::RelationForm::Refinement)
        {
            continue;
        }
        let Some(CheckedTerm::Refinement {
            abstract_key,
            protocol,
            class_catalog: Some(catalog),
            ..
        }) = graph.terms[position].as_ref()
        else {
            continue;
        };
        let root = node_at(position).key("body").key("class_catalog");
        let local = local_protocol_nodes(&graph);
        let mut concrete = protocol_classes(
            &local,
            &graph.nodes[position].semantic_type,
            None,
            None,
            CatalogSignatures::Local(owners),
            &root,
            meter,
        )?;
        if let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = graph
            .local_node(&graph.nodes[position].semantic_type)
            .and_then(|node| node.owner.as_ref())
        {
            for operation in owners.operation_classes(identity, meter)? {
                concrete.push(DeclaredGraphClass {
                    class: format!("{}::{}", operation.object, operation.operation),
                    node: RefinementKey::Local(operation.node),
                    binder_types: operation.binder_types.map_or(
                        BinderSignature::Unspecified,
                        |types| {
                            BinderSignature::Known(
                                types.into_iter().map(RefinementKey::Local).collect(),
                            )
                        },
                    ),
                });
            }
            concrete.sort_unstable_by(|left, right| left.class.cmp(&right.class));
        }
        compare_declared_classes(
            &catalog.concrete,
            &concrete,
            &graph,
            root.clone().key("concrete"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        let concrete_memory = protocol_memory_models(
            &local,
            &graph.nodes[position].semantic_type,
            None,
            &root,
            meter,
        )?;
        compare_concrete_memory(
            &catalog.concrete,
            &concrete_memory,
            root.clone().key("concrete"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        let (abstract_classes, abstract_memory) = match protocol.as_deref() {
            Some(RefinementKey::Local(clause)) => match abstract_key.as_ref() {
                RefinementKey::Local(model) => (
                    protocol_classes(
                        &local,
                        model,
                        Some(clause),
                        None,
                        CatalogSignatures::Local(owners),
                        &root,
                        meter,
                    )?,
                    protocol_memory_models(&local, model, Some(clause), &root, meter)?,
                ),
                RefinementKey::External(_) => (Vec::new(), BTreeSet::new()),
            },
            Some(RefinementKey::External(clause)) => match abstract_key.as_ref() {
                RefinementKey::External(model) if model.package == clause.package => {
                    let selected = graph
                        .dependencies
                        .selected_package(&clause.package)
                        .ok_or_else(|| {
                            ValidationFailure::refused_at(
                                CheckedPackageRefusalCode::InvalidModelBinding,
                                node_at(position).key("body").key("protocol"),
                                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                                graph.nodes[position].node_id.clone(),
                            )
                        })?;
                    let dependency = dependency_protocol_nodes(selected);
                    (
                        protocol_classes(
                            &dependency,
                            &model.node,
                            Some(&clause.node),
                            Some(&clause.package),
                            CatalogSignatures::External(&selected.models),
                            &root,
                            meter,
                        )?,
                        protocol_memory_models(
                            &dependency,
                            &model.node,
                            Some(&clause.node),
                            &root,
                            meter,
                        )?,
                    )
                }
                _ => (Vec::new(), BTreeSet::new()),
            },
            None => {
                let operations = match abstract_key.as_ref() {
                    RefinementKey::Local(model) => graph
                        .local_node(model)
                        .and_then(|node| match node.owner.as_ref() {
                            Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) => {
                                Some(identity.as_ref())
                            }
                            _ => None,
                        })
                        .map(|identity| owners.operation_classes(identity, meter))
                        .transpose()?
                        .unwrap_or_default(),
                    RefinementKey::External(model) => graph
                        .dependencies
                        .selected_package(&model.package)
                        .and_then(|selected| {
                            selected
                                .graph()
                                .nodes
                                .iter()
                                .find(|node| node.node_id == model.node)
                                .and_then(|node| match node.owner.as_ref() {
                                    Some(super::CheckedNodeOwner::ModelRoot {
                                        identity, ..
                                    }) => Some((selected, identity.as_ref())),
                                    _ => None,
                                })
                        })
                        .map(|(selected, identity)| {
                            selected.models.operation_classes(identity, meter)
                        })
                        .transpose()?
                        .unwrap_or_default(),
                };
                let mut classes = operations
                    .into_iter()
                    .map(|operation| {
                        let key = |node| match abstract_key.as_ref() {
                            RefinementKey::Local(_) => RefinementKey::Local(node),
                            RefinementKey::External(model) => {
                                RefinementKey::External(RefinementPackageNodeKey {
                                    package: model.package.clone(),
                                    node,
                                })
                            }
                        };
                        DeclaredGraphClass {
                            class: format!("{}::{}", operation.object, operation.operation),
                            node: key(operation.node),
                            binder_types: operation.binder_types.map_or(
                                BinderSignature::Unspecified,
                                |types| {
                                    BinderSignature::Known(types.into_iter().map(key).collect())
                                },
                            ),
                        }
                    })
                    .collect::<Vec<_>>();
                classes.sort_unstable_by(|left, right| left.class.cmp(&right.class));
                (classes, BTreeSet::new())
            }
        };
        compare_abstract_classes(
            &catalog.r#abstract,
            &abstract_classes,
            &graph,
            root.clone().key("abstract"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        compare_abstract_memory(
            &catalog.r#abstract,
            &abstract_memory,
            root.key("abstract"),
            &graph.nodes[position].node_id,
            meter,
        )?;
    }
    Ok(())
}

#[derive(Clone)]
struct DeclaredGraphClass {
    class: String,
    node: RefinementKey,
    binder_types: BinderSignature<RefinementKey>,
}

#[derive(Clone)]
enum BinderSignature<T> {
    Known(Vec<T>),
    Unspecified,
    Ambiguous,
}

impl<T> BinderSignature<T> {
    fn map<U>(self, convert: impl FnMut(T) -> U) -> BinderSignature<U> {
        match self {
            Self::Known(types) => BinderSignature::Known(types.into_iter().map(convert).collect()),
            Self::Unspecified => BinderSignature::Unspecified,
            Self::Ambiguous => BinderSignature::Ambiguous,
        }
    }
}

#[derive(Clone, Copy)]
enum CatalogSignatures<'a, 'm> {
    Local(&'a super::model_members::ModelOwners<'m>),
    External(&'a super::model_fields::RetainedModels),
}

impl CatalogSignatures<'_, '_> {
    fn record_binder_types(
        &self,
        key: &CheckedNodeId,
    ) -> Option<super::model_fields::RecordBinderTypes> {
        match self {
            Self::Local(owners) => {
                let owner = owners.record_by_key(key)?;
                owner.package.record_binder_types(owner.node)
            }
            Self::External(models) => models.record_binder_types(key),
        }
    }

    fn is_record(&self, key: &CheckedNodeId) -> bool {
        match self {
            Self::Local(owners) => owners.record_by_key(key).is_some(),
            Self::External(models) => models.is_record_type(key),
        }
    }

    fn operation_binder_types(
        &self,
        context: &CheckedNodeId,
        name: &str,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
        match self {
            Self::Local(owners) => {
                let Some(owner) = owners.owner_by_key(context) else {
                    return Ok(None);
                };
                let mut budget = super::model_members::Budget::new(
                    meter,
                    owner.package.root,
                    owner.package.bytes,
                );
                owner
                    .package
                    .effective_operation_binder_types(owner.node, name, &mut budget)
            }
            Self::External(models) => models.operation_binder_types(context, name, meter),
        }
    }
}

struct ProtocolCatalogNode<'a> {
    id: &'a CheckedNodeId,
    kind: super::CheckedNodeKind,
    name: Option<&'a super::CheckedDeclaration>,
    owner: Option<&'a super::CheckedNodeOwner>,
    owner_model: Option<&'a CheckedNodeId>,
    semantic_type: &'a CheckedNodeId,
    term: Option<&'a CheckedTerm>,
}

fn local_protocol_nodes<'a>(graph: &'a RefinementGraph<'_, '_>) -> Vec<ProtocolCatalogNode<'a>> {
    graph
        .index
        .values()
        .map(|&position| {
            let node = &graph.nodes[position];
            ProtocolCatalogNode {
                id: &node.node_id,
                kind: graph.kinds[position],
                name: node.declaration.as_ref(),
                owner: node.owner.as_ref(),
                owner_model: node.owner_model.as_ref(),
                semantic_type: &node.semantic_type,
                term: graph.terms[position].as_ref(),
            }
        })
        .collect()
}

fn dependency_protocol_nodes(package: &super::CheckedPackageV2) -> Vec<ProtocolCatalogNode<'_>> {
    package
        .graph()
        .nodes
        .iter()
        .zip(package.node_kinds())
        .map(|(node, &kind)| ProtocolCatalogNode {
            id: &node.node_id,
            kind,
            name: node.declaration.as_ref(),
            owner: node.owner.as_ref(),
            owner_model: node.owner_model.as_ref(),
            semantic_type: &node.semantic_type,
            term: Some(&node.body),
        })
        .collect()
}

fn protocol_clause_prefix(member: &[Box<str>], path: &[Box<str>]) -> bool {
    member.len() > path.len()
        && member.starts_with(path)
        && member.get(path.len()).is_some_and(|segment| {
            matches!(
                segment.as_ref(),
                "role" | "channel" | "compensate" | "finish" | "terminal" | "run"
            )
        })
}

fn protocol_classes(
    nodes: &[ProtocolCatalogNode<'_>],
    model: &CheckedNodeId,
    selected_clause: Option<&CheckedNodeId>,
    package: Option<&str>,
    signatures: CatalogSignatures<'_, '_>,
    at: &JsonPointer,
    meter: &mut super::WorkMeter,
) -> Result<Vec<DeclaredGraphClass>, ValidationFailure> {
    let mut classes = Vec::new();
    let selected_identity = nodes.iter().find(|node| node.id == model).and_then(|node| {
        if let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = node.owner {
            Some(identity.as_ref())
        } else {
            None
        }
    });
    let key = |id: &CheckedNodeId| match package {
        Some(package) => RefinementKey::External(RefinementPackageNodeKey {
            package: package.into(),
            node: id.clone(),
        }),
        None => RefinementKey::Local(id.clone()),
    };
    meter.charge(super::model_members::units(nodes.len()), || at.clone())?;
    let clause_paths: Vec<_> = nodes
        .iter()
        .filter(|node| {
            node.kind == super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
        })
        .filter_map(|node| {
            node.name
                .map(|name| (node.id, name.qualified_name.as_slice()))
        })
        .collect();
    let mut closest_clauses = Vec::with_capacity(nodes.len());
    for member in nodes {
        let closest = if matches!(member.kind, super::CheckedNodeKind::Protocol(_)) {
            if let Some(name) = member.name {
                meter.charge(super::model_members::units(clause_paths.len()), || {
                    at.clone()
                })?;
                clause_paths
                    .iter()
                    .filter(|(_, path)| protocol_clause_prefix(&name.qualified_name, path))
                    .max_by_key(|(_, path)| path.len())
                    .map(|(id, _)| *id)
            } else {
                None
            }
        } else {
            None
        };
        closest_clauses.push(closest);
    }
    for clause in nodes {
        let clause_identity = clause
            .owner_model
            .and_then(|owner| nodes.iter().find(|node| node.id == owner))
            .and_then(|owner| match (owner.kind, owner.owner) {
                (
                    super::CheckedNodeKind::Model(super::ModelForm::ObjectType),
                    Some(super::CheckedNodeOwner::Model { identity, .. }),
                ) => Some(identity.as_ref()),
                _ => None,
            });
        if clause.kind != super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
            || clause_identity != selected_identity
            || selected_identity.is_none()
            || selected_clause.is_some_and(|selected| selected != clause.id)
        {
            continue;
        }
        let Some(clause_name) = clause.name else {
            continue;
        };
        if let Some(CheckedTerm::Application { arguments, .. }) = clause.term {
            if let Some(activation) = super::protocol_body::binding_value(arguments, "activation") {
                if super::protocol_body::group_value(activation, "kind")
                    .and_then(super::protocol_body::text_value)
                    == Some("each")
                {
                    meter.charge(super::model_members::units(nodes.len()), || at.clone())?;
                    let trigger_type = super::protocol_body::group_value(activation, "trigger")
                        .and_then(|trigger| match trigger {
                            CheckedTerm::Reference { target } => Some(target),
                            _ => None,
                        })
                        .and_then(|trigger| nodes.iter().find(|node| node.id == trigger))
                        .map(|trigger| trigger.semantic_type);
                    if let (Some(name), Some(trigger_type)) = (
                        protocol_clause_path(&clause_name.qualified_name),
                        trigger_type,
                    ) {
                        classes.push(DeclaredGraphClass {
                            class: format!("activate {name}"),
                            node: key(clause.id),
                            binder_types: binder_signature(trigger_type, signatures)
                                .map(|id| key(&id)),
                        });
                    }
                }
            }
        }
        for (member, closest_clause) in nodes.iter().zip(&closest_clauses) {
            meter.charge(1, || at.clone())?;
            let Some(name) = member.name else {
                continue;
            };
            if *closest_clause != Some(clause.id)
                || !matches!(member.kind, super::CheckedNodeKind::Protocol(_))
                || name.qualified_name.len() <= clause_name.qualified_name.len()
                || !name.qualified_name.starts_with(&clause_name.qualified_name)
            {
                continue;
            }
            let Some(CheckedTerm::Application {
                operation,
                arguments,
                ..
            }) = member.term
            else {
                continue;
            };
            let Some(operator) = operation
                .get("member")
                .and_then(|member| member.get("operator"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let Some(operator) = catalog_operator(member.kind, operator) else {
                continue;
            };
            let Some(classes_for_node) = protocol_member_classes(
                operator,
                arguments,
                &clause_name.qualified_name,
                &name.qualified_name,
            ) else {
                continue;
            };
            let binder_types = if matches!(operator, CatalogOperator::Role) {
                BinderSignature::Unspecified
            } else {
                protocol_binder_types(operator, arguments, nodes, signatures, meter)?
                    .map(|id| key(&id))
            };
            for class in classes_for_node {
                let class_binders = if class.empty_binders {
                    BinderSignature::Known(Vec::new())
                } else {
                    binder_types.clone()
                };
                classes.push(DeclaredGraphClass {
                    class: class.name,
                    node: key(member.id),
                    binder_types: class_binders,
                });
            }
        }
    }
    classes.sort_unstable_by(|left, right| left.class.cmp(&right.class));
    Ok(classes)
}

/// Collect only source defaults from parallel controls of the selected subject.
#[ix_trace_rs::string_edge]
fn protocol_memory_models(
    nodes: &[ProtocolCatalogNode<'_>],
    model: &CheckedNodeId,
    selected_clause: Option<&CheckedNodeId>,
    at: &JsonPointer,
    meter: &mut super::WorkMeter,
) -> Result<BTreeSet<RefinementMemoryModel>, ValidationFailure> {
    let mut by_id = BTreeMap::new();
    for node in nodes {
        meter.charge(1, || at.clone())?;
        by_id.insert(node.id, node);
    }
    let selected_identity = by_id.get(model).and_then(|node| {
        if let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = node.owner {
            Some(identity.as_ref())
        } else {
            None
        }
    });
    let mut models = BTreeSet::new();
    for clause in nodes {
        let clause_identity = clause
            .owner_model
            .and_then(|owner| by_id.get(owner))
            .and_then(|owner| match owner.owner {
                Some(super::CheckedNodeOwner::Model { identity, .. }) => Some(identity.as_ref()),
                _ => None,
            });
        if clause.kind != super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
            || selected_identity.is_none()
            || clause_identity != selected_identity
            || selected_clause.is_some_and(|selected| selected != clause.id)
        {
            continue;
        }
        let Some(clause_name) = clause.name else {
            continue;
        };
        for member in nodes {
            meter.charge(1, || at.clone())?;
            let Some(name) = member.name else {
                continue;
            };
            if member.kind != super::CheckedNodeKind::Protocol(super::ProtocolForm::Control)
                || name.qualified_name.len() <= clause_name.qualified_name.len()
                || !name.qualified_name.starts_with(&clause_name.qualified_name)
            {
                continue;
            }
            let Some(CheckedTerm::Application {
                operation,
                arguments,
                ..
            }) = member.term
            else {
                continue;
            };
            if operation
                .get("member")
                .and_then(|member| member.get("operator"))
                .and_then(Value::as_str)
                != Some("parallel")
            {
                continue;
            }
            let Some(CheckedTerm::Aggregate { members }) =
                super::protocol_body::binding_value(arguments, "memory")
            else {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at.clone(),
                ));
            };
            let selected = match members.as_slice() {
                [] => "sc",
                [value] => super::protocol_body::text_value(value).ok_or_else(|| {
                    ValidationFailure::refused(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at.clone(),
                    )
                })?,
                _ => {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at.clone(),
                    ))
                }
            };
            let model = match selected {
                "sc" => RefinementMemoryModel::Sc,
                "tso" => RefinementMemoryModel::Tso,
                "ra" => RefinementMemoryModel::Ra,
                _ => {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at.clone(),
                    ))
                }
            };
            models.insert(model);
        }
    }
    Ok(models)
}

fn expected_memory_classes(
    models: &BTreeSet<RefinementMemoryModel>,
) -> BTreeSet<(&'static str, RefinementMemoryModel)> {
    let mut expected = BTreeSet::new();
    for model in models {
        expected.insert(("memory drain", *model));
        if *model == RefinementMemoryModel::Tso {
            expected.insert(("memory flush", *model));
        }
    }
    expected
}

fn compare_memory<'a>(
    actual: impl Iterator<Item = (usize, &'a RefinementMemoryClass)>,
    models: &BTreeSet<RefinementMemoryModel>,
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected_memory_classes(models);
    for (index, memory) in actual {
        let entry_at = at.clone().index(index);
        meter.charge(1, || entry_at.clone())?;
        if !expected.remove(&(memory.class(), memory.model())) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                entry_at,
                None,
                locus.clone(),
            ));
        }
    }
    if !expected.is_empty() {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at,
            None,
            locus.clone(),
        ));
    }
    Ok(())
}

fn compare_concrete_memory(
    actual: &[RefinementConcreteClass],
    models: &BTreeSet<RefinementMemoryModel>,
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    compare_memory(
        actual
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                RefinementConcreteClass::Memory(memory) => Some((index, memory)),
                RefinementConcreteClass::Declared { .. } => None,
            }),
        models,
        at,
        locus,
        meter,
    )
}

fn compare_abstract_memory(
    actual: &[RefinementAbstractClass],
    models: &BTreeSet<RefinementMemoryModel>,
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    compare_memory(
        actual
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                RefinementAbstractClass::Memory(memory) => Some((index, memory)),
                RefinementAbstractClass::Declared { .. } => None,
            }),
        models,
        at,
        locus,
        meter,
    )
}

#[derive(Clone, Copy)]
enum CatalogOperator {
    Role,
    Channel,
    Parallel,
    Attempt,
    Compensation,
    Event,
    Send,
    Receive,
    Fence,
    /// The timeout step owned by an await node (FR-426), with FR-432's bare
    /// node-path spelling.
    TimeoutFromAwait,
    Finish,
}

/// Classify the admitted protocol operator once, together with its checked
/// node form; catalog reconstruction below uses only this closed set.
#[ix_trace_rs::string_edge]
fn catalog_operator(kind: super::CheckedNodeKind, operator: &str) -> Option<CatalogOperator> {
    use super::{CheckedNodeKind::Protocol, ProtocolForm};
    Some(match (kind, operator) {
        (Protocol(ProtocolForm::Role), "role") => CatalogOperator::Role,
        (Protocol(ProtocolForm::Channel), "channel") => CatalogOperator::Channel,
        (Protocol(ProtocolForm::Control), "parallel") => CatalogOperator::Parallel,
        (Protocol(ProtocolForm::Control), "attempt") => CatalogOperator::Attempt,
        (Protocol(ProtocolForm::Compensation), "compensate") => CatalogOperator::Compensation,
        (Protocol(ProtocolForm::Control), "event" | "effect" | "commit") => CatalogOperator::Event,
        (Protocol(ProtocolForm::Control), "send") => CatalogOperator::Send,
        (Protocol(ProtocolForm::Control), "receive") => CatalogOperator::Receive,
        (Protocol(ProtocolForm::Control), "fence") => CatalogOperator::Fence,
        (Protocol(ProtocolForm::Control), "await") => CatalogOperator::TimeoutFromAwait,
        (Protocol(ProtocolForm::Obligation), "finish") => CatalogOperator::Finish,
        _ => return None,
    })
}

fn protocol_binder_types(
    operator: CatalogOperator,
    arguments: &[CheckedTerm],
    nodes: &[ProtocolCatalogNode<'_>],
    signatures: CatalogSignatures<'_, '_>,
    meter: &mut super::WorkMeter,
) -> Result<BinderSignature<CheckedNodeId>, ValidationFailure> {
    if matches!(
        operator,
        CatalogOperator::Attempt | CatalogOperator::Compensation
    ) {
        let Some(anchor) = bound_reference(arguments, "operation")
            .and_then(|id| nodes.iter().find(|node| node.id == id))
            .and_then(|node| node.term)
            .and_then(super::structural::typed_anchor_body)
        else {
            return Ok(BinderSignature::Unspecified);
        };
        return Ok(signatures
            .operation_binder_types(&anchor.context, anchor.operation, meter)?
            .map_or(BinderSignature::Unspecified, BinderSignature::Known));
    }
    let binder_type = match operator {
        CatalogOperator::Send | CatalogOperator::Receive => {
            let Some(channel) = bound_reference(arguments, "channel")
                .and_then(|id| nodes.iter().find(|node| node.id == id))
                .and_then(|node| node.term)
            else {
                return Ok(BinderSignature::Unspecified);
            };
            let CheckedTerm::Application {
                arguments: channel_arguments,
                ..
            } = channel
            else {
                return Ok(BinderSignature::Unspecified);
            };
            let Some(carries) = bound_reference(channel_arguments, "carries") else {
                return Ok(BinderSignature::Unspecified);
            };
            carries
        }
        CatalogOperator::Event | CatalogOperator::Finish => {
            let Some(binder) = bound_reference(arguments, "binder")
                .and_then(|id| nodes.iter().find(|node| node.id == id))
            else {
                return Ok(BinderSignature::Unspecified);
            };
            binder.semantic_type
        }
        CatalogOperator::Role
        | CatalogOperator::Channel
        | CatalogOperator::Parallel
        | CatalogOperator::Attempt
        | CatalogOperator::Compensation
        | CatalogOperator::Fence
        | CatalogOperator::TimeoutFromAwait => return Ok(BinderSignature::Known(Vec::new())),
    };
    Ok(binder_signature(binder_type, signatures))
}

fn binder_signature(
    binder_type: &CheckedNodeId,
    signatures: CatalogSignatures<'_, '_>,
) -> BinderSignature<CheckedNodeId> {
    if signatures.is_record(binder_type) {
        match signatures.record_binder_types(binder_type) {
            Some(super::model_fields::RecordBinderTypes::Known(types)) => {
                BinderSignature::Known(types)
            }
            Some(super::model_fields::RecordBinderTypes::Ambiguous) => BinderSignature::Ambiguous,
            None => BinderSignature::Unspecified,
        }
    } else {
        BinderSignature::Known(vec![binder_type.clone()])
    }
}

fn bound_reference<'a>(arguments: &'a [CheckedTerm], field: &str) -> Option<&'a CheckedNodeId> {
    arguments.iter().find_map(|argument| match argument {
        CheckedTerm::Binding { name, value } if name.as_ref() == field => match value.as_ref() {
            CheckedTerm::Reference { target } => Some(target),
            _ => None,
        },
        _ => None,
    })
}

struct ProtocolMemberClass {
    name: String,
    empty_binders: bool,
}

/// Interprets the admitted delivery and replication literals that determine
/// FR-432 channel and role classes.
#[ix_trace_rs::string_edge]
fn protocol_member_classes(
    operator: CatalogOperator,
    arguments: &[CheckedTerm],
    clause_name: &[Box<str>],
    name: &[Box<str>],
) -> Option<Vec<ProtocolMemberClass>> {
    let path = format!("{}::{}", protocol_clause_path(clause_name)?, name.last()?);
    let inherited = |name| ProtocolMemberClass {
        name,
        empty_binders: false,
    };
    let classes = match operator {
        CatalogOperator::Parallel => Some(vec![format!("fork {path}"), format!("join {path}")]),
        CatalogOperator::Compensation => {
            return Some(vec![
                inherited(path.clone()),
                ProtocolMemberClass {
                    name: format!("cend {path}"),
                    empty_binders: true,
                },
            ]);
        }
        CatalogOperator::Attempt
        | CatalogOperator::Event
        | CatalogOperator::Send
        | CatalogOperator::Receive
        | CatalogOperator::Fence
        | CatalogOperator::TimeoutFromAwait
        | CatalogOperator::Finish => Some(vec![path]),
        CatalogOperator::Channel => {
            let delivery = super::protocol_body::binding_value(arguments, "delivery")
                .and_then(super::protocol_body::text_value)?;
            Some(match delivery {
                "unknown" => vec![format!("duplicate {path}"), format!("lose {path}")],
                "at-least-once" => vec![format!("duplicate {path}")],
                "at-most-once" => vec![format!("lose {path}")],
                "exactly-once-premise" => Vec::new(),
                _ => return None,
            })
        }
        CatalogOperator::Role => {
            let replication = super::protocol_body::binding_value(arguments, "replication")?;
            let CheckedTerm::Aggregate { members } = replication else {
                return None;
            };
            if members.is_empty() {
                return Some(Vec::new());
            }
            let mut classes = vec![format!("spawn {path}")];
            if super::protocol_body::group_value(replication, "lifetime")
                .and_then(super::protocol_body::text_value)
                == Some("until")
            {
                classes.push(format!("retire {path}"));
            }
            Some(classes)
        }
    }?;
    Some(classes.into_iter().map(inherited).collect())
}

/// Preserve every authored clause segment in the class identity.
fn protocol_clause_path(name: &[Box<str>]) -> Option<String> {
    (!name.is_empty()).then(|| {
        name.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            .join("::")
    })
}

fn compare_declared_classes(
    actual: &[RefinementConcreteClass],
    expected: &[DeclaredGraphClass],
    graph: &RefinementGraph<'_, '_>,
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected.iter().peekable();
    for (index, entry) in actual.iter().enumerate() {
        if matches!(entry, RefinementConcreteClass::Memory(_)) {
            continue;
        }
        let entry_at = at.clone().index(index);
        meter.charge(1, || entry_at.clone())?;
        if expected
            .peek()
            .is_some_and(|item| item.class.as_str() < entry.class())
        {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at,
                None,
                locus.clone(),
            ));
        }
        let Some(found) = expected.next() else {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                entry_at,
                None,
                locus.clone(),
            ));
        };
        if found.class != entry.class() {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                entry_at,
                None,
                locus.clone(),
            ));
        }
        match entry {
            RefinementConcreteClass::Declared {
                node, binder_types, ..
            } => {
                if matches!(found.binder_types, BinderSignature::Ambiguous) {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::AmbiguousDeclaration,
                        entry_at.key("binder_types"),
                        Some(CheckedPackageRefusalCause::AmbiguousName),
                        locus.clone(),
                    ));
                }
                if found.node.local() != Some(node)
                    || matches!(&found.binder_types, BinderSignature::Known(expected)
                        if binder_types.iter().map(|id| super::structural_key(id, graph.index, graph.structural_keys))
                            .ne(expected.iter().map(|key| graph.resolved_key(key))))
                {
                    return Err(wrong_binding(entry_at, locus));
                }
            }
            RefinementConcreteClass::Memory(_) => unreachable!("memory entry skipped above"),
        }
    }
    if expected.next().is_some() {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at,
            None,
            locus.clone(),
        ));
    }
    Ok(())
}

fn compare_abstract_classes(
    actual: &[RefinementAbstractClass],
    expected: &[DeclaredGraphClass],
    graph: &RefinementGraph<'_, '_>,
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected.iter().peekable();
    for (index, entry) in actual.iter().enumerate() {
        if matches!(entry, RefinementAbstractClass::Memory(_)) {
            continue;
        }
        let entry_at = at.clone().index(index);
        meter.charge(1, || entry_at.clone())?;
        if expected
            .peek()
            .is_some_and(|item| item.class.as_str() < entry.class())
        {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at,
                None,
                locus.clone(),
            ));
        }
        let Some(found) = expected.next() else {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                entry_at,
                None,
                locus.clone(),
            ));
        };
        if found.class != entry.class() {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                entry_at,
                None,
                locus.clone(),
            ));
        }
        match entry {
            RefinementAbstractClass::Declared {
                node, binder_types, ..
            } => {
                if matches!(found.binder_types, BinderSignature::Ambiguous) {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::AmbiguousDeclaration,
                        entry_at.key("binder_types"),
                        Some(CheckedPackageRefusalCause::AmbiguousName),
                        locus.clone(),
                    ));
                }
                if node != &found.node
                    || matches!(&found.binder_types, BinderSignature::Known(expected)
                        if binder_types.len() != expected.len()
                            || binder_types.iter().zip(expected).any(|(actual, expected)|
                                !graph.same_key(actual, expected)))
                {
                    return Err(wrong_binding(entry_at, locus));
                }
            }
            RefinementAbstractClass::Memory(_) => unreachable!("memory entry skipped above"),
        }
    }
    if expected.next().is_some() {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at,
            None,
            locus.clone(),
        ));
    }
    Ok(())
}

fn add_key(targets: &mut BTreeSet<CheckedNodeId>, key: &RefinementKey) {
    if let Some(local) = key.local() {
        targets.insert(local.clone());
    }
}

fn add_term(targets: &mut BTreeSet<CheckedNodeId>, term: &CheckedTerm) {
    targets.extend(term.reference_targets().into_iter().cloned());
    if let Some(application) = super::structural::application_dependencies(term) {
        targets.extend(application);
    }
}

fn local_dependencies(body: &CheckedTerm) -> Option<Vec<CheckedNodeId>> {
    let CheckedTerm::Refinement {
        abstract_key,
        protocol,
        populations,
        objects,
        steps,
        histories,
        assume,
        ensure,
        class_catalog,
        ..
    } = body
    else {
        return None;
    };
    let mut targets = BTreeSet::new();
    add_key(&mut targets, abstract_key);
    if let Some(protocol) = protocol {
        add_key(&mut targets, protocol);
    }
    for row in populations {
        add_key(&mut targets, &row.r#abstract);
        targets.insert(row.concrete.clone());
    }
    for row in objects {
        add_key(&mut targets, &row.abstract_type);
        targets.insert(row.concrete_type.clone());
        for field in &row.fields {
            add_term(&mut targets, &field.expression);
        }
    }
    for row in steps {
        if let Some(node) = &row.source_node {
            targets.insert(node.clone());
        }
        match &row.target {
            RefinementStepTarget::Operation {
                operation,
                receiver,
                arguments,
            } => {
                add_key(&mut targets, operation);
                add_term(&mut targets, receiver);
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        add_term(&mut targets, term);
                    }
                }
            }
            RefinementStepTarget::ProtocolStep {
                node, arguments, ..
            } => {
                if let Some(node) = node {
                    add_key(&mut targets, node);
                }
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        add_term(&mut targets, term);
                    }
                }
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for row in histories {
        targets.insert(row.concrete_type.clone());
        targets.insert(row.type_id.clone());
        add_term(&mut targets, &row.initial);
        for update in &row.updates {
            targets.insert(update.operation.clone());
            add_term(&mut targets, &update.expression);
        }
    }
    for row in assume {
        targets.insert(row.operation.clone());
    }
    for row in ensure {
        add_key(&mut targets, &row.operation);
    }
    if let Some(catalog) = class_catalog {
        for entry in &catalog.concrete {
            if let RefinementConcreteClass::Declared {
                node, binder_types, ..
            } = entry
            {
                targets.insert(node.clone());
                targets.extend(binder_types.iter().cloned());
            }
        }
        for entry in &catalog.r#abstract {
            if let RefinementAbstractClass::Declared {
                node, binder_types, ..
            } = entry
            {
                add_key(&mut targets, node);
                for binder in binder_types {
                    add_key(&mut targets, binder);
                }
            }
        }
    }
    Some(targets.into_iter().collect())
}

/// Check the exact local dependency vector at the graph-shape stage.
pub(super) fn validate_dependencies(
    nodes: &[super::RawSemanticNodeV2],
    kinds: &[super::CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
) -> Result<(), ValidationFailure> {
    for (position, ((node, kind), term)) in nodes.iter().zip(kinds).zip(terms).enumerate() {
        if *kind != super::CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
        let expected = term.as_ref().and_then(local_dependencies).ok_or_else(|| {
            ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                crate::checked_package::common::node_pointer(position).key("body"),
            )
        })?;
        let expected = expected
            .iter()
            .map(|id| super::structural_key(id, index, structural_keys))
            .collect::<BTreeSet<_>>();
        if !node
            .dependencies
            .iter()
            .map(|id| super::structural_key(id, index, structural_keys))
            .eq(expected)
        {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                crate::checked_package::common::node_pointer(position).key("dependencies"),
            ));
        }
    }
    Ok(())
}

/// Reference terms in the three row collections that can hold expressions.
pub(super) fn visit_reference_targets<'a>(
    objects: &'a [RefinementObject],
    steps: &'a [RefinementStep],
    histories: &'a [RefinementHistory],
    visitor: &mut impl FnMut(&'a CheckedNodeId),
) {
    for object in objects {
        for field in &object.fields {
            field.expression.visit_reference_targets(visitor);
        }
    }
    for step in steps {
        match &step.target {
            RefinementStepTarget::Operation {
                receiver,
                arguments,
                ..
            } => {
                receiver.visit_reference_targets(visitor);
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        term.visit_reference_targets(visitor);
                    }
                }
            }
            RefinementStepTarget::ProtocolStep { arguments, .. } => {
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        term.visit_reference_targets(visitor);
                    }
                }
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for history in histories {
        history.initial.visit_reference_targets(visitor);
        for update in &history.updates {
            update.expression.visit_reference_targets(visitor);
        }
    }
}
