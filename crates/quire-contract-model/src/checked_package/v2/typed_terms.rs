//! Typed admitted V2 body vocabulary.
//!
//! The wire reader validates the flat QSpec grammar and each body before
//! decoding this representation for admitted graph joins. The flat grammar
//! has bounded term depth, so serde's recursion here is independent of input
//! byte length.

use super::{ApplicationOperator, BodyTerm, LiteralKind};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId, CheckedSemanticId};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{error::Category, Value};

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

/// Why a validated wire body could not become its retained typed term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TermConversionCause {
    MissingTerm,
    UnknownTerm,
    MissingMember {
        member: &'static str,
    },
    MalformedMember {
        member: &'static str,
        category: Category,
    },
}

impl CheckedTerm {
    /// Visit every nested reference target in term order, without visiting
    /// type IDs or other node identities.
    pub fn visit_reference_targets(&self, visitor: &mut impl FnMut(&CheckedNodeId)) {
        match self {
            Self::Reference { target } => visitor(target),
            Self::Application { arguments, .. } => {
                for argument in arguments {
                    argument.visit_reference_targets(visitor);
                }
            }
            Self::Aggregate { members } => {
                for member in members {
                    member.visit_reference_targets(visitor);
                }
            }
            Self::Binding { value, .. } => value.visit_reference_targets(visitor),
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.visit_reference_targets(visitor);
                }
            }
            Self::Literal { .. }
            | Self::DependencyReference { .. }
            | Self::Frame { .. }
            | Self::AbstractionRelation { .. } => {}
        }
    }

    /// Substitute an exact node identity in this term's typed fields and
    /// closed operation payloads without changing ordinary strings.
    pub fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        fn replace(id: &mut CheckedNodeId, old: &CheckedNodeId, new: &CheckedNodeId) {
            if id == old {
                *id = new.clone();
            }
        }
        match self {
            Self::Literal { type_id, value, .. } => {
                replace(type_id, old, new);
                replace_node_ids_in_value(value, old, new);
            }
            Self::Reference { target } => replace(target, old, new),
            Self::Application {
                operation,
                result_type,
                arguments,
                ..
            } => {
                replace_node_ids_in_value(operation, old, new);
                replace(result_type, old, new);
                for argument in arguments {
                    argument.replace_node_id(old, new);
                }
            }
            Self::Aggregate { members } => {
                for member in members {
                    member.replace_node_id(old, new);
                }
            }
            Self::Binding { value, .. } => value.replace_node_id(old, new),
            Self::DependencyReference { node, .. } => replace(node, old, new),
            Self::Frame {
                modifies,
                creates,
                deletes,
                ..
            } => {
                for modification in modifies {
                    match modification {
                        CheckedFrameModification::Relationship { declaration }
                        | CheckedFrameModification::Field { declaration, .. } => {
                            replace(declaration, old, new)
                        }
                    }
                }
                for id in creates.iter_mut().chain(deletes) {
                    replace(id, old, new);
                }
            }
            Self::AbstractionRelation {
                objects,
                populations,
                frames,
            } => {
                for object in objects {
                    replace(&mut object.type_id, old, new);
                }
                for population in populations {
                    replace(&mut population.population, old, new);
                }
                for frame in frames {
                    replace(&mut frame.context, old, new);
                }
            }
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.replace_node_id(old, new);
                }
            }
        }
    }
    /// Construct a checked reference body without an untyped JSON intermediate.
    pub fn reference(target: CheckedNodeId) -> Self {
        Self::Reference { target }
    }

    /// Construct an ordered checked aggregate body.
    pub fn aggregate(members: Vec<Self>) -> Self {
        Self::Aggregate { members }
    }

    /// Construct one named aggregate member.
    pub fn binding(name: impl Into<Box<str>>, value: Self) -> Self {
        Self::Binding {
            name: name.into(),
            value: Box::new(value),
        }
    }

    /// Construct a text literal; the wire payload is a JSON string.
    pub fn literal_text(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Text,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a rational literal; the wire payload is a JSON string.
    pub fn literal_rational(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Rational,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a quantity literal; the wire payload is a JSON string.
    pub fn literal_quantity(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Quantity,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a nonnegative integer literal; the wire payload is a JSON number.
    pub fn literal_integer(type_id: CheckedNodeId, value: u64) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Integer,
            value: Value::Number(value.into()),
        }
    }

    /// Decodes only a body that the reader's flat and closed-shape checks
    /// already accepted. This is deliberately not a public Deserialize impl:
    /// an arbitrary recursive wire term has no standalone depth bound.
    pub(super) fn decode_admitted(value: &Value) -> Result<Self, TermConversionCause> {
        fn field<T: DeserializeOwned>(
            value: &Value,
            name: &'static str,
        ) -> Result<T, TermConversionCause> {
            let member = value
                .get(name)
                .ok_or(TermConversionCause::MissingMember { member: name })?;
            serde_json::from_value(member.clone()).map_err(|error| {
                TermConversionCause::MalformedMember {
                    member: name,
                    category: error.classify(),
                }
            })
        }

        fn children(
            value: &Value,
            name: &'static str,
        ) -> Result<Vec<CheckedTerm>, TermConversionCause> {
            value
                .get(name)
                .ok_or(TermConversionCause::MissingMember { member: name })?
                .as_array()
                .ok_or(TermConversionCause::MalformedMember {
                    member: name,
                    category: Category::Data,
                })?
                .iter()
                .map(CheckedTerm::decode_admitted)
                .collect()
        }

        let tag = value
            .get("term")
            .ok_or(TermConversionCause::MissingTerm)?
            .as_str()
            .ok_or(TermConversionCause::MalformedMember {
                member: "term",
                category: Category::Data,
            })?;
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
                value: Box::new(Self::decode_admitted(
                    value
                        .get("value")
                        .ok_or(TermConversionCause::MissingMember { member: "value" })?,
                )?),
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
            None => Err(TermConversionCause::UnknownTerm),
        }
    }
}

