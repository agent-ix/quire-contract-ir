//! Typed admitted V2 body vocabulary.
//!
//! The wire reader validates the flat QSpec grammar and each body before
//! decoding this representation for admitted graph joins. The flat grammar
//! has bounded term depth, so serde's recursion here is independent of input
//! byte length.

use super::checked_json::CheckedJson;
use super::{
    ApplicationOperator, BodyTerm, LawRole, LiteralKind, OperationConstraintKind,
    OperationMemberKind, OperationModeKind, StateClauseKind,
};
use super::{
    CheckedComponentClause, CheckedCompositionComponent, CheckedCompositionPremise,
    CheckedEnvironmentEntry, CheckedFairnessSupply, CheckedGuaranteeProgress,
};
use super::{
    RefinementAssume, RefinementEnsure, RefinementHistory, RefinementKey, RefinementObject,
    RefinementPopulation, RefinementStep, RefinementStepArgument, RefinementStepTarget,
};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment};
use crate::identity::ExecutionPoint;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use quire_exact::Integer;
#[cfg(test)]
use serde::de::DeserializeOwned;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::error::Category;
#[cfg(test)]
use serde_json::Value;

/// An application operation whose state-clause point has a typed carrier.
/// Equality compares the represented record and point; admission views are
/// private provenance and do not affect represented-body equality.
#[derive(Clone, Debug)]
pub struct CheckedApplicationOperation {
    record: Box<CheckedApplicationOperationRecord>,
    execution_point: Option<ExecutionPoint>,
    views: Option<CheckedOperationViews>,
    #[cfg(feature = "test-support")]
    test_fixture: bool,
}

impl PartialEq for CheckedApplicationOperation {
    fn eq(&self, other: &Self) -> bool {
        self.record.identity == other.record.identity
            && self.record.laws == other.record.laws
            && self.record.mode == other.record.mode
            && self.record.member == other.record.member
            && self.record.leaves == other.record.leaves
            && self.record.malformed == other.record.malformed
            && self.execution_point == other.execution_point
    }
}

impl Eq for CheckedApplicationOperation {}

/// The typed wire record retained before operation-catalog admission.
/// Its member is a view into the caller-charged package document: malformed
/// member shapes keep the operation step's semantic refusal precedence.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckedApplicationOperationRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) identity: Option<Box<str>>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    pub(super) laws: Vec<CheckedOperationLawWire>,
    #[serde(deserialize_with = "super::required_nullable")]
    pub(super) mode: Option<CheckedOperationModeWire>,
    #[serde(deserialize_with = "super::required_nullable")]
    pub(super) member: Option<CheckedJson>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    pub(super) leaves: Vec<CheckedOperationLeafWire>,
    #[serde(skip)]
    pub(super) member_ids: CheckedOperationMemberIds,
    #[serde(skip)]
    pub(super) malformed: Option<CheckedJson>,
}

/// Node-valued member fields decoded with the operation's operand row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CheckedOperationMemberIds {
    pub(super) declaration: Option<CheckedNodeId>,
    pub(super) context: Option<CheckedNodeId>,
    pub(super) destination_type: Option<CheckedNodeId>,
    pub(super) population: Option<CheckedNodeId>,
    pub(super) target_declaration: Option<CheckedNodeId>,
    pub(super) target_node: Option<CheckedNodeId>,
    pub(super) target_parallel: Option<CheckedNodeId>,
}

impl CheckedApplicationOperationRecord {
    pub(super) fn malformed(raw: CheckedJson) -> Self {
        Self {
            identity: None,
            laws: Vec::new(),
            mode: None,
            member: None,
            leaves: Vec::new(),
            member_ids: CheckedOperationMemberIds::default(),
            malformed: Some(raw),
        }
    }

    /// Check wire shape by borrowing the retained document, before decoding fields.
    #[ix_trace_rs::string_edge]
    pub(super) fn wire_shape(node: super::TermNode<'_>) -> bool {
        fn text(node: Option<super::TermNode<'_>>) -> bool {
            node.and_then(super::TermNode::as_str).is_some()
        }
        fn mode(node: super::TermNode<'_>) -> bool {
            node.is_null()
                || (node.exact_members(&["kind", "value"])
                    && text(node.get("kind"))
                    && text(node.get("value")))
        }
        fn laws(node: super::TermNode<'_>) -> bool {
            node.items().is_some_and(|mut items| {
                items.all(|law| {
                    law.exact_members(&["role", "definition"])
                        && text(law.get("role"))
                        && law.get("definition").is_some_and(|definition| {
                            definition.exact_members(&["authority", "identity"])
                                && text(definition.get("authority"))
                                && text(definition.get("identity"))
                        })
                })
            })
        }
        node.members().is_some_and(|mut fields| {
            fields
                .all(|(name, _)| matches!(name, "identity" | "laws" | "mode" | "member" | "leaves"))
        }) && node
            .get("identity")
            .is_none_or(|identity| identity.as_str().is_some())
            && node.get("laws").is_some_and(laws)
            && node.get("mode").is_some_and(mode)
            && node.get("member").is_some()
            && node
                .get("leaves")
                .and_then(super::TermNode::items)
                .is_some_and(|mut leaves| {
                    leaves.all(|leaf| {
                        leaf.members().is_some_and(|mut fields| {
                            fields.all(|(name, _)| matches!(name, "path" | "laws" | "mode"))
                        }) && leaf
                            .get("path")
                            .and_then(super::TermNode::items)
                            .is_some_and(|mut path| path.all(|part| part.as_str().is_some()))
                            && leaf.get("laws").is_some_and(laws)
                            && leaf.get("mode").is_none_or(mode)
                    })
                })
    }

    pub(super) fn decode_member_ids(&mut self, context: &super::document_de::DecodeContext<'_>) {
        let Some(member) = self.member.as_ref().map(super::TermNode::of) else {
            return;
        };
        let decode = |value: Option<super::TermNode<'_>>| {
            value
                .and_then(|value| {
                    context
                        .value_by_id(value.node_ref().id())
                        .and_then(|value| CheckedNodeId::deserialize(value).ok())
                })
                .filter(CheckedNodeId::is_well_formed)
        };
        self.member_ids = CheckedOperationMemberIds {
            declaration: decode(member.get("declaration")),
            context: decode(member.get("context")),
            destination_type: decode(member.get("destination_type")),
            population: decode(member.get("population")),
            target_declaration: decode(
                member
                    .get("target")
                    .and_then(|target| target.get("declaration")),
            ),
            target_node: decode(member.get("target").and_then(|target| target.get("node"))),
            target_parallel: decode(
                member
                    .get("target")
                    .and_then(|target| target.get("parallel")),
            ),
        };
    }

    pub(super) fn copy_member_context(
        &self,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<CheckedNodeId>, crate::checked_package::common::ValidationFailure> {
        self.member_ids
            .context
            .as_ref()
            .map(|id| meter.clone_id(id))
            .transpose()
    }
    pub(super) fn copy_member_declaration(
        &self,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<CheckedNodeId>, crate::checked_package::common::ValidationFailure> {
        self.member_ids
            .declaration
            .as_ref()
            .map(|id| meter.clone_id(id))
            .transpose()
    }

    pub(super) fn member_id(&self, field: &str) -> Option<&CheckedNodeId> {
        match field {
            "declaration" => self.member_ids.declaration.as_ref(),
            "context" => self.member_ids.context.as_ref(),
            "destination_type" => self.member_ids.destination_type.as_ref(),
            "population" => self.member_ids.population.as_ref(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckedOperationLawWire {
    pub(super) role: Box<str>,
    pub(super) definition: CheckedArtifactRef,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckedOperationModeWire {
    pub(super) kind: Box<str>,
    pub(super) value: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckedOperationLeafWire {
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    pub(super) path: Vec<Box<str>>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    pub(super) laws: Vec<CheckedOperationLawWire>,
    pub(super) mode: Option<CheckedOperationModeWire>,
}

impl Encode for CheckedApplicationOperationRecord {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        if let Some(raw) = &self.malformed {
            return raw.encode_into(writer);
        }
        writer.begin_object()?;
        super::encode::present(writer, "identity", &self.identity)?;
        super::encode::member(writer, "laws", &self.laws)?;
        super::encode::member(writer, "leaves", &self.leaves)?;
        writer.name("member")?;
        match &self.member {
            Some(member) => member.encode_into(writer)?,
            None => writer.null()?,
        }
        super::encode::member(writer, "mode", &self.mode)?;
        writer.end_object()
    }
}

impl Serialize for CheckedApplicationOperationRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(raw) = &self.malformed {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(identity) = &self.identity {
            map.serialize_entry("identity", identity)?;
        }
        map.serialize_entry("laws", &self.laws)?;
        map.serialize_entry("mode", &self.mode)?;
        map.serialize_entry("member", &self.member)?;
        map.serialize_entry("leaves", &self.leaves)?;
        map.end()
    }
}

#[cfg(test)]
impl TryFrom<Value> for CheckedApplicationOperationRecord {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let raw = CheckedJson::from_serializable(&value).map_err(serde::de::Error::custom)?;
        CheckedApplicationOperation::outside_read_raw(raw)
            .map(|operation| *operation.record)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
impl TryFrom<Value> for Box<CheckedApplicationOperationRecord> {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        CheckedApplicationOperationRecord::try_from(value).map(Box::new)
    }
}

/// Read a closed execution point without serde's internally-tagged scratch map.
#[ix_trace_rs::string_edge]
pub(super) fn point_from_node(
    value: super::TermNode<'_>,
    context: &super::document_de::DecodeContext<'_>,
) -> Option<ExecutionPoint> {
    use crate::identity::ExecutionPointName;
    let kind = value.get("kind")?.as_str()?;
    let field = match kind {
        "initialization" | "handler" => "name",
        "pre" | "post" => "operation",
        _ => return None,
    };
    if !value.exact_members(&["kind", field]) {
        return None;
    }
    let text = value.get(field)?.as_str()?;
    if text.is_empty() || !context.charge_bytes(text.len()) {
        return None;
    }
    let name = ExecutionPointName::new(Box::<str>::from(text)).ok()?;
    Some(match kind {
        "initialization" => ExecutionPoint::Initialization { name },
        "handler" => ExecutionPoint::Handler { name },
        "pre" => ExecutionPoint::Pre { operation: name },
        "post" => ExecutionPoint::Post { operation: name },
        _ => return None,
    })
}

/// The document deserializer offers a typed record with its charged point.
pub(super) const OPERATION_TOKEN: &str = "$quire::ApplicationOperation";
thread_local! {
    static OFFERED_OPERATION: std::cell::RefCell<Option<CheckedApplicationOperation>> = const { std::cell::RefCell::new(None) };
}

/// An operation view was requested without reader admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedOperationViewError {
    /// The operation was decoded directly or read from unauthenticated V1 bytes.
    #[error("operation views are not admitted")]
    NotAdmitted,
}

/// Typed, validated input for a test-only application operation.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedApplicationOperationFixture {
    /// Optional operation identity, including intentional absence.
    pub identity: Option<Box<str>>,
    /// Optional closed mode kind and its value.
    pub mode: Option<(OperationModeKind, Box<str>)>,
    /// Ordered definitions with the fixed `integer_division` law role.
    pub law_definitions: Vec<CheckedArtifactRef>,
    /// Optional member declaration key.
    pub member_declaration: Option<CheckedNodeId>,
}

/// A malformed typed test operation input.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedApplicationOperationFixtureError {
    /// A present operation identity was empty.
    #[error("operation identity is empty")]
    EmptyIdentity,
    /// A present mode value was empty.
    #[error("operation mode value is empty")]
    EmptyModeValue,
    /// A law definition had an empty authority.
    #[error("law definition authority is empty")]
    EmptyLawAuthority,
    /// A law definition had an empty identity.
    #[error("law definition identity is empty")]
    EmptyLawIdentity,
    /// The member declaration has an empty package or path, or an ordinal
    /// outside the exact JSON integer range.
    #[error("member declaration node id is invalid")]
    InvalidMemberNodeId,
}

/// The admitted kind and value of an operation's mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedOperationModeView<'a> {
    /// The catalogued mode kind.
    pub kind: OperationModeKind,
    /// The admitted mode value.
    pub value: &'a str,
}

/// One selected operation law, in wire order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationLaw {
    /// Closed law role.
    pub role: LawRole,
    /// Selected definition.
    pub definition: CheckedArtifactRef,
}

/// One catalogued cross-operand constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationConstraint {
    /// Closed constraint choice.
    pub kind: OperationConstraintKind,
    /// Operand positions, in catalog order.
    pub operands: Vec<u64>,
    /// Required family or group when the constraint names one.
    pub family: Option<Box<str>>,
}

/// The closed shape of an admitted operation member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedOperationMember {
    /// A named field of a declaration.
    Field {
        /// Declaring type node.
        declaration: CheckedNodeId,
        /// Admitted field name.
        name: Box<str>,
        /// Source read form for a model-object projection; absent for record fields.
        read_form: Option<CheckedFieldReadForm>,
    },
    /// A declared tuple position.
    Position {
        /// Declaring tuple node.
        declaration: CheckedNodeId,
        /// Exact position, including beyond the JSON safe integer range.
        position: Integer,
    },
    /// An element type of a collection.
    Element {
        /// Declaring collection node.
        declaration: CheckedNodeId,
    },
    /// A named relationship end.
    RelationshipEnd {
        /// Declaring relationship node.
        declaration: CheckedNodeId,
        /// Named end role.
        name: Box<str>,
        /// Producer supplied destination type node.
        destination_type: CheckedNodeId,
        /// Population node, when the operation links one.
        population: Option<CheckedNodeId>,
    },
    /// A named dispatched operation.
    Operation {
        /// Declaring model node.
        declaration: CheckedNodeId,
        /// Admitted operation name.
        name: Box<str>,
    },
    /// An explicit type argument.
    TypeArgument {
        /// Type argument declaration node.
        declaration: CheckedNodeId,
    },
    /// An operator selected by a profile.
    ProfileOperator {
        /// Explicit operator, when present in the admitted record.
        operator: Option<Box<str>>,
    },
    /// A state clause.
    StateClause {
        /// Closed clause kind.
        clause: StateClauseKind,
    },
    /// An optional temporal interval.
    TemporalInterval {
        /// Interval, or an unbounded interval when absent.
        interval: Option<CheckedOperationInterval>,
    },
    /// A fairness clause.
    Fairness {
        /// Weak or strong fairness.
        fairness_kind: CheckedFairnessKind,
        /// Whole or per participant granularity.
        granularity: CheckedFairnessGranularity,
        /// Exact producer-selected fairness subject.
        target: CheckedFairnessTarget,
    },
}

/// A closed fairness subject carrying its admitted structural identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckedFairnessTarget {
    /// An effective model operation.
    Operation {
        /// Declaring model node.
        declaration: CheckedNodeId,
        /// Original operation name.
        name: Box<str>,
    },
    /// The owning protocol clause.
    ProtocolRoot {
        /// Protocol clause node.
        node: CheckedNodeId,
    },
    /// An attempt control in the owning protocol.
    AttemptNode {
        /// Attempt control node.
        node: CheckedNodeId,
    },
    /// One named branch of a parallel control.
    ParallelBranch {
        /// Exact parallel control node.
        parallel: CheckedNodeId,
        /// Authored branch label.
        branch: Box<str>,
    },
    /// An owning protocol's compensation template.
    CompensationTemplate {
        /// Compensation node.
        node: CheckedNodeId,
    },
    /// A transition of a hybrid model or mode.
    HybridTransition {
        /// Declaring hybrid model or mode.
        declaration: CheckedNodeId,
        /// Original transition name.
        name: Box<str>,
    },
}

