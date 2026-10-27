//! Closed FR-375 refinement body vocabulary and reader checks.

use super::encode::RefinementNodePreimage;
use super::CheckedTerm;
use crate::checked_package::common::{
    decoder_pointer, locate_closed_value_failure, validate_term, ReferenceVisitor, TermGrammar,
    Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSemanticId, CheckedSourceMapEntry,
    CheckedSourceRef, CheckedSourceRegion,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Source region of one canonical refinement step row, outside package identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedRefinementStepRowRegion {
    /// Retained refinement relation node.
    pub node_id: CheckedNodeId,
    /// Zero-based position in its canonical steps array.
    pub ordinal: u64,
    /// The checked source row's single region.
    pub region: CheckedSourceRegion,
}

fn node_at(position: usize) -> crate::checked_package::shared::JsonPointer {
    crate::checked_package::common::node_pointer(position)
}

pub(super) fn stale_refinement_key(
    node: &super::RawSemanticNodeV2,
    position: usize,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let computed = quire_canonical::sha256(
        &RefinementNodePreimage {
            semantic_type: &node.semantic_type,
            body: &node.body,
        },
        quire_canonical::Limits::new(bytes),
    )
    .map_err(|_| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::MalformedWire,
            node_at(position).key("body"),
        )
    })?;
    if computed.to_string() == node.node_id.digest.as_ref() {
        return Ok(());
    }
    Err(ValidationFailure::refused_stale_node_key(
        node_at(position).key("node_id"),
        node.node_id.clone(),
        CheckedNodeId {
            domain: node.node_id.domain.clone(),
            digest: computed.to_string().into(),
        },
    ))
}

/// A refinement target is either a local node or a node in a directly selected package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(untagged)]
pub enum RefinementKey {
    /// A declaration in this graph.
    Local(CheckedNodeId),
    /// A declaration in a directly selected dependency package.
    External(RefinementPackageNodeKey),
}

/// A node of a directly selected dependency package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct RefinementPackageNodeKey {
    /// The selected package identity.
    pub package: CheckedSemanticId,
    /// The target node identity in that package.
    pub node: CheckedNodeId,
}

/// One abstract and concrete population pair.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct RefinementClassCatalog {
    /// Concrete operation, protocol and memory classes.
    pub concrete: Vec<RefinementConcreteClass>,
    /// Abstract operation, protocol and memory classes.
    pub r#abstract: Vec<RefinementAbstractClass>,
}

/// A concrete class with a declaration or a source-default memory step.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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

/// One memory step defined by a selected source-default memory model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct RefinementMemoryClass {
    /// Alias-free `memory <kind>` spelling.
    pub class: Box<str>,
    /// The selected source-default memory model.
    pub memory_model: RefinementMemoryModel,
}

/// Closed source-default memory model vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
            Self::Declared { class, .. } | Self::Memory(RefinementMemoryClass { class, .. }) => {
                class
            }
        }
    }
}

