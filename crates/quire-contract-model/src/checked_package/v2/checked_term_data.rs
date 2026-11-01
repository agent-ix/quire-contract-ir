//! Producer-captured checked-term decisions retained by the V2 reader.
//!
//! QSpec FR-322 defines this optional closed sidecar. A present empty dispatch
//! candidate list is an admitted producer decision; an absent table is not.

use super::CheckedRational;
use crate::checked_package::shared::{CheckedNodeId, CheckedOccurrence};
use serde::{Deserialize, Serialize};

/// The three producer-captured decision arrays in a present V2 sidecar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedTermDataV2 {
    /// Selected equality schedules and their operands.
    pub equality_decisions: Vec<CheckedEqualityDecision>,
    /// Selected semantic locations for protocol terms.
    pub protocol_locations: Vec<CheckedProtocolSemanticLocation>,
    /// Selected model-operation candidate tables.
    pub dispatch_tables: Vec<CheckedDispatchCandidateTable>,
}

/// One captured equality decision at an exact application occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedEqualityDecision {
    /// Local application node.
    pub application: CheckedNodeId,
    /// Exact source occurrence of the application.
    pub occurrence: CheckedOccurrence,
    /// Child steps from the application's body root to the compared application.
    pub term_path: Vec<CheckedEqualityTermPathStep>,
    /// Equality operation selected by the producer.
    pub operator: CheckedEqualityOperator,
    /// Left operand decision.
    pub left: CheckedEqualityOperandDecision,
    /// Right operand decision.
    pub right: CheckedEqualityOperandDecision,
    /// Selected equality schedule.
    pub schedule: CheckedEqualitySchedule,
    /// Only the units consulted by the producer for this comparison.
    pub units: Vec<CheckedResolvedEqualityUnit>,
    /// Only the compared enum's retained members.
    pub enum_members: Vec<CheckedEqualityEnumMember>,
}

/// One typed step from a node body root to a nested equality application.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedEqualityTermPathStep {
    /// An application argument by position.
    Argument {
        /// Zero-based argument index.
        index: CheckedKeyOrdinal,
    },
    /// An aggregate member by position.
    Member {
        /// Zero-based member index.
        index: CheckedKeyOrdinal,
    },
    /// The value of a named aggregate binding.
    BindingValue,
}

/// The closed equality operation vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedEqualityOperator {
    /// Equality.
    Equal,
    /// Inequality.
    NotEqual,
}

/// One operand's original position, type and selected conversion target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedEqualityOperandDecision {
    /// Zero-based application argument position; only zero and one are valid.
    pub operand: u8,
    /// Admitted source type node.
    pub source_type: CheckedNodeId,
    /// Producer-selected target type, or no conversion.
    #[serde(deserialize_with = "super::required_nullable")]
    pub conversion_target: Option<CheckedNodeId>,
}

/// The producer-selected equality procedure.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedEqualitySchedule {
    /// Text comparison.
    Text,
    /// Enum comparison.
    Enum,
    /// Quantity comparison.
    Quantity,
    /// Plan comparison.
    Plan,
}

/// One resolved unit fact used by a quantity comparison.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedResolvedEqualityUnit {
    /// Quantity type whose unit was resolved.
    pub quantity_type: CheckedNodeId,
    /// Admitted unit node.
    pub unit: CheckedNodeId,
    /// Declared or compound resolution.
    pub form: CheckedResolvedEqualityUnitForm,
}

/// A producer-resolved declared or compound unit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedResolvedEqualityUnitForm {
    /// A declared unit and its ordered conversion path.
    Declared {
        /// Dimension declaration.
        dimension_node: CheckedNodeId,
        /// Normalized base-dimension powers.
        dimension_terms: Vec<CheckedUnitDimensionTerm>,
        /// Root declared unit.
        root_unit: CheckedNodeId,
        /// Ordered exact conversion edges.
        path: Vec<CheckedUnitConversionEdge>,
    },
    /// A compound unit with canonical root-unit powers.
    Compound {
        /// Normalized base-dimension powers.
        dimension_terms: Vec<CheckedUnitDimensionTerm>,
        /// Canonical root-unit powers.
        terms: Vec<CheckedCompoundEqualityUnitTerm>,
    },
}

