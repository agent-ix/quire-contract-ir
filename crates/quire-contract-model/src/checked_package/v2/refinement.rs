//! Closed FR-375 refinement body vocabulary and reader checks.

use super::graph_storage::{GraphIndex, ResolvedKey, SortedTable};
use super::protocol_body::ChannelDelivery;
use super::stage_storage::{
    charge_amount_at, charge_at, node_path, refuse_at, refuse_unlocated, Path,
};
use super::CheckedTerm;
use crate::checked_package::common::{
    validate_term, Ids, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSourceMapEntry, CheckedSourceRef,
    CheckedSourceRegion,
};
use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
use crate::checked_package::terms::TermSource;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
#[cfg(test)]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::Value;

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

fn node_at(position: usize) -> Path<'static> {
    node_path(position)
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

/// Convert the untrusted argument-only special term before row admission.
#[ix_trace_rs::string_edge]
fn open_term_kind(term: &str) -> Option<RefinementOpenTerm> {
    (term == "open").then_some(RefinementOpenTerm::Open)
}

#[cfg(test)]
fn member<'a>(
    value: &'a Value,
    name: &str,
    at: &Trail<'_>,
) -> Result<&'a Value, ValidationFailure> {
    value.get(name).ok_or_else(|| invalid(&at.key(name)))
}

#[cfg(test)]
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

#[cfg(test)]
fn typed<T: DeserializeOwned>(value: &Value, at: &Trail<'_>) -> Result<T, ValidationFailure> {
    serde_json::from_value(value.clone()).map_err(|_| invalid(at))
}

#[cfg(test)]
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

#[cfg(test)]
fn text(value: &Value, at: &Trail<'_>) -> Result<Box<str>, ValidationFailure> {
    value.as_str().map(Into::into).ok_or_else(|| invalid(at))
}

#[cfg(test)]
fn node(value: &Value, at: &Trail<'_>) -> Result<CheckedNodeId, ValidationFailure> {
    CheckedNodeId::from_value(value).ok_or_else(|| invalid(at))
}

#[cfg(test)]
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

#[cfg(test)]
fn term(value: &Value, at: &Trail<'_>) -> Result<CheckedTerm, ValidationFailure> {
    super::read_charge::with_unmetered_walk(|walk| {
        super::flat_wire::check_refinement_term(value, at, walk)
    })?;
    CheckedTerm::decode_admitted(value).map_err(|_| invalid(at))
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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
#[cfg(test)]
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

/// The member `name` of an object the closed body check has passed.
fn node_member<'a, V: TermSource<'a>>(
    value: V,
    name: &str,
    at: &Trail<'_>,
) -> Result<V, ValidationFailure> {
    value.member(name).ok_or_else(|| invalid(&at.key(name)))
}

/// The elements of the array member `name`.
fn node_items<'a, V: TermSource<'a>>(
    value: V,
    name: &str,
    at: &Trail<'_>,
) -> Result<impl Iterator<Item = V> + use<'a, V>, ValidationFailure> {
    node_member(value, name, at)?
        .items()
        .ok_or_else(|| invalid(&at.key(name)))
}

/// Walk each actual semantic term in FR-375 row order. The special `open`
/// argument is not a semantic term and is excluded here.
fn row_terms<'a, V: TermSource<'a>>(
    body: V,
    at: &Trail<'_>,
    mut visit: impl FnMut(V, &Trail<'_>) -> Result<(), ValidationFailure>,
) -> Result<(), ValidationFailure> {
    for (object_index, object) in node_items(body, "objects", at)?.enumerate() {
        let objects_at = at.key("objects");
        let object_at = objects_at.index(object_index);
        for (field_index, field) in node_items(object, "fields", &object_at)?.enumerate() {
            let fields_at = object_at.key("fields");
            let field_at = fields_at.index(field_index);
            visit(
                node_member(field, "expression", &field_at)?,
                &field_at.key("expression"),
            )?;
        }
    }
    for (step_index, step) in node_items(body, "steps", at)?.enumerate() {
        let steps_at = at.key("steps");
        let step_at = steps_at.index(step_index);
        let target_at = step_at.key("target");
        let target = node_member(step, "target", &step_at)?;
        let kind = node_member(target, "kind", &target_at)?.text();
        if kind == Some("operation") {
            visit(
                node_member(target, "receiver", &target_at)?,
                &target_at.key("receiver"),
            )?;
        }
        if matches!(kind, Some("operation" | "protocol_step")) {
            for (argument_index, argument) in
                node_items(target, "arguments", &target_at)?.enumerate()
            {
                if argument.member("term").and_then(V::text) != Some("open") {
                    visit(argument, &target_at.key("arguments").index(argument_index))?;
                }
            }
        }
    }
    for (history_index, history) in node_items(body, "histories", at)?.enumerate() {
        let histories_at = at.key("histories");
        let history_at = histories_at.index(history_index);
        visit(
            node_member(history, "initial", &history_at)?,
            &history_at.key("initial"),
        )?;
        for (update_index, update) in node_items(history, "updates", &history_at)?.enumerate() {
            let updates_at = history_at.key("updates");
            let update_at = updates_at.index(update_index);
            visit(
                node_member(update, "expression", &update_at)?,
                &update_at.key("expression"),
            )?;
        }
    }
    Ok(())
}

/// Check the complete closed body before graph admission, including the
/// nested flat-term rule that applies to each row expression independently.
pub(super) fn check_wire<'a, V: TermSource<'a>>(
    body: V,
    typed: Option<&CheckedTerm>,
    at: &Trail<'_>,
    walk: &mut super::read_charge::WalkReservation<'_>,
) -> Result<(), ValidationFailure> {
    let Some(CheckedTerm::Refinement {
        steps,
        histories,
        abstract_internal,
        protocol,
        ..
    }) = typed
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
        super::flat_wire::check_refinement_term(term, term_at, walk)
    })
}

/// Convert the outer raw object shape before typed semantic checks.
#[ix_trace_rs::string_edge]
#[cfg(test)]
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
/// The whole body has already passed `check_wire`. Each term's allocations are
/// charged to `ids`, and its node IDs are read through the node's operand row.
pub(super) fn validate_body<'a, V: TermSource<'a>>(
    body: V,
    at: &Trail<'a>,
    visitor: &mut ReferenceVisitor<'_>,
    mut ids: Ids<'_, '_>,
) -> Result<u64, ValidationFailure> {
    let mut work = 1_u64;
    row_terms(body, at, |term, term_at| {
        work = work.saturating_add(validate_term(
            term.shorter(),
            TermGrammar::V2,
            true,
            term_at,
            visitor,
            ids.reborrow(),
        )?);
        Ok(())
    })?;
    Ok(work)
}

impl RefinementKey {
    fn local(&self) -> Option<&CheckedNodeId> {
        match self {
            Self::Local(node) => Some(node),
            Self::External(_) => None,
        }
    }
}

/// A refusal about the value at `at` with the `cause` this stage determined,
/// its diagnostic charged before it is built.
fn refuse_because(
    meter: &mut super::WorkMeter,
    code: CheckedPackageRefusalCode,
    at: Path<'_>,
    cause: CheckedPackageRefusalCause,
) -> ValidationFailure {
    let site = super::natural::site(meter, at);
    super::stage_storage::refuse(
        meter,
        &site,
        super::stage_storage::Refusal::new(code, at).because(cause),
    )
}

fn order_defect(
    meter: &mut super::WorkMeter,
    position: usize,
    array: &str,
    index: usize,
) -> ValidationFailure {
    refuse_unlocated(
        meter,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        node_at(position).key("body").key(array).index(index),
    )
}