impl RefinementAbstractClass {
    fn class(&self) -> &str {
        match self {
            Self::Declared { class, .. } | Self::Memory(RefinementMemoryClass { class, .. }) => {
                class
            }
        }
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
        writer.serialize(&self.abstract_type)?;
        writer.name("concrete_type")?;
        writer.serialize(&self.concrete_type)?;
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
                writer.serialize(operation)?;
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
                    writer.serialize(node)?;
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
            writer.serialize(node)?;
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
        writer.serialize(&self.operation)?;
        writer.name("expression")?;
        self.expression.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for RefinementHistory {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("concrete_type")?;
        writer.serialize(&self.concrete_type)?;
        writer.name("name")?;
        writer.string(&self.name)?;
        writer.name("type")?;
        writer.serialize(&self.type_id)?;
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

/// Check the raw source-default memory class grammar at the catalog intake.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RefinementMemoryClassKind {
    Valid,
}

#[ix_trace_rs::string_edge]
fn memory_class_kind(class: &str) -> Option<RefinementMemoryClassKind> {
    class
        .strip_prefix("memory ")
        .filter(|kind| !kind.is_empty() && !kind.contains(' '))
        .map(|_| RefinementMemoryClassKind::Valid)
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

/// Check the complete closed body before hashing the package, including the
/// nested flat-term rule that applies to each row expression independently.
pub(super) fn check_wire(body: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    check_body_header(body, at)?;
    if let Some(catalog) = body.get("class_catalog") {
        check_catalog_wire(catalog, &at.key("class_catalog"))?;
    }
    let typed = serde_json::from_value::<CheckedTerm>(body.clone()).map_err(|_| {
        let path = locate_closed_value_failure::<CheckedTerm>(body)
            .map(|(path, _)| decoder_pointer(at.pointer(), &path))
            .unwrap_or_else(|| at.pointer());
        ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
    })?;
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
            let valid = if memory {
                memory_class_kind(class).is_some()
                    && entry.len() == 2
                    && entry.get("memory_model").is_some_and(|value| {
                        serde_json::from_value::<RefinementMemoryModel>(value.clone()).is_ok()
                    })
            } else {
                !class.is_empty()
                    && entry.len() == 3
                    && entry.contains_key("node")
                    && entry.contains_key("binder_types")
            };
            if !valid {
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
    fn digest(&self) -> &str {
        match self {
            Self::Local(node) => &node.digest,
            Self::External(key) => &key.node.digest,
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

/// Check the six FR-375 row arrays and their nested arrays in published order.
fn canonical_order(body: &CheckedTerm, position: usize) -> Result<(), ValidationFailure> {
    let CheckedTerm::Refinement {
        populations,
        objects,
        steps,
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
        if pair[0].r#abstract.digest() >= pair[1].r#abstract.digest() {
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
        if pair[0].abstract_type.digest() >= pair[1].abstract_type.digest() {
            return Err(order_defect(position, "objects", index + 1));
        }
    }
    for (index, pair) in steps.windows(2).enumerate() {
        fn key(step: &RefinementStep) -> (bool, &str, &str) {
            (
                step.source_node.is_none(),
                step.source_node
                    .as_ref()
                    .map(|node| node.digest.as_ref())
                    .unwrap_or(""),
                step.source_class.as_ref(),
            )
        }
        if key(&pair[0]) >= key(&pair[1]) {
            return Err(order_defect(position, "steps", index + 1));
        }
    }
    for (index, history) in histories.iter().enumerate() {
        for (update_index, pair) in history.updates.windows(2).enumerate() {
            if pair[0].operation.digest >= pair[1].operation.digest {
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
        if (pair[0].concrete_type.digest.as_ref(), pair[0].name.as_ref())
            >= (pair[1].concrete_type.digest.as_ref(), pair[1].name.as_ref())
        {
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
        fn key(row: &RefinementAssume) -> (&str, &str, &str) {
            (
                &row.operation.digest,
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        if key(&pair[0]) >= key(&pair[1]) {
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
        fn key(row: &RefinementEnsure) -> (&str, &str, &str) {
            (
                row.operation.digest(),
                kind(row.kind),
                granularity(row.granularity),
            )
        }
        if key(&pair[0]) >= key(&pair[1]) {
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
            if pair[0].class() >= pair[1].class() {
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
            if pair[0].class() >= pair[1].class() {
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
    dependencies: &'a super::SuppliedDependencies<'e>,
    package_id: &'a CheckedSemanticId,
}

impl<'a, 'e> RefinementGraph<'a, 'e> {
    pub(super) fn new(
        nodes: &'a [super::RawSemanticNodeV2],
        kinds: &'a [super::CheckedNodeKind],
        terms: &'a [Option<CheckedTerm>],
        index: &'a BTreeMap<&'a CheckedNodeId, usize>,
        dependencies: &'a super::SuppliedDependencies<'e>,
        package_id: &'a CheckedSemanticId,
    ) -> Self {
        Self {
            nodes,
            kinds,
            terms,
            index,
            dependencies,
            package_id,
        }
    }
}

impl RefinementGraph<'_, '_> {
    fn local_term(&self, id: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.terms
            .get(*self.index.get(id)?)
            .and_then(Option::as_ref)
    }

    fn local_node(&self, id: &CheckedNodeId) -> Option<&super::RawSemanticNodeV2> {
        self.nodes.get(*self.index.get(id)?)
    }

    fn key_owner_model(&self, key: &RefinementKey) -> Option<&CheckedNodeId> {
        match key {
            RefinementKey::Local(id) => self.local_node(id)?.owner_model.as_ref(),
            RefinementKey::External(external) => self
                .dependencies
                .selected_package(&external.package)?
                .node(&external.node)?
                .owner_model
                .as_ref(),
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
                if &key.package == self.package_id {
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
        subject: &CheckedNodeId,
        at: JsonPointer,
        locus: &CheckedNodeId,
    ) -> Result<Box<str>, ValidationFailure> {
        self.require_kind(
            key,
            super::CheckedNodeKind::State(super::StateForm::OperationAnchor),
            at.clone(),
            locus,
        )?;
        let Some((context, operation)) = self.key_anchor(key) else {
            return Err(wrong_binding(at, locus));
        };
        if context != *subject {
            return Err(wrong_binding(at, locus));
        }
        Ok(operation)
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
        super::CheckedNodeKind::State(super::StateForm::OperationAnchor)
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
            .and_then(|owner_model| graph.local_node(owner_model))
            .and_then(|model| model.owner.as_ref())
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
    let abstract_id = match abstract_key.as_ref() {
        RefinementKey::Local(id) => id,
        RefinementKey::External(external) => &external.node,
    };
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
        } else if let Some(source) = &row.source_node {
            let operation = graph.require_anchor_subject(
                &RefinementKey::Local(source.clone()),
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
                    abstract_id,
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
            graph.require_anchor_subject(
                &RefinementKey::Local(update.operation.clone()),
                &node.semantic_type,
                update_at.key("operation"),
                locus,
            )?;
        }
    }
    for (index, row) in assume.iter().enumerate() {
        let at = root.clone().key("assume").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_anchor_subject(
            &RefinementKey::Local(row.operation.clone()),
            &node.semantic_type,
            at.key("operation"),
            locus,
        )?;
    }
    for (index, row) in ensure.iter().enumerate() {
        let at = root.clone().key("ensure").index(index);
        meter.charge(1, || at.clone())?;
        graph.require_anchor_subject(&row.operation, abstract_id, at.key("operation"), locus)?;
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
        let abstract_id = match abstract_key.as_ref() {
            RefinementKey::Local(id) => id,
            RefinementKey::External(key) => &key.node,
        };
        if graph.key_owner_model(protocol) != Some(abstract_id) {
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
        let ordered = previous.is_none_or(|(node, ordinal)| {
            (node.digest.as_ref(), ordinal) < (row.node_id.digest.as_ref(), row.ordinal)
        });
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
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != super::CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
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
    bytes: u64,
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
        stale_refinement_key(node, position, bytes)?;
        canonical_order(body, position)?;
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
        let concrete = protocol_classes(
            &local,
            &graph.nodes[position].semantic_type,
            None,
            None,
            &root,
            meter,
        )?;
        compare_declared_classes(
            &catalog.concrete,
            &concrete,
            root.clone().key("concrete"),
            &graph.nodes[position].node_id,
            meter,
        )?;
        let abstract_classes = match protocol.as_deref() {
            Some(RefinementKey::Local(clause)) => match abstract_key.as_ref() {
                RefinementKey::Local(model) => {
                    protocol_classes(&local, model, Some(clause), None, &root, meter)?
                }
                RefinementKey::External(_) => Vec::new(),
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
                    protocol_classes(
                        &dependency,
                        &model.node,
                        Some(&clause.node),
                        Some(&clause.package),
                        &root,
                        meter,
                    )?
                }
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        compare_abstract_classes(
            &catalog.r#abstract,
            &abstract_classes,
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
    binder_types: Vec<RefinementKey>,
}

struct ProtocolCatalogNode<'a> {
    id: &'a CheckedNodeId,
    kind: super::CheckedNodeKind,
    name: Option<&'a super::CheckedDeclaration>,
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
            owner_model: node.owner_model.as_ref(),
            semantic_type: &node.semantic_type,
            term: Some(&node.body),
        })
        .collect()
}

fn protocol_classes(
    nodes: &[ProtocolCatalogNode<'_>],
    model: &CheckedNodeId,
    selected_clause: Option<&CheckedNodeId>,
    package: Option<&CheckedSemanticId>,
    at: &JsonPointer,
    meter: &mut super::WorkMeter,
) -> Result<Vec<DeclaredGraphClass>, ValidationFailure> {
    let mut classes = Vec::new();
    let key = |id: &CheckedNodeId| match package {
        Some(package) => RefinementKey::External(RefinementPackageNodeKey {
            package: package.clone(),
            node: id.clone(),
        }),
        None => RefinementKey::Local(id.clone()),
    };
    for clause in nodes {
        if clause.kind != super::CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolClause)
            || clause.owner_model != Some(model)
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
            if !matches!(member.kind, super::CheckedNodeKind::Protocol(_))
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
            let Some(classes_for_node) = protocol_member_classes(operator, &name.qualified_name)
            else {
                continue;
            };
            let binder_types: Vec<RefinementKey> = arguments
                .iter()
                .find_map(|argument| match argument {
                    CheckedTerm::Binding { name, value } if name.as_ref() == "binder" => {
                        if let CheckedTerm::Reference { target } = value.as_ref() {
                            nodes
                                .iter()
                                .find(|candidate| candidate.id == target)
                                .map(|binder| vec![key(binder.semantic_type)])
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .unwrap_or_default();
            for class in classes_for_node {
                classes.push(DeclaredGraphClass {
                    class,
                    node: key(member.id),
                    binder_types: binder_types.clone(),
                });
            }
        }
    }
    classes.sort_unstable_by(|left, right| left.class.cmp(&right.class));
    Ok(classes)
}

#[ix_trace_rs::string_edge]
fn protocol_member_classes(operator: &str, name: &[Box<str>]) -> Option<Vec<String>> {
    let clause = name.get(1)?.as_ref();
    let node = name.last()?.as_ref();
    let path = format!("{clause}::{node}");
    match operator {
        "parallel" => Some(vec![format!("fork {path}"), format!("join {path}")]),
        "attempt" | "event" | "send" | "receive" | "fence" | "await" | "finish" => Some(vec![path]),
        _ => None,
    }
}

fn compare_declared_classes(
    actual: &[RefinementConcreteClass],
    expected: &[DeclaredGraphClass],
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
                if found.node.local() != Some(node)
                    || binder_types
                        .iter()
                        .map(|binder| Some(binder))
                        .ne(found.binder_types.iter().map(RefinementKey::local))
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
    at: JsonPointer,
    locus: &CheckedNodeId,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = expected.iter();
    for (index, entry) in actual.iter().enumerate() {
        if matches!(entry, RefinementAbstractClass::Memory(_)) {
            continue;
        }
        let entry_at = at.clone().index(index);
        meter.charge(1, || entry_at.clone())?;
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
                if node != &found.node || binder_types != &found.binder_types {
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
        if node.dependencies != expected {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn refinement_preimage_matches_jcs_value() {
        let semantic_type = CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: "a".repeat(64).into(),
        };
        let body = json!({"term":"refinement","abstract":semantic_type,"profile":{"authority":"test","identity":"quire.temporal.infinite-trace/v1"},"populations":[],"objects":[],"steps":[],"histories":[],"assume":[],"ensure":[]});
        let value =
            json!({"version":"quire.refinement-node/v1","semantic_type":semantic_type,"body":body});
        let limits = quire_canonical::Limits::new(u64::MAX);
        let expected = quire_canonical::to_vec(&value, limits).expect("value");
        let actual = quire_canonical::to_vec(
            &RefinementNodePreimage {
                semantic_type: &semantic_type,
                body: &body,
            },
            limits,
        )
        .expect("preimage");
        assert_eq!(actual, expected);
    }
}