impl<'de> Deserialize<'de> for CheckedFairnessTarget {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TargetVisitor;
        impl<'de> serde::de::Visitor<'de> for TargetVisitor {
            type Value = CheckedFairnessTarget;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("one closed typed fairness target")
            }
            #[ix_trace_rs::string_edge]
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                use serde::de::Error as _;
                let (mut kind, mut declaration, mut node, mut parallel, mut name, mut branch) = (
                    None::<Box<str>>,
                    None::<CheckedNodeId>,
                    None::<CheckedNodeId>,
                    None::<CheckedNodeId>,
                    None::<Box<str>>,
                    None::<Box<str>>,
                );
                while let Some(key) = map.next_key::<Box<str>>()? {
                    match key.as_ref() {
                        "kind" if kind.is_none() => kind = Some(map.next_value()?),
                        "declaration" if declaration.is_none() => {
                            declaration = Some(map.next_value()?)
                        }
                        "node" if node.is_none() => node = Some(map.next_value()?),
                        "parallel" if parallel.is_none() => parallel = Some(map.next_value()?),
                        "name" if name.is_none() => name = Some(map.next_value()?),
                        "branch" if branch.is_none() => branch = Some(map.next_value()?),
                        _ => {
                            return Err(M::Error::custom(
                                "unknown or duplicate fairness target member",
                            ));
                        }
                    }
                }
                match (kind.as_deref(), declaration, node, parallel, name, branch) {
                    (Some("operation"), Some(declaration), None, None, Some(name), None) => {
                        Ok(CheckedFairnessTarget::Operation { declaration, name })
                    }
                    (
                        Some("hybrid_transition"),
                        Some(declaration),
                        None,
                        None,
                        Some(name),
                        None,
                    ) => Ok(CheckedFairnessTarget::HybridTransition { declaration, name }),
                    (Some("protocol_root"), None, Some(node), None, None, None) => {
                        Ok(CheckedFairnessTarget::ProtocolRoot { node })
                    }
                    (Some("attempt_node"), None, Some(node), None, None, None) => {
                        Ok(CheckedFairnessTarget::AttemptNode { node })
                    }
                    (Some("compensation_template"), None, Some(node), None, None, None) => {
                        Ok(CheckedFairnessTarget::CompensationTemplate { node })
                    }
                    (Some("parallel_branch"), None, None, Some(parallel), None, Some(branch)) => {
                        Ok(CheckedFairnessTarget::ParallelBranch { parallel, branch })
                    }
                    _ => Err(M::Error::custom("invalid fairness target shape")),
                }
            }
        }
        deserializer.deserialize_map(TargetVisitor)
    }
}

impl CheckedFairnessTarget {
    pub(super) fn from_validated_node(
        value: super::TermNode<'_>,
        ids: &CheckedOperationMemberIds,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<Self>, crate::checked_package::common::ValidationFailure> {
        use super::TermNode;
        macro_rules! need {
            ($value:expr) => {
                match $value {
                    Some(value) => value,
                    None => return Ok(None),
                }
            };
        }
        let kind = need!(value.get("kind").and_then(TermNode::as_str));
        Ok(Some(match kind {
            "operation" | "hybrid_transition" => {
                let declaration = meter.clone_id(need!(ids.target_declaration.as_ref()))?;
                let name = meter.boxed_str(need!(value.get("name").and_then(TermNode::as_str)))?;
                if kind == "operation" {
                    Self::Operation { declaration, name }
                } else {
                    Self::HybridTransition { declaration, name }
                }
            }
            "protocol_root" => Self::ProtocolRoot {
                node: meter.clone_id(need!(ids.target_node.as_ref()))?,
            },
            "attempt_node" => Self::AttemptNode {
                node: meter.clone_id(need!(ids.target_node.as_ref()))?,
            },
            "compensation_template" => Self::CompensationTemplate {
                node: meter.clone_id(need!(ids.target_node.as_ref()))?,
            },
            "parallel_branch" => Self::ParallelBranch {
                parallel: meter.clone_id(need!(ids.target_parallel.as_ref()))?,
                branch: meter.boxed_str(need!(value.get("branch").and_then(TermNode::as_str)))?,
            },
            _ => return Ok(None),
        }))
    }

    #[cfg(test)]
    #[ix_trace_rs::string_edge]
    #[cfg(test)]
    fn from_validated_value(value: &Value) -> Option<Self> {
        let kind = value.get("kind")?.as_str()?;
        let declaration = || structural_member_id_value(value.get("declaration")?);
        let node = || structural_member_id_value(value.get("node")?);
        let name = || Some(Box::<str>::from(value.get("name")?.as_str()?));
        Some(match kind {
            "operation" => Self::Operation {
                declaration: declaration()?,
                name: name()?,
            },
            "hybrid_transition" => Self::HybridTransition {
                declaration: declaration()?,
                name: name()?,
            },
            "protocol_root" => Self::ProtocolRoot { node: node()? },
            "attempt_node" => Self::AttemptNode { node: node()? },
            "compensation_template" => Self::CompensationTemplate { node: node()? },
            "parallel_branch" => Self::ParallelBranch {
                parallel: structural_member_id_value(value.get("parallel")?)?,
                branch: value.get("branch")?.as_str()?.into(),
            },
            _ => return None,
        })
    }

    pub(super) fn node_mut(&mut self) -> &mut CheckedNodeId {
        match self {
            Self::Operation { declaration, .. } | Self::HybridTransition { declaration, .. } => {
                declaration
            }
            Self::ProtocolRoot { node }
            | Self::AttemptNode { node }
            | Self::CompensationTemplate { node } => node,
            Self::ParallelBranch { parallel, .. } => parallel,
        }
    }

    pub(super) fn node(&self) -> &CheckedNodeId {
        match self {
            Self::Operation { declaration, .. } | Self::HybridTransition { declaration, .. } => {
                declaration
            }
            Self::ProtocolRoot { node }
            | Self::AttemptNode { node }
            | Self::CompensationTemplate { node } => node,
            Self::ParallelBranch { parallel, .. } => parallel,
        }
    }
}

/// Source syntax retained for a model-object field read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedFieldReadForm {
    /// A direct source field read, such as `view.total`.
    Bare,
    /// A source field read with explicit dereference, such as `deref(view).total`.
    ExplicitDeref,
}

impl CheckedFieldReadForm {
    #[ix_trace_rs::string_edge]
    pub(super) fn from_wire(value: &str) -> Option<Self> {
        match value {
            "bare" => Some(Self::Bare),
            "explicit_deref" => Some(Self::ExplicitDeref),
            _ => None,
        }
    }
}

/// A closed temporal interval shape.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CheckedOperationInterval {
    /// Integer positions, including an unbounded upper end.
    Integer {
        /// Inclusive lower position.
        #[serde(deserialize_with = "decode_decimal_integer")]
        lower: Integer,
        /// Inclusive upper position, or no upper limit.
        #[serde(deserialize_with = "decode_optional_decimal_integer")]
        upper: Option<Integer>,
    },
    /// Time magnitudes with explicit open or closed ends.
    Timed {
        /// Lower time magnitude.
        lower: CheckedOperationRational,
        /// Upper time magnitude, when bounded.
        upper: Option<CheckedOperationRational>,
        /// Lower endpoint inclusion.
        lower_end: CheckedIntervalEnd,
        /// Upper endpoint inclusion.
        upper_end: CheckedIntervalEnd,
    },
}

/// An exact nonnegative rational interval bound.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationRational {
    /// Decimal numerator.
    #[serde(deserialize_with = "decode_decimal_integer")]
    pub numerator: Integer,
    /// Decimal denominator.
    #[serde(deserialize_with = "decode_decimal_integer")]
    pub denominator: Integer,
}

/// Whether an interval includes its endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedIntervalEnd {
    /// Includes the endpoint.
    Closed,
    /// Excludes the endpoint.
    Open,
}

/// The strength of a fairness clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedFairnessKind {
    /// Weak fairness.
    Weak,
    /// Strong fairness.
    Strong,
}

/// Whether fairness applies to the whole or each participant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedFairnessGranularity {
    /// Whole transition.
    Whole,
    /// Each participant.
    Each,
}

/// A leaf's selected mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationMode {
    /// Closed mode kind.
    pub kind: OperationModeKind,
    /// Catalogued mode value.
    pub value: Box<str>,
}

/// One structural leaf, in the checked operation's order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationLeaf {
    /// Structural path segments.
    pub path: Vec<Box<str>>,
    /// Selected laws in order.
    pub laws: Vec<CheckedOperationLaw>,
    /// Mode selected for this leaf.
    pub mode: Option<CheckedOperationMode>,
}

/// Complete owned admission data retained behind the borrowed read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CheckedOperationRecord {
    pub(super) identity: Box<str>,
    pub(super) mode: Option<CheckedOperationMode>,
    pub(super) laws: Vec<CheckedOperationLaw>,
    pub(super) member: Option<CheckedOperationMember>,
    pub(super) constraints: Vec<CheckedOperationConstraint>,
    pub(super) leaves: Vec<CheckedOperationLeaf>,
}

/// One borrowed read of a reader-admitted operation and its catalog rules.
#[derive(Clone, Copy, Debug)]
pub struct CheckedOperationRead<'a> {
    /// Catalogued identity.
    pub identity: &'a str,
    /// Admitted mode, when present.
    pub mode: Option<CheckedOperationModeView<'a>>,
    /// Ordered selected laws.
    pub laws: &'a [CheckedOperationLaw],
    /// Closed typed member.
    pub member: Option<&'a CheckedOperationMember>,
    /// State-clause execution point, when the member carries one.
    pub execution_point: Option<&'a ExecutionPoint>,
    /// Catalogued constraints in order.
    pub constraints: &'a [CheckedOperationConstraint],
    /// Ordered structural leaves.
    pub leaves: &'a [CheckedOperationLeaf],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CheckedOperationViews {
    pub(super) record: Option<Box<CheckedOperationRecord>>,
    pub(super) state_clause_kind: Option<StateClauseKind>,
    pub(super) projection_field: Option<(CheckedNodeId, Box<str>)>,
    pub(super) member_declaration: Option<CheckedNodeId>,
    pub(super) mode: Option<(OperationModeKind, Box<str>)>,
    pub(super) law_definitions: Vec<CheckedArtifactRef>,
}

impl CheckedOperationViews {
    fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        if let Some(record) = &mut self.record {
            if let Some(member) = &mut record.member {
                member.replace_node_id(old, new);
            }
        }
        if let Some((declaration, _)) = &mut self.projection_field {
            if declaration == old {
                *declaration = new.clone();
            }
        }
        if let Some(declaration) = &mut self.member_declaration {
            if declaration == old {
                *declaration = new.clone();
            }
        }
    }
}

impl CheckedOperationMember {
    pub(super) fn map_node_ids(
        &mut self,
        mut map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
    ) -> Option<()> {
        let mut replace = |id: &mut CheckedNodeId| {
            *id = map(id)?;
            Some(())
        };
        match self {
            Self::Field { declaration, .. }
            | Self::Position { declaration, .. }
            | Self::Element { declaration }
            | Self::Operation { declaration, .. }
            | Self::TypeArgument { declaration } => replace(declaration)?,
            Self::Fairness { target, .. } => replace(target.node_mut())?,
            Self::RelationshipEnd {
                declaration,
                destination_type,
                population,
                ..
            } => {
                replace(declaration)?;
                replace(destination_type)?;
                if let Some(population) = population {
                    replace(population)?;
                }
            }
            Self::ProfileOperator { .. }
            | Self::StateClause { .. }
            | Self::TemporalInterval { .. } => {}
        }
        Some(())
    }

