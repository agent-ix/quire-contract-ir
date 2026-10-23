//! Typed admitted V2 body vocabulary.
//!
//! The wire reader validates the flat QSpec grammar and each body before
//! decoding this representation for admitted graph joins. The flat grammar
//! has bounded term depth, so serde's recursion here is independent of input
//! byte length.

use super::{ApplicationOperator, BodyTerm, LiteralKind};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId, CheckedSemanticId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// A body term in an admitted checked-package V2 node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

impl CheckedTerm {
    /// Decodes only a body that the reader's flat and closed-shape checks
    /// already accepted. This is deliberately not a public Deserialize impl:
    /// an arbitrary recursive wire term has no standalone depth bound.
    pub(super) fn decode_admitted(value: &Value) -> Result<Self, ()> {
        fn field<T: DeserializeOwned>(value: &Value, name: &str) -> Result<T, ()> {
            serde_json::from_value(value.get(name).ok_or(())?.clone()).map_err(|_| ())
        }

        fn children(value: &Value, name: &str) -> Result<Vec<CheckedTerm>, ()> {
            value
                .get(name)
                .and_then(Value::as_array)
                .ok_or(())?
                .iter()
                .map(CheckedTerm::decode_admitted)
                .collect()
        }

        let tag = value.get("term").and_then(Value::as_str).ok_or(())?;
        match BodyTerm::from_wire(tag) {
            Some(BodyTerm::Literal) => Ok(Self::Literal {
                type_id: field(value, "type")?,
                value_kind: field(value, "value_kind")?,
                value: field(value, "value")?,
            }),
            Some(BodyTerm::Reference) => Ok(Self::Reference {
                target: field(value, "target")?,
            }),
            Some(BodyTerm::Application) => Ok(Self::Application {
                operator: field(value, "operator")?,
                operation: field(value, "operation")?,
                result_type: field(value, "result_type")?,
                arguments: children(value, "arguments")?,
            }),
            Some(BodyTerm::Aggregate) => Ok(Self::Aggregate {
                members: children(value, "members")?,
            }),
            Some(BodyTerm::Binding) => Ok(Self::Binding {
                name: field(value, "name")?,
                value: Box::new(Self::decode_admitted(value.get("value").ok_or(())?)?),
            }),
            Some(BodyTerm::DependencyReference) => Ok(Self::DependencyReference {
                package: field(value, "package")?,
                node: field(value, "node")?,
            }),
            Some(BodyTerm::Frame) => Ok(Self::Frame {
                modifies: field(value, "modifies")?,
                creates: field(value, "creates")?,
                deletes: field(value, "deletes")?,
                deletes_scoped: field(value, "deletes_scoped")?,
            }),
            Some(BodyTerm::AbstractionRelation) => Ok(Self::AbstractionRelation {
                objects: field(value, "objects")?,
                populations: field(value, "populations")?,
                frames: field(value, "frames")?,
            }),
            None if tag == "extension_form" => Ok(Self::ExtensionForm {
                definition: field(value, "definition")?,
                node_kind: field(value, "node_kind")?,
                clauses: children(value, "clauses")?,
            }),
            None => Err(()),
        }
    }
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
    #[serde(rename = "rust_type")]
    pub type_path: Vec<Box<str>>,
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
    #[serde(rename = "rust_field")]
    pub field_ident: Box<str>,
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
    #[serde(rename = "rust_parameter")]
    pub parameter_ident: Box<str>,
}

#[cfg(test)]
mod tests {
    use super::CheckedTerm;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-048")]
    #[test]
    fn checked_term_conversion_causes_distinguish_invalid_inputs() {
        let missing = CheckedTerm::decode_admitted(&json!({})).expect_err("missing term");
        let unknown = CheckedTerm::decode_admitted(&json!({"term":"not_a_term"}))
            .expect_err("unknown term");
        assert_ne!(missing, unknown, "missing and unknown term causes differ");
    }

    #[trace("FR-349-AC-1")]
    #[test]
    fn checked_term_variants_round_trip_the_closed_wire_arms() {
        let node = json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": "a".repeat(64)
        });
        let package = json!({
            "domain": "quire.package.semantic/v2",
            "algorithm": "sha256",
            "digest": "b".repeat(64)
        });
        let literal = json!({"term":"literal","type":node,"value_kind":"integer","value":"1"});
        let reference = json!({"term":"reference","target":node});
        let dependency = json!({"term":"dependency_reference","package":package,"node":node});
        let binding = json!({"term":"binding","name":"x","value":literal});
        let aggregate = json!({"term":"aggregate","members":[binding,reference,dependency]});
        let application = json!({
            "term":"application","operator":"binary",
            "operation":{"identity":"quire.op.test","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":node,"arguments":[aggregate]
        });
        let frame = json!({
            "term":"frame","modifies":[],"creates":[],"deletes":[],"deletes_scoped":[]
        });
        let abstraction = json!({
            "term":"abstraction_relation","objects":[],"populations":[],"frames":[]
        });
        let extension = json!({
            "term":"extension_form",
            "definition":{"authority":"test","identity":"extension"},
            "node_kind":"retention","clauses":[]
        });

        for (body, expected) in [
            (literal, "literal"),
            (reference, "reference"),
            (dependency, "dependency_reference"),
            (binding, "binding"),
            (aggregate, "aggregate"),
            (application, "application"),
            (frame, "frame"),
            (abstraction, "abstraction_relation"),
            (extension, "extension_form"),
        ] {
            let term = CheckedTerm::decode_admitted(&body).expect("closed term");
            let actual = match &term {
                CheckedTerm::Literal { .. } => "literal",
                CheckedTerm::Reference { .. } => "reference",
                CheckedTerm::DependencyReference { .. } => "dependency_reference",
                CheckedTerm::Binding { .. } => "binding",
                CheckedTerm::Aggregate { .. } => "aggregate",
                CheckedTerm::Application { .. } => "application",
                CheckedTerm::Frame { .. } => "frame",
                CheckedTerm::AbstractionRelation { .. } => "abstraction_relation",
                CheckedTerm::ExtensionForm { .. } => "extension_form",
            };
            assert_eq!(actual, expected);
            if let CheckedTerm::Application { arguments, .. } = &term {
                let [CheckedTerm::Aggregate { members }] = arguments.as_slice() else {
                    panic!("application argument must be a typed aggregate");
                };
                assert!(matches!(members[0], CheckedTerm::Binding { .. }));
                assert!(matches!(members[1], CheckedTerm::Reference { .. }));
                assert!(matches!(
                    members[2],
                    CheckedTerm::DependencyReference { .. }
                ));
            }
            assert_eq!(serde_json::to_value(term).expect("term serializes"), body);
        }
        assert!(
            CheckedTerm::decode_admitted(&json!({"term":"group_reference","ordinal":0})).is_err(),
            "a preimage-only group reference cannot become a wire term"
        );
    }
}