/// Check the ordered FR-375 row arrays and their nested arrays in published order.
fn canonical_order(
    body: &CheckedTerm,
    position: usize,
    graph: &RefinementGraph<'_, '_>,
    meter: &mut super::WorkMeter,
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
        return Err(refuse_unlocated(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    for (index, pair) in populations.windows(2).enumerate() {
        if graph
            .resolved_key(&pair[0].r#abstract)
            .order(&graph.resolved_key(&pair[1].r#abstract), meter)?
            != std::cmp::Ordering::Less
        {
            return Err(order_defect(meter, position, "populations", index + 1));
        }
    }
    for (index, object) in objects.iter().enumerate() {
        for (field_index, pair) in object.fields.windows(2).enumerate() {
            if pair[0].name >= pair[1].name {
                return Err(refuse_unlocated(
                    meter,
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
        if graph
            .resolved_key(&pair[0].abstract_type)
            .order(&graph.resolved_key(&pair[1].abstract_type), meter)?
            != std::cmp::Ordering::Less
        {
            return Err(order_defect(meter, position, "objects", index + 1));
        }
    }
    for (index, history) in histories.iter().enumerate() {
        for (update_index, pair) in history.updates.windows(2).enumerate() {
            if pair[0].operation >= pair[1].operation {
                return Err(refuse_unlocated(
                    meter,
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
        let first = graph
            .index
            .resolved(&pair[0].concrete_type, graph.structural_keys);
        let second = graph
            .index
            .resolved(&pair[1].concrete_type, graph.structural_keys);
        if first
            .order(&second, meter)?
            .then_with(|| pair[0].name.as_ref().cmp(pair[1].name.as_ref()))
            != std::cmp::Ordering::Less
        {
            return Err(order_defect(meter, position, "histories", index + 1));
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
        ) -> (ResolvedKey<'a>, &'static str, &'static str) {
            (
                graph.index.resolved(&row.operation, graph.structural_keys),
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        let (first, second) = (key(&pair[0], graph), key(&pair[1], graph));
        if first
            .0
            .order(&second.0, meter)?
            .then_with(|| (first.1, first.2).cmp(&(second.1, second.2)))
            != std::cmp::Ordering::Less
        {
            return Err(order_defect(meter, position, "assume", index + 1));
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
        ) -> (ResolvedKey<'a>, &'static str, &'static str) {
            (
                graph.resolved_key(&row.operation),
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        let (first, second) = (key(&pair[0], graph), key(&pair[1], graph));
        if first
            .0
            .order(&second.0, meter)?
            .then_with(|| (first.1, first.2).cmp(&(second.1, second.2)))
            != std::cmp::Ordering::Less
        {
            return Err(order_defect(meter, position, "ensure", index + 1));
        }
    }
    if let Some(internal) = abstract_internal {
        for (index, pair) in internal.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(order_defect(
                    meter,
                    position,
                    "abstract_internal",
                    index + 1,
                ));
            }
        }
    }
    if let Some(catalog) = class_catalog {
        for (index, pair) in catalog.concrete.windows(2).enumerate() {
            if concrete_catalog_key(&pair[0]) >= concrete_catalog_key(&pair[1]) {
                return Err(refuse_unlocated(
                    meter,
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
                return Err(refuse_unlocated(
                    meter,
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
    index: &'a GraphIndex<'a>,
    structural_keys: &'a [std::sync::Arc<CheckedNodeId>],
    dependencies: &'a super::SuppliedDependencies<'e>,
    package_name: &'a str,
}

impl<'a, 'e> RefinementGraph<'a, 'e> {
    pub(super) fn new(
        nodes: &'a [super::RawSemanticNodeV2],
        kinds: &'a [super::CheckedNodeKind],
        terms: &'a [Option<CheckedTerm>],
        index: &'a GraphIndex<'a>,
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
    /// The graph key a reference names. A local key compares by its graph
    /// rank; a key of a selected dependency package is another graph's and
    /// compares by `CheckedNodeId::cmp`.
    fn resolved_key<'a>(&'a self, key: &'a RefinementKey) -> ResolvedKey<'a> {
        match key {
            RefinementKey::Local(id) => self.index.resolved(id, self.structural_keys),
            RefinementKey::External(external) => ResolvedKey::foreign(
                self.dependencies
                    .selected_package(&external.package)
                    .and_then(|package| package.structural_key(&external.node))
                    .unwrap_or(&external.node),
            ),
        }
    }

    fn same_key(
        &self,
        actual: &RefinementKey,
        expected: &RefinementKey,
        meter: &mut super::WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let same = match (actual, expected) {
            (RefinementKey::Local(_), RefinementKey::Local(_)) => true,
            (RefinementKey::External(actual_ref), RefinementKey::External(expected_ref)) => {
                actual_ref.package == expected_ref.package
            }
            _ => false,
        };
        Ok(same
            && self
                .resolved_key(actual)
                .order(&self.resolved_key(expected), meter)?
                == std::cmp::Ordering::Equal)
    }

    fn local_term(&self, id: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.terms
            .get(*self.index.get(id)?)
            .and_then(Option::as_ref)
    }

    fn local_node(&self, id: &CheckedNodeId) -> Option<&super::RawSemanticNodeV2> {
        self.nodes.get(*self.index.get(id)?)
    }

    fn local_owner_identity(&self, id: &CheckedNodeId) -> Option<&str> {
        let owner = self.local_node(id)?.owner_model.as_ref()?;
        match self.local_node(owner)?.owner.as_ref()? {
            super::CheckedNodeOwner::Model { identity, .. } => Some(identity),
            _ => None,
        }
    }

    fn key_owner_identity(&self, key: &RefinementKey) -> Option<&str> {
        let owner = match key {
            RefinementKey::Local(id) => return self.local_owner_identity(id),
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

    fn local_anchor(&self, id: &CheckedNodeId) -> Option<(&CheckedNodeId, &str)> {
        let anchor = super::structural::typed_anchor_body(self.local_term(id)?)?;
        Some((anchor.context, anchor.operation))
    }

    fn key_anchor(&self, key: &RefinementKey) -> Option<(&CheckedNodeId, &str)> {
        let term = match key {
            RefinementKey::Local(id) => return self.local_anchor(id),
            RefinementKey::External(external) => self
                .dependencies
                .selected_package(&external.package)?
                .term(&external.node)?,
        };
        let anchor = super::structural::typed_anchor_body(term)?;
        Some((anchor.context, anchor.operation))
    }

    fn local_kind(
        &self,
        meter: &mut super::WorkMeter,
        id: &CheckedNodeId,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<super::CheckedNodeKind, ValidationFailure> {
        let Some(&position) = self.index.get(id) else {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::MissingDeclaration,
                Some(CheckedPackageRefusalCause::MissingName),
                at,
                locus,
            ));
        };
        self.kinds.get(position).copied().ok_or_else(|| {
            refuse_unlocated(meter, CheckedPackageRefusalCode::InvalidSemanticGraph, at)
        })
    }

    fn key_kind(
        &self,
        meter: &mut super::WorkMeter,
        key: &RefinementKey,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<super::CheckedNodeKind, ValidationFailure> {
        match key {
            RefinementKey::Local(id) => self.local_kind(meter, id, at, locus),
            RefinementKey::External(key) => {
                if key.package.as_ref() == self.package_name {
                    return Err(refuse_at(
                        meter,
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        None,
                        at.key("package"),
                        locus,
                    ));
                }
                let selected = self
                    .dependencies
                    .selected_package(&key.package)
                    .ok_or_else(|| {
                        refuse_at(
                            meter,
                            CheckedPackageRefusalCode::MissingDeclaration,
                            Some(CheckedPackageRefusalCause::MissingSelection),
                            at.key("package"),
                            locus,
                        )
                    })?;
                selected.node_kind(&key.node).ok_or_else(|| {
                    refuse_at(
                        meter,
                        CheckedPackageRefusalCode::MissingDeclaration,
                        Some(CheckedPackageRefusalCause::MissingName),
                        at,
                        locus,
                    )
                })
            }
        }
    }

    fn require_kind(
        &self,
        meter: &mut super::WorkMeter,
        key: &RefinementKey,
        expected: super::CheckedNodeKind,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        let actual = self.key_kind(meter, key, at, locus)?;
        if actual == expected {
            Ok(())
        } else {
            Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidModelBinding,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                at,
                locus,
            ))
        }
    }

    fn require_local_kind(
        &self,
        meter: &mut super::WorkMeter,
        id: &CheckedNodeId,
        expected: super::CheckedNodeKind,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<(), ValidationFailure> {
        let actual = self.local_kind(meter, id, at, locus)?;
        if actual == expected {
            Ok(())
        } else {
            Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidModelBinding,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                at,
                locus,
            ))
        }
    }

    fn require_anchor_subject<'s>(
        &'s self,
        meter: &mut super::WorkMeter,
        key: &RefinementKey,
        subject: &RefinementKey,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<&'s str, ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner, ModelForm};
        self.require_kind(
            meter,
            key,
            K::State(super::StateForm::OperationAnchor),
            at,
            locus,
        )?;
        let Some((context, operation)) = self.key_anchor(key) else {
            return Err(wrong_binding(meter, at, locus));
        };
        // An anchor names its declaring type, while `subject` names the selected model root.
        let (declaring_kind, declaring_identity, selected_identity) = match (key, subject) {
            (RefinementKey::Local(_), RefinementKey::Local(root)) => {
                let kind = self.local_kind(meter, context, at, locus)?;
                let declaring = self.local_node(context).and_then(|node| match &node.owner {
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
                    return Err(wrong_binding(meter, at, locus));
                };
                let Some(kind) = selected.node_kind(context) else {
                    return Err(wrong_binding(meter, at, locus));
                };
                let declaring = selected.node(context).and_then(|node| match &node.owner {
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
            _ => return Err(wrong_binding(meter, at, locus)),
        };
        if !matches!(
            declaring_kind,
            K::Model(ModelForm::ObjectType | ModelForm::Process | ModelForm::StateMachine)
        ) || declaring_identity.is_none()
            || declaring_identity != selected_identity
        {
            return Err(wrong_binding(meter, at, locus));
        }
        Ok(operation)
    }

    fn require_concrete_step_anchor<'s>(
        &'s self,
        meter: &mut super::WorkMeter,
        source: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: Path<'_>,
        locus: &CheckedNodeId,
    ) -> Result<&'s str, ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner, ModelForm};
        self.require_local_kind(
            meter,
            source,
            K::State(super::StateForm::OperationAnchor),
            at,
            locus,
        )?;
        let Some((context, operation)) = self.local_anchor(source) else {
            return Err(wrong_binding(meter, at, locus));
        };
        let declaring_kind = self
            .index
            .get(context)
            .and_then(|&index| self.kinds.get(index));
        if !matches!(
            declaring_kind,
            Some(K::Model(
                ModelForm::ObjectType | ModelForm::Process | ModelForm::StateMachine
            ))
        ) {
            return Err(wrong_binding(meter, at, locus));
        }
        let selected_identity = self.local_node(selected_root).and_then(|root| {
            if let Some(CheckedNodeOwner::ModelRoot { identity, .. }) = &root.owner {
                Some(identity.as_ref())
            } else {
                None
            }
        });
        let declaring_identity = self.local_node(context).and_then(|declaration| {
            if let Some(CheckedNodeOwner::Model { identity, .. }) = &declaration.owner {
                Some(identity.as_ref())
            } else {
                None
            }
        });
        if selected_identity.is_none() || selected_identity != declaring_identity {
            return Err(wrong_binding(meter, at, locus));
        }
        Ok(operation)
    }

    fn require_concrete_owner(
        &self,
        meter: &mut super::WorkMeter,
        declaration: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: Path<'_>,
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
            Err(wrong_binding(meter, at, locus))
        }
    }

    fn require_concrete_class_source(
        &self,
        source: &CheckedNodeId,
        selected_root: &CheckedNodeId,
        at: Path<'_>,
        locus: &CheckedNodeId,
        meter: &mut super::WorkMeter,
    ) -> Result<(), ValidationFailure> {
        use super::{CheckedNodeKind as K, CheckedNodeOwner};
        let kind = self.local_kind(meter, source, at, locus)?;
        if kind == K::State(super::StateForm::OperationAnchor) {
            self.require_concrete_step_anchor(meter, source, selected_root, at, locus)?;
            return Ok(());
        }
        if matches!(kind, K::Model(_)) {
            return self.require_concrete_owner(meter, source, selected_root, at, locus);
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
                charge_at(meter, at)?;
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
            return Err(wrong_binding(meter, at, locus));
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
        let declared_identity = self.local_owner_identity(clause);
        if selected_identity.is_some() && selected_identity == declared_identity {
            Ok(())
        } else {
            Err(wrong_binding(meter, at, locus))
        }
    }
}

fn wrong_binding(
    meter: &mut super::WorkMeter,
    at: Path<'_>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    refuse_at(
        meter,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        at,
        locus,
    )
}

fn missing_class(
    meter: &mut super::WorkMeter,
    at: Path<'_>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    refuse_at(
        meter,
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingName),
        at,
        locus,
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
        charge_at(meter, root)?;
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
        charge_at(meter, root)?;
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
        return Err(refuse_unlocated(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            root,
        ));
    }
    let Some(catalog) = class_catalog else {
        return Ok(());
    };
    if protocol.is_none() && catalog.concrete.is_empty() && catalog.r#abstract.is_empty() {
        return Err(refuse_unlocated(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            root,
        ));
    }
    for (index, entry) in catalog.concrete.iter().enumerate() {
        let at = root.key("concrete").index(index);
        charge_at(meter, at)?;
        if let RefinementConcreteClass::Declared {
            node, binder_types, ..
        } = entry
        {
            let node_at = at.key("node");
            if !class_node_kind(graph.local_kind(meter, node, node_at, locus)?) {
                return Err(wrong_binding(meter, node_at, locus));
            }
            for (binder_index, binder) in binder_types.iter().enumerate() {
                let binder_at = at.key("binder_types").index(binder_index);
                charge_at(meter, binder_at)?;
                if !class_type_kind(graph.local_kind(meter, binder, binder_at, locus)?) {
                    return Err(wrong_binding(meter, binder_at, locus));
                }
            }
        }
    }
    for (index, entry) in catalog.r#abstract.iter().enumerate() {
        let at = root.key("abstract").index(index);
        charge_at(meter, at)?;
        if let RefinementAbstractClass::Declared {
            node, binder_types, ..
        } = entry
        {
            let node_at = at.key("node");
            if !class_node_kind(graph.key_kind(meter, node, node_at, locus)?) {
                return Err(wrong_binding(meter, node_at, locus));
            }
            for (binder_index, binder) in binder_types.iter().enumerate() {
                let binder_at = at.key("binder_types").index(binder_index);
                charge_at(meter, binder_at)?;
                if !class_type_kind(graph.key_kind(meter, binder, binder_at, locus)?) {
                    return Err(wrong_binding(meter, binder_at, locus));
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
        return Err(refuse_unlocated(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    let root = node_at(position).key("body");
    let locus = &node.node_id;
    for (index, row) in populations.iter().enumerate() {
        let at = root.key("populations").index(index);
        charge_at(meter, at)?;
        graph.require_kind(
            meter,
            &row.r#abstract,
            K::Relation(RelationForm::Population),
            at.key("abstract"),
            locus,
        )?;
        graph.require_local_kind(
            meter,
            &row.concrete,
            K::Relation(RelationForm::Population),
            at.key("concrete"),
            locus,
        )?;
        graph.require_concrete_owner(
            meter,
            &row.concrete,
            &node.semantic_type,
            at.key("concrete"),
            locus,
        )?;
    }
    for (index, row) in objects.iter().enumerate() {
        let at = root.key("objects").index(index);
        charge_at(meter, at)?;
        graph.require_kind(
            meter,
            &row.abstract_type,
            K::Model(ModelForm::ObjectType),
            at.key("abstract_type"),
            locus,
        )?;
        graph.require_local_kind(
            meter,
            &row.concrete_type,
            K::Model(ModelForm::ObjectType),
            at.key("concrete_type"),
            locus,
        )?;
        graph.require_concrete_owner(
            meter,
            &row.concrete_type,
            &node.semantic_type,
            at.key("concrete_type"),
            locus,
        )?;
        let known = abstract_fields(&row.abstract_type, owners, graph.dependencies);
        for (field_index, field) in row.fields.iter().enumerate() {
            let field_at = at.key("fields").index(field_index);
            charge_at(meter, field_at)?;
            if !known
                .as_ref()
                .is_some_and(|names| names.contains(&field.name))
            {
                return Err(refuse_at(
                    meter,
                    CheckedPackageRefusalCode::MissingDeclaration,
                    Some(CheckedPackageRefusalCause::MissingName),
                    field_at.key("name"),
                    locus,
                ));
            }
        }
    }
    for (index, row) in steps.iter().enumerate() {
        let at = root.key("steps").index(index);
        charge_at(meter, at)?;
        if let Some(catalog) = class_catalog {
            let Some(class) = catalog
                .concrete
                .iter()
                .find(|entry| entry.class() == row.source_class.as_ref())
            else {
                return Err(missing_class(meter, at.key("source_class"), locus));
            };
            let matches_node = match (class, &row.source_node) {
                (RefinementConcreteClass::Declared { node, .. }, Some(source)) => node == source,
                (RefinementConcreteClass::Memory(_), None) => true,
                _ => false,
            };
            if !matches_node {
                return Err(wrong_binding(meter, at.key("source_node"), locus));
            }
            if let Some(source) = &row.source_node {
                graph.require_concrete_class_source(
                    source,
                    &node.semantic_type,
                    at.key("source_node"),
                    locus,
                    meter,
                )?;
            }
        } else if let Some(source) = &row.source_node {
            let operation = graph.require_concrete_step_anchor(
                meter,
                source,
                &node.semantic_type,
                at.key("source_node"),
                locus,
            )?;
            if row.source_class.as_ref() != operation {
                return Err(wrong_binding(meter, at.key("source_class"), locus));
            }
        } else {
            return Err(missing_class(meter, at.key("source_class"), locus));
        }
        let target_at = at.key("target");
        match &row.target {
            RefinementStepTarget::Operation { operation, .. } => {
                graph.require_anchor_subject(
                    meter,
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
                    .ok_or_else(|| missing_class(meter, target_at.key("class"), locus))?;
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
                    return Err(wrong_binding(meter, target_at.key("node"), locus));
                }
                let expected = binders.map_or(0, <[_]>::len);
                if !arguments.is_empty() && arguments.len() != expected {
                    return Err(refuse_at(
                        meter,
                        CheckedPackageRefusalCode::IllTyped,
                        Some(CheckedPackageRefusalCause::OperatorIneligible),
                        target_at.key("arguments"),
                        locus,
                    ));
                }
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for (index, row) in histories.iter().enumerate() {
        let at = root.key("histories").index(index);
        charge_at(meter, at)?;
        graph.require_local_kind(
            meter,
            &row.concrete_type,
            K::Model(ModelForm::ObjectType),
            at.key("concrete_type"),
            locus,
        )?;
        graph.require_concrete_owner(
            meter,
            &row.concrete_type,
            &node.semantic_type,
            at.key("concrete_type"),
            locus,
        )?;
        graph.local_kind(meter, &row.type_id, at.key("type"), locus)?;
        if let CheckedTerm::Reference { target } = &row.initial {
            graph.require_local_kind(
                meter,
                target,
                K::Value(super::ValueForm::Literal),
                at.key("initial").key("target"),
                locus,
            )?;
        }
        for (update_index, update) in row.updates.iter().enumerate() {
            let update_at = at.key("updates").index(update_index);
            charge_at(meter, update_at)?;
            graph.require_concrete_step_anchor(
                meter,
                &update.operation,
                &node.semantic_type,
                update_at.key("operation"),
                locus,
            )?;
        }
    }
    for (index, row) in assume.iter().enumerate() {
        let at = root.key("assume").index(index);
        charge_at(meter, at)?;
        graph.require_concrete_step_anchor(
            meter,
            &row.operation,
            &node.semantic_type,
            at.key("operation"),
            locus,
        )?;
    }
    for (index, row) in ensure.iter().enumerate() {
        let at = root.key("ensure").index(index);
        charge_at(meter, at)?;
        graph.require_anchor_subject(
            meter,
            &row.operation,
            abstract_key,
            at.key("operation"),
            locus,
        )?;
    }
    Ok(())
}

fn check_headers(
    meter: &mut super::WorkMeter,
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
        return Err(refuse_unlocated(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_at(position).key("body"),
        ));
    };
    let root = node_at(position);
    let locus = &node.node_id;
    graph.require_local_kind(
        meter,
        &node.semantic_type,
        K::Model(ModelForm::ModelRoot),
        root.key("semantic_type"),
        locus,
    )?;
    graph.require_kind(
        meter,
        abstract_key,
        K::Model(ModelForm::ModelRoot),
        root.key("body").key("abstract"),
        locus,
    )?;
    if let Some(protocol) = protocol {
        graph.require_kind(
            meter,
            protocol,
            K::Protocol(ProtocolForm::ProtocolClause),
            root.key("body").key("protocol"),
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
                meter,
                root.key("body").key("abstract"),
                locus,
            ));
        };
        if graph.key_owner_identity(protocol) != Some(identity.as_ref()) {
            return Err(wrong_binding(
                meter,
                root.key("body").key("protocol"),
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
        return Err(refuse_at(
            meter,
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            root.key("body").key("profile"),
            locus,
        ));
    }
    Ok(())
}

fn abstract_fields<'a>(
    key: &RefinementKey,
    owners: &'a super::model_members::ModelOwners<'_>,
    dependencies: &'a super::SuppliedDependencies<'_>,
) -> Option<&'a [Box<str>]> {
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
        charge_amount_at(
            meter,
            node_at(position).key("body").key("objects").index(index),
            u64::try_from(fields.len()).unwrap_or(u64::MAX),
        )?;
        let mut mapped: Vec<&str> = meter.vec(object.fields.len())?;
        mapped.extend(object.fields.iter().map(|field| field.name.as_ref()));
        mapped.sort_unstable();
        if fields
            .iter()
            .any(|field| mapped.binary_search(&field.as_ref()).is_err())
        {
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
    meter.at_node(0, "refinement");
    let refinements = (0..nodes.len())
        .filter(|&position| {
            kinds[position] == super::CheckedNodeKind::Relation(super::RelationForm::Refinement)
        })
        .count();
    let mut entries = meter.vec(refinements)?;
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != super::CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
        if let Some(CheckedTerm::Refinement { steps, .. }) = terms[position].as_ref() {
            charge_amount_at(
                meter,
                node_at(position).key("body").key("steps"),
                u64::try_from(steps.len()).unwrap_or(u64::MAX),
            )?;
            entries.push((&node.node_id, steps.len()));
        }
    }
    // Node ids are distinct, so each has its one step count.
    let expected = SortedTable::from_vec(entries, |left, right| right.cmp(left));
    let count = expected
        .iter()
        .try_fold(0usize, |total, (_, len)| total.checked_add(*len))
        .ok_or_else(|| invalid_step_rows(meter, None))?;
    if count == 0 && rows.is_none() {
        return Ok(());
    }
    if count == 0 {
        return Err(invalid_step_rows(meter, None));
    }
    let Some(rows) = rows else {
        return Err(invalid_step_rows(meter, None));
    };
    charge_amount_at(
        meter,
        Path::root().key("lock").key("sources"),
        u64::try_from(locked_sources.len()).unwrap_or(u64::MAX),
    )?;
    let mut locked: Vec<&CheckedSourceRef> = meter.vec(locked_sources.len())?;
    locked.extend(locked_sources.iter());
    locked.sort_unstable();
    let mut previous: Option<(&CheckedNodeId, u64)> = None;
    for (index, row) in rows.iter().enumerate() {
        charge_at(meter, {
            Path::root().key("refinement_step_rows").index(index)
        })?;
        let in_range = expected
            .get(&&row.node_id)
            .is_some_and(|len| usize::try_from(row.ordinal).is_ok_and(|ordinal| ordinal < *len));
        let ordered =
            previous.is_none_or(|(node, ordinal)| (node, ordinal) < (&row.node_id, row.ordinal));
        if !in_range || !ordered {
            return Err(invalid_step_rows(meter, Some(index)));
        }
        if locked.binary_search(&&row.region.source).is_err() {
            return Err(invalid_step_row_region(meter, index, true));
        }
        if row.region.start >= row.region.end {
            return Err(invalid_step_row_region(meter, index, false));
        }
        previous = Some((&row.node_id, row.ordinal));
    }
    if rows.len() != count {
        return Err(invalid_step_rows(meter, None));
    }
    Ok(())
}

fn invalid_step_row_region(
    meter: &mut super::WorkMeter,
    index: usize,
    source: bool,
) -> ValidationFailure {
    let region = Path::root()
        .key("refinement_step_rows")
        .index(index)
        .key("region");
    refuse_because(
        meter,
        CheckedPackageRefusalCode::InvalidPackage,
        if source { region.key("source") } else { region },
        CheckedPackageRefusalCause::InvalidValue,
    )
}

fn invalid_step_rows(meter: &mut super::WorkMeter, index: Option<usize>) -> ValidationFailure {
    let path = Path::root().key("refinement_step_rows");
    let path = index.map_or(path, |index| path.index(index));
    refuse_because(
        meter,
        CheckedPackageRefusalCode::InvalidPackage,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}

/// Check the source-map occurrences against declaration and step-row regions.
pub(super) fn validate_occurrences(
    graph: RefinementGraph<'_, '_>,
    source_map: &[CheckedSourceMapEntry],
    rows: Option<&[CheckedRefinementStepRowRegion]>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    meter.at_node(0, "refinement");
    let mut row_entries = meter.vec(rows.map_or(0, <[_]>::len))?;
    if let Some(rows) = rows {
        for (index, row) in rows.iter().enumerate() {
            charge_at(meter, {
                Path::root().key("refinement_step_rows").index(index)
            })?;
            row_entries.push(((&row.node_id, row.ordinal), &row.region));
        }
    }
    // Of two rows with one key the later is kept, as a map would hold.
    let row_regions = SortedTable::from_vec(row_entries, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let is_declaration = |entry: &&CheckedSourceMapEntry| {
        entry.role == CheckedOccurrenceRole::Declaration && entry.ordinal == 0
    };
    let mut declaration_entries = meter.vec(source_map.iter().filter(is_declaration).count())?;
    for (index, entry) in source_map.iter().enumerate() {
        charge_at(meter, Path::root().key("source_map").index(index))?;
        if is_declaration(&entry) {
            declaration_entries.push((&entry.node_id, entry.regions.as_slice()));
        }
    }
    let declarations = SortedTable::from_vec(declaration_entries, |left, right| {
        right.as_ptr().cmp(&left.as_ptr())
    });
    let is_refinement = |position: &usize| {
        graph.kinds[*position] == super::CheckedNodeKind::Relation(super::RelationForm::Refinement)
    };
    let mut refinement_positions: Vec<usize> =
        meter.vec((0..graph.nodes.len()).filter(is_refinement).count())?;
    refinement_positions.extend((0..graph.nodes.len()).filter(is_refinement));
    refinement_positions.sort_unstable_by(|&left, &right| {
        graph
            .index
            .resolved_at(left, graph.structural_keys)
            .cmp(&graph.index.resolved_at(right, graph.structural_keys))
    });
    for position in refinement_positions {
        let node = &graph.nodes[position];
        let Some(body) = graph.terms[position].as_ref() else {
            continue;
        };
        let Some(generated_count) =
            proof_eligible(body, owners, graph.dependencies, meter, position)?
        else {
            continue;
        };
        let declaration = declarations.get(&&node.node_id);
        let mut present: Vec<(CheckedOccurrenceRole, u64)> = Vec::new();
        for (entry_index, entry) in source_map.iter().enumerate() {
            charge_at(meter, Path::root().key("source_map").index(entry_index))?;
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
                return Err(refuse_at(
                    meter,
                    CheckedPackageRefusalCode::InvalidPackage,
                    Some(CheckedPackageRefusalCause::InvalidValue),
                    Path::root().key("source_map").index(entry_index),
                    &node.node_id,
                ));
            }
            meter.insert_sorted(&mut present, (entry.role.clone(), entry.ordinal))?;
        }
        let has = |role: CheckedOccurrenceRole, ordinal: u64| {
            present.binary_search(&(role, ordinal)).is_ok()
        };
        if !has(CheckedOccurrenceRole::Declaration, 0)
            || (0..generated_count).any(|ordinal| {
                u64::try_from(ordinal).map_or(true, |ordinal| {
                    !has(CheckedOccurrenceRole::Generated, ordinal)
                })
            })
        {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidPackage,
                Some(CheckedPackageRefusalCause::InvalidValue),
                Path::root().key("source_map"),
                &node.node_id,
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
                refuse_unlocated(
                    meter,
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_at(position).key("body"),
                )
            })?;
        charge_at(meter, node_at(position))?;
        canonical_order(body, position, &graph, meter)?;
        check_headers(meter, node, body, position, &graph, lock)?;
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
        meter.at_node(position, "refinement");
        let local = local_protocol_nodes(&graph, meter)?;
        let mut concrete = protocol_classes(
            &local,
            &graph.nodes[position].semantic_type,
            None,
            None,
            CatalogSignatures::Local(owners),
            root,
            meter,
        )?;
        if let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = graph
            .local_node(&graph.nodes[position].semantic_type)
            .and_then(|node| node.owner.as_ref())
        {
            for operation in owners.operation_classes(identity, meter)? {
                let class = operation_class(operation, None, meter)?;
                meter.push(&mut concrete, class)?;
            }
            concrete.sort_unstable_by(|left, right| left.class.cmp(&right.class));
        }
        compare_declared_classes(
            &catalog.concrete,
            &concrete,
            &graph,
            root.key("concrete"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        let concrete_memory = protocol_memory_models(
            &local,
            &graph.nodes[position].semantic_type,
            None,
            root,
            meter,
        )?;
        compare_concrete_memory(
            &catalog.concrete,
            concrete_memory,
            root.key("concrete"),
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
                        root,
                        meter,
                    )?,
                    protocol_memory_models(&local, model, Some(clause), root, meter)?,
                ),
                RefinementKey::External(_) => (Vec::new(), MemoryModels::default()),
            },
            Some(RefinementKey::External(clause)) => match abstract_key.as_ref() {
                RefinementKey::External(model) if model.package == clause.package => {
                    let selected = graph
                        .dependencies
                        .selected_package(&clause.package)
                        .ok_or_else(|| {
                            refuse_at(
                                meter,
                                CheckedPackageRefusalCode::InvalidModelBinding,
                                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                                node_at(position).key("body").key("protocol"),
                                &graph.nodes[position].node_id,
                            )
                        })?;
                    let dependency = dependency_protocol_nodes(selected, meter)?;
                    (
                        protocol_classes(
                            &dependency,
                            &model.node,
                            Some(&clause.node),
                            Some(&clause.package),
                            CatalogSignatures::External(&selected.models),
                            root,
                            meter,
                        )?,
                        protocol_memory_models(
                            &dependency,
                            &model.node,
                            Some(&clause.node),
                            root,
                            meter,
                        )?,
                    )
                }
                _ => (Vec::new(), MemoryModels::default()),
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
                                .resolved_graph()
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
                let package = match abstract_key.as_ref() {
                    RefinementKey::Local(_) => None,
                    RefinementKey::External(model) => Some(model.package.as_ref()),
                };
                let mut classes = meter.vec(operations.len())?;
                for operation in operations {
                    classes.push(operation_class(operation, package, meter)?);
                }
                classes.sort_unstable_by(|left, right| left.class.cmp(&right.class));
                (classes, MemoryModels::default())
            }
        };
        compare_abstract_classes(
            &catalog.r#abstract,
            &abstract_classes,
            &graph,
            root.key("abstract"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        compare_abstract_memory(
            &catalog.r#abstract,
            abstract_memory,
            root.key("abstract"),
            &graph.nodes[position].node_id,
            meter,
        )?;
    }
    Ok(())
}

struct DeclaredGraphClass {
    class: String,
    node: RefinementKey,
    binder_types: BinderSignature<RefinementKey>,
}

enum BinderSignature<T> {
    Known(Vec<T>),
    Unspecified,
    Ambiguous,
}

impl<T> BinderSignature<T> {
    fn map<U>(
        self,
        meter: &mut super::WorkMeter,
        mut convert: impl FnMut(T, &mut super::WorkMeter) -> Result<U, ValidationFailure>,
    ) -> Result<BinderSignature<U>, ValidationFailure> {
        Ok(match self {
            Self::Known(types) => {
                let mut mapped = meter.vec(types.len())?;
                for member in types {
                    mapped.push(convert(member, meter)?);
                }
                BinderSignature::Known(mapped)
            }
            Self::Unspecified => BinderSignature::Unspecified,
            Self::Ambiguous => BinderSignature::Ambiguous,
        })
    }
}

impl BinderSignature<RefinementKey> {
    fn duplicate(&self, meter: &mut super::WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(match self {
            Self::Known(types) => {
                let mut copied = meter.vec(types.len())?;
                for key in types {
                    let copied_key = match key {
                        RefinementKey::Local(id) => copy_class_key(id, None, meter)?,
                        RefinementKey::External(key) => {
                            copy_class_key(&key.node, Some(&key.package), meter)?
                        }
                    };
                    copied.push(copied_key);
                }
                Self::Known(copied)
            }
            Self::Unspecified => Self::Unspecified,
            Self::Ambiguous => Self::Ambiguous,
        })
    }
}

fn owned_class_key(
    node: CheckedNodeId,
    package: Option<&str>,
    meter: &mut super::WorkMeter,
) -> Result<RefinementKey, ValidationFailure> {
    Ok(match package {
        Some(package) => RefinementKey::External(RefinementPackageNodeKey {
            package: meter.boxed_str(package)?,
            node,
        }),
        None => RefinementKey::Local(node),
    })
}

fn copy_class_key(
    node: &CheckedNodeId,
    package: Option<&str>,
    meter: &mut super::WorkMeter,
) -> Result<RefinementKey, ValidationFailure> {
    let node = meter.clone_id(node)?;
    owned_class_key(node, package, meter)
}

fn operation_class(
    operation: super::model_members::ModelOperationClass,
    package: Option<&str>,
    meter: &mut super::WorkMeter,
) -> Result<DeclaredGraphClass, ValidationFailure> {
    let class = meter
        .formatted(format_args!(
            "{}::{}",
            operation.object, operation.operation
        ))?
        .into_string();
    let node = owned_class_key(operation.node, package, meter)?;
    let binder_types = match operation.binder_types {
        Some(types) => BinderSignature::Known(types)
            .map(meter, |id, meter| owned_class_key(id, package, meter))?,
        None => BinderSignature::Unspecified,
    };
    Ok(DeclaredGraphClass {
        class,
        node,
        binder_types,
    })
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
        meter: &mut super::WorkMeter,
    ) -> Result<Option<super::model_fields::RecordBinderTypes>, ValidationFailure> {
        match self {
            Self::Local(owners) => {
                let Some(owner) = owners.record_by_key(key) else {
                    return Ok(None);
                };
                let mut budget = super::model_members::Budget::new(
                    meter,
                    owner.package.root,
                    owner.package.bytes,
                );
                owner.package.record_binder_types(owner.node, &mut budget)
            }
            Self::External(models) => models.record_binder_types(key, meter),
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

fn local_protocol_nodes<'a>(
    graph: &'a RefinementGraph<'_, '_>,
    meter: &mut super::WorkMeter,
) -> Result<Vec<ProtocolCatalogNode<'a>>, ValidationFailure> {
    let mut nodes = meter.vec(graph.index.len())?;
    nodes.extend(graph.index.values().map(|&position| {
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
    }));
    Ok(nodes)
}

fn dependency_protocol_nodes<'a>(
    package: &'a super::CheckedPackageV2,
    meter: &mut super::WorkMeter,
) -> Result<Vec<ProtocolCatalogNode<'a>>, ValidationFailure> {
    let graph = package.resolved_graph();
    let mut nodes = meter.vec(graph.nodes.len())?;
    nodes.extend(
        graph
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
            }),
    );
    Ok(nodes)
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
    at: Path<'_>,
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
    charge_amount_at(meter, at, super::model_members::units(nodes.len()))?;
    let clause_count = nodes
        .iter()
        .filter(|node| {
            node.kind == super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
                && node.name.is_some()
        })
        .count();
    let mut clause_paths = meter.vec(clause_count)?;
    clause_paths.extend(
        nodes
            .iter()
            .filter(|node| {
                node.kind == super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
            })
            .filter_map(|node| {
                node.name
                    .map(|name| (node.id, name.qualified_name.as_slice()))
            }),
    );
    let mut closest_clauses = meter.vec(nodes.len())?;
    for member in nodes {
        let closest = if matches!(member.kind, super::CheckedNodeKind::Protocol(_)) {
            if let Some(name) = member.name {
                charge_amount_at(meter, at, super::model_members::units(clause_paths.len()))?;
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
                    charge_amount_at(meter, at, super::model_members::units(nodes.len()))?;
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
                        let class = meter
                            .formatted(format_args!("activate {name}"))?
                            .into_string();
                        let node = copy_class_key(clause.id, package, meter)?;
                        let binder_types = binder_signature(trigger_type, signatures, meter)?
                            .map(meter, |id, meter| owned_class_key(id, package, meter))?;
                        meter.push(
                            &mut classes,
                            DeclaredGraphClass {
                                class,
                                node,
                                binder_types,
                            },
                        )?;
                    }
                }
            }
        }
        for (member, closest_clause) in nodes.iter().zip(&closest_clauses) {
            charge_at(meter, at)?;
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
            let Some(operator) = operation.member_operator() else {
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
                meter,
            )?
            else {
                continue;
            };
            let binder_types = if matches!(operator, CatalogOperator::Role) {
                BinderSignature::Unspecified
            } else {
                protocol_binder_types(operator, arguments, nodes, signatures, meter)?
                    .map(meter, |id, meter| owned_class_key(id, package, meter))?
            };
            for class in classes_for_node {
                let class_binders = if class.empty_binders {
                    BinderSignature::Known(Vec::new())
                } else {
                    binder_types.duplicate(meter)?
                };
                let node = copy_class_key(member.id, package, meter)?;
                meter.push(
                    &mut classes,
                    DeclaredGraphClass {
                        class: class.name,
                        node,
                        binder_types: class_binders,
                    },
                )?;
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
    at: Path<'_>,
    meter: &mut super::WorkMeter,
) -> Result<MemoryModels, ValidationFailure> {
    let mut entries = meter.vec(nodes.len())?;
    for node in nodes {
        charge_at(meter, at)?;
        entries.push((node.id, node));
    }
    // Of two nodes with one id the later is kept, as a map would hold.
    let by_id = SortedTable::from_vec(entries, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let selected_identity = by_id.get(&model).and_then(|node| {
        if let Some(super::CheckedNodeOwner::ModelRoot { identity, .. }) = node.owner {
            Some(identity.as_ref())
        } else {
            None
        }
    });
    let mut models = MemoryModels::default();
    for clause in nodes {
        let clause_identity = clause
            .owner_model
            .and_then(|owner| by_id.get(&owner))
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
            charge_at(meter, at)?;
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
            if operation.member_operator() != Some("parallel") {
                continue;
            }
            let Some(CheckedTerm::Aggregate { members }) =
                super::protocol_body::binding_value(arguments, "memory")
            else {
                return Err(refuse_unlocated(
                    meter,
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at,
                ));
            };
            let selected = match members.as_slice() {
                [] => "sc",
                [value] => super::protocol_body::text_value(value).ok_or_else(|| {
                    refuse_unlocated(meter, CheckedPackageRefusalCode::InvalidSemanticGraph, at)
                })?,
                _ => {
                    return Err(refuse_unlocated(
                        meter,
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at,
                    ));
                }
            };
            let model = match selected {
                "sc" => RefinementMemoryModel::Sc,
                "tso" => RefinementMemoryModel::Tso,
                "ra" => RefinementMemoryModel::Ra,
                _ => {
                    return Err(refuse_unlocated(
                        meter,
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at,
                    ));
                }
            };
            models.insert(model);
        }
    }
    Ok(models)
}

/// The memory models a protocol selects, one bit per model. There are three,
/// so the set holds no heap block.
#[derive(Clone, Copy, Default)]
struct MemoryModels(u8);

impl MemoryModels {
    fn bit(model: RefinementMemoryModel) -> u8 {
        match model {
            RefinementMemoryModel::Sc => 1,
            RefinementMemoryModel::Tso => 2,
            RefinementMemoryModel::Ra => 4,
        }
    }

    fn insert(&mut self, model: RefinementMemoryModel) {
        self.0 |= Self::bit(model);
    }

    fn contains(self, model: RefinementMemoryModel) -> bool {
        self.0 & Self::bit(model) != 0
    }

    /// Removes `model`, reporting whether it was present.
    fn remove(&mut self, model: RefinementMemoryModel) -> bool {
        let present = self.contains(model);
        self.0 &= !Self::bit(model);
        present
    }
}

fn compare_memory<'a>(
    actual: impl Iterator<Item = (usize, &'a RefinementMemoryClass)>,
    models: MemoryModels,
    at: Path<'_>,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    // Each selected model has a drain; the total-store-order model a flush.
    let (mut drains, mut flush) = (models, models.contains(RefinementMemoryModel::Tso));
    for (index, memory) in actual {
        let entry_at = at.index(index);
        charge_at(meter, entry_at)?;
        let expected = match memory {
            RefinementMemoryClass::Drain { memory_model, .. } => drains.remove(*memory_model),
            RefinementMemoryClass::Flush { .. } => std::mem::take(&mut flush),
        };
        if !expected {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                entry_at,
                locus,
            ));
        }
    }
    if drains.0 != 0 || flush {
        return Err(refuse_at(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            None,
            at,
            locus,
        ));
    }
    Ok(())
}

fn compare_concrete_memory(
    actual: &[RefinementConcreteClass],
    models: MemoryModels,
    at: Path<'_>,
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
    models: MemoryModels,
    at: Path<'_>,
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
            .operation_binder_types(anchor.context, anchor.operation, meter)?
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
    binder_signature(binder_type, signatures, meter)
}

fn binder_signature(
    binder_type: &CheckedNodeId,
    signatures: CatalogSignatures<'_, '_>,
    meter: &mut super::WorkMeter,
) -> Result<BinderSignature<CheckedNodeId>, ValidationFailure> {
    Ok(if signatures.is_record(binder_type) {
        match signatures.record_binder_types(binder_type, meter)? {
            Some(super::model_fields::RecordBinderTypes::Known(types)) => {
                BinderSignature::Known(types)
            }
            Some(super::model_fields::RecordBinderTypes::Ambiguous) => BinderSignature::Ambiguous,
            None => BinderSignature::Unspecified,
        }
    } else {
        let mut types = meter.vec(1)?;
        types.push(meter.clone_id(binder_type)?);
        BinderSignature::Known(types)
    })
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
    meter: &mut super::WorkMeter,
) -> Result<Option<Vec<ProtocolMemberClass>>, ValidationFailure> {
    let Some(path) = protocol_clause_path(clause_name) else {
        return Ok(None);
    };
    let Some(member) = name.last() else {
        return Ok(None);
    };
    let prefixes: &[(&str, bool)] = match operator {
        CatalogOperator::Parallel => &[("fork ", false), ("join ", false)],
        CatalogOperator::Compensation => &[("", false), ("cend ", true)],
        CatalogOperator::Attempt
        | CatalogOperator::Event
        | CatalogOperator::Send
        | CatalogOperator::Receive
        | CatalogOperator::Fence
        | CatalogOperator::TimeoutFromAwait
        | CatalogOperator::Finish => &[("", false)],
        CatalogOperator::Channel => {
            let Some(delivery) = super::protocol_body::binding_value(arguments, "delivery")
                .and_then(super::protocol_body::text_value)
                .and_then(ChannelDelivery::parse)
            else {
                return Ok(None);
            };
            match delivery {
                ChannelDelivery::Unknown => &[("duplicate ", false), ("lose ", false)],
                ChannelDelivery::AtLeastOnce => &[("duplicate ", false)],
                ChannelDelivery::AtMostOnce => &[("lose ", false)],
                ChannelDelivery::ExactlyOncePremise | ChannelDelivery::Between { .. } => &[],
            }
        }
        CatalogOperator::Role => {
            let Some(replication @ CheckedTerm::Aggregate { members }) =
                super::protocol_body::binding_value(arguments, "replication")
            else {
                return Ok(None);
            };
            if members.is_empty() {
                &[]
            } else if super::protocol_body::group_value(replication, "lifetime")
                .and_then(super::protocol_body::text_value)
                == Some("until")
            {
                &[("spawn ", false), ("retire ", false)]
            } else {
                &[("spawn ", false)]
            }
        }
    };
    let mut classes = meter.vec(prefixes.len())?;
    for (prefix, empty_binders) in prefixes {
        let name = meter
            .formatted(format_args!("{prefix}{path}::{member}"))?
            .into_string();
        classes.push(ProtocolMemberClass {
            name,
            empty_binders: *empty_binders,
        });
    }
    Ok(Some(classes))
}

/// A clause name rendered from its borrowed segments without an intermediate list or string.
struct ProtocolClausePath<'a>(&'a [Box<str>]);
impl std::fmt::Display for ProtocolClausePath<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, segment) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str("::")?;
            }
            formatter.write_str(segment)?;
        }
        Ok(())
    }
}
fn protocol_clause_path(name: &[Box<str>]) -> Option<ProtocolClausePath<'_>> {
    (!name.is_empty()).then_some(ProtocolClausePath(name))
}

fn compare_declared_classes(
    actual: &[RefinementConcreteClass],
    expected: &[DeclaredGraphClass],
    graph: &RefinementGraph<'_, '_>,
    at: Path<'_>,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected.iter().peekable();
    for (index, entry) in actual.iter().enumerate() {
        if matches!(entry, RefinementConcreteClass::Memory(_)) {
            continue;
        }
        let entry_at = at.index(index);
        charge_at(meter, entry_at)?;
        if expected
            .peek()
            .is_some_and(|item| item.class.as_str() < entry.class())
        {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                at,
                locus,
            ));
        }
        let Some(found) = expected.next() else {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                entry_at,
                locus,
            ));
        };
        if found.class != entry.class() {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                entry_at,
                locus,
            ));
        }
        match entry {
            RefinementConcreteClass::Declared {
                node, binder_types, ..
            } => {
                if matches!(found.binder_types, BinderSignature::Ambiguous) {
                    return Err(refuse_at(
                        meter,
                        CheckedPackageRefusalCode::AmbiguousDeclaration,
                        Some(CheckedPackageRefusalCause::AmbiguousName),
                        entry_at.key("binder_types"),
                        locus,
                    ));
                }
                let mut same = found.node.local() == Some(node);
                if let (true, BinderSignature::Known(expected)) = (same, &found.binder_types) {
                    same = binder_types.len() == expected.len();
                    for (id, key) in binder_types.iter().zip(expected) {
                        if !same {
                            break;
                        }
                        same = graph
                            .index
                            .resolved(id, graph.structural_keys)
                            .order(&graph.resolved_key(key), meter)?
                            == std::cmp::Ordering::Equal;
                    }
                }
                if !same {
                    return Err(wrong_binding(meter, entry_at, locus));
                }
            }
            RefinementConcreteClass::Memory(_) => unreachable!("memory entry skipped above"),
        }
    }
    if expected.next().is_some() {
        return Err(refuse_at(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            None,
            at,
            locus,
        ));
    }
    Ok(())
}

fn compare_abstract_classes(
    actual: &[RefinementAbstractClass],
    expected: &[DeclaredGraphClass],
    graph: &RefinementGraph<'_, '_>,
    at: Path<'_>,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected.iter().peekable();
    for (index, entry) in actual.iter().enumerate() {
        if matches!(entry, RefinementAbstractClass::Memory(_)) {
            continue;
        }
        let entry_at = at.index(index);
        charge_at(meter, entry_at)?;
        if expected
            .peek()
            .is_some_and(|item| item.class.as_str() < entry.class())
        {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                at,
                locus,
            ));
        }
        let Some(found) = expected.next() else {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                entry_at,
                locus,
            ));
        };
        if found.class != entry.class() {
            return Err(refuse_at(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                None,
                entry_at,
                locus,
            ));
        }
        match entry {
            RefinementAbstractClass::Declared {
                node, binder_types, ..
            } => {
                if matches!(found.binder_types, BinderSignature::Ambiguous) {
                    return Err(refuse_at(
                        meter,
                        CheckedPackageRefusalCode::AmbiguousDeclaration,
                        Some(CheckedPackageRefusalCause::AmbiguousName),
                        entry_at.key("binder_types"),
                        locus,
                    ));
                }
                let mut same = node == &found.node;
                if let (true, BinderSignature::Known(expected)) = (same, &found.binder_types) {
                    same = binder_types.len() == expected.len();
                    for (actual, expected) in binder_types.iter().zip(expected) {
                        if !same {
                            break;
                        }
                        same = graph.same_key(actual, expected, meter)?;
                    }
                }
                if !same {
                    return Err(wrong_binding(meter, entry_at, locus));
                }
            }
            RefinementAbstractClass::Memory(_) => unreachable!("memory entry skipped above"),
        }
    }
    if expected.next().is_some() {
        return Err(refuse_at(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            None,
            at,
            locus,
        ));
    }
    Ok(())
}

fn add_key(
    targets: &mut Vec<CheckedNodeId>,
    key: &RefinementKey,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    if let Some(local) = key.local() {
        meter.insert_id(targets, local)?;
    }
    Ok(())
}

fn add_term(
    targets: &mut Vec<CheckedNodeId>,
    term: &CheckedTerm,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut failure = None;
    let mut found: Vec<&CheckedNodeId> = Vec::new();
    // The targets are borrowed from the term; the walk allocates nothing
    // itself, so the borrowed list is charged as it grows.
    term.visit_reference_targets(&mut |target| {
        if failure.is_none() {
            if let Err(refusal) = meter.push(&mut found, target) {
                failure = Some(refusal);
            }
        }
    });
    if let Some(refusal) = failure {
        return Err(refusal);
    }
    for target in found {
        meter.insert_id(targets, target)?;
    }
    let mut application = Vec::new();
    if super::structural::collect_application_dependencies(term, &mut application, meter)? {
        for id in application {
            meter.insert_owned_id(targets, id)?;
        }
    }
    Ok(())
}

fn local_dependencies(
    body: &CheckedTerm,
    meter: &mut super::WorkMeter,
) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
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
        return Ok(None);
    };
    let mut targets: Vec<CheckedNodeId> = Vec::new();
    add_key(&mut targets, abstract_key, meter)?;
    if let Some(protocol) = protocol {
        add_key(&mut targets, protocol, meter)?;
    }
    for row in populations {
        add_key(&mut targets, &row.r#abstract, meter)?;
        meter.insert_id(&mut targets, &row.concrete)?;
    }
    for row in objects {
        add_key(&mut targets, &row.abstract_type, meter)?;
        meter.insert_id(&mut targets, &row.concrete_type)?;
        for field in &row.fields {
            add_term(&mut targets, &field.expression, meter)?;
        }
    }
    for row in steps {
        if let Some(node) = &row.source_node {
            meter.insert_id(&mut targets, node)?;
        }
        match &row.target {
            RefinementStepTarget::Operation {
                operation,
                receiver,
                arguments,
            } => {
                add_key(&mut targets, operation, meter)?;
                add_term(&mut targets, receiver, meter)?;
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        add_term(&mut targets, term, meter)?;
                    }
                }
            }
            RefinementStepTarget::ProtocolStep {
                node, arguments, ..
            } => {
                if let Some(node) = node {
                    add_key(&mut targets, node, meter)?;
                }
                for argument in arguments {
                    if let RefinementStepArgument::Term(term) = argument {
                        add_term(&mut targets, term, meter)?;
                    }
                }
            }
            RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
        }
    }
    for row in histories {
        meter.insert_id(&mut targets, &row.concrete_type)?;
        meter.insert_id(&mut targets, &row.type_id)?;
        add_term(&mut targets, &row.initial, meter)?;
        for update in &row.updates {
            meter.insert_id(&mut targets, &update.operation)?;
            add_term(&mut targets, &update.expression, meter)?;
        }
    }
    for row in assume {
        meter.insert_id(&mut targets, &row.operation)?;
    }
    for row in ensure {
        add_key(&mut targets, &row.operation, meter)?;
    }
    if let Some(catalog) = class_catalog {
        for entry in &catalog.concrete {
            if let RefinementConcreteClass::Declared {
                node, binder_types, ..
            } = entry
            {
                meter.insert_id(&mut targets, node)?;
                for binder in binder_types {
                    meter.insert_id(&mut targets, binder)?;
                }
            }
        }
        for entry in &catalog.r#abstract {
            if let RefinementAbstractClass::Declared {
                node, binder_types, ..
            } = entry
            {
                add_key(&mut targets, node, meter)?;
                for binder in binder_types {
                    add_key(&mut targets, binder, meter)?;
                }
            }
        }
    }
    Ok(Some(targets))
}