    pub(super) fn from_validated_node(
        value: super::TermNode<'_>,
        ids: &CheckedOperationMemberIds,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<Self>, crate::checked_package::common::ValidationFailure> {
        use super::TermNode;
        use crate::checked_package::common::ValidationFailure;
        macro_rules! need {
            ($value:expr) => {
                match $value {
                    Some(value) => value,
                    None => return Ok(None),
                }
            };
        }
        fn integer(
            value: TermNode<'_>,
            meter: &mut super::WorkMeter,
        ) -> Result<Option<Integer>, ValidationFailure> {
            let text = match value.node_ref().node() {
                quire_canonical::Node::String(text) => text,
                quire_canonical::Node::Number(number) => number.text(),
                _ => return Ok(None),
            };
            meter.exact_integer(text)
        }
        fn rational(
            value: TermNode<'_>,
            meter: &mut super::WorkMeter,
        ) -> Result<Option<CheckedOperationRational>, ValidationFailure> {
            let numerator = need!(integer(need!(value.get("numerator")), meter)?);
            let denominator = need!(integer(need!(value.get("denominator")), meter)?);
            Ok(Some(CheckedOperationRational {
                numerator,
                denominator,
            }))
        }
        fn interval(
            value: TermNode<'_>,
            meter: &mut super::WorkMeter,
        ) -> Result<Option<CheckedOperationInterval>, ValidationFailure> {
            let lower = need!(value.get("lower"));
            let upper = need!(value.get("upper"));
            if value.exact_members(&["lower", "upper"]) {
                return Ok(Some(CheckedOperationInterval::Integer {
                    lower: need!(integer(lower, meter)?),
                    upper: if upper.is_null() {
                        None
                    } else {
                        Some(need!(integer(upper, meter)?))
                    },
                }));
            }
            if !value.exact_members(&["lower", "upper", "lower_end", "upper_end"]) {
                return Ok(None);
            }
            let end = |value: Option<TermNode<'_>>| match value.and_then(TermNode::as_str) {
                Some("open") => Some(CheckedIntervalEnd::Open),
                Some("closed") => Some(CheckedIntervalEnd::Closed),
                _ => None,
            };
            Ok(Some(CheckedOperationInterval::Timed {
                lower: need!(rational(lower, meter)?),
                upper: if upper.is_null() {
                    None
                } else {
                    Some(need!(rational(upper, meter)?))
                },
                lower_end: need!(end(value.get("lower_end"))),
                upper_end: need!(end(value.get("upper_end"))),
            }))
        }
        let kind = need!(value
            .get("kind")
            .and_then(TermNode::as_str)
            .and_then(OperationMemberKind::from_wire));
        let declaration = ids.declaration.as_ref();
        let name = value.get("name").and_then(TermNode::as_str);
        Ok(Some(match kind {
            OperationMemberKind::Field => Self::Field {
                declaration: meter.clone_id(need!(declaration))?,
                name: meter.boxed_str(need!(name))?,
                read_form: match value.get("read_form") {
                    Some(value) => Some(need!(value
                        .as_str()
                        .and_then(CheckedFieldReadForm::from_wire))),
                    None => None,
                },
            },
            OperationMemberKind::Position => Self::Position {
                declaration: meter.clone_id(need!(declaration))?,
                position: need!(integer(need!(value.get("position")), meter)?),
            },
            OperationMemberKind::Element => Self::Element {
                declaration: meter.clone_id(need!(declaration))?,
            },
            OperationMemberKind::RelationshipEnd => Self::RelationshipEnd {
                declaration: meter.clone_id(need!(declaration))?,
                name: meter.boxed_str(need!(name))?,
                destination_type: meter.clone_id(need!(ids.destination_type.as_ref()))?,
                population: if value.get("population").is_some() {
                    Some(meter.clone_id(need!(ids.population.as_ref()))?)
                } else {
                    None
                },
            },
            OperationMemberKind::Operation => Self::Operation {
                declaration: meter.clone_id(need!(declaration))?,
                name: meter.boxed_str(need!(name))?,
            },
            OperationMemberKind::TypeArgument => Self::TypeArgument {
                declaration: meter.clone_id(need!(declaration))?,
            },
            OperationMemberKind::ProfileOperator => Self::ProfileOperator {
                operator: value
                    .get("operator")
                    .and_then(TermNode::as_str)
                    .map(|text| meter.boxed_str(text))
                    .transpose()?,
            },
            OperationMemberKind::StateClause => Self::StateClause {
                clause: need!(value
                    .get("clause")
                    .and_then(TermNode::as_str)
                    .and_then(StateClauseKind::from_wire)),
            },
            OperationMemberKind::TemporalInterval => Self::TemporalInterval {
                interval: {
                    let value = need!(value.get("interval"));
                    if value.is_null() {
                        None
                    } else {
                        Some(need!(interval(value, meter)?))
                    }
                },
            },
            OperationMemberKind::Fairness => Self::Fairness {
                fairness_kind: match value.get("fairness_kind").and_then(TermNode::as_str) {
                    Some("weak") => CheckedFairnessKind::Weak,
                    Some("strong") => CheckedFairnessKind::Strong,
                    _ => return Ok(None),
                },
                granularity: match value.get("granularity").and_then(TermNode::as_str) {
                    Some("whole") => CheckedFairnessGranularity::Whole,
                    Some("each") => CheckedFairnessGranularity::Each,
                    _ => return Ok(None),
                },
                target: need!(CheckedFairnessTarget::from_validated_node(
                    need!(value.get("target")),
                    ids,
                    meter
                )?),
            },
        }))
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(super) fn from_validated_value(value: &Value) -> Option<Self> {
        let kind = OperationMemberKind::from_wire(value.get("kind")?.as_str()?)?;
        let declaration = || structural_member_id_value(value.get("declaration")?);
        let name = || Some(Box::<str>::from(value.get("name")?.as_str()?));
        let node = |key| structural_member_id_value(value.get(key)?);
        Some(match kind {
            OperationMemberKind::Field => Self::Field {
                declaration: declaration()?,
                name: name()?,
                read_form: match value.get("read_form") {
                    Some(value) => Some(CheckedFieldReadForm::from_wire(value.as_str()?)?),
                    None => None,
                },
            },
            OperationMemberKind::Position => {
                let position = value.get("position")?;
                let text = match position {
                    Value::Number(number) => number.to_string(),
                    Value::String(text) => text.clone(),
                    _ => return None,
                };
                Self::Position {
                    declaration: declaration()?,
                    position: text.parse().ok()?,
                }
            }
            OperationMemberKind::Element => Self::Element {
                declaration: declaration()?,
            },
            OperationMemberKind::RelationshipEnd => Self::RelationshipEnd {
                declaration: declaration()?,
                name: name()?,
                destination_type: node("destination_type")?,
                population: match value.get("population") {
                    Some(_) => Some(node("population")?),
                    None => None,
                },
            },
            OperationMemberKind::Operation => Self::Operation {
                declaration: declaration()?,
                name: name()?,
            },
            OperationMemberKind::TypeArgument => Self::TypeArgument {
                declaration: declaration()?,
            },
            OperationMemberKind::ProfileOperator => Self::ProfileOperator {
                operator: value
                    .get("operator")
                    .and_then(Value::as_str)
                    .map(Into::into),
            },
            OperationMemberKind::StateClause => Self::StateClause {
                clause: StateClauseKind::from_wire(value.get("clause")?.as_str()?)?,
            },
            OperationMemberKind::TemporalInterval => Self::TemporalInterval {
                interval: match value.get("interval")? {
                    Value::Null => None,
                    interval => Some(serde_json::from_value(interval.clone()).ok()?),
                },
            },
            OperationMemberKind::Fairness => Self::Fairness {
                fairness_kind: serde_json::from_value(value.get("fairness_kind")?.clone()).ok()?,
                granularity: serde_json::from_value(value.get("granularity")?.clone()).ok()?,
                target: CheckedFairnessTarget::from_validated_value(value.get("target")?)?,
            },
        })
    }

    fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        let replace = |id: &mut CheckedNodeId| {
            if id == old {
                *id = new.clone();
            }
        };
        match self {
            Self::Field { declaration, .. }
            | Self::Position { declaration, .. }
            | Self::Element { declaration }
            | Self::Operation { declaration, .. }
            | Self::TypeArgument { declaration } => replace(declaration),
            Self::Fairness { target, .. } => replace(target.node_mut()),
            Self::RelationshipEnd {
                declaration,
                destination_type,
                population,
                ..
            } => {
                replace(declaration);
                replace(destination_type);
                if let Some(population) = population {
                    replace(population);
                }
            }
            Self::ProfileOperator { .. }
            | Self::StateClause { .. }
            | Self::TemporalInterval { .. } => {}
        }
    }
}

fn decode_decimal_integer<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Integer, D::Error> {
    let text = Box::<str>::deserialize(deserializer)?;
    text.parse()
        .map_err(|_| serde::de::Error::custom("expected a canonical decimal integer"))
}

fn decode_optional_decimal_integer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Integer>, D::Error> {
    let text = Option::<Box<str>>::deserialize(deserializer)?;
    text.map(|text| {
        text.parse()
            .map_err(|_| serde::de::Error::custom("expected a canonical decimal integer"))
    })
    .transpose()
}

impl CheckedApplicationOperation {
    /// Construct the catalogued state-clause operation without a JSON payload.
    pub fn state_clause(clause: StateClauseKind, execution_point: Option<ExecutionPoint>) -> Self {
        #[derive(Serialize)]
        struct Member<'a> {
            kind: &'static str,
            clause: StateClauseKind,
            #[serde(skip_serializing_if = "Option::is_none")]
            execution_point: Option<&'a ExecutionPoint>,
        }
        let member = CheckedJson::from_serializable(&Member {
            kind: "state_clause",
            clause,
            execution_point: execution_point.as_ref(),
        })
        .expect("closed state-clause member");
        Self {
            record: Box::new(CheckedApplicationOperationRecord {
                identity: Some("quire.op.state.clause".into()),
                laws: Vec::new(),
                mode: None,
                member: Some(member),
                leaves: Vec::new(),
                member_ids: CheckedOperationMemberIds::default(),
                malformed: None,
            }),
            execution_point,
            views: Some(CheckedOperationViews {
                state_clause_kind: Some(clause),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        }
    }

    /// Construct a test operation without admitting a package or accepting a JSON view.
    #[cfg(feature = "test-support")]
    #[ix_trace_rs::string_edge]
    pub fn from_test_fixture(
        fixture: CheckedApplicationOperationFixture,
    ) -> Result<Self, CheckedApplicationOperationFixtureError> {
        use CheckedApplicationOperationFixtureError as FixtureError;

        if fixture.identity.as_deref() == Some("") {
            return Err(FixtureError::EmptyIdentity);
        }
        if fixture
            .mode
            .as_ref()
            .is_some_and(|(_, value)| value.is_empty())
        {
            return Err(FixtureError::EmptyModeValue);
        }
        for definition in &fixture.law_definitions {
            if definition.authority.is_empty() {
                return Err(FixtureError::EmptyLawAuthority);
            }
            if definition.identity.is_empty() {
                return Err(FixtureError::EmptyLawIdentity);
            }
        }
        if fixture
            .member_declaration
            .as_ref()
            .is_some_and(|key| !key.is_well_formed())
        {
            return Err(FixtureError::InvalidMemberNodeId);
        }

        #[derive(Serialize)]
        struct Member<'a> {
            kind: &'static str,
            declaration: &'a CheckedNodeId,
        }
        let member = fixture.member_declaration.as_ref().map(|declaration| {
            CheckedJson::from_serializable(&Member {
                kind: "type_argument",
                declaration,
            })
            .expect("validated fixture member")
        });
        let laws = fixture
            .law_definitions
            .iter()
            .map(|definition| CheckedOperationLawWire {
                role: "integer_division".into(),
                definition: definition.clone(),
            })
            .collect();

        Ok(Self {
            record: Box::new(CheckedApplicationOperationRecord {
                identity: fixture.identity.map(Into::into),
                laws,
                mode: fixture
                    .mode
                    .as_ref()
                    .map(|(kind, value)| CheckedOperationModeWire {
                        kind: kind.as_wire().into(),
                        value: value.clone(),
                    }),
                member,
                leaves: Vec::new(),
                member_ids: CheckedOperationMemberIds::default(),
                malformed: None,
            }),
            execution_point: None,
            views: Some(CheckedOperationViews {
                member_declaration: fixture.member_declaration,
                mode: fixture.mode,
                law_definitions: fixture.law_definitions,
                ..CheckedOperationViews::default()
            }),
            test_fixture: true,
        })
    }

    /// Whether this value came from the feature-gated fixture constructor.
    #[cfg(feature = "test-support")]
    pub fn is_test_fixture(&self) -> bool {
        self.test_fixture
    }

    /// The retained point of a state-clause application, if present.
    pub fn execution_point(&self) -> Option<&ExecutionPoint> {
        self.execution_point.as_ref()
    }

    /// The catalogued identity of this application operation.
    pub fn identity(&self) -> Option<&str> {
        self.record.identity.as_deref().or_else(|| {
            super::TermNode::of(self.record.malformed.as_ref()?)
                .get("identity")?
                .as_str()
        })
    }

    /// The admitted state-clause kind, or `None` for another member shape.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn state_clause_kind(&self) -> Result<Option<StateClauseKind>, CheckedOperationViewError> {
        Ok(self.admitted_views()?.state_clause_kind)
    }

    /// The admitted field declaration and name, or `None` for another member shape.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn projection_field(
        &self,
    ) -> Result<Option<(&CheckedNodeId, &str)>, CheckedOperationViewError> {
        Ok(self
            .admitted_views()?
            .projection_field
            .as_ref()
            .map(|(declaration, name)| (declaration, name.as_ref())))
    }

    /// The admitted member declaration, when the member carries one.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn member_declaration(&self) -> Result<Option<&CheckedNodeId>, CheckedOperationViewError> {
        Ok(self.admitted_views()?.member_declaration.as_ref())
    }

    /// The admitted operation mode kind and value, when present.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn mode(&self) -> Result<Option<CheckedOperationModeView<'_>>, CheckedOperationViewError> {
        Ok(self
            .admitted_views()?
            .mode
            .as_ref()
            .map(|(kind, value)| CheckedOperationModeView { kind: *kind, value }))
    }

    /// The admitted law definitions in operation order.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn law_definitions(&self) -> Result<&[CheckedArtifactRef], CheckedOperationViewError> {
        Ok(&self.admitted_views()?.law_definitions)
    }

    /// Borrow the complete reader-admitted operation and catalog decisions.
    /// Direct decoding and test fixtures return `NotAdmitted`.
    pub fn read(&self) -> Result<CheckedOperationRead<'_>, CheckedOperationViewError> {
        let record = self
            .admitted_views()?
            .record
            .as_ref()
            .ok_or(CheckedOperationViewError::NotAdmitted)?;
        Ok(CheckedOperationRead {
            identity: &record.identity,
            mode: record.mode.as_ref().map(|mode| CheckedOperationModeView {
                kind: mode.kind,
                value: &mode.value,
            }),
            laws: &record.laws,
            member: record.member.as_ref(),
            execution_point: self.execution_point.as_ref(),
            constraints: &record.constraints,
            leaves: &record.leaves,
        })
    }

    pub(super) fn admitted_views(
        &self,
    ) -> Result<&CheckedOperationViews, CheckedOperationViewError> {
        self.views
            .as_ref()
            .ok_or(CheckedOperationViewError::NotAdmitted)
    }

    pub(super) fn install_admitted_views(&mut self, views: CheckedOperationViews) {
        self.views = Some(views);
    }

    pub(super) fn record(&self) -> &CheckedApplicationOperationRecord {
        &self.record
    }

    /// The unadmitted member, borrowed from the retained document.
    pub(super) fn member_node(&self) -> Option<super::TermNode<'_>> {
        self.record.member.as_ref().map(super::TermNode::of)
    }

    /// One member spelling, without allocation or semantic admission.
    pub(super) fn member_text(&self, field: &str) -> Option<&str> {
        self.member_node()?.get(field)?.as_str()
    }

    pub(super) fn member_kind(&self) -> Option<OperationMemberKind> {
        OperationMemberKind::from_wire(self.member_text("kind")?)
    }

    pub(super) fn member_operator(&self) -> Option<&str> {
        self.member_text("operator")
    }
    pub(super) fn member_name(&self) -> Option<&str> {
        self.member_text("name")
    }

    pub(super) fn member_key(&self, field: &str) -> Option<&CheckedNodeId> {
        self.record.member_id(field)
    }

    #[cfg(test)]
    pub(super) fn record_mut(&mut self) -> OperationRecordEdit<'_> {
        let value = serde_json::to_value(&self.record).expect("typed fixture record serializes");
        OperationRecordEdit {
            operation: self,
            value,
        }
    }

    #[ix_trace_rs::string_edge]
    pub(super) fn fairness_target_syntax(
        &self,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<CheckedFairnessTarget>, crate::checked_package::common::ValidationFailure>
    {
        let Some(member) = self.member_node() else {
            return Ok(None);
        };
        if !super::temporal::member_is_well_formed_node(
            OperationMemberKind::Fairness,
            member,
            &self.record.member_ids,
        ) {
            return Ok(None);
        }
        let Some(target) = member.get("target") else {
            return Ok(None);
        };
        CheckedFairnessTarget::from_validated_node(target, &self.record.member_ids, meter)
    }

    #[ix_trace_rs::string_edge]
    pub(super) fn hybrid_target_for_join(
        &self,
        meter: &mut super::WorkMeter,
    ) -> Result<Option<CheckedFairnessTarget>, crate::checked_package::common::ValidationFailure>
    {
        let Some(target) = self.member_node().and_then(|member| member.get("target")) else {
            return Ok(None);
        };
        if target.get("kind").and_then(super::TermNode::as_str) != Some("hybrid_transition") {
            return Ok(None);
        }
        let Some(declaration) = self.record.member_ids.target_declaration.as_ref() else {
            return Ok(None);
        };
        Ok(Some(CheckedFairnessTarget::HybridTransition {
            declaration: meter.clone_id(declaration)?,
            name: meter.boxed_str(
                target
                    .get("name")
                    .and_then(super::TermNode::as_str)
                    .unwrap_or_default(),
            )?,
        }))
    }

    pub(super) fn from_typed_record(
        record: CheckedApplicationOperationRecord,
        execution_point: Option<ExecutionPoint>,
    ) -> Self {
        Self {
            record: Box::new(record),
            execution_point,
            views: None,
            #[cfg(feature = "test-support")]
            test_fixture: false,
        }
    }

    pub(super) fn offer(operation: Self) {
        OFFERED_OPERATION.with(|offered| *offered.borrow_mut() = Some(operation));
    }

    fn outside_read_raw(raw: CheckedJson) -> Result<Self, super::document_de::Error> {
        let record = if CheckedApplicationOperationRecord::wire_shape(super::TermNode::of(&raw)) {
            let mut budget =
                super::NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(u64::MAX);
            let context = super::document_de::DecodeContext::new(raw.document(), &mut budget);
            CheckedApplicationOperationRecord::deserialize(
                context.value_by_id(raw.node().id()).expect("retained node"),
            )?
        } else {
            CheckedApplicationOperationRecord::malformed(raw)
        };
        Ok(Self::outside_read(record))
    }

    fn outside_read(mut record: CheckedApplicationOperationRecord) -> Self {
        let execution_point = if let Some(member) = record.member.clone() {
            let mut budget =
                super::NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(u64::MAX);
            let context = super::document_de::DecodeContext::new(member.document(), &mut budget);
            record.decode_member_ids(&context);
            if record.identity.as_deref() == Some("quire.op.state.clause") {
                super::TermNode::of(&member)
                    .get("execution_point")
                    .and_then(|point| point_from_node(point, &context))
            } else {
                None
            }
        } else {
            None
        };
        Self::from_typed_record(record, execution_point)
    }
}