/// One dimension and exact exponent of a resolved unit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedUnitDimensionTerm {
    /// Selected dimension node.
    pub dimension: CheckedNodeId,
    /// Canonical integer exponent text.
    pub exponent: Box<str>,
}

/// One exact edge on a declared unit's path to its root.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedUnitConversionEdge {
    /// Target unit of the edge.
    pub target: CheckedNodeId,
    /// Exact multiplicative scale.
    pub scale: CheckedRational,
    /// Exact additive offset.
    pub offset: CheckedRational,
}

/// One root unit and exponent in a compound unit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedCompoundEqualityUnitTerm {
    /// Root declared unit.
    pub root_unit: CheckedNodeId,
    /// Canonical integer exponent text.
    pub exponent: Box<str>,
}

/// A selected enum member retained for one equality comparison.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedEqualityEnumMember {
    /// Compared enum declaration.
    pub enum_type: CheckedNodeId,
    /// Selected member identity.
    pub member: CheckedNodeId,
    /// Authored case name.
    pub case: Box<str>,
    /// Producer-selected case position.
    pub ordinal: u64,
    /// Whether the enum is ordered.
    pub ordered: bool,
}

/// One producer-selected semantic location for a protocol term.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedProtocolSemanticLocation {
    /// Local application node.
    pub application: CheckedNodeId,
    /// Exact occurrence of that application.
    pub occurrence: CheckedOccurrence,
    /// Zero-based term coordinate within the occurrence.
    pub term_ordinal: u64,
    /// Retained origin and child-index path.
    pub location: CheckedProtocolLocation,
}

/// The semantic origin and child path of one protocol term.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedProtocolLocation {
    /// Authored declaration or protocol attempt origin.
    pub origin: CheckedProtocolLocationOrigin,
    /// Child positions from the origin to this term.
    pub path: Vec<CheckedKeyOrdinal>,
}

/// The two closed kinds of protocol semantic origin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedProtocolLocationOrigin {
    /// A named type declaration.
    TypeDeclaration {
        /// Selected declaration name.
        name: Box<str>,
    },
    /// One attempt within a selected protocol.
    ProtocolAttempt {
        /// Protocol position.
        protocol: u64,
        /// Attempt position.
        attempt: u64,
    },
}

/// A graph position represented by a safe JSON integer or a larger canonical string.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CheckedKeyOrdinal {
    /// JSON integer within the exact safe range.
    Safe(u64),
    /// Canonical decimal text strictly above the exact safe range.
    BeyondSafe(Box<str>),
}

/// An ordered candidate table captured for one selected operation call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDispatchCandidateTable {
    /// Local call application node.
    pub application: CheckedNodeId,
    /// Exact occurrence of that call.
    pub occurrence: CheckedOccurrence,
    /// Producer-selected operation node.
    pub operation: CheckedNodeId,
    /// Receiver's selected model context node.
    pub context: CheckedNodeId,
    /// Candidates in producer order, including an authoritative empty set.
    pub candidates: Vec<CheckedDispatchCandidate>,
}

/// One candidate's structural selection key and executable term handles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDispatchCandidate {
    /// FR-150 structural identity of the receiver's effective declaration.
    pub selection_key: CheckedEffectiveId,
    /// Admitted function body handle.
    pub operation_body: CheckedNodeId,
    /// Effective precondition body handle, or deliberately no body.
    #[serde(deserialize_with = "super::required_nullable")]
    pub effective_precondition: Option<CheckedNodeId>,
}

/// FR-150 structural effective declaration identity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedEffectiveId {
    /// Owning effective type, absent for an effective type declaration.
    #[serde(deserialize_with = "super::required_nullable")]
    pub owner_effective_type: Option<Box<CheckedEffectiveId>>,
    /// Exact original declaration key.
    pub original: CheckedOriginalDeclarationId,
}

/// The package and IR node identity of an original model declaration.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOriginalDeclarationId {
    /// Selected domain package identity.
    pub package: Box<str>,
    /// Original IR declaration identity.
    pub node: Box<str>,
}