impl Encode for CheckedTerm {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Literal {
                type_id,
                value_kind,
                value,
            } => {
                writer.name("term")?;
                writer.string("literal")?;
                writer.name("type")?;
                writer.serialize(type_id)?;
                writer.name("value_kind")?;
                writer.serialize(value_kind)?;
                writer.name("value")?;
                value.encode_into(writer)?;
            }
            Self::Reference { target } => {
                writer.name("term")?;
                writer.string("reference")?;
                writer.name("target")?;
                writer.serialize(target)?;
            }
            Self::Application {
                operator,
                operation,
                result_type,
                arguments,
            } => {
                writer.name("term")?;
                writer.string("application")?;
                writer.name("operator")?;
                writer.serialize(operator)?;
                writer.name("operation")?;
                operation.encode_into(writer)?;
                writer.name("result_type")?;
                writer.serialize(result_type)?;
                writer.name("arguments")?;
                writer.begin_array()?;
                for argument in arguments {
                    argument.encode_into(writer)?;
                }
                writer.end_array()?;
            }
            Self::Aggregate { members } => {
                writer.name("term")?;
                writer.string("aggregate")?;
                writer.name("members")?;
                writer.begin_array()?;
                for member in members {
                    member.encode_into(writer)?;
                }
                writer.end_array()?;
            }
            Self::Binding { name, value } => {
                writer.name("term")?;
                writer.string("binding")?;
                writer.name("name")?;
                writer.string(name)?;
                writer.name("value")?;
                value.encode_into(writer)?;
            }
            Self::DependencyReference { package, node } => {
                writer.name("term")?;
                writer.string("dependency_reference")?;
                writer.name("package")?;
                writer.serialize(package)?;
                writer.name("node")?;
                writer.serialize(node)?;
            }
            Self::Frame {
                modifies,
                creates,
                deletes,
                deletes_scoped,
            } => {
                writer.name("term")?;
                writer.string("frame")?;
                writer.name("modifies")?;
                writer.serialize(modifies)?;
                writer.name("creates")?;
                writer.serialize(creates)?;
                writer.name("deletes")?;
                writer.serialize(deletes)?;
                writer.name("deletes_scoped")?;
                writer.serialize(deletes_scoped)?;
            }
            Self::AbstractionRelation {
                objects,
                populations,
                frames,
            } => {
                writer.name("term")?;
                writer.string("abstraction_relation")?;
                writer.name("objects")?;
                writer.serialize(objects)?;
                writer.name("populations")?;
                writer.serialize(populations)?;
                writer.name("frames")?;
                writer.serialize(frames)?;
            }
            Self::ExtensionForm {
                definition,
                node_kind,
                clauses,
            } => {
                writer.name("term")?;
                writer.string("extension_form")?;
                writer.name("definition")?;
                writer.serialize(definition)?;
                writer.name("node_kind")?;
                writer.string(node_kind)?;
                writer.name("clauses")?;
                writer.begin_array()?;
                for clause in clauses {
                    clause.encode_into(writer)?;
                }
                writer.end_array()?;
            }
        }
        writer.end_object()
    }
}