#[cfg(test)]
pub(super) struct OperationRecordEdit<'a> {
    operation: &'a mut CheckedApplicationOperation,
    value: Value,
}
#[cfg(test)]
impl std::ops::Deref for OperationRecordEdit<'_> {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.value
    }
}
#[cfg(test)]
impl std::ops::DerefMut for OperationRecordEdit<'_> {
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.value
    }
}
#[cfg(test)]
impl Drop for OperationRecordEdit<'_> {
    fn drop(&mut self) {
        self.operation.record = self
            .value
            .take()
            .try_into()
            .expect("edited fixture retains operation record shape");
    }
}

impl Serialize for CheckedApplicationOperation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.record.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CheckedApplicationOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OperationVisitor;
        impl<'de> serde::de::Visitor<'de> for OperationVisitor {
            type Value = CheckedApplicationOperation;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an application operation record")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                OFFERED_OPERATION
                    .with(|offered| offered.borrow_mut().take())
                    .ok_or_else(|| E::custom("missing decoded operation record"))
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                let raw = CheckedJson::deserialize(deserializer)?;
                CheckedApplicationOperation::outside_read_raw(raw).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_newtype_struct(OPERATION_TOKEN, OperationVisitor)
    }
}

impl Encode for CheckedApplicationOperation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        self.record.encode_into(writer)
    }
}

/// An admitted operation anchor body with three expression bindings and an
/// optional bare point binding. The point itself is not an expression term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationAnchorBody {
    /// Context, operation and frame bindings, in wire order.
    pub members: [CheckedTerm; 3],
    /// The optional closed fourth binding's bare value.
    pub execution_point: Option<ExecutionPoint>,
}

impl Serialize for CheckedOperationAnchorBody {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Members<'a>(&'a CheckedOperationAnchorBody);
        impl Serialize for Members<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #[derive(Serialize)]
                struct PointBinding<'a> {
                    term: &'static str,
                    name: &'static str,
                    value: &'a ExecutionPoint,
                }
                let mut sequence = serializer.serialize_seq(Some(
                    self.0.members.len() + usize::from(self.0.execution_point.is_some()),
                ))?;
                for member in &self.0.members {
                    sequence.serialize_element(member)?;
                }
                if let Some(point) = &self.0.execution_point {
                    sequence.serialize_element(&PointBinding {
                        term: "binding",
                        name: "execution_point",
                        value: point,
                    })?;
                }
                sequence.end()
            }
        }
        let mut object = serializer.serialize_map(Some(2))?;
        object.serialize_entry("term", "aggregate")?;
        object.serialize_entry("members", &Members(self))?;
        object.end()
    }
}

/// An admitted binding name with the closed contextual names decoded once.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedBindingName {
    raw: Box<str>,
    kind: super::TermBindingName,
}

/// The contextual meaning of an admitted binding name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedBindingNameKind<'a> {
    /// The `parameters` binding.
    Parameters,
    /// The `body` binding.
    Body,
    /// The `optional` binding.
    Optional,
    /// The `clause` binding.
    Clause,
    /// Any other name, retained verbatim.
    Other(&'a str),
}

impl CheckedBindingName {
    pub(super) fn internal_kind(&self) -> super::TermBindingName {
        self.kind
    }

    /// Read the contextual name without losing an unrecognized exact name.
    pub fn kind(&self) -> CheckedBindingNameKind<'_> {
        match self.kind {
            super::TermBindingName::Parameters => CheckedBindingNameKind::Parameters,
            super::TermBindingName::Body => CheckedBindingNameKind::Body,
            super::TermBindingName::Optional => CheckedBindingNameKind::Optional,
            super::TermBindingName::Clause => CheckedBindingNameKind::Clause,
            super::TermBindingName::Other => CheckedBindingNameKind::Other(&self.raw),
        }
    }
}

impl From<Box<str>> for CheckedBindingName {
    fn from(raw: Box<str>) -> Self {
        let kind = super::TermBindingName::from_wire(&raw);
        Self { raw, kind }
    }
}

impl From<&str> for CheckedBindingName {
    fn from(raw: &str) -> Self {
        Box::<str>::from(raw).into()
    }
}

impl From<String> for CheckedBindingName {
    fn from(raw: String) -> Self {
        raw.into_boxed_str().into()
    }
}

impl AsRef<str> for CheckedBindingName {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl std::fmt::Display for CheckedBindingName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(formatter)
    }
}

impl std::ops::Deref for CheckedBindingName {
    type Target = str;
    fn deref(&self) -> &str {
        &self.raw
    }
}

impl Serialize for CheckedBindingName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CheckedBindingName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<str>::deserialize(deserializer).map(Self::from)
    }
}

/// A literal's decoded value. Exact numbers are retained without floating point conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedLiteralValue<'a> {
    /// Boolean value.
    Boolean(bool),
    /// Arbitrary precision signed integer.
    Integer(Integer),
    /// Reduced rational or quantity magnitude.
    Rational {
        numerator: Integer,
        denominator: Integer,
    },
    /// Decimal coefficient and the exact written scale.
    Decimal { coefficient: Integer, scale: usize },
    /// IEEE binary32 bits, including signed zero and NaN payloads.
    Float32Bits(u32),
    /// IEEE binary64 bits, including signed zero and NaN payloads.
    Float64Bits(u64),
    /// Text value.
    Text(&'a str),
    /// Native string value.
    String(&'a str),
    /// Canonical UUID spelling.
    Uuid(&'a str),
    /// Signed nanoseconds from the POSIX epoch.
    Timestamp(i128),
    /// Exact enumeration value.
    Enum(&'a str),
    /// Absent option value.
    None,
}

/// A borrowed read of one literal's type, kind, and decoded value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedLiteralRead<'a> {
    /// The literal's declared type node.
    pub type_id: &'a CheckedNodeId,
    /// The admitted literal kind.
    pub kind: LiteralKind,
    /// The decoded value.
    pub value: CheckedLiteralValue<'a>,
}

fn rational_parts(value: &CheckedJson) -> Option<(Integer, Integer)> {
    let text = value.as_str()?;
    let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
    Some((numerator.parse().ok()?, denominator.parse().ok()?))
}

fn literal_value(kind: LiteralKind, value: &CheckedJson) -> Option<CheckedLiteralValue<'_>> {
    Some(match kind {
        LiteralKind::Boolean => CheckedLiteralValue::Boolean(value.as_bool()?),
        LiteralKind::Integer => {
            let text = value.as_str().or_else(|| value.number_text())?;
            CheckedLiteralValue::Integer(text.parse().ok()?)
        }
        LiteralKind::Rational | LiteralKind::Quantity => {
            let (numerator, denominator) = rational_parts(value)?;
            CheckedLiteralValue::Rational {
                numerator,
                denominator,
            }
        }
        LiteralKind::Decimal => {
            let text = value.as_str()?;
            let (whole, fractional) = text.split_once('.').unwrap_or((text, ""));
            let scale = fractional.len();
            let digits = format!("{whole}{fractional}");
            let negative = digits.starts_with('-');
            let magnitude = digits.trim_start_matches('-').trim_start_matches('0');
            let canonical = if magnitude.is_empty() {
                "0".to_owned()
            } else if negative {
                format!("-{magnitude}")
            } else {
                magnitude.to_owned()
            };
            CheckedLiteralValue::Decimal {
                coefficient: canonical.parse().ok()?,
                scale,
            }
        }
        LiteralKind::Float32Bits => {
            CheckedLiteralValue::Float32Bits(u32::from_str_radix(value.as_str()?, 16).ok()?)
        }
        LiteralKind::Float64Bits => {
            CheckedLiteralValue::Float64Bits(u64::from_str_radix(value.as_str()?, 16).ok()?)
        }
        LiteralKind::Text => CheckedLiteralValue::Text(value.as_str()?),
        LiteralKind::String => CheckedLiteralValue::String(value.as_str()?),
        LiteralKind::Uuid => CheckedLiteralValue::Uuid(value.as_str()?),
        LiteralKind::Timestamp => CheckedLiteralValue::Timestamp(value.as_str()?.parse().ok()?),
        LiteralKind::Enum => CheckedLiteralValue::Enum(value.as_str()?),
        LiteralKind::None if value.is_null() => CheckedLiteralValue::None,
        LiteralKind::None => return None,
    })
}

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
        value: CheckedJson,
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
        operation: CheckedApplicationOperation,
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
        name: CheckedBindingName,
        /// The member value.
        value: Box<CheckedTerm>,
    },
    /// A reference to a selected dependency package node.
    DependencyReference {
        /// Dependency package identity.
        package: Box<str>,
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
    /// The closed implementation guarantee body (QSpec FR-490).
    ImplementationGuarantee {
        /// The temporal clause node.
        clause: CheckedNodeId,
        /// The abstraction relation node.
        relation: CheckedNodeId,
        /// The Rust path of the initial-state function.
        initial: Vec<Box<str>>,
        /// No-divergence premises.
        progress: Vec<CheckedGuaranteeProgress>,
        /// Environment premises.
        environment: Vec<CheckedEnvironmentEntry>,
    },
    /// The closed component composition body (QSpec FR-491).
    ComponentComposition {
        /// The composition rule spelling.
        rule: Box<str>,
        /// The component subjects.
        components: Vec<CheckedCompositionComponent>,
        /// The component guarantees.
        guarantees: Vec<CheckedComponentClause>,
        /// Safety premises.
        relies: Vec<CheckedCompositionPremise>,
        /// Liveness premises.
        liveness_dependencies: Vec<CheckedCompositionPremise>,
        /// Supplied scheduling constraints.
        supplies: Vec<CheckedFairnessSupply>,
        /// Environment premises.
        environment: Vec<CheckedEnvironmentEntry>,
    },
    /// A closed model or protocol refinement declaration.
    Refinement {
        /// Abstract model declaration.
        #[serde(rename = "abstract")]
        abstract_key: Box<RefinementKey>,
        /// Selected infinite-trace profile.
        profile: CheckedArtifactRef,
        /// Population mappings.
        populations: Vec<RefinementPopulation>,
        /// Object mappings.
        objects: Vec<RefinementObject>,
        /// Step mappings.
        steps: Vec<RefinementStep>,
        /// History fields.
        histories: Vec<RefinementHistory>,
        /// Concrete fairness assumptions.
        assume: Vec<RefinementAssume>,
        /// Abstract fairness guarantees.
        ensure: Vec<RefinementEnsure>,
        /// Abstract protocol clause, when selected.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        protocol: Option<Box<RefinementKey>>,
        /// Nonempty alias-free internal list, when supplied.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abstract_internal: Option<Vec<Box<str>>>,
        /// Identity-bound checked classes of the selected subjects.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        class_catalog: Option<super::refinement::RefinementClassCatalog>,
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
    /// Closed source-authored enum, dimension, unit or equivalence body.
    #[serde(untagged, skip_deserializing)]
    NominalBody(Box<super::identity::NominalBody>),
    /// A dedicated operation anchor body; its bare point is not a term.
    #[serde(untagged, skip_deserializing)]
    OperationAnchorBody(Box<CheckedOperationAnchorBody>),
}

enum NodeMapWork<'a> {
    Term(&'a mut CheckedTerm),
    Id(&'a mut CheckedNodeId),
    Operation(&'a mut CheckedApplicationOperation),
    Nominal(&'a mut super::identity::NominalBody),
}

fn push_refinement_key<'a>(work: &mut Vec<NodeMapWork<'a>>, key: &'a mut RefinementKey) {
    match key {
        RefinementKey::Local(node) => work.push(NodeMapWork::Id(node)),
        RefinementKey::External(external) => work.push(NodeMapWork::Id(&mut external.node)),
    }
}

// Operation records retain their authored JSON, including structural IDs whose
// path operands are whole node IDs. The wire decoder intentionally accepts only
// flat IDs, so decode these retained member slots without recursive descent.
fn structural_member_id(value: super::TermNode<'_>) -> Option<CheckedNodeId> {
    enum Task<'a> {
        Node(super::TermNode<'a>),
        Segment(super::TermNode<'a>),
        Finish {
            package: &'a str,
            ordinal: u64,
            len: usize,
        },
    }

    let mut pending = vec![Task::Node(value)];
    let mut segments = Vec::new();
    let mut root = None;
    while let Some(task) = pending.pop() {
        match task {
            Task::Node(value) => {
                if !value.exact_members(&["package", "path", "ordinal"]) {
                    return None;
                }
                let package = value.get("package")?.as_str()?;
                let path = value.get("path")?.items()?;
                let ordinal = value.get("ordinal")?.as_u64()?;
                if package.is_empty() || ordinal > 9_007_199_254_740_991 {
                    return None;
                }
                let path: Vec<_> = path.collect();
                if path.is_empty() {
                    return None;
                }
                pending.push(Task::Finish {
                    package,
                    ordinal,
                    len: path.len(),
                });
                for segment in path.iter().rev() {
                    pending.push(Task::Segment(*segment));
                }
            }
            Task::Segment(segment) if segment.as_str().is_some() => {
                segments.push(CheckedNodePathSegment::Name(segment.as_str()?.into()));
            }
            Task::Segment(segment) if segment.exact_members(&["node"]) => {
                let index = segment.get("node")?.as_u64()?;
                if index > 9_007_199_254_740_991 {
                    return None;
                }
                segments.push(CheckedNodePathSegment::Index(index));
            }
            Task::Segment(segment) if segment.exact_members(&["operand"]) => {
                let slot = segment.get("operand")?.as_u64()?;
                if slot > 9_007_199_254_740_991 {
                    return None;
                }
                segments.push(CheckedNodePathSegment::Operand(slot));
            }
            Task::Segment(value) if value.exact_members(&["package", "path", "ordinal"]) => {
                pending.push(Task::Node(value));
            }
            Task::Segment(_) => return None,
            Task::Finish {
                package,
                ordinal,
                len,
            } => {
                let path = segments.split_off(segments.len().checked_sub(len)?);
                let node = CheckedNodeId {
                    package: package.into(),
                    path,
                    ordinal,
                };
                if pending.is_empty() {
                    root = Some(node);
                } else {
                    segments.push(CheckedNodePathSegment::Node(std::sync::Arc::new(node)));
                }
            }
        }
    }
    root
}

#[cfg(test)]
fn structural_member_id_value(value: &Value) -> Option<CheckedNodeId> {
    let checked = CheckedJson::from_serializable(value).ok()?;
    structural_member_id(super::TermNode::of(&checked))
}

struct RemapNode<'a> {
    node: super::TermNode<'a>,
    remapped: &'a [(CheckedNodeId, CheckedNodeId)],
}

impl Serialize for RemapNode<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(id) = structural_member_id(self.node) {
            if let Some((_, replacement)) = self.remapped.iter().find(|(old, _)| *old == id) {
                return replacement.serialize(serializer);
            }
        }
        match self.node.node_ref().node() {
            quire_canonical::Node::Null => serializer.serialize_unit(),
            quire_canonical::Node::Bool(value) => serializer.serialize_bool(value),
            quire_canonical::Node::Number(number) => {
                if let Ok(value) = number.text().parse::<i64>() {
                    serializer.serialize_i64(value)
                } else if let Ok(value) = number.text().parse::<u64>() {
                    serializer.serialize_u64(value)
                } else {
                    serializer.serialize_f64(number.value())
                }
            }
            quire_canonical::Node::String(value) => serializer.serialize_str(value),
            quire_canonical::Node::Array(_) => {
                let items = self.node.items().expect("array node has items");
                let mut sequence = serializer.serialize_seq(None)?;
                for node in items {
                    sequence.serialize_element(&Self {
                        node,
                        remapped: self.remapped,
                    })?;
                }
                sequence.end()
            }
            quire_canonical::Node::Object(_) => {
                let members = self.node.members().expect("object node has members");
                let mut map = serializer.serialize_map(None)?;
                for (name, node) in members {
                    map.serialize_entry(
                        name,
                        &Self {
                            node,
                            remapped: self.remapped,
                        },
                    )?;
                }
                map.end()
            }
        }
    }
}

#[ix_trace_rs::string_edge]
fn map_operation_ids(
    operation: &mut CheckedApplicationOperation,
    map: &mut impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
) {
    let Some(member_handle) = operation.record.member.as_ref() else {
        return;
    };
    let root = super::TermNode::of(member_handle);
    let mut remapped_ids = Vec::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Some(id) = structural_member_id(node) {
            if let Some(replacement) = map(&id) {
                remapped_ids.push((id, replacement));
                continue;
            }
        }
        if let Some(items) = node.items() {
            pending.extend(items);
        } else if let Some(members) = node.members() {
            pending.extend(members.map(|(_, value)| value));
        }
    }
    operation.record.member = Some(
        CheckedJson::from_serializable(&RemapNode {
            node: root,
            remapped: &remapped_ids,
        })
        .expect("remapped member remains canonical"),
    );

    if let Some(member) = operation.record.member.clone() {
        let mut budget =
            super::NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(u64::MAX);
        let context = super::document_de::DecodeContext::new(member.document(), &mut budget);
        operation.record.decode_member_ids(&context);
    }
    let Some(views) = &mut operation.views else {
        return;
    };
    for (old, new) in &remapped_ids {
        views.replace_node_id(old, new);
    }
    if let Some(record) = &mut views.record {
        if let Some(member) = &mut record.member {
            match member {
                CheckedOperationMember::Field { declaration, .. }
                | CheckedOperationMember::Position { declaration, .. }
                | CheckedOperationMember::Element { declaration }
                | CheckedOperationMember::Operation { declaration, .. }
                | CheckedOperationMember::TypeArgument { declaration } => {
                    for (old, new) in &remapped_ids {
                        if *declaration == *old {
                            *declaration = new.clone();
                        }
                    }
                }
                CheckedOperationMember::Fairness { target, .. } => {
                    for (old, new) in &remapped_ids {
                        if target.node() == old {
                            *target.node_mut() = new.clone();
                        }
                    }
                }
                CheckedOperationMember::RelationshipEnd {
                    declaration,
                    destination_type,
                    population,
                    ..
                } => {
                    for (old, new) in &remapped_ids {
                        if *declaration == *old {
                            *declaration = new.clone();
                        }
                        if *destination_type == *old {
                            *destination_type = new.clone();
                        }
                        if population.as_ref() == Some(old) {
                            *population = Some(new.clone());
                        }
                    }
                }
                CheckedOperationMember::ProfileOperator { .. }
                | CheckedOperationMember::StateClause { .. }
                | CheckedOperationMember::TemporalInterval { .. } => {}
            }
        }
    }
    if let Some((declaration, _)) = &mut views.projection_field {
        for (old, new) in &remapped_ids {
            if *declaration == *old {
                *declaration = new.clone();
            }
        }
    }
    if let Some(declaration) = &mut views.member_declaration {
        for (old, new) in &remapped_ids {
            if *declaration == *old {
                *declaration = new.clone();
            }
        }
    }
}