/// Check the exact local dependency vector at the graph-shape stage.
pub(super) fn validate_dependencies(
    nodes: &[super::RawSemanticNodeV2],
    kinds: &[super::CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    for (position, ((node, kind), term)) in nodes.iter().zip(kinds).zip(terms).enumerate() {
        if *kind != super::CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
        meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
        let expected = match term {
            Some(term) => local_dependencies(term, meter)?,
            None => None,
        }
        .ok_or_else(|| {
            refuse_unlocated(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_at(position).key("body"),
            )
        })?;
        let mut keys = meter.vec(expected.len())?;
        keys.extend(
            expected
                .iter()
                .map(|id| index.resolved(id, structural_keys)),
        );
        keys.sort_unstable();
        keys.dedup();
        if !node
            .dependencies
            .iter()
            .map(|id| index.resolved(id, structural_keys))
            .eq(keys)
        {
            return Err(refuse_unlocated(
                meter,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_at(position).key("dependencies"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-500", "FR-038-AC-511")]
    #[test]
    fn refinement_row_walks_charge_the_callers_budget() {
        let id = |name| json!({"package":"refinement-test","path":[name],"ordinal":0});
        let wire = json!({
            "term":"refinement", "abstract":id("a"),
            "profile":{"authority":"test","identity":"quire.temporal.infinite-trace/v1"},
            "populations":[], "objects":[], "steps":[],
            "histories":[{"concrete_type":id("b"),"name":"past","type":id("c"),
                "initial":{"term":"reference","target":id("d")},
                "updates":[{"operation":id("e"),"expression":{
                    "term":"aggregate","members":[{"term":"aggregate","members":[
                        {"term":"reference","target":id("a")}
                    ]}]
                }}]}], "assume":[], "ensure":[]
        });
        let typed = CheckedTerm::decode_admitted(&wire).expect("closed refinement");
        let document = super::super::CheckedJson::from_value(&wire).expect("document");
        let run = |limit| {
            let mut budget =
                super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let meter = super::super::read_charge::Meter::new(&mut budget);
                let site = super::super::read_charge::Site::new(
                    super::super::PackageReadStage::WireChecks,
                    7,
                    "refinement",
                );
                outcome = Some(check_wire(
                    super::super::TermNode::of(&document),
                    Some(&typed),
                    &Trail::Base(&[]),
                    &mut meter.reservation(site),
                ));
            });
            (
                outcome.expect("ran"),
                measured.bytes_total,
                budget.read_storage_consumed(),
                budget.first_unpaid_locus(),
            )
        };
        let (outcome, allocated, charged, _) = run(u64::MAX);
        outcome.expect("refinement rows admitted");
        assert!(allocated > 0);
        assert_eq!(allocated, charged);
        let (outcome, allocated, paid, _) = run(charged);
        outcome.expect("exact budget admits");
        assert_eq!(allocated, paid);
        let (outcome, allocated, paid, locus) = run(charged - 1);
        assert!(matches!(outcome, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, paid, "denied request allocates nothing");
        let locus = locus.expect("unpaid row walk locus");
        assert_eq!(
            locus.source,
            super::super::NativeJoinLocusSource::PackageRead(
                super::super::PackageReadStage::WireChecks,
                7
            )
        );
        assert!(matches!(locus.form, "refinement" | "walk"));
    }

    fn channel_classes(delivery: &str) -> Option<Vec<String>> {
        let argument = CheckedTerm::decode_admitted(&json!({
            "term": "binding", "name": "delivery", "value": {
                "term": "literal", "value_kind": "text", "value": delivery,
                "type": {"package": "test/protocol", "path": ["text"], "ordinal": 0}
            }
        }))
        .expect("valid delivery binding");
        let clause: Vec<Box<str>> = vec!["Example".into(), "OrderFlow".into()];
        let name: Vec<Box<str>> = vec!["Charges".into()];
        protocol_member_classes(
            CatalogOperator::Channel,
            &[argument],
            &clause,
            &name,
            &mut super::super::WorkMeter::new(u64::MAX),
        )
        .expect("unmetered class derivation")
        .map(|classes| classes.into_iter().map(|class| class.name).collect())
    }

    /// QSpec FR-465 and FR-426: a delivery count interval enables neither the
    /// `duplicate` nor the `lose` step, so it contributes no channel class;
    /// the four policy words keep their classes.
    /// Trace: FR-038-AC-325
    #[trace("TC-947")]
    #[test]
    fn channel_classes_follow_delivery_and_an_interval_has_none() {
        let path = "Example::OrderFlow::Charges";
        assert_eq!(
            channel_classes("unknown"),
            Some(vec![format!("duplicate {path}"), format!("lose {path}")])
        );
        assert_eq!(
            channel_classes("at-least-once"),
            Some(vec![format!("duplicate {path}")])
        );
        assert_eq!(
            channel_classes("at-most-once"),
            Some(vec![format!("lose {path}")])
        );
        assert_eq!(channel_classes("exactly-once-premise"), Some(Vec::new()));
        assert_eq!(channel_classes("[0,1]"), Some(Vec::new()));
        assert_eq!(channel_classes("[2,2]"), Some(Vec::new()));
        assert_eq!(channel_classes("[2,1]"), None);
    }

    /// The refinement's local-dependency vector is built from charged blocks
    /// only: the bytes the allocator is asked for equal the bytes charged, the
    /// measured requirement admits and one byte below is denied
    /// (FR-038-AC-504). Reference targets and member IDs are flat wire IDs.
    /// An application's member IDs are read out of Value members and charged
    /// only after they are parsed until the body migration lands.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn local_dependencies_allocate_exactly_what_they_charge() {
        let id = |fill: char| json!({"package": "refinement-test", "path": [fill.to_string()], "ordinal": 0});
        let profile = json!({"authority": "test", "identity": "quire.temporal.infinite-trace/v1"});
        let body = CheckedTerm::decode_admitted(&json!({
            "term": "refinement", "abstract": id('a'), "profile": profile,
            "populations": [{"abstract": id('b'), "concrete": id('c')}],
            "objects": [{"abstract_type": id('d'), "concrete_type": id('e'), "fields": []}],
            "steps": [],
            "histories": [{
                "concrete_type": id('e'), "name": "past", "type": id('f'),
                "initial": {"term": "reference", "target": id('g')},
                "updates": [{"operation": id('h'),
                    "expression": {"term": "reference", "target": id('a')}}]
            }],
            "assume": [{"operation": id('h'), "kind": "weak", "granularity": "whole"}],
            "ensure": []
        }))
        .expect("closed refinement");
        let run = |limit: u64| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut found = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = super::super::WorkMeter::over(&mut budget);
                found = Some(local_dependencies(&body, &mut meter));
            });
            (
                found.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (found, allocated, charged) = run(u64::MAX);
        let expected: Vec<CheckedNodeId> = "abcdefgh"
            .chars()
            .map(|fill| CheckedNodeId::from_value(&id(fill)).expect("node id"))
            .collect();
        let found = found.expect("ran").expect("a refinement body");
        assert_eq!(
            found, expected,
            "abstract, rows, types, history and operations"
        );
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(
            run(charged).0,
            Some(Some(found)),
            "the measured limit admits"
        );
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }
    /// The live graph-to-class traversal pays its temporary indexes, rendered
    /// class names, output rows and dependency-qualified node copies.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn protocol_classes_allocate_exactly_what_they_charge() {
        let id = |name: &str| {
            CheckedNodeId::from_value(&json!({
                "package":"test/protocol", "path":[name], "ordinal":0
            }))
            .expect("node ID")
        };
        let root = id("root");
        let object = id("object");
        let clause = id("clause");
        let member = id("parallel");
        let root_owner = super::super::CheckedNodeOwner::ModelRoot {
            identity: "example".into(),
        };
        let object_owner = super::super::CheckedNodeOwner::Model {
            identity: "example".into(),
            node: "Object".into(),
        };
        let clause_name = super::super::CheckedDeclaration {
            qualified_name: vec!["Example".into(), "Flow".into()],
        };
        let member_name = super::super::CheckedDeclaration {
            qualified_name: vec!["Example".into(), "Flow".into(), "run".into(), "Fork".into()],
        };
        let body = CheckedTerm::decode_admitted(&json!({
            "term":"application", "operator":"protocol_control",
            "operation":{"identity":"quire.op.protocol.control", "laws":[], "mode":null,
                "member":{"kind":"profile_operator", "operator":"parallel"}, "leaves":[]},
            "result_type":root, "arguments":[]
        }))
        .expect("admitted parallel operation");
        use super::super::{CheckedNodeKind as Kind, ModelForm, ProtocolForm};
        let nodes = [
            ProtocolCatalogNode {
                id: &root,
                kind: Kind::Model(ModelForm::ObjectType),
                name: None,
                owner: Some(&root_owner),
                owner_model: None,
                semantic_type: &root,
                term: None,
            },
            ProtocolCatalogNode {
                id: &object,
                kind: Kind::Model(ModelForm::ObjectType),
                name: None,
                owner: Some(&object_owner),
                owner_model: None,
                semantic_type: &root,
                term: None,
            },
            ProtocolCatalogNode {
                id: &clause,
                kind: Kind::Protocol(ProtocolForm::ProtocolClause),
                name: Some(&clause_name),
                owner: None,
                owner_model: Some(&object),
                semantic_type: &root,
                term: None,
            },
            ProtocolCatalogNode {
                id: &member,
                kind: Kind::Protocol(ProtocolForm::Control),
                name: Some(&member_name),
                owner: None,
                owner_model: None,
                semantic_type: &root,
                term: Some(&body),
            },
        ];
        let retained = super::super::model_fields::RetainedModels::default();
        let at = Path::root();
        for package in [None, Some("dependency/package")] {
            let run = |limit| {
                let mut budget =
                    crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    result = Some(protocol_classes(
                        &nodes,
                        &root,
                        None,
                        package,
                        CatalogSignatures::External(&retained),
                        at,
                        &mut super::super::WorkMeter::over(&mut budget),
                    ));
                });
                (
                    result.expect("ran"),
                    measured.bytes_total,
                    budget.read_storage_consumed(),
                )
            };
            let (result, allocated, charged) = run(u64::MAX);
            let classes = result.expect("classes");
            assert_eq!(
                classes
                    .iter()
                    .map(|class| class.class.as_str())
                    .collect::<Vec<_>>(),
                ["fork Example::Flow::Fork", "join Example::Flow::Fork"]
            );
            assert!(charged > 0);
            assert_eq!(allocated, charged);
            assert!(run(charged).0.is_ok(), "exact allowance admits");
            assert!(run(charged - 1).0.is_err(), "one byte short refuses");
        }
    }
}
