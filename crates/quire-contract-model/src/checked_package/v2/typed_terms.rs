//! Typed admitted V2 body vocabulary.
//!
//! The wire reader validates the flat QSpec grammar and all semantic joins
//! before decoding this representation. The flat grammar has bounded term
//! depth, so serde's recursion here is independent of input byte length.

use super::{ApplicationOperator, LiteralKind};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId, CheckedSemanticId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A body term in an admitted checked-package V2 node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "term", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedTerm {
    /// A typed scalar literal.
    Literal {
        /// The scalar type node.
        #[serde(rename = "type")]
        type_id: CheckedNodeId,
        /// The closed literal kind.
        value_kind: LiteralKind,
        /// The literal payload; its shape depends on `value_kind`.
        value: Value,
    },
    /// A reference to another graph node.
    Reference {
        /// The referenced node.
        target: CheckedNodeId,
    },
    /// An operation application at a node body root.
    Application {
        /// The closed operator class.
        operator: ApplicationOperator,
        /// The catalogued operation's closed wire record.
        operation: Value,
        /// The declared result type node.
        result_type: CheckedNodeId,
        /// Typed argument terms in source order.
        arguments: Vec<CheckedTerm>,
    },
    /// An ordered aggregate.
    Aggregate {
        /// Typed members in source order.
        members: Vec<CheckedTerm>,
    },
    /// A named aggregate member.
    Binding {
        /// The member name.
        name: Box<str>,
        /// The member value.
        value: Box<CheckedTerm>,
    },
    /// A reference to a selected dependency package node.
    DependencyReference {
        /// Dependency package identity.
        package: CheckedSemanticId,
        /// The dependency's node key.
        node: CheckedNodeId,
    },
    /// The closed state frame body.
    Frame {
        /// Write grants.
        modifies: Vec<CheckedFrameModification>,
        /// Created nodes.
        creates: Vec<CheckedNodeId>,
        /// Deleted nodes.
        deletes: Vec<CheckedNodeId>,
        /// Scoped deletion grants.
        deletes_scoped: Vec<CheckedScopedDeletion>,
    },
    /// The closed abstraction relation body.
    AbstractionRelation {
        /// Object mappings.
        objects: Vec<CheckedAbstractionObject>,
        /// Population mappings.
        populations: Vec<CheckedAbstractionPopulation>,
        /// Frame mappings.
        frames: Vec<CheckedAbstractionFrame>,
    },
    /// The closed extension declaration body.
    ExtensionForm {
        /// Selected definition.
        definition: CheckedArtifactRef,
        /// Definition-owned node kind.
        node_kind: Box<str>,
        /// Ordered clause bindings.
        clauses: Vec<CheckedTerm>,
    },
}

/// One frame write grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedFrameModification {
    /// A relationship declaration.
    Relationship {
        /// The relationship node.
        declaration: CheckedNodeId,
    },
    /// A field declaration and its scope.
    Field {
        /// The field declaration node.
        declaration: CheckedNodeId,
        /// Field name.
        name: Box<str>,
        /// The write scope.
        scope: CheckedFieldScope,
    },
}

/// The scope of one frame field grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedFieldScope {
    /// Every object in the frame.
    EveryObject,
    /// The receiver only.
    Receiver,
}

/// One scoped deletion grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedScopedDeletion {
    /// Delete the receiver.
    Receiver,
    /// Delete a named parameter.
    Parameter {
        /// Parameter name.
        name: Box<str>,
    },
}

/// One abstraction object mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionObject {
    /// Model type node.
    #[serde(rename = "type")]
    pub type_id: CheckedNodeId,
    /// Rust type path.
    pub rust_type: Vec<Box<str>>,
    /// Field mappings.
    pub fields: Vec<CheckedAbstractionField>,
}

/// One abstraction field mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionField {
    /// Model field.
    pub name: Box<str>,
    /// Rust field.
    pub rust_field: Box<str>,
}

/// One abstraction population mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionPopulation {
    /// Population node.
    pub population: CheckedNodeId,
    /// Rust collection path.
    pub collection: Vec<Box<str>>,
}

/// One abstraction frame mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionFrame {
    /// Context node.
    pub context: CheckedNodeId,
    /// Operation name.
    pub operation: Box<str>,
    /// Rust function path.
    pub function: Vec<Box<str>>,
    /// Rust receiver name.
    pub receiver: Box<str>,
    /// Parameter mappings.
    pub parameters: Vec<CheckedAbstractionParameter>,
}

/// One abstraction parameter mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionParameter {
    /// Model parameter.
    pub name: Box<str>,
    /// Rust parameter.
    pub rust_parameter: Box<str>,
}