/// Borrowed, one-level decoding for early family wire checks. Children are
/// checked in their original order, with their own refusal pointers, before
/// the complete admitted tree is constructed.
#[cfg(test)]
pub(in crate::checked_package) enum CheckedTermView<'a> {
    Application { arguments: &'a [Value] },
    Aggregate { members: &'a [Value] },
    Binding { value: &'a Value },
    Reference,
    Literal,
}

/// An edge exposed by the bounded selected ordinary-term visitor.
#[allow(dead_code)] // The complete selected closure reader will use this visitor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SelectedTermEdge<'a> {
    Reference,
    Type,
    OperationMember,
    Dependency(&'a str),
    Unsupported,
}

/// One charged step of a borrowed ordinary-term walk.
#[allow(dead_code)] // The complete selected closure reader will use this visitor.
pub(super) enum SelectedTermVisit<'a> {
    Term,
    Edge(SelectedTermEdge<'a>, &'a CheckedNodeId),
}

/// Admission hooks around the shared ordinary-term traversal core.
pub(super) enum SelectedTermTraversal<'a> {
    Step,
    StackStorage(u128),
    Visit(SelectedTermVisit<'a>),
}

impl CheckedTerm {
    /// Read a scalar literal without inspecting its JSON wire payload.
    /// Returns `None` for another term or a directly decoded malformed literal.
    pub fn literal_read(&self) -> Option<CheckedLiteralRead<'_>> {
        let Self::Literal {
            type_id,
            value_kind,
            value,
        } = self
        else {
            return None;
        };
        Some(CheckedLiteralRead {
            type_id,
            kind: *value_kind,
            value: literal_value(*value_kind, value)?,
        })
    }

    #[cfg(test)]
    pub(in crate::checked_package) fn view(value: &Value) -> Option<CheckedTermView<'_>> {
        Self::view_from_wire(crate::checked_package::common::WireTerm::decode(value))
    }

    #[cfg(test)]
    pub(in crate::checked_package) fn view_from_wire(
        wire: crate::checked_package::common::WireTerm<'_>,
    ) -> Option<CheckedTermView<'_>> {
        use crate::checked_package::common::{exact_members, TermField, TermTag};
        let object = wire.object()?;
        match wire.tag() {
            TermTag::Known(BodyTerm::Application)
                if exact_members(
                    object,
                    &["term", "operator", "operation", "result_type", "arguments"],
                ) =>
            {
                wire.operator()?;
                wire.field(TermField::ResultType)?;
                Some(CheckedTermView::Application {
                    arguments: wire.array(TermField::Arguments)?,
                })
            }
            TermTag::Known(BodyTerm::Aggregate) if exact_members(object, &["term", "members"]) => {
                Some(CheckedTermView::Aggregate {
                    members: wire.array(TermField::Members)?,
                })
            }
            TermTag::Known(BodyTerm::Binding)
                if exact_members(object, &["term", "name", "value"]) =>
            {
                Some(CheckedTermView::Binding {
                    value: wire.field(TermField::Value)?,
                })
            }
            TermTag::Known(BodyTerm::Reference) if exact_members(object, &["term", "target"]) => {
                Some(CheckedTermView::Reference)
            }
            TermTag::Known(BodyTerm::Literal)
                if exact_members(object, &["term", "type", "value_kind", "value"]) =>
            {
                Some(CheckedTermView::Literal)
            }
            _ => None,
        }
    }
}