fn replace_node_ids_in_value(value: &mut Value, old: &CheckedNodeId, new: &CheckedNodeId) {
    match value {
        Value::Array(items) => {
            for item in items {
                replace_node_ids_in_value(item, old, new);
            }
        }
        Value::Object(object) => {
            if object.len() == 2
                && object.get("domain").and_then(Value::as_str) == Some(old.domain.as_ref())
                && object.get("digest").and_then(Value::as_str) == Some(old.digest.as_ref())
            {
                object.insert("domain".into(), Value::String(new.domain.to_string()));
                object.insert("digest".into(), Value::String(new.digest.to_string()));
            } else {
                for member in object.values_mut() {
                    replace_node_ids_in_value(member, old, new);
                }
            }
        }
        _ => {}
    }
}

/// One frame write grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedFieldScope {
    /// Every object in the frame.
    EveryObject,
    /// The receiver only.
    Receiver,
}

/// One scoped deletion grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionField {
    /// Model field.
    pub name: Box<str>,
    /// Rust field.
    #[serde(rename = "rust_field")]
    pub field_ident: Box<str>,
}

/// One abstraction population mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionPopulation {
    /// Population node.
    pub population: CheckedNodeId,
    /// Rust collection path.
    pub collection: Vec<Box<str>>,
}

/// One abstraction frame mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
    use super::{CheckedTerm, TermConversionCause};
    use ix_trace_rs::trace;
    use serde_json::{error::Category, json};

    #[trace("TC-048")]
    #[test]
    fn checked_term_conversion_causes_distinguish_invalid_inputs() {
        let missing = CheckedTerm::decode_admitted(&json!({})).expect_err("missing term");
        let unknown =
            CheckedTerm::decode_admitted(&json!({"term":"not_a_term"})).expect_err("unknown term");
        assert_ne!(missing, unknown, "missing and unknown term causes differ");
        assert_eq!(missing, TermConversionCause::MissingTerm);
        assert_eq!(unknown, TermConversionCause::UnknownTerm);
        for (body, expected) in [
            (
                json!({"term":false}),
                TermConversionCause::MalformedMember {
                    member: "term",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"reference"}),
                TermConversionCause::MissingMember { member: "target" },
            ),
            (
                json!({"term":"reference","target":0}),
                TermConversionCause::MalformedMember {
                    member: "target",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"aggregate"}),
                TermConversionCause::MissingMember { member: "members" },
            ),
            (
                json!({"term":"aggregate","members":false}),
                TermConversionCause::MalformedMember {
                    member: "members",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"binding","name":"x"}),
                TermConversionCause::MissingMember { member: "value" },
            ),
        ] {
            assert_eq!(CheckedTerm::decode_admitted(&body), Err(expected), "{body}");
        }
    }

    #[trace("TC-048")]
    #[test]
    fn checked_term_conversion_keeps_member_and_child_failure_order() {
        assert_eq!(
            CheckedTerm::decode_admitted(&json!({"term":"binding","name":0,"value":{}})),
            Err(TermConversionCause::MalformedMember {
                member: "name",
                category: Category::Data,
            }),
            "the binding name is decoded before its value"
        );
        assert_eq!(
            CheckedTerm::decode_admitted(&json!({"term":"literal","type":false})),
            Err(TermConversionCause::MalformedMember {
                member: "type",
                category: Category::Data,
            }),
            "the malformed type precedes the absent literal kind"
        );
        for (members, expected) in [
            (
                json!([{"term":"unknown"}, {}]),
                TermConversionCause::UnknownTerm,
            ),
            (
                json!([{}, {"term":"unknown"}]),
                TermConversionCause::MissingTerm,
            ),
        ] {
            let aggregate = json!({"term":"aggregate","members":members});
            assert_eq!(CheckedTerm::decode_admitted(&aggregate), Err(expected));
            assert_eq!(
                CheckedTerm::decode_admitted(
                    &json!({"term":"binding","name":"x","value":aggregate})
                ),
                Err(expected),
                "a recursive binding preserves the first child's exact cause"
            );
        }
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