/// Why a validated wire body could not become its retained typed term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::checked_package) enum TermConversionCause {
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
    /// Inspect ordinary typed-term IDs without recursing through nested terms.
    /// The caller charges each term and edge before following it. Specialized
    /// bodies are reported instead of silently losing their IDs.
    #[allow(dead_code)] // The complete selected closure reader will use this visitor.
    pub(super) fn visit_selected_edges<'a, E>(
        &'a self,
        at: &'a CheckedNodeId,
        mut visit: impl FnMut(SelectedTermVisit<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        self.visit_selected_edges_charged(at, |event| match event {
            SelectedTermTraversal::Visit(event) => visit(event),
            SelectedTermTraversal::Step | SelectedTermTraversal::StackStorage(_) => Ok(()),
        })
    }

    pub(super) fn visit_selected_edges_charged<'a, E>(
        &'a self,
        at: &'a CheckedNodeId,
        mut traversal: impl FnMut(SelectedTermTraversal<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        enum Pending<'a> {
            Term(&'a CheckedTerm),
            Children(std::slice::Iter<'a, CheckedTerm>),
        }

        traversal(SelectedTermTraversal::StackStorage(2))?;
        let mut pending = vec![Pending::Term(self)];
        while !pending.is_empty() {
            traversal(SelectedTermTraversal::Step)?;
            let next = pending.pop().expect("nonempty term worklist");
            let current = match next {
                Pending::Term(term) => term,
                Pending::Children(mut children) => {
                    if let Some(child) = children.next() {
                        traversal(SelectedTermTraversal::StackStorage(2))?;
                        pending.push(Pending::Children(children));
                        pending.push(Pending::Term(child));
                    }
                    continue;
                }
            };
            traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Term))?;
            match current {
                Self::Literal { type_id, .. } => traversal(SelectedTermTraversal::Visit(
                    SelectedTermVisit::Edge(SelectedTermEdge::Type, type_id),
                ))?,
                Self::Reference { target } => traversal(SelectedTermTraversal::Visit(
                    SelectedTermVisit::Edge(SelectedTermEdge::Reference, target),
                ))?,
                Self::Application {
                    operation,
                    result_type,
                    arguments,
                    ..
                } => {
                    traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Edge(
                        SelectedTermEdge::Type,
                        result_type,
                    )))?;
                    let Ok(read) = operation.read() else {
                        traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Edge(
                            SelectedTermEdge::Unsupported,
                            at,
                        )))?;
                        continue;
                    };
                    if let Some(member) = read.member {
                        let mut member_id = |id| {
                            traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Edge(
                                SelectedTermEdge::OperationMember,
                                id,
                            )))
                        };
                        match member {
                            CheckedOperationMember::Field { declaration, .. }
                            | CheckedOperationMember::Position { declaration, .. }
                            | CheckedOperationMember::Element { declaration }
                            | CheckedOperationMember::Operation { declaration, .. }
                            | CheckedOperationMember::TypeArgument { declaration } => {
                                member_id(declaration)?;
                            }
                            CheckedOperationMember::RelationshipEnd {
                                declaration,
                                destination_type,
                                population,
                                ..
                            } => {
                                member_id(declaration)?;
                                member_id(destination_type)?;
                                if let Some(population) = population {
                                    member_id(population)?;
                                }
                            }
                            CheckedOperationMember::Fairness { target, .. } => {
                                let id = match target {
                                    CheckedFairnessTarget::Operation { declaration, .. }
                                    | CheckedFairnessTarget::HybridTransition {
                                        declaration, ..
                                    } => declaration,
                                    CheckedFairnessTarget::ProtocolRoot { node }
                                    | CheckedFairnessTarget::AttemptNode { node }
                                    | CheckedFairnessTarget::CompensationTemplate { node } => node,
                                    CheckedFairnessTarget::ParallelBranch { parallel, .. } => {
                                        parallel
                                    }
                                };
                                member_id(id)?;
                            }
                            CheckedOperationMember::ProfileOperator { .. }
                            | CheckedOperationMember::StateClause { .. }
                            | CheckedOperationMember::TemporalInterval { .. } => {}
                        }
                    }
                    traversal(SelectedTermTraversal::StackStorage(1))?;
                    pending.push(Pending::Children(arguments.iter()));
                }
                Self::Aggregate { members } => {
                    traversal(SelectedTermTraversal::StackStorage(1))?;
                    pending.push(Pending::Children(members.iter()));
                }
                Self::Binding { value, .. } => {
                    traversal(SelectedTermTraversal::StackStorage(1))?;
                    pending.push(Pending::Term(value));
                }
                Self::DependencyReference { package, node } => {
                    traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Edge(
                        SelectedTermEdge::Dependency(package),
                        node,
                    )))?;
                }
                Self::Frame { .. }
                | Self::AbstractionRelation { .. }
                | Self::ImplementationGuarantee { .. }
                | Self::ComponentComposition { .. }
                | Self::ExtensionForm { .. }
                | Self::OperationAnchorBody(_)
                | Self::Refinement { .. } => traversal(SelectedTermTraversal::Visit(
                    SelectedTermVisit::Edge(SelectedTermEdge::Unsupported, at),
                ))?,
                Self::NominalBody(body) => {
                    body.try_visit_reference_targets(&mut |id| {
                        traversal(SelectedTermTraversal::Visit(SelectedTermVisit::Edge(
                            SelectedTermEdge::Reference,
                            id,
                        )))?;
                        Ok(())
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Return nested reference targets in term order, excluding type IDs and
    /// other node identities.
    pub fn reference_targets(&self) -> Vec<&CheckedNodeId> {
        let mut targets = Vec::new();
        self.visit_reference_targets(&mut |target| targets.push(target));
        targets
    }

    pub(super) fn visit_reference_targets<'a>(
        &'a self,
        visitor: &mut impl FnMut(&'a CheckedNodeId),
    ) {
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
            Self::OperationAnchorBody(body) => {
                for member in &body.members {
                    member.visit_reference_targets(visitor);
                }
            }
            Self::NominalBody(body) => body.visit_reference_targets(visitor),
            Self::Binding { value, .. } => value.visit_reference_targets(visitor),
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.visit_reference_targets(visitor);
                }
            }
            Self::Refinement {
                objects,
                steps,
                histories,
                ..
            } => {
                super::refinement::visit_reference_targets(objects, steps, histories, visitor);
            }
            Self::Literal { .. }
            | Self::DependencyReference { .. }
            | Self::Frame { .. }
            | Self::AbstractionRelation { .. }
            | Self::ImplementationGuarantee { .. }
            | Self::ComponentComposition { .. } => {}
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
            // A literal's payload is a scalar and holds no node id.
            Self::Literal { type_id, .. } => replace(type_id, old, new),
            Self::Reference { target } => replace(target, old, new),
            Self::Application {
                operation,
                result_type,
                arguments,
                ..
            } => {
                let record = CheckedJson::from_serializable(&operation.record)
                    .expect("typed operation serializes");
                let root = super::TermNode::of(&record);
                let mut ids = Vec::new();
                let mut pending = vec![root];
                while let Some(node) = pending.pop() {
                    if structural_member_id(node).as_ref() == Some(old) {
                        ids.push((old.clone(), new.clone()));
                        continue;
                    }
                    if let Some(items) = node.items() {
                        pending.extend(items);
                    } else if let Some(members) = node.members() {
                        pending.extend(members.map(|(_, value)| value));
                    }
                }
                let remapped = CheckedJson::from_serializable(&RemapNode {
                    node: root,
                    remapped: &ids,
                })
                .expect("remapped operation is valid");
                operation.record = CheckedApplicationOperation::outside_read_raw(remapped)
                    .expect("remapped operation shape")
                    .record;
                if let Some(views) = &mut operation.views {
                    views.replace_node_id(old, new);
                }
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
            Self::OperationAnchorBody(body) => {
                for member in &mut body.members {
                    member.replace_node_id(old, new);
                }
            }
            Self::NominalBody(body) => body.replace_node_id(old, new),
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
            Self::ImplementationGuarantee { .. } | Self::ComponentComposition { .. } => {
                super::guarantee_terms::replace_node_ids(self, old, new);
            }
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.replace_node_id(old, new);
                }
            }
            Self::Refinement { .. } => {
                super::refinement::replace_node_ids(self, old, new);
            }
        }
    }

    /// Rewrite each logical typed node identity slot once, in term preorder.
    ///
    /// The walk uses an explicit stack so term nesting does not consume the
    /// call stack. Replacements are applied only at their original slots and
    /// are not passed through the callback again.
    pub fn map_node_ids(&mut self, mut map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {
        let mut work = vec![NodeMapWork::Term(self)];
        while let Some(item) = work.pop() {
            match item {
                NodeMapWork::Id(id) => {
                    if let Some(replacement) = map(id) {
                        *id = replacement;
                    }
                }
                NodeMapWork::Operation(operation) => map_operation_ids(operation, &mut map),
                NodeMapWork::Nominal(body) => body.map_node_ids(&mut map),
                NodeMapWork::Term(term) => match term {
                    Self::Literal { type_id, .. } => work.push(NodeMapWork::Id(type_id)),
                    Self::Reference { target } => work.push(NodeMapWork::Id(target)),
                    Self::Application {
                        operation,
                        result_type,
                        arguments,
                        ..
                    } => {
                        work.extend(arguments.iter_mut().rev().map(NodeMapWork::Term));
                        work.push(NodeMapWork::Id(result_type));
                        work.push(NodeMapWork::Operation(operation));
                    }
                    Self::Aggregate { members } => {
                        work.extend(members.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::Binding { value, .. } => work.push(NodeMapWork::Term(value)),
                    Self::DependencyReference { node, .. } => work.push(NodeMapWork::Id(node)),
                    Self::Frame {
                        modifies,
                        creates,
                        deletes,
                        ..
                    } => {
                        work.extend(deletes.iter_mut().rev().map(NodeMapWork::Id));
                        work.extend(creates.iter_mut().rev().map(NodeMapWork::Id));
                        for modification in modifies.iter_mut().rev() {
                            let declaration = match modification {
                                CheckedFrameModification::Relationship { declaration }
                                | CheckedFrameModification::Field { declaration, .. } => {
                                    declaration
                                }
                            };
                            work.push(NodeMapWork::Id(declaration));
                        }
                    }
                    Self::ImplementationGuarantee { .. } | Self::ComponentComposition { .. } => {
                        for id in super::guarantee_terms::ids_mut(term).into_iter().rev() {
                            work.push(NodeMapWork::Id(id));
                        }
                    }
                    Self::AbstractionRelation {
                        objects,
                        populations,
                        frames,
                    } => {
                        for frame in frames.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut frame.context));
                        }
                        for population in populations.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut population.population));
                        }
                        for object in objects.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut object.type_id));
                        }
                    }
                    Self::ExtensionForm { clauses, .. } => {
                        work.extend(clauses.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::OperationAnchorBody(body) => {
                        work.extend(body.members.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::NominalBody(body) => work.push(NodeMapWork::Nominal(body)),
                    Self::Refinement {
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
                    } => {
                        if let Some(catalog) = class_catalog {
                            for entry in catalog.r#abstract.iter_mut().rev() {
                                if let super::refinement::RefinementAbstractClass::Declared {
                                    node,
                                    binder_types,
                                    ..
                                } = entry
                                {
                                    for binder in binder_types.iter_mut().rev() {
                                        push_refinement_key(&mut work, binder);
                                    }
                                    push_refinement_key(&mut work, node);
                                }
                            }
                            for entry in catalog.concrete.iter_mut().rev() {
                                if let super::refinement::RefinementConcreteClass::Declared {
                                    node,
                                    binder_types,
                                    ..
                                } = entry
                                {
                                    work.extend(binder_types.iter_mut().rev().map(NodeMapWork::Id));
                                    work.push(NodeMapWork::Id(node));
                                }
                            }
                        }
                        if let Some(key) = protocol {
                            push_refinement_key(&mut work, key);
                        }
                        for row in ensure.iter_mut().rev() {
                            push_refinement_key(&mut work, &mut row.operation);
                        }
                        for row in assume.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut row.operation));
                        }
                        for row in histories.iter_mut().rev() {
                            for update in row.updates.iter_mut().rev() {
                                work.push(NodeMapWork::Term(&mut update.expression));
                                work.push(NodeMapWork::Id(&mut update.operation));
                            }
                            work.push(NodeMapWork::Term(&mut row.initial));
                            work.push(NodeMapWork::Id(&mut row.type_id));
                            work.push(NodeMapWork::Id(&mut row.concrete_type));
                        }
                        for row in steps.iter_mut().rev() {
                            match &mut row.target {
                                RefinementStepTarget::Operation {
                                    operation,
                                    receiver,
                                    arguments,
                                } => {
                                    for argument in arguments.iter_mut().rev() {
                                        if let RefinementStepArgument::Term(term) = argument {
                                            work.push(NodeMapWork::Term(term));
                                        }
                                    }
                                    work.push(NodeMapWork::Term(receiver));
                                    push_refinement_key(&mut work, operation);
                                }
                                RefinementStepTarget::ProtocolStep {
                                    node, arguments, ..
                                } => {
                                    for argument in arguments.iter_mut().rev() {
                                        if let RefinementStepArgument::Term(term) = argument {
                                            work.push(NodeMapWork::Term(term));
                                        }
                                    }
                                    if let Some(key) = node {
                                        push_refinement_key(&mut work, key);
                                    }
                                }
                                RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
                            }
                            if let Some(node) = &mut row.source_node {
                                work.push(NodeMapWork::Id(node));
                            }
                        }
                        for row in objects.iter_mut().rev() {
                            work.extend(
                                row.fields
                                    .iter_mut()
                                    .rev()
                                    .map(|field| NodeMapWork::Term(&mut field.expression)),
                            );
                            work.push(NodeMapWork::Id(&mut row.concrete_type));
                            push_refinement_key(&mut work, &mut row.abstract_type);
                        }
                        for row in populations.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut row.concrete));
                            push_refinement_key(&mut work, &mut row.r#abstract);
                        }
                        push_refinement_key(&mut work, abstract_key);
                    }
                },
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
            name: name.into().into(),
            value: Box::new(value),
        }
    }

    /// Construct a text literal; the wire payload is a JSON string.
    pub fn literal_text(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Text,
            value: CheckedJson::scalar(&value.into()),
        }
    }

    /// Construct a rational literal; the wire payload is a JSON string.
    pub fn literal_rational(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Rational,
            value: CheckedJson::scalar(&value.into()),
        }
    }

    /// Construct a quantity literal; the wire payload is a JSON string.
    pub fn literal_quantity(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Quantity,
            value: CheckedJson::scalar(&value.into()),
        }
    }

    /// Construct a nonnegative integer literal; the wire payload is a JSON number.
    pub fn literal_integer(type_id: CheckedNodeId, value: u64) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Integer,
            value: CheckedJson::scalar(&value),
        }
    }

    /// Decodes only a body that the reader's flat and closed-shape checks
    /// already accepted. Public serde decoding does not provide that bound.
    #[cfg(test)]
    pub(super) fn decode_admitted(value: &Value) -> Result<Self, TermConversionCause> {
        if value.get("term").and_then(Value::as_str) == Some("refinement") {
            return super::refinement::decode_body(
                value,
                &crate::checked_package::common::Trail::Base(&[]),
            )
            .map_err(|_| TermConversionCause::MalformedMember {
                member: "refinement",
                category: Category::Data,
            });
        }
        fn field<T: DeserializeOwned>(
            wire: crate::checked_package::common::WireTerm<'_>,
            name: crate::checked_package::common::TermField,
        ) -> Result<T, TermConversionCause> {
            let member = wire.field(name).ok_or(TermConversionCause::MissingMember {
                member: name.as_wire(),
            })?;
            serde_json::from_value(member.clone()).map_err(|error| {
                TermConversionCause::MalformedMember {
                    member: name.as_wire(),
                    category: error.classify(),
                }
            })
        }

        fn children(
            wire: crate::checked_package::common::WireTerm<'_>,
            name: crate::checked_package::common::TermField,
        ) -> Result<Vec<CheckedTerm>, TermConversionCause> {
            wire.field(name)
                .ok_or(TermConversionCause::MissingMember {
                    member: name.as_wire(),
                })?
                .as_array()
                .ok_or(TermConversionCause::MalformedMember {
                    member: name.as_wire(),
                    category: Category::Data,
                })?
                .iter()
                .map(CheckedTerm::decode_admitted)
                .collect()
        }

        use crate::checked_package::common::{TermField, TermTag, WireTerm};
        let wire = WireTerm::decode(value);
        match wire.tag() {
            TermTag::Known(BodyTerm::Literal) => Ok(Self::Literal {
                type_id: field(wire, TermField::Type)?,
                value_kind: field(wire, TermField::ValueKind)?,
                value: field(wire, TermField::Value)?,
            }),
            TermTag::Known(BodyTerm::Reference) => Ok(Self::Reference {
                target: field(wire, TermField::Target)?,
            }),
            TermTag::Known(BodyTerm::Application) => Ok(Self::Application {
                operator: field(wire, TermField::Operator)?,
                operation: field(wire, TermField::Operation)?,
                result_type: field(wire, TermField::ResultType)?,
                arguments: children(wire, TermField::Arguments)?,
            }),
            TermTag::Known(BodyTerm::Aggregate) => Ok(Self::Aggregate {
                members: children(wire, TermField::Members)?,
            }),
            TermTag::Known(BodyTerm::Binding) => Ok(Self::Binding {
                name: field(wire, TermField::Name)?,
                value: Box::new(Self::decode_admitted(
                    wire.field(TermField::Value)
                        .ok_or(TermConversionCause::MissingMember { member: "value" })?,
                )?),
            }),
            TermTag::Known(BodyTerm::DependencyReference) => Ok(Self::DependencyReference {
                package: field(wire, TermField::Package)?,
                node: field(wire, TermField::Node)?,
            }),
            TermTag::Known(BodyTerm::Frame) => Ok(Self::Frame {
                modifies: field(wire, TermField::Modifies)?,
                creates: field(wire, TermField::Creates)?,
                deletes: field(wire, TermField::Deletes)?,
                deletes_scoped: field(wire, TermField::DeletesScoped)?,
            }),
            TermTag::Known(BodyTerm::AbstractionRelation) => Ok(Self::AbstractionRelation {
                objects: field(wire, TermField::Objects)?,
                populations: field(wire, TermField::Populations)?,
                frames: field(wire, TermField::Frames)?,
            }),
            TermTag::Known(BodyTerm::ImplementationGuarantee) => {
                Ok(Self::ImplementationGuarantee {
                    clause: field(wire, TermField::Clause)?,
                    relation: field(wire, TermField::Relation)?,
                    initial: field(wire, TermField::Initial)?,
                    progress: field(wire, TermField::Progress)?,
                    environment: field(wire, TermField::Environment)?,
                })
            }
            TermTag::Known(BodyTerm::ComponentComposition) => Ok(Self::ComponentComposition {
                rule: field(wire, TermField::Rule)?,
                components: field(wire, TermField::Components)?,
                guarantees: field(wire, TermField::Guarantees)?,
                relies: field(wire, TermField::Relies)?,
                liveness_dependencies: field(wire, TermField::LivenessDependencies)?,
                supplies: field(wire, TermField::Supplies)?,
                environment: field(wire, TermField::Environment)?,
            }),
            TermTag::ExtensionForm => Ok(Self::ExtensionForm {
                definition: field(wire, TermField::Definition)?,
                node_kind: field(wire, TermField::NodeKind)?,
                clauses: children(wire, TermField::Clauses)?,
            }),
            TermTag::Missing => Err(TermConversionCause::MissingTerm),
            TermTag::NonString => Err(TermConversionCause::MalformedMember {
                member: "term",
                category: Category::Data,
            }),
            TermTag::Unknown => Err(TermConversionCause::UnknownTerm),
        }
    }
}

impl Encode for CheckedTerm {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        fn encode_rational<S: Sink + ?Sized>(
            value: &super::identity::CheckedRational,
            writer: &mut Writer<'_, S>,
        ) -> Result<(), Error> {
            writer.begin_object()?;
            writer.name("numerator")?;
            writer.string(&value.numerator)?;
            writer.name("denominator")?;
            writer.string(&value.denominator)?;
            writer.end_object()
        }
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
                type_id.encode_into(writer)?;
                writer.name("value_kind")?;
                writer.serialize(value_kind)?;
                writer.name("value")?;
                value.encode_into(writer)?;
            }
            Self::Reference { target } => {
                writer.name("term")?;
                writer.string("reference")?;
                writer.name("target")?;
                target.encode_into(writer)?;
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
                result_type.encode_into(writer)?;
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
                node.encode_into(writer)?;
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
                super::refinement::encode_rows(modifies, writer)?;
                writer.name("creates")?;
                super::refinement::encode_rows(creates, writer)?;
                writer.name("deletes")?;
                super::refinement::encode_rows(deletes, writer)?;
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
                super::refinement::encode_rows(objects, writer)?;
                writer.name("populations")?;
                super::refinement::encode_rows(populations, writer)?;
                writer.name("frames")?;
                super::refinement::encode_rows(frames, writer)?;
            }
            Self::ImplementationGuarantee {
                clause,
                relation,
                initial,
                progress,
                environment,
            } => {
                writer.name("term")?;
                writer.string("implementation_guarantee")?;
                writer.name("clause")?;
                clause.encode_into(writer)?;
                writer.name("relation")?;
                relation.encode_into(writer)?;
                writer.name("initial")?;
                writer.serialize(initial)?;
                writer.name("progress")?;
                super::guarantee_terms::encode_all(progress, writer)?;
                writer.name("environment")?;
                super::guarantee_terms::encode_all(environment, writer)?;
            }
            Self::ComponentComposition {
                rule,
                components,
                guarantees,
                relies,
                liveness_dependencies,
                supplies,
                environment,
            } => {
                writer.name("term")?;
                writer.string("component_composition")?;
                writer.name("rule")?;
                writer.serialize(rule)?;
                writer.name("components")?;
                super::guarantee_terms::encode_all(components, writer)?;
                writer.name("guarantees")?;
                super::guarantee_terms::encode_all(guarantees, writer)?;
                writer.name("relies")?;
                super::guarantee_terms::encode_all(relies, writer)?;
                writer.name("liveness_dependencies")?;
                super::guarantee_terms::encode_all(liveness_dependencies, writer)?;
                writer.name("supplies")?;
                super::guarantee_terms::encode_all(supplies, writer)?;
                writer.name("environment")?;
                super::guarantee_terms::encode_all(environment, writer)?;
            }
            Self::Refinement {
                abstract_key,
                profile,
                populations,
                objects,
                steps,
                histories,
                assume,
                ensure,
                protocol,
                abstract_internal,
                class_catalog,
            } => {
                writer.name("term")?;
                writer.string("refinement")?;
                writer.name("abstract")?;
                abstract_key.encode_into(writer)?;
                writer.name("profile")?;
                writer.serialize(profile)?;
                writer.name("populations")?;
                super::refinement::encode_rows(populations, writer)?;
                writer.name("objects")?;
                super::refinement::encode_rows(objects, writer)?;
                writer.name("steps")?;
                super::refinement::encode_rows(steps, writer)?;
                writer.name("histories")?;
                super::refinement::encode_rows(histories, writer)?;
                writer.name("assume")?;
                super::refinement::encode_rows(assume, writer)?;
                writer.name("ensure")?;
                super::refinement::encode_rows(ensure, writer)?;
                if let Some(protocol) = protocol {
                    writer.name("protocol")?;
                    protocol.encode_into(writer)?;
                }
                if let Some(internal) = abstract_internal {
                    writer.name("abstract_internal")?;
                    writer.serialize(internal)?;
                }
                if let Some(catalog) = class_catalog {
                    writer.name("class_catalog")?;
                    catalog.encode_into(writer)?;
                }
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
            Self::NominalBody(body) => match body.kind() {
                super::identity::NominalBodyKind::Enum { ordered, members } => {
                    writer.name("term")?;
                    writer.string("enum")?;
                    writer.name("ordered")?;
                    writer.serialize(ordered)?;
                    writer.name("members")?;
                    writer.serialize(members)?;
                }
                super::identity::NominalBodyKind::Dimension { base, terms } => {
                    writer.name("term")?;
                    writer.string("dimension")?;
                    writer.name("base")?;
                    writer.serialize(base)?;
                    writer.name("terms")?;
                    writer.begin_array()?;
                    for term in terms {
                        writer.begin_object()?;
                        writer.name("dimension_node_id")?;
                        term.dimension_node_id.encode_into(writer)?;
                        writer.name("exponent")?;
                        writer.string(&term.exponent)?;
                        writer.end_object()?;
                    }
                    writer.end_array()?;
                }
                super::identity::NominalBodyKind::Unit {
                    dimension_node_id,
                    target_unit_node_id,
                    scale,
                    offset,
                } => {
                    writer.name("term")?;
                    writer.string("unit")?;
                    writer.name("dimension_node_id")?;
                    dimension_node_id.encode_into(writer)?;
                    writer.name("target_unit_node_id")?;
                    if let Some(target) = target_unit_node_id {
                        target.encode_into(writer)?;
                    } else {
                        writer.serialize(&Option::<u8>::None)?;
                    }
                    writer.name("scale")?;
                    encode_rational(scale, writer)?;
                    writer.name("offset")?;
                    encode_rational(offset, writer)?;
                }
                super::identity::NominalBodyKind::DimensionEquivalence {
                    dimension_node_ids,
                    root_scale,
                } => {
                    writer.name("term")?;
                    writer.string("dimension_equivalence")?;
                    writer.name("dimension_node_ids")?;
                    writer.begin_array()?;
                    for operand in dimension_node_ids {
                        match operand {
                            super::identity::CheckedDimensionOperand::Local(id) => {
                                id.encode_into(writer)?;
                            }
                            super::identity::CheckedDimensionOperand::Package(reference) => {
                                writer.begin_object()?;
                                writer.name("package")?;
                                writer.string(&reference.package)?;
                                writer.name("node")?;
                                reference.node.node_id().encode_into(writer)?;
                                writer.end_object()?;
                            }
                        }
                    }
                    writer.end_array()?;
                    writer.name("root_scale")?;
                    encode_rational(root_scale, writer)?;
                }
            },
            Self::OperationAnchorBody(body) => {
                writer.name("term")?;
                writer.string("aggregate")?;
                writer.name("members")?;
                writer.begin_array()?;
                for member in &body.members {
                    member.encode_into(writer)?;
                }
                if let Some(point) = &body.execution_point {
                    writer.begin_object()?;
                    writer.name("term")?;
                    writer.string("binding")?;
                    writer.name("name")?;
                    writer.string("execution_point")?;
                    writer.name("value")?;
                    writer.serialize(point)?;
                    writer.end_object()?;
                }
                writer.end_array()?;
            }
        }
        writer.end_object()
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionParameter {
    /// Model parameter.
    pub name: Box<str>,
    /// Rust parameter.
    #[serde(rename = "rust_parameter")]
    pub parameter_ident: Box<str>,
}

impl Encode for CheckedFrameModification {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Relationship { declaration } => {
                writer.name("kind")?;
                writer.string("relationship")?;
                writer.name("declaration")?;
                declaration.encode_into(writer)?;
            }
            Self::Field {
                declaration,
                name,
                scope,
            } => {
                writer.name("kind")?;
                writer.string("field")?;
                writer.name("declaration")?;
                declaration.encode_into(writer)?;
                writer.name("name")?;
                writer.serialize(name)?;
                writer.name("scope")?;
                writer.serialize(scope)?;
            }
        }
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionObject {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("type")?;
        self.type_id.encode_into(writer)?;
        writer.name("rust_type")?;
        writer.serialize(&self.type_path)?;
        writer.name("fields")?;
        writer.serialize(&self.fields)?;
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionPopulation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("population")?;
        self.population.encode_into(writer)?;
        writer.name("collection")?;
        writer.serialize(&self.collection)?;
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionFrame {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("context")?;
        self.context.encode_into(writer)?;
        writer.name("operation")?;
        writer.serialize(&self.operation)?;
        writer.name("function")?;
        writer.serialize(&self.function)?;
        writer.name("receiver")?;
        writer.serialize(&self.receiver)?;
        writer.name("parameters")?;
        writer.serialize(&self.parameters)?;
        writer.end_object()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApplicationOperator, CheckedApplicationOperation, CheckedBindingName,
        CheckedBindingNameKind, CheckedFairnessGranularity, CheckedFairnessKind,
        CheckedFairnessTarget, CheckedOperationMember, CheckedOperationRecord,
        CheckedOperationViews, CheckedTerm, ExecutionPoint, TermConversionCause,
    };
    use crate::checked_package::shared::{CheckedNodeId, CheckedNodePathSegment};
    use ix_trace_rs::trace;
    use quire_canonical::{to_vec, Limits};
    use serde_json::{error::Category, json};

    /// The record and its member keys are decoded from retained views; every
    /// requested string and vector block is paid for before allocation.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn operation_document_decode_allocates_only_charged_storage() {
        use super::super::read_charge::Site;
        use super::super::{CheckedJson, NativeJoinBudget, PackageReadStage, TermNode};
        let id = json!({"package":"record-test", "path":["decl", {"operand":0}], "ordinal":0});
        for (identity, member, has_point) in [
            (
                "test.operation",
                json!({"kind":"field", "declaration":id, "name":"count"}),
                false,
            ),
            (
                "quire.op.state.clause",
                json!({"kind":"state_clause", "clause":"invariant", "execution_point":{"kind":"initialization", "name":"Start"}}),
                true,
            ),
        ] {
            let value = json!({"term":"application", "operator":"unary", "operation":{
                "identity":identity, "laws":[{"role":"text_profile", "definition":{"authority":"test", "identity":"profile"}}],
                "mode":{"kind":"text_profile", "value":"nfc"}, "member":member, "leaves":[]
            }, "result_type":id, "arguments":[]});
            let json = CheckedJson::from_serializable(&value).expect("canonical application");
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    result = Some(TermNode::of(&json).decode_in(
                        json.document(),
                        &mut budget,
                        Site::new(PackageReadStage::Graph, 7, "application"),
                        &[9],
                    ));
                })
                .bytes_total;
                (
                    result.expect("decode ran"),
                    allocated,
                    budget.read_storage_consumed(),
                )
            };
            let (result, allocated, charged) = run(u64::MAX);
            let CheckedTerm::Application { operation, .. } =
                result.expect("sufficient budget").expect("closed term")
            else {
                panic!("application");
            };
            assert_eq!(operation.identity(), Some(identity));
            assert_eq!(operation.execution_point().is_some(), has_point);
            assert_eq!(operation.member_key("declaration").is_some(), !has_point);
            if let Some(key) = operation.member_key("declaration") {
                assert!(
                    matches!(key.path.last(), Some(CheckedNodePathSegment::Index(9))),
                    "member operand slot resolves through its application's row: {key:?}"
                );
            }
            assert!(charged > 0);
            assert_eq!(
                allocated, charged,
                "{identity}: every owned record field was charged"
            );
            assert!(matches!(run(charged).0, Ok(Ok(_))));
            assert!(
                run(charged - 1).0.is_err(),
                "one byte short cannot complete the decode"
            );
        }
    }

    #[trace("FR-048-AC-7")]
    #[test]
    fn application_member_rewrite_keeps_wire_and_admitted_views_in_step() {
        let original = CheckedNodeId {
            package: "operation-map".into(),
            path: vec![CheckedNodePathSegment::Name("old".into())],
            ordinal: 0,
        };
        let replacement = CheckedNodeId {
            package: original.package.clone(),
            path: vec![CheckedNodePathSegment::Name("new".into())],
            ordinal: original.ordinal,
        };
        let member = CheckedOperationMember::Field {
            declaration: original.clone(),
            name: "field".into(),
            read_form: None,
        };
        let operation = CheckedApplicationOperation {
            record: json!({
                "identity": "quire.op.field.read", "laws": [], "mode": null,
                "member": {"kind": "field", "declaration": original, "name": "field"},
                "leaves": []
            })
            .try_into()
            .expect("closed operation fixture"),
            execution_point: None,
            views: Some(CheckedOperationViews {
                record: Some(Box::new(CheckedOperationRecord {
                    identity: "quire.op.field.read".into(),
                    mode: None,
                    laws: Vec::new(),
                    member: Some(member),
                    constraints: Vec::new(),
                    leaves: Vec::new(),
                })),
                projection_field: Some((original.clone(), "field".into())),
                member_declaration: Some(original.clone()),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        };
        let mut term = CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            result_type: original.clone(),
            arguments: Vec::new(),
        };
        let mut visits = 0;
        term.map_node_ids(|id| {
            assert_eq!(id, &original);
            visits += 1;
            Some(replacement.clone())
        });
        assert_eq!(visits, 2, "member and result are distinct logical slots");
        let CheckedTerm::Application {
            operation,
            result_type,
            ..
        } = term
        else {
            panic!("application changed form");
        };
        assert_eq!(result_type, replacement);
        assert_eq!(operation.member_declaration(), Ok(Some(&replacement)));
        assert_eq!(
            operation.projection_field(),
            Ok(Some((&replacement, "field")))
        );
        assert!(matches!(operation.read().expect("admitted view").member,
            Some(CheckedOperationMember::Field { declaration, .. }) if declaration == &replacement));
        assert_eq!(
            serde_json::to_value(&operation).expect("operation wire")["member"]["declaration"],
            serde_json::to_value(&replacement).expect("node ID wire")
        );
    }

    #[trace("FR-048-AC-7")]
    #[test]
    fn relationship_end_uses_one_callback_per_member_slot() {
        let original = CheckedNodeId {
            package: "operation-map".into(),
            path: vec![
                CheckedNodePathSegment::Name("shared".into()),
                CheckedNodePathSegment::Node(std::sync::Arc::new(CheckedNodeId {
                    package: "operation-map".into(),
                    path: vec![CheckedNodePathSegment::Name("nested".into())],
                    ordinal: 0,
                })),
            ],
            ordinal: 0,
        };
        let member = CheckedOperationMember::RelationshipEnd {
            declaration: original.clone(),
            name: "original".into(),
            destination_type: original.clone(),
            population: Some(original.clone()),
        };
        let operation = CheckedApplicationOperation {
            record: json!({
                "identity": "quire.op.relationship.read", "laws": [], "mode": null,
                "member": {"kind": "relationship_end", "declaration": original,
                    "name": "original", "destination_type": original, "population": original},
                "leaves": []
            })
            .try_into()
            .expect("closed operation fixture"),
            execution_point: None,
            views: Some(CheckedOperationViews {
                record: Some(Box::new(CheckedOperationRecord {
                    identity: "quire.op.relationship.read".into(),
                    mode: None,
                    laws: Vec::new(),
                    member: Some(member),
                    constraints: Vec::new(),
                    leaves: Vec::new(),
                })),
                member_declaration: Some(original.clone()),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        };
        let mut term = CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            result_type: original.clone(),
            arguments: Vec::new(),
        };
        let mut visits = 0;
        term.map_node_ids(|seen| {
            assert_eq!(seen, &original);
            visits += 1;
            Some(CheckedNodeId {
                package: original.package.clone(),
                path: original.path.clone(),
                ordinal: visits,
            })
        });
        assert_eq!(visits, 4, "three member slots and the result type");
        let CheckedTerm::Application {
            operation,
            result_type,
            ..
        } = term
        else {
            panic!("application changed form");
        };
        let read = operation.read().expect("admitted operation view");
        let Some(CheckedOperationMember::RelationshipEnd {
            declaration,
            destination_type,
            population: Some(population),
            name,
        }) = read.member
        else {
            panic!("relationship member changed form");
        };
        assert_eq!(
            (
                declaration.ordinal,
                destination_type.ordinal,
                population.ordinal
            ),
            (1, 2, 3)
        );
        assert_eq!(result_type.ordinal, 4);
        assert_eq!(operation.member_declaration(), Ok(Some(declaration)));
        assert_eq!(name.as_ref(), "original");
        let wire = serde_json::to_value(&operation).expect("operation wire");
        for (field, id) in [
            ("declaration", declaration),
            ("destination_type", destination_type),
            ("population", population),
        ] {
            assert_eq!(
                wire["member"][field],
                serde_json::to_value(id).expect("node ID wire")
            );
        }
    }

    #[trace("FR-048-AC-7")]
    #[test]
    fn validated_member_views_retain_nested_structural_ids() {
        let nested = CheckedNodeId {
            package: "operation-map".into(),
            path: vec![
                CheckedNodePathSegment::Name("outer".into()),
                CheckedNodePathSegment::Node(std::sync::Arc::new(CheckedNodeId {
                    package: "operation-map".into(),
                    path: vec![CheckedNodePathSegment::Name("inner".into())],
                    ordinal: 0,
                })),
            ],
            ordinal: 1,
        };
        let relationship = json!({
            "kind": "relationship_end", "declaration": nested,
            "destination_type": nested, "population": nested, "name": "end"
        });
        assert!(
            matches!(CheckedOperationMember::from_validated_value(&relationship),
            Some(CheckedOperationMember::RelationshipEnd {
                declaration, destination_type, population: Some(population), name
            }) if declaration == nested && destination_type == nested
                && population == nested && name.as_ref() == "end")
        );
        let fairness = json!({
            "kind": "fairness", "fairness_kind": "weak", "granularity": "whole",
            "target": {"kind": "parallel_branch", "parallel": nested, "branch": "left"}
        });
        assert!(
            matches!(CheckedOperationMember::from_validated_value(&fairness),
            Some(CheckedOperationMember::Fairness {
                target: CheckedFairnessTarget::ParallelBranch { parallel, branch }, ..
            }) if parallel == nested && branch.as_ref() == "left")
        );
    }

    #[trace("FR-048-AC-7")]
    #[test]
    fn single_declaration_member_variants_map_nested_ids_once() {
        let original = CheckedNodeId {
            package: "member-map".into(),
            path: vec![
                CheckedNodePathSegment::Name("outer".into()),
                CheckedNodePathSegment::Node(std::sync::Arc::new(CheckedNodeId {
                    package: "member-map".into(),
                    path: vec![CheckedNodePathSegment::Name("inner".into())],
                    ordinal: 0,
                })),
            ],
            ordinal: 0,
        };
        let replacement = CheckedNodeId {
            package: "member-map".into(),
            path: vec![CheckedNodePathSegment::Name("replacement".into())],
            ordinal: 0,
        };
        for member_wire in [
            json!({"kind":"field", "declaration":original, "name":"named"}),
            json!({"kind":"position", "declaration":original, "position":"0"}),
            json!({"kind":"element", "declaration":original}),
            json!({"kind":"operation", "declaration":original, "name":"named"}),
            json!({"kind":"type_argument", "declaration":original}),
        ] {
            let member = CheckedOperationMember::from_validated_value(&member_wire)
                .expect("validated structural member");
            let mut operation = CheckedApplicationOperation {
                record: json!({"identity":"test", "laws":[], "mode":null,
                    "member":member_wire, "leaves":[]})
                .try_into()
                .expect("closed operation fixture"),
                execution_point: None,
                views: Some(CheckedOperationViews {
                    record: Some(Box::new(CheckedOperationRecord {
                        identity: "test".into(),
                        mode: None,
                        laws: Vec::new(),
                        member: Some(member),
                        constraints: Vec::new(),
                        leaves: Vec::new(),
                    })),
                    member_declaration: Some(original.clone()),
                    ..CheckedOperationViews::default()
                }),
                #[cfg(feature = "test-support")]
                test_fixture: false,
            };
            let mut visits = 0;
            super::map_operation_ids(&mut operation, &mut |seen| {
                assert_eq!(seen, &original);
                visits += 1;
                Some(replacement.clone())
            });
            assert_eq!(visits, 1);
            assert_eq!(operation.member_declaration(), Ok(Some(&replacement)));
            assert_eq!(
                serde_json::to_value(operation.record()).expect("operation record")["member"]
                    ["declaration"],
                json!(replacement)
            );
            let Some(mapped) = operation.read().unwrap().member else {
                panic!("mapped member view");
            };
            let declaration = match mapped {
                CheckedOperationMember::Field { declaration, .. }
                | CheckedOperationMember::Position { declaration, .. }
                | CheckedOperationMember::Element { declaration }
                | CheckedOperationMember::Operation { declaration, .. }
                | CheckedOperationMember::TypeArgument { declaration } => declaration,
                _ => panic!("single-declaration member changed kind"),
            };
            assert_eq!(declaration, &replacement);
            assert_eq!(
                serde_json::to_value(operation.record()).expect("operation record")["member"]
                    ["name"],
                member_wire["name"]
            );
        }
    }

    #[trace("FR-040-AC-31", "FR-040-AC-32", "FR-040-AC-36")]
    #[test]
    fn anchor_point_is_a_closed_bare_typed_object() {
        for wire in [
            json!({"kind": "pre", "operation": "scaled"}),
            json!({"kind": "post", "operation": "scaled"}),
        ] {
            let point = ExecutionPoint::from_wire(&wire).expect("closed point");
            assert!(CheckedTerm::decode_admitted(&wire).is_err());
            assert_eq!(serde_json::to_value(&point).expect("serialize point"), wire);
            assert_eq!(
                to_vec(&point, Limits::new(u64::MAX)).expect("encode point"),
                to_vec(&wire, Limits::new(u64::MAX)).expect("encode wire")
            );
        }
        for malformed in [
            json!({"kind": "pre", "operation": ""}),
            json!({"kind": "pre", "operation": "scaled", "extra": true}),
            json!({"kind": "unknown", "operation": "scaled"}),
        ] {
            assert!(ExecutionPoint::from_wire(&malformed).is_none());
            assert!(CheckedTerm::decode_admitted(&malformed).is_err());
        }
    }
    #[trace("TC-460")]
    #[test]
    fn admitted_binding_name_retains_its_class_without_changing_wire_text() {
        let known: CheckedBindingName = serde_json::from_value(json!("parameters")).unwrap();
        assert_eq!(known.kind(), CheckedBindingNameKind::Parameters);
        assert_eq!(serde_json::to_value(&known).unwrap(), json!("parameters"));
        let open: CheckedBindingName = serde_json::from_value(json!("user-field")).unwrap();
        assert_eq!(open.kind(), CheckedBindingNameKind::Other("user-field"));
        assert_eq!(open.as_ref(), "user-field");
    }

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
            "package": "test-package", "path": ["node-a"], "ordinal": 0
        });
        let package = "dependency-package";
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
        let refinement = json!({
            "term":"refinement", "abstract":node,
            "profile":{"authority":"test","identity":"trace"},
            "populations":[{"abstract":node,"concrete":node}],
            "objects":[{"abstract_type":node,"concrete_type":node,
                "fields":[{"name":"history","expression":reference}]}],
            "steps":[
                {"source_class":"finish","source_node":node,"target":{
                    "kind":"operation","operation":node,"receiver":reference,
                    "arguments":[{"term":"open"},reference]}},
                {"source_class":"memory flush","target":{
                    "kind":"protocol_step","class":"memory flush",
                    "arguments":[{"term":"open"}]}},
                {"source_class":"memory read","target":{"kind":"stutter"}},
                {"source_class":"memory write","target":{"kind":"any"}}
            ],
            "histories":[{"concrete_type":node,"name":"history","type":node,
                "initial":literal,"updates":[{"operation":node,"expression":reference}]}],
            "assume":[{"operation":node,"kind":"weak","granularity":"each"}],
            "ensure":[{"operation":node,"kind":"strong","granularity":"whole"}],
            "protocol":node,"abstract_internal":["history"],
            "class_catalog":{"concrete":[{"class":"finish","node":node,
                "binder_types":[node]},{"class":"memory flush","memory_model":"tso"}],
                "abstract":[{"class":"finish","node":node,"binder_types":[node]}]}
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
            (refinement, "refinement"),
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
                CheckedTerm::ImplementationGuarantee { .. } => "implementation_guarantee",
                CheckedTerm::ComponentComposition { .. } => "component_composition",
                CheckedTerm::Refinement { .. } => "refinement",
                CheckedTerm::ExtensionForm { .. } => "extension_form",
                CheckedTerm::OperationAnchorBody(_) => "operation_anchor_body",
                CheckedTerm::NominalBody(_) => "nominal_body",
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
            assert_eq!(serde_json::to_value(&term).expect("term serializes"), body);
            assert_eq!(
                to_vec(&term, Limits::new(u64::MAX)).expect("term encodes"),
                to_vec(&body, Limits::new(u64::MAX)).expect("wire encodes"),
                "typed {expected} canonical bytes"
            );
        }
        assert!(
            CheckedTerm::decode_admitted(&json!({"term":"group_reference","ordinal":0})).is_err(),
            "an internal group reference cannot become a wire term"
        );
    }

    #[trace("FR-349-AC-1", "FR-349-AC-3")]
    #[test]
    fn public_producer_helpers_build_typed_terms_and_visit_reference_targets() {
        let node = serde_json::from_value(json!({
            "package": "test-package", "path": ["node-a"], "ordinal": 0
        }))
        .expect("node identity");
        let replacement: crate::checked_package::shared::CheckedNodeId =
            serde_json::from_value(json!({
                "package": "test-package", "path": ["node-b"], "ordinal": 0
            }))
            .expect("replacement identity");
        let mut body = CheckedTerm::aggregate(vec![
            CheckedTerm::binding("member", CheckedTerm::reference(node)),
            CheckedTerm::literal_text(replacement.clone(), "plain text"),
        ]);
        let targets = body.reference_targets();
        assert_eq!(targets.len(), 1);
        let old = targets[0].clone();
        body.replace_node_id(&old, &replacement);
        assert_eq!(body.reference_targets(), [&replacement]);
        let wire = serde_json::to_value(&body).expect("producer term serializes");
        assert_eq!(
            wire["members"][0]["value"]["target"]["path"],
            json!(["node-b"])
        );
        assert_eq!(wire["members"][1]["value"], "plain text");

        let refinement_wire = json!({
            "term":"refinement", "abstract":old,
            "profile":{"authority":"test","identity":"trace"},
            "populations":[],"objects":[],
            "steps":[{"source_class":"finish","source_node":old,
                "target":{"kind":"stutter"}}],
            "histories":[],"assume":[],"ensure":[],
            "class_catalog":{"concrete":[{"class":"finish","node":old,
                "binder_types":[old]}],"abstract":[]}
        });
        let mut refinement = CheckedTerm::decode_admitted(&refinement_wire)
            .expect("typed refinement through the admitted decoder");
        refinement.replace_node_id(&old, &replacement);
        let wire = serde_json::to_value(&refinement).expect("refinement serializes");
        let replacement_wire = serde_json::to_value(&replacement).expect("node serializes");
        assert_eq!(wire["abstract"], replacement_wire);
        assert_eq!(wire["steps"][0]["source_node"], replacement_wire);
        assert_eq!(
            wire["class_catalog"]["concrete"][0]["node"],
            replacement_wire
        );
        assert_eq!(
            wire["class_catalog"]["concrete"][0]["binder_types"][0],
            replacement_wire
        );
        assert_eq!(wire["steps"][0]["source_class"], "finish");
    }
    #[trace("TC-048", "FR-048-AC-7", "FR-038-AC-98", "FR-349-AC-9")]
    #[test]
    fn fairness_targets_map_one_identity_and_keep_all_views_in_sync() {
        let original = CheckedNodeId {
            package: "fair-map".into(),
            path: vec![
                CheckedNodePathSegment::Name("before".into()),
                CheckedNodePathSegment::Node(std::sync::Arc::new(CheckedNodeId {
                    package: "fair-map".into(),
                    path: vec![CheckedNodePathSegment::Name("nested".into())],
                    ordinal: 0,
                })),
            ],
            ordinal: 0,
        };
        let replacement = CheckedNodeId {
            package: original.package.clone(),
            path: vec![CheckedNodePathSegment::Name("after".into())],
            ordinal: 0,
        };
        let targets = [
            CheckedFairnessTarget::Operation {
                declaration: original.clone(),
                name: "step".into(),
            },
            CheckedFairnessTarget::ProtocolRoot {
                node: original.clone(),
            },
            CheckedFairnessTarget::AttemptNode {
                node: original.clone(),
            },
            CheckedFairnessTarget::ParallelBranch {
                parallel: original.clone(),
                branch: "left".into(),
            },
            CheckedFairnessTarget::CompensationTemplate {
                node: original.clone(),
            },
            CheckedFairnessTarget::HybridTransition {
                declaration: original.clone(),
                name: "Stopped".into(),
            },
        ];
        for target in targets {
            let declares = matches!(
                target,
                CheckedFairnessTarget::Operation { .. }
                    | CheckedFairnessTarget::HybridTransition { .. }
            );
            let operation = CheckedApplicationOperation {
                record: json!({"identity":"quire.op.temporal.fair", "laws":[], "mode":null, "member":{"kind":"fairness", "fairness_kind":"weak", "granularity":"whole", "target":target}, "leaves":[]}).try_into().expect("closed operation fixture"),
                execution_point: None,
                views: Some(CheckedOperationViews {
                    record: Some(Box::new(CheckedOperationRecord {
                        identity: "quire.op.temporal.fair".into(),
                        mode: None,
                        laws: Vec::new(),
                        member: Some(CheckedOperationMember::Fairness {
                            fairness_kind: CheckedFairnessKind::Weak,
                            granularity: CheckedFairnessGranularity::Whole,
                            target: target.clone(),
                        }),
                        constraints: Vec::new(),
                        leaves: Vec::new(),
                    })),
                    member_declaration: declares.then(|| original.clone()),
                    ..CheckedOperationViews::default()
                }),
                #[cfg(feature = "test-support")]
                test_fixture: false,
            };
            let mut term = CheckedTerm::Application {
                operator: ApplicationOperator::TemporalFairness,
                operation,
                result_type: original.clone(),
                arguments: Vec::new(),
            };
            let mut visits = 0;
            term.map_node_ids(|id| {
                assert_eq!(id, &original);
                visits += 1;
                Some(replacement.clone())
            });
            assert_eq!(visits, 2, "one target slot and one result slot");
            let CheckedTerm::Application {
                operation,
                result_type,
                ..
            } = term
            else {
                panic!("application")
            };
            assert_eq!(result_type, replacement);
            assert_eq!(
                operation.member_declaration().unwrap(),
                declares.then_some(&replacement)
            );
            let Some(CheckedOperationMember::Fairness { target: mapped, .. }) =
                operation.read().unwrap().member
            else {
                panic!("fairness")
            };
            let mut expected = target;
            *expected.node_mut() = replacement.clone();
            assert_eq!(mapped, &expected);
            assert_eq!(
                serde_json::to_value(mapped).unwrap(),
                serde_json::to_value(operation.record()).expect("operation record")["member"]
                    ["target"]
            );
        }
    }
}

#[cfg(test)]
mod selected_edge_budget_tests {
    use super::{CheckedTerm, SelectedTermTraversal, SelectedTermVisit};
    use crate::checked_package::v2::native_join_budget::{read_charge, NativeReadDenied};
    use crate::checked_package::v2::{
        CheckedNodeId, CheckedNodePathSegment, NativeJoinBudget, NativeJoinChargeKind,
    };

    #[ix_trace_rs::trace("TC-470", "FR-360-AC-2", "FR-360-AC-3")]
    #[test]
    fn runtime_term_stack_storage_stops_before_nested_aggregate_growth() {
        let id = CheckedNodeId {
            package: "test/package".into(),
            path: vec![CheckedNodePathSegment::Name("nested".into())],
            ordinal: 0,
        };
        let mut term = CheckedTerm::reference(id.clone());
        for _ in 0..64 {
            term = CheckedTerm::aggregate(vec![term]);
        }
        for (storage, expected_terms) in [(0, 0), (3, 1)] {
            let mut budget = NativeJoinBudget::new(u64::MAX, storage);
            let mut terms = 0;
            let result =
                term.visit_selected_edges_charged(&id, |event| -> Result<(), NativeReadDenied> {
                    match event {
                        SelectedTermTraversal::StackStorage(amount) => {
                            read_charge(&mut budget, NativeJoinChargeKind::Storage, amount)
                        }
                        SelectedTermTraversal::Step => {
                            read_charge(&mut budget, NativeJoinChargeKind::Work, 1)
                        }
                        SelectedTermTraversal::Visit(event) => {
                            read_charge(&mut budget, NativeJoinChargeKind::Work, 1)?;
                            if matches!(event, SelectedTermVisit::Term) {
                                terms += 1;
                            }
                            Ok(())
                        }
                    }
                });
            assert_eq!(result, Err(NativeReadDenied));
            assert_eq!(terms, expected_terms);
            assert_eq!(budget.consumed().storage, storage);
            assert_eq!(
                budget.first_unpaid().unwrap().kind,
                NativeJoinChargeKind::Storage
            );
            assert_eq!(budget.first_unpaid().unwrap().amount, 2);
        }
        let mut legacy_terms = 0;
        term.visit_selected_edges(&id, |event| {
            if matches!(event, SelectedTermVisit::Term) {
                legacy_terms += 1;
            }
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(legacy_terms, 65);
    }
}
