//! FR-035 and QSpec FR-330: the single target-neutral ContractPackage wire.

mod guard_context;
mod result_context;
mod scalar_operands;

pub use guard_context::{
    ContractGuardContextErrorV1, ContractGuardContextFailureCauseV1, ContractGuardContextFailureV1,
    ContractGuardContextViewV1, ContractGuardReferenceV1, ContractLoweredGuardContextV1,
    ContractPathGuardV1,
};
pub use result_context::{
    ContractLoweredResultV1, ContractOccurrenceKeyV1, ContractResultBoundV1,
    ContractResultContextV1, ContractResultViewV1,
};
pub use scalar_operands::{
    ContractLoweredScalarOperandsV1, ContractScalarOperandChildV1, ContractScalarOperandErrorV1,
    ContractScalarOperandV1, ContractScalarOperandViewV1, ContractScalarOwnRangeV1,
};

use super::dependency_references::{
    check_derived_closure, DependencyReferences, Referrer, SuppliedDependencies,
};
use super::encode::{elements, encoded, member as encode_member};
use super::lower::{
    CompleteContractNodeV2, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    ContractPackageDependencyV2, LoweringProfileRef, NodeSourceMaps, CONTRACT_PACKAGE_VERSION,
};
use super::{
    CheckedNodeKind, CheckedNodeTag, CheckedPackageLockV2, CheckedPackageV2, CheckedSelectionRole,
    CheckedTerm, RawSemanticNodeV2 as CheckedSemanticNodeV2,
};
use crate::checked_package::common::{
    read_typed_prefix, strict_parse, Step, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedOccurrenceRole, CheckedPackageIncomplete,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
use quire_canonical::{Encode, Error as CanonicalError, FixedShape, Sink, Writer};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

/// The source package's exact admitted identity and lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractSourceV1 {
    /// Originating admitted checked-package identity.
    pub name: Box<str>,
    /// Exact originating lock.
    pub lock: CheckedPackageLockV2,
}

/// Source provenance and the caller's explicit lowering choice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractProvenanceV1 {
    /// Source identity and lock.
    pub source: ContractSourceV1,
    /// Selected lowering definition.
    pub lowering_profile: LoweringProfileRef,
}

/// Exact admitted semantic nodes in canonical recursion-aware order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSemanticGraphV1 {
    /// The FR-322 graph version.
    pub graph_version: Box<str>,
    /// Nodes represented by at least one lowered request, including their closure.
    pub nodes: Vec<CheckedSemanticNodeV2>,
}

/// One represented graph node's exact source occurrences.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractNodeSourcesV1 {
    /// Represented semantic node key.
    pub node_id: CheckedNodeId,
    /// Exact FR-322 source-map entries for the node.
    pub occurrences: Vec<CheckedSourceMapEntry>,
}

/// One requested item's source occurrences, including an absent request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRequestSourcesV1 {
    /// Exact requested key.
    pub requested_item: CheckedNodeId,
    /// Available source occurrences, empty for an absent key.
    pub occurrences: Vec<CheckedSourceMapEntry>,
}

/// The seven terminal disposition names.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum ContractDispositionV1 {
    /// Exact represented meaning.
    Lowered,
    /// Unsupported family.
    Unsupported,
    /// Missing required bound.
    RequiresBound,
    /// Invalid request.
    InvalidInput,
    /// Resource charge failed.
    Failed,
    /// Defensive body refusal.
    InvalidBody,
    /// Defensive body limit stop.
    BodyIncomplete,
}

impl ContractDispositionV1 {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Option<Self> {
        serde_json::from_value(Value::String(value.to_owned())).ok()
    }
}

/// A refused request's exact available sources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRefusalSourcesV1 {
    /// Exact requested key.
    pub requested_item: CheckedNodeId,
    /// Matching terminal disposition.
    pub disposition: ContractDispositionV1,
    /// Available source occurrences.
    pub occurrences: Vec<CheckedSourceMapEntry>,
}

/// Total source correspondence for graph nodes and requests.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSourceMapV1 {
    /// One entry per represented node, in semantic-graph order.
    pub nodes: Vec<ContractNodeSourcesV1>,
    /// One entry per request, in request order.
    pub requested_items: Vec<ContractRequestSourcesV1>,
    /// One entry per non-lowered request, in request order.
    pub refusals: Vec<ContractRefusalSourcesV1>,
}

/// Closed projection of a defensive FR-322 refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractBodyRefusalV1 {
    /// FR-322 refusal code.
    pub code: CheckedPackageRefusalCode,
    /// Exact present source-document pointer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<JsonPointer>,
    /// Exact present FR-272 cause.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<CheckedPackageRefusalCause>,
    /// Exact present node locus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locus: Option<CheckedNodeId>,
    /// Required only for an unknown contract version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<Box<str>>,
    /// Exact present supplied-document pointer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_pointer: Option<JsonPointer>,
}

impl From<&CheckedPackageRefusal> for ContractBodyRefusalV1 {
    fn from(value: &CheckedPackageRefusal) -> Self {
        Self {
            code: value.code,
            path: value.primary_path().cloned(),
            cause: value.cause,
            locus: value.locus.clone(),
            contract_version: value.contract_version.clone(),
            document_pointer: value.document_pointer.clone(),
        }
    }
}

/// Closed projection of a defensive FR-322 limit stop.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractBodyIncompleteV1 {
    /// Exact limit kind.
    pub limit_kind: CheckedPackageLimit,
    /// Canonical unsigned decimal u64.
    pub limit: Box<str>,
    /// Canonical unsigned decimal u64.
    pub consumed: Box<str>,
    /// Exact present pointer, omitted for bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<JsonPointer>,
}

impl From<&CheckedPackageIncomplete> for ContractBodyIncompleteV1 {
    fn from(value: &CheckedPackageIncomplete) -> Self {
        Self {
            limit_kind: value.limit_kind,
            limit: value.limit.to_string().into_boxed_str(),
            consumed: value.consumed.to_string().into_boxed_str(),
            path: value.path.clone(),
        }
    }
}

/// One exact terminal outcome, serialized as the closed FR-330 tagged record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "disposition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContractFeatureRecordV1 {
    /// Exact represented root.
    Lowered { node_id: CheckedNodeId },
    /// First unsupported node in key order.
    Unsupported {
        node_id: CheckedNodeId,
        unsupported_node_id: CheckedNodeId,
        node_tag: Box<str>,
    },
    /// First unbounded type in key order.
    RequiresBound {
        node_id: CheckedNodeId,
        unbounded_type: CheckedNodeId,
    },
    /// Absent or invalid requested key.
    InvalidInput { node_id: CheckedNodeId },
    /// Failed measured charge.
    Failed {
        node_id: CheckedNodeId,
        limit_kind: CheckedPackageLimit,
        limit: Box<str>,
        consumed: Box<str>,
    },
    /// Defensive body refusal.
    InvalidBody {
        node_id: CheckedNodeId,
        body_node_id: CheckedNodeId,
        refusal: ContractBodyRefusalV1,
    },
    /// Defensive body limit stop.
    BodyIncomplete {
        node_id: CheckedNodeId,
        body_node_id: CheckedNodeId,
        incomplete: ContractBodyIncompleteV1,
    },
}

impl<'de> Deserialize<'de> for ContractFeatureRecordV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        fn take(map: &mut serde_json::Map<String, Value>, name: &str) -> Result<Value, String> {
            map.remove(name).ok_or_else(|| format!("missing {name}"))
        }

        fn node(
            map: &mut serde_json::Map<String, Value>,
            name: &str,
        ) -> Result<CheckedNodeId, String> {
            CheckedNodeId::from_value(&take(map, name)?).ok_or_else(|| format!("invalid {name}"))
        }

        fn typed<T: serde::de::DeserializeOwned>(
            map: &mut serde_json::Map<String, Value>,
            name: &str,
        ) -> Result<T, String> {
            serde_json::from_value(take(map, name)?).map_err(|error| error.to_string())
        }

        let mut map = match Value::deserialize(deserializer)? {
            Value::Object(map) => map,
            _ => return Err(D::Error::custom("feature record must be an object")),
        };
        let disposition: ContractDispositionV1 =
            typed(&mut map, "disposition").map_err(D::Error::custom)?;
        let node_id = node(&mut map, "node_id").map_err(D::Error::custom)?;
        let record = match disposition {
            ContractDispositionV1::Lowered => Self::Lowered { node_id },
            ContractDispositionV1::Unsupported => Self::Unsupported {
                node_id,
                unsupported_node_id: node(&mut map, "unsupported_node_id")
                    .map_err(D::Error::custom)?,
                node_tag: typed(&mut map, "node_tag").map_err(D::Error::custom)?,
            },
            ContractDispositionV1::RequiresBound => Self::RequiresBound {
                node_id,
                unbounded_type: node(&mut map, "unbounded_type").map_err(D::Error::custom)?,
            },
            ContractDispositionV1::InvalidInput => Self::InvalidInput { node_id },
            ContractDispositionV1::Failed => Self::Failed {
                node_id,
                limit_kind: typed(&mut map, "limit_kind").map_err(D::Error::custom)?,
                limit: typed(&mut map, "limit").map_err(D::Error::custom)?,
                consumed: typed(&mut map, "consumed").map_err(D::Error::custom)?,
            },
            ContractDispositionV1::InvalidBody => {
                let mut refusal: Value = take(&mut map, "refusal").map_err(D::Error::custom)?;
                let locus = refusal
                    .as_object_mut()
                    .and_then(|object| object.remove("locus"))
                    .map(|value| {
                        CheckedNodeId::from_value(&value)
                            .ok_or_else(|| D::Error::custom("invalid refusal locus"))
                    })
                    .transpose()?;
                let mut refusal: ContractBodyRefusalV1 =
                    serde_json::from_value(refusal).map_err(D::Error::custom)?;
                refusal.locus = locus;
                Self::InvalidBody {
                    node_id,
                    body_node_id: node(&mut map, "body_node_id").map_err(D::Error::custom)?,
                    refusal,
                }
            }
            ContractDispositionV1::BodyIncomplete => Self::BodyIncomplete {
                node_id,
                body_node_id: node(&mut map, "body_node_id").map_err(D::Error::custom)?,
                incomplete: typed(&mut map, "incomplete").map_err(D::Error::custom)?,
            },
        };
        if !map.is_empty() {
            return Err(D::Error::custom("unknown feature record member"));
        }
        Ok(record)
    }
}

impl ContractFeatureRecordV1 {
    /// Exact requested key.
    pub fn node_id(&self) -> &CheckedNodeId {
        match self {
            Self::Lowered { node_id }
            | Self::Unsupported { node_id, .. }
            | Self::RequiresBound { node_id, .. }
            | Self::InvalidInput { node_id }
            | Self::Failed { node_id, .. }
            | Self::InvalidBody { node_id, .. }
            | Self::BodyIncomplete { node_id, .. } => node_id,
        }
    }

    /// Closed disposition name.
    pub const fn disposition(&self) -> ContractDispositionV1 {
        match self {
            Self::Lowered { .. } => ContractDispositionV1::Lowered,
            Self::Unsupported { .. } => ContractDispositionV1::Unsupported,
            Self::RequiresBound { .. } => ContractDispositionV1::RequiresBound,
            Self::InvalidInput { .. } => ContractDispositionV1::InvalidInput,
            Self::Failed { .. } => ContractDispositionV1::Failed,
            Self::InvalidBody { .. } => ContractDispositionV1::InvalidBody,
            Self::BodyIncomplete { .. } => ContractDispositionV1::BodyIncomplete,
        }
    }
}

impl From<&CompleteLoweringRecordV2> for ContractFeatureRecordV1 {
    fn from(record: &CompleteLoweringRecordV2) -> Self {
        match record {
            CompleteLoweringRecordV2::Lowered { node } => Self::Lowered {
                node_id: node.node.node_id.clone(),
            },
            CompleteLoweringRecordV2::Unsupported {
                node_id,
                unsupported_node_id,
                node_tag,
            } => Self::Unsupported {
                node_id: node_id.clone(),
                unsupported_node_id: unsupported_node_id.clone(),
                node_tag: node_tag.as_wire().into(),
            },
            CompleteLoweringRecordV2::RequiresBound {
                node_id,
                unbounded_type,
            } => Self::RequiresBound {
                node_id: node_id.clone(),
                unbounded_type: unbounded_type.clone(),
            },
            CompleteLoweringRecordV2::InvalidInput { node_id } => Self::InvalidInput {
                node_id: node_id.clone(),
            },
            CompleteLoweringRecordV2::Failed {
                node_id,
                limit_kind,
                limit,
                consumed,
            } => Self::Failed {
                node_id: node_id.clone(),
                limit_kind: *limit_kind,
                limit: limit.to_string().into_boxed_str(),
                consumed: consumed.to_string().into_boxed_str(),
            },
            CompleteLoweringRecordV2::InvalidBody {
                node_id,
                body_node_id,
                refusal,
            } => Self::InvalidBody {
                node_id: node_id.clone(),
                body_node_id: body_node_id.clone(),
                refusal: refusal.into(),
            },
            CompleteLoweringRecordV2::BodyIncomplete {
                node_id,
                body_node_id,
                incomplete,
            } => Self::BodyIncomplete {
                node_id: node_id.clone(),
                body_node_id: body_node_id.clone(),
                incomplete: incomplete.into(),
            },
        }
    }
}

/// The one closed six-member FR-330 package value.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractPackageWireV1 {
    /// Exactly `quire.contract-package/v1`.
    pub contract_version: Box<str>,
    /// Exact source provenance and explicit lowering profile.
    pub package: ContractProvenanceV1,
    /// One canonically ordered semantic node collection.
    pub semantic_graph: ContractSemanticGraphV1,
    /// Source-frame node identities, closed under their indexed operands.
    pub source_keys: Vec<CheckedNodeId>,
    /// Total source correspondence.
    pub source_map: ContractSourceMapV1,
    /// One result per request, in request order.
    pub feature_accounting: Vec<ContractFeatureRecordV1>,
}

#[derive(Serialize)]
struct ExpandedContractWire<'a> {
    contract_version: &'a str,
    package: &'a ContractProvenanceV1,
    semantic_graph: &'a ContractSemanticGraphV1,
    source_keys: &'a [CheckedNodeId],
    source_map: &'a ContractSourceMapV1,
    feature_accounting: &'a [ContractFeatureRecordV1],
}

fn expanded_wire_value(wire: &ContractPackageWireV1) -> Result<Value, serde_json::Error> {
    serde_json::to_value(ExpandedContractWire {
        contract_version: &wire.contract_version,
        package: &wire.package,
        semantic_graph: &wire.semantic_graph,
        source_keys: &wire.source_keys,
        source_map: &wire.source_map,
        feature_accounting: &wire.feature_accounting,
    })
}

impl ContractPackageWireV1 {
    fn raw_wire_value(&self) -> Result<Value, String> {
        let mut value = expanded_wire_value(self).map_err(|error| error.to_string())?;
        let positions = self
            .source_keys
            .iter()
            .enumerate()
            .map(|(position, id)| (id, position))
            .collect::<BTreeMap<_, _>>();
        for at in source_ref_paths(&value) {
            let id = value
                .pointer(at.as_str())
                .and_then(CheckedNodeId::from_value)
                .ok_or_else(|| format!("invalid source-frame id at {at}"))?;
            let position = *positions
                .get(&id)
                .ok_or_else(|| format!("unlisted source-frame id at {at}"))?;
            let reference = value
                .pointer_mut(at.as_str())
                .ok_or_else(|| format!("missing source-frame id at {at}"))?;
            *reference = serde_json::json!({ "source_key": position });
        }
        Ok(value)
    }
}

impl Serialize for ContractPackageWireV1 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw_wire_value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

/// A ContractPackage produced from an admitted source or admitted by the sole
/// strict reader. Consumers take this type so a raw wire value cannot bypass
/// admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPackageV1 {
    wire: ContractPackageWireV1,
    frames: ContractFrameKeys,
    node_kinds: Vec<CheckedNodeKind>,
    terms: Vec<Option<CheckedTerm>>,
    node_index: BTreeMap<CheckedNodeId, usize>,
    source_index: BTreeMap<CheckedNodeId, usize>,
    unsupported_tags: Vec<Option<CheckedNodeTag>>,
    unsupported_source_kinds: Option<Vec<Option<CheckedNodeKind>>>,
}

/// One admitted structural identity for each raw coordinate in the two
/// independent FR-330 frames. The maps are shallow wire-key lookups; their
/// values are shared Arc DAG keys rebuilt by the common FR-322 resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ContractFrameKeys {
    output: BTreeMap<CheckedNodeId, Arc<CheckedNodeId>>,
    source: BTreeMap<CheckedNodeId, Arc<CheckedNodeId>>,
}

impl ContractFrameKeys {
    fn output(&self, id: &CheckedNodeId) -> Option<&CheckedNodeId> {
        self.output.get(id).map(Arc::as_ref)
    }

    fn source(&self, id: &CheckedNodeId) -> Option<&CheckedNodeId> {
        self.source.get(id).map(Arc::as_ref)
    }

    fn source_equals_output(&self, source: &CheckedNodeId, output: &CheckedNodeId) -> bool {
        self.source(source)
            .zip(self.output(output))
            .is_some_and(|(a, b)| a == b)
    }

    fn source_equals_source(&self, first: &CheckedNodeId, second: &CheckedNodeId) -> bool {
        self.source(first)
            .zip(self.source(second))
            .is_some_and(|(a, b)| a == b)
    }
}

/// The requested-root kind has no authenticated V2 source in a plain V1 read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ContractUnsupportedSourceKindError {
    /// The query requires a producer or an authenticated source handoff.
    #[error("requested-root kind was not admitted from a V2 source")]
    NotAdmitted,
}

pub(super) struct GuardContextWork {
    limit: Option<u64>,
    consumed: u64,
}

impl GuardContextWork {
    pub(super) fn unbounded_view() -> Self {
        Self {
            limit: None,
            consumed: 0,
        }
    }

    #[cfg(feature = "fault-injection")]
    pub(super) fn with_limit(limit: u64) -> Self {
        Self {
            limit: Some(limit),
            consumed: 0,
        }
    }

    pub(super) fn bounded_query() -> Self {
        Self {
            limit: Some(CheckedPackageReadLimits::bounded().work),
            consumed: 0,
        }
    }

    pub(super) fn charge(&mut self, amount: u64) -> Result<(), CheckedPackageIncomplete> {
        let consumed = self.consumed.saturating_add(amount);
        if self.limit.is_some_and(|limit| consumed > limit) {
            return Err(CheckedPackageIncomplete {
                limit_kind: CheckedPackageLimit::Work,
                limit: self.limit.unwrap_or(u64::MAX),
                consumed,
                path: None,
            });
        }
        self.consumed = consumed;
        Ok(())
    }
}

pub(super) struct ContractPackageTraversalIndex<'a> {
    positions: &'a BTreeMap<CheckedNodeId, usize>,
    sources: BTreeMap<&'a CheckedNodeId, &'a [CheckedSourceMapEntry]>,
    accounting: BTreeMap<&'a CheckedNodeId, &'a ContractFeatureRecordV1>,
}

fn json_value_count(root: &Value) -> Option<u64> {
    let mut pending = vec![root];
    let mut count = 0_u64;
    while let Some(value) = pending.pop() {
        count = count.checked_add(1)?;
        match value {
            Value::Array(values) => pending.extend(values.iter()),
            Value::Object(values) => pending.extend(values.values()),
            _ => {}
        }
    }
    Some(count)
}

impl<'a> ContractPackageTraversalIndex<'a> {
    pub(super) fn build(
        package: &'a ContractPackageV1,
        work: &mut GuardContextWork,
    ) -> Result<Self, CheckedPackageIncomplete> {
        let node_count = u64::try_from(package.wire.semantic_graph.nodes.len()).unwrap_or(u64::MAX);
        let source_count = package
            .wire
            .source_map
            .nodes
            .iter()
            .try_fold(0_u64, |total, entry| {
                total
                    .checked_add(1)?
                    .checked_add(u64::try_from(entry.occurrences.len()).ok()?)
            })
            .unwrap_or(u64::MAX);
        let accounting_count =
            u64::try_from(package.wire.feature_accounting.len()).unwrap_or(u64::MAX);
        work.charge(
            node_count
                .saturating_add(source_count)
                .saturating_add(accounting_count),
        )?;

        let sources = package
            .wire
            .source_map
            .nodes
            .iter()
            .map(|entry| (&entry.node_id, entry.occurrences.as_slice()))
            .collect();
        let mut accounting = BTreeMap::new();
        for record in &package.wire.feature_accounting {
            accounting
                .entry(record.node_id())
                .and_modify(|existing: &mut &ContractFeatureRecordV1| {
                    if existing.disposition() != ContractDispositionV1::Lowered
                        && record.disposition() == ContractDispositionV1::Lowered
                    {
                        *existing = record;
                    }
                })
                .or_insert(record);
        }
        Ok(Self {
            positions: &package.node_index,
            sources,
            accounting,
        })
    }
}

/// One reached local node and its exact admitted source correspondence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractReachedNodeV1<'a> {
    /// The admitted semantic node, addressed by its existing typed key.
    pub node: &'a CheckedSemanticNodeV2,
    /// The node's admitted typed family and form.
    pub kind: CheckedNodeKind,
    /// Every admitted source-map occurrence for that node.
    pub sources: &'a [CheckedSourceMapEntry],
}

/// A selected external reference at its admitted package boundary.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContractSelectedReferenceV1 {
    /// Identity of the selected dependency package.
    pub package: Box<str>,
    /// Typed target key in that selected package.
    pub node: CheckedNodeId,
}

/// The exact local closure of one lowered request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractReachedClosureV1<'a> {
    /// Requested root, also present in `nodes`.
    pub root: &'a CheckedSemanticNodeV2,
    /// The requested root's admitted typed family and form.
    pub root_kind: CheckedNodeKind,
    /// Reached local nodes once each, in ascending typed-key order.
    pub nodes: Vec<ContractReachedNodeV1<'a>>,
    /// Selected external package/key pairs, never flattened into `nodes`.
    pub selected_references: Vec<ContractSelectedReferenceV1>,
    /// Reached bounded-domain keys excluding the root, in typed-key order.
    pub bounds: Vec<CheckedNodeId>,
    /// Reached claim keys excluding the root, in typed-key order.
    pub claims: Vec<CheckedNodeId>,
}

/// Exact result of querying a requested key on an admitted package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractRootViewV1<'a> {
    /// The key has a lowered request record and its reached closure.
    Lowered(ContractReachedClosureV1<'a>),
    /// The exact non-lowered request record, with its typed disposition.
    NonLowered(&'a ContractFeatureRecordV1),
    /// No request record exists for this exact key.
    Absent,
}

/// A defensive failure if the admitted body walk can no longer be reproduced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractRootViewError {
    /// The shared body validator refused a reached node.
    Refused(Box<CheckedPackageRefusal>),
    /// The shared body validator reached its declared limit.
    Incomplete(CheckedPackageIncomplete),
    /// An admitted graph or source-map edge no longer has correspondence.
    Inconsistent,
}

impl From<ValidationFailure> for ContractRootViewError {
    fn from(failure: ValidationFailure) -> Self {
        match failure {
            ValidationFailure::Refused(refusal) => Self::Refused(refusal),
            ValidationFailure::Incomplete(incomplete) => Self::Incomplete(incomplete),
        }
    }
}

impl ContractPackageV1 {
    pub(super) fn reached_closure_indexed<'a>(
        &'a self,
        requested: &CheckedNodeId,
        index: &ContractPackageTraversalIndex<'a>,
        work: &mut GuardContextWork,
    ) -> Result<ContractRootViewV1<'a>, ContractRootViewError> {
        work.charge(1).map_err(ContractRootViewError::Incomplete)?;
        let Some(record) = index.accounting.get(requested).copied() else {
            return Ok(ContractRootViewV1::Absent);
        };
        if record.disposition() != ContractDispositionV1::Lowered {
            return Ok(ContractRootViewV1::NonLowered(record));
        }
        let nodes = &self.wire.semantic_graph.nodes;
        let Some(&root_position) = index.positions.get(requested) else {
            return Err(ContractRootViewError::Inconsistent);
        };
        let root = &nodes[root_position];
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([requested.clone()]);
        let mut selected_references = BTreeSet::new();
        while let Some(key) = pending.pop_front() {
            work.charge(1).map_err(ContractRootViewError::Incomplete)?;
            if !visited.insert(key.clone()) {
                continue;
            }
            let Some(&position) = index.positions.get(&key) else {
                return Err(ContractRootViewError::Inconsistent);
            };
            let node = &nodes[position];
            let edge_count = u64::try_from(node.dependencies.len())
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            work.charge(edge_count)
                .map_err(ContractRootViewError::Incomplete)?;
            pending.push_back(node.semantic_type.clone());
            pending.extend(node.dependencies.iter().cloned());
            let kind = *self
                .node_kinds
                .get(position)
                .ok_or(ContractRootViewError::Inconsistent)?;
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(position),
                Step::Key("body"),
            ];
            let body_units = json_value_count(&node.body).unwrap_or(u64::MAX);
            work.charge(body_units.saturating_mul(4))
                .map_err(ContractRootViewError::Incomplete)?;
            super::validate_body(
                kind,
                &node.body,
                &Trail::Base(&steps),
                &mut |target, _site, _at| pending.push_back(target.clone()),
            )?;
            let body = self
                .terms
                .get(position)
                .and_then(Option::as_ref)
                .ok_or(ContractRootViewError::Inconsistent)?;
            let selected = super::dependency_references::selected_reference_targets(body);
            for (package, node) in selected {
                work.charge(1).map_err(ContractRootViewError::Incomplete)?;
                selected_references.insert(ContractSelectedReferenceV1 { package, node });
            }
        }
        work.charge(u64::try_from(visited.len()).unwrap_or(u64::MAX))
            .map_err(ContractRootViewError::Incomplete)?;
        let reached = visited
            .iter()
            .map(|key| {
                let position = index
                    .positions
                    .get(key)
                    .ok_or(ContractRootViewError::Inconsistent)?;
                let node = nodes
                    .get(*position)
                    .ok_or(ContractRootViewError::Inconsistent)?;
                let sources = index
                    .sources
                    .get(key)
                    .ok_or(ContractRootViewError::Inconsistent)?;
                work.charge(1_u64.saturating_add(u64::try_from(sources.len()).unwrap_or(u64::MAX)))
                    .map_err(ContractRootViewError::Incomplete)?;
                let kind = *self
                    .node_kinds
                    .get(*position)
                    .ok_or(ContractRootViewError::Inconsistent)?;
                Ok(ContractReachedNodeV1 {
                    node,
                    kind,
                    sources,
                })
            })
            .collect::<Result<Vec<_>, ContractRootViewError>>()?;
        work.charge(u64::try_from(reached.len()).unwrap_or(u64::MAX))
            .map_err(ContractRootViewError::Incomplete)?;
        let classified = reached
            .iter()
            .map(|entry| (&entry.node.node_id, entry.kind.tag()));
        let (bounds, claims) = super::lower::classify_reached_keys(requested, classified);
        Ok(ContractRootViewV1::Lowered(ContractReachedClosureV1 {
            root,
            root_kind: *self
                .node_kinds
                .get(root_position)
                .ok_or(ContractRootViewError::Inconsistent)?,
            nodes: reached,
            selected_references: selected_references.into_iter().collect(),
            bounds,
            claims,
        }))
    }

    /// The admitted six-member wire, available for read-only inspection.
    pub fn wire(&self) -> &ContractPackageWireV1 {
        &self.wire
    }

    /// A represented local node, addressed by its admitted typed key.
    pub fn node(&self, key: &CheckedNodeId) -> Option<&CheckedSemanticNodeV2> {
        self.wire
            .semantic_graph
            .nodes
            .get(*self.node_index.get(key)?)
    }

    /// The admitted typed body of a represented local node, if it has one.
    pub fn term(&self, key: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.terms.get(*self.node_index.get(key)?)?.as_ref()
    }

    /// The retained kind of a represented local node, if present.
    pub fn node_kind(&self, key: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.node_kinds.get(*self.node_index.get(key)?).copied()
    }

    /// The retained unsupported family at a `feature_accounting` index.
    pub fn unsupported_tag_at(&self, accounting_index: usize) -> Option<CheckedNodeTag> {
        self.unsupported_tags
            .get(accounting_index)
            .copied()
            .flatten()
    }

    /// The admitted V2 kind of an unsupported request's root at this accounting index.
    /// Plain strict reads return `NotAdmitted` for unsupported records.
    pub fn unsupported_source_kind_at(
        &self,
        accounting_index: usize,
    ) -> Result<Option<CheckedNodeKind>, ContractUnsupportedSourceKindError> {
        if self.unsupported_tag_at(accounting_index).is_none() {
            return Ok(None);
        }
        self.unsupported_source_kinds
            .as_ref()
            .and_then(|kinds| kinds.get(accounting_index))
            .copied()
            .ok_or(ContractUnsupportedSourceKindError::NotAdmitted)
    }

    /// Remove one admitted source-map node to exercise defensive query errors.
    /// This is available only to the root crate's test-only fault-injection
    /// feature; production callers cannot mutate an admitted package.
    #[cfg(feature = "fault-injection")]
    pub fn fault_inject_missing_source_for_test(&mut self, key: &CheckedNodeId) -> bool {
        let before = self.wire.source_map.nodes.len();
        self.wire
            .source_map
            .nodes
            .retain(|entry| &entry.node_id != key);
        self.wire.source_map.nodes.len() != before
    }

    /// Replace one admitted node body with an invalid value to exercise
    /// defensive reached-view validation after admission.
    #[cfg(feature = "fault-injection")]
    pub fn fault_inject_invalid_body_for_test(&mut self, key: &CheckedNodeId) -> bool {
        let Some(node) = self
            .wire
            .semantic_graph
            .nodes
            .iter_mut()
            .find(|node| &node.node_id == key)
        else {
            return false;
        };
        node.body = Value::Null;
        true
    }

    /// Reads one exact requested key and, when lowered, its local reached
    /// closure. The six-member wire is never changed.
    pub fn reached_closure(
        &self,
        requested: &CheckedNodeId,
    ) -> Result<ContractRootViewV1<'_>, ContractRootViewError> {
        let mut work = GuardContextWork::unbounded_view();
        let index = ContractPackageTraversalIndex::build(self, &mut work)
            .map_err(ContractRootViewError::Incomplete)?;
        self.reached_closure_indexed(requested, &index, &mut work)
    }
}

/// One typed FR-330 read outcome.
// The required inline typed refusal makes this public result larger than its
// incomplete case; changing the variant to a box would change its API.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractPackageReadResult {
    /// Fully admitted package.
    Admitted(Box<ContractPackageV1>),
    /// Located refusal.
    Refused(CheckedPackageRefusal),
    /// A caller limit stopped admission.
    Incomplete(CheckedPackageIncomplete),
}

/// An admitted V1 package does not correspond to the supplied admitted V2 source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractSourceHandoffError {
    /// The source package identity differs from the represented provenance.
    SourceIdentity,
    /// The source lock differs from the represented provenance.
    SourceLock,
    /// A represented key has no node in the admitted source.
    MissingNode(CheckedNodeId),
    /// A represented node differs from the admitted source node.
    NodeMismatch(CheckedNodeId),
}

/// Supplied dependency packages are keyed by their selected library identity.
pub type ContractDependencies<'a> = BTreeMap<&'a str, ContractDependencyInput<'a>>;

/// An admitted dependency's selected library identity and exact package.
#[derive(Clone, Copy, Debug)]
pub struct ContractDependencyInput<'a> {
    /// The library identity the caller associates with this admitted package.
    pub identity: &'a str,
    /// The FR-322 admitted dependency package.
    pub package: &'a CheckedPackageV2,
}

impl Encode for ContractSemanticGraphV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encode_member(writer, "graph_version", &self.graph_version)?;
        elements(writer, "nodes", &self.nodes)?;
        writer.end_object()
    }
}

impl Encode for ContractNodeSourcesV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encoded(writer, "node_id", &self.node_id)?;
        elements(writer, "occurrences", &self.occurrences)?;
        writer.end_object()
    }
}

impl Encode for ContractRequestSourcesV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encoded(writer, "requested_item", &self.requested_item)?;
        elements(writer, "occurrences", &self.occurrences)?;
        writer.end_object()
    }
}

impl Encode for ContractRefusalSourcesV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encoded(writer, "requested_item", &self.requested_item)?;
        encode_member(writer, "disposition", &self.disposition)?;
        elements(writer, "occurrences", &self.occurrences)?;
        writer.end_object()
    }
}

impl Encode for ContractSourceMapV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        elements(writer, "nodes", &self.nodes)?;
        elements(writer, "requested_items", &self.requested_items)?;
        elements(writer, "refusals", &self.refusals)?;
        writer.end_object()
    }
}

impl Encode for ContractBodyRefusalV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encode_member(writer, "code", &self.code)?;
        if let Some(path) = &self.path {
            encode_member(writer, "path", path)?;
        }
        if let Some(cause) = &self.cause {
            encode_member(writer, "cause", cause)?;
        }
        if let Some(locus) = &self.locus {
            encoded(writer, "locus", locus)?;
        }
        if let Some(version) = &self.contract_version {
            encode_member(writer, "contract_version", version)?;
        }
        if let Some(pointer) = &self.document_pointer {
            encode_member(writer, "document_pointer", pointer)?;
        }
        writer.end_object()
    }
}

impl Encode for ContractFeatureRecordV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encode_member(writer, "disposition", &self.disposition())?;
        encoded(writer, "node_id", self.node_id())?;
        match self {
            Self::Lowered { .. } | Self::InvalidInput { .. } => {}
            Self::Unsupported {
                unsupported_node_id,
                node_tag,
                ..
            } => {
                encoded(writer, "unsupported_node_id", unsupported_node_id)?;
                encode_member(writer, "node_tag", node_tag)?;
            }
            Self::RequiresBound { unbounded_type, .. } => {
                encoded(writer, "unbounded_type", unbounded_type)?;
            }
            Self::Failed {
                limit_kind,
                limit,
                consumed,
                ..
            } => {
                encode_member(writer, "limit_kind", limit_kind)?;
                encode_member(writer, "limit", limit)?;
                encode_member(writer, "consumed", consumed)?;
            }
            Self::InvalidBody {
                body_node_id,
                refusal,
                ..
            } => {
                encoded(writer, "body_node_id", body_node_id)?;
                encoded(writer, "refusal", refusal)?;
            }
            Self::BodyIncomplete {
                body_node_id,
                incomplete,
                ..
            } => {
                encoded(writer, "body_node_id", body_node_id)?;
                encode_member(writer, "incomplete", incomplete)?;
            }
        }
        writer.end_object()
    }
}

impl Encode for ContractPackageWireV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        self.raw_wire_value()
            .map_err(CanonicalError::Serialize)?
            .encode_into(writer)
    }
}

fn canonical<T: Encode>(value: &T, ceiling: u64) -> Result<Vec<u8>, u64> {
    quire_canonical::to_vec(value, quire_canonical::Limits::new(ceiling)).map_err(|error| {
        match error {
            quire_canonical::Error::Limit(limit) => limit.required,
            _ => ceiling.saturating_add(1),
        }
    })
}

/// Sorts group blocks by their least represented key and preserves the
/// admitted order of members inside each block. Ungrouped nodes are one-node
/// blocks, so a graph without recursion keeps its ordinary key order.
fn canonical_node_order<'a, Body>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a super::CheckedSemanticNodeV2<Body>)>,
) -> Vec<&'a super::CheckedSemanticNodeV2<Body>> {
    let mut nodes = nodes.collect::<Vec<_>>();
    let mut least = BTreeMap::<&str, &CheckedNodeId>::new();
    for &(structural_key, node) in &nodes {
        if let Some(group) = node.recursion_group.as_deref() {
            least
                .entry(group)
                .and_modify(|least_key| {
                    if structural_key < *least_key {
                        *least_key = structural_key;
                    }
                })
                .or_insert(structural_key);
        }
    }
    nodes.sort_by_key(|(key, node)| {
        node.recursion_group
            .as_deref()
            .and_then(|group| least.get(group).copied())
            .unwrap_or(key)
    });
    nodes.into_iter().map(|(_, node)| node).collect()
}

fn remap_projected_indexes(value: &mut Value, positions: &[Option<usize>]) -> Result<bool, ()> {
    let mut changed = false;
    match value {
        // A selected dependency key owns its own graph coordinates.
        Value::Object(members)
            if members.contains_key("package")
                && members.contains_key("node")
                && !members.contains_key("path") => {}
        Value::Object(members)
            if members.len() == 3
                && members.contains_key("package")
                && members.contains_key("path")
                && members.contains_key("ordinal") =>
        {
            for segment in members["path"].as_array_mut().ok_or(())? {
                let Value::Object(index) = segment else {
                    continue;
                };
                if index.len() != 1 || !index.contains_key("node") {
                    return Err(());
                }
                let source = index["node"].as_u64().ok_or(())?;
                let source = usize::try_from(source).map_err(|_| ())?;
                let projected = positions
                    .get(source)
                    .and_then(|position| *position)
                    .ok_or(())?;
                changed |= source != projected;
                index.insert("node".to_owned(), Value::from(projected));
            }
        }
        Value::Object(members) => {
            for member in members.values_mut() {
                changed |= remap_projected_indexes(member, positions)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                changed |= remap_projected_indexes(item, positions)?;
            }
        }
        _ => {}
    }
    Ok(changed)
}

fn remap_checked_id(id: &CheckedNodeId, positions: &[Option<usize>]) -> Option<CheckedNodeId> {
    let mut value = serde_json::to_value(id).ok()?;
    remap_projected_indexes(&mut value, positions).ok()?;
    CheckedNodeId::from_value(&value)
}

fn restore_operation_views(
    source: &super::CheckedTerm,
    projected: &mut super::CheckedTerm,
    positions: &[Option<usize>],
) -> Result<(), ()> {
    use super::CheckedTerm;
    match (source, projected) {
        (
            CheckedTerm::Application {
                operation: source_operation,
                arguments: source_arguments,
                ..
            },
            CheckedTerm::Application {
                operation: projected_operation,
                arguments: projected_arguments,
                ..
            },
        ) => {
            if let Ok(source_views) = source_operation.admitted_views() {
                let mut views = source_views.clone();
                if let Some((id, _)) = &mut views.projection_field {
                    *id = remap_checked_id(id, positions).ok_or(())?;
                }
                if let Some(id) = &mut views.member_declaration {
                    *id = remap_checked_id(id, positions).ok_or(())?;
                }
                projected_operation.install_admitted_views(views);
            }
            for (source, projected) in source_arguments.iter().zip(projected_arguments) {
                restore_operation_views(source, projected, positions)?;
            }
        }
        (
            CheckedTerm::Aggregate { members: source },
            CheckedTerm::Aggregate { members: projected },
        )
        | (
            CheckedTerm::ExtensionForm {
                clauses: source, ..
            },
            CheckedTerm::ExtensionForm {
                clauses: projected, ..
            },
        ) => {
            for (source, projected) in source.iter().zip(projected) {
                restore_operation_views(source, projected, positions)?;
            }
        }
        (
            CheckedTerm::Binding { value: source, .. },
            CheckedTerm::Binding {
                value: projected, ..
            },
        ) => restore_operation_views(source, projected, positions)?,
        _ => {}
    }
    Ok(())
}

impl ContractPackageWireV1 {
    pub(super) fn from_lowering(
        source: &CheckedPackageV2,
        profile: &CompleteLoweringProfileV2,
        records: &[CompleteLoweringRecordV2],
        lowered: &[CompleteContractNodeV2],
        dependencies: &[ContractPackageDependencyV2],
        source_maps: &NodeSourceMaps<'_>,
        ceiling: u64,
    ) -> Result<(ContractPackageV1, Box<[u8]>), u64> {
        let mut nodes = BTreeSet::new();
        for represented in lowered {
            nodes.insert(&represented.node.node_id);
        }
        for represented in dependencies {
            nodes.insert(&represented.node.node_id);
        }
        // Preserve each recursion group's source member order in the emitted
        // graph. Group blocks sort by their least structural node ID; no
        // graph-order coordinate is derived for a member's node ID.
        let nodes = canonical_node_order(
            source
                .graph()
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| nodes.contains(&node.node_id))
                .map(|(position, node)| {
                    (
                        source
                            .structural_key_at(position)
                            .expect("admitted graph key at every position"),
                        node,
                    )
                }),
        );
        let source_positions = source
            .graph()
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<BTreeMap<_, _>>();
        let mut projected_positions = vec![None; source.graph().nodes.len()];
        for (position, node) in nodes.iter().enumerate() {
            projected_positions[source_positions[&node.node_id]] = Some(position);
        }
        let source_kinds = source
            .graph()
            .nodes
            .iter()
            .zip(source.node_kinds())
            .map(|(node, kind)| (&node.node_id, *kind))
            .collect::<BTreeMap<_, _>>();
        let node_kinds: Vec<CheckedNodeKind> = nodes
            .iter()
            .map(|node| source_kinds[&node.node_id])
            .collect();
        let unsupported_tags = records
            .iter()
            .map(|record| match record {
                CompleteLoweringRecordV2::Unsupported { node_tag, .. } => Some(*node_tag),
                _ => None,
            })
            .collect();
        let unsupported_source_kinds = records
            .iter()
            .map(|record| match record {
                CompleteLoweringRecordV2::Unsupported { node_id, .. } => source.node_kind(node_id),
                _ => None,
            })
            .collect();
        let node_sources = nodes
            .iter()
            .map(|node_id| ContractNodeSourcesV1 {
                node_id: node_id.node_id.clone(),
                occurrences: source_maps.of(&node_id.node_id),
            })
            .collect();
        let accounting: Vec<ContractFeatureRecordV1> = records.iter().map(Into::into).collect();
        let requested_items = accounting
            .iter()
            .map(|record| ContractRequestSourcesV1 {
                requested_item: record.node_id().clone(),
                occurrences: source_maps.of(record.node_id()),
            })
            .collect();
        let refusals = accounting
            .iter()
            .filter(|record| record.disposition() != ContractDispositionV1::Lowered)
            .map(|record| ContractRefusalSourcesV1 {
                requested_item: record.node_id().clone(),
                disposition: record.disposition(),
                occurrences: source_maps.of(record.node_id()),
            })
            .collect();
        let package = ContractProvenanceV1 {
            source: ContractSourceV1 {
                name: source.name().into(),
                lock: source.lock().clone(),
            },
            lowering_profile: profile.reference.clone(),
        };
        let semantic_graph = ContractSemanticGraphV1 {
            graph_version: source.graph().graph_version.clone(),
            nodes: nodes.into_iter().map(|node| node.to_raw()).collect(),
        };
        let terms: Vec<Option<super::CheckedTerm>> = semantic_graph
            .nodes
            .iter()
            .map(|node| source.term(&node.node_id).cloned())
            .collect();
        let source_map = ContractSourceMapV1 {
            nodes: node_sources,
            requested_items,
            refusals,
        };
        let wire = Self {
            contract_version: CONTRACT_PACKAGE_VERSION.into(),
            package,
            semantic_graph,
            source_keys: Vec::new(),
            source_map,
            feature_accounting: accounting,
        };
        // FR-322 indexes name positions in this projected document, not in
        // the originating checked package. Preserve provenance untouched.
        let mut projected = expanded_wire_value(&wire).map_err(|_| ceiling.saturating_add(1))?;
        remap_projected_indexes(&mut projected["semantic_graph"], &projected_positions)
            .map_err(|()| ceiling.saturating_add(1))?;
        for mapped in projected["source_map"]["nodes"]
            .as_array_mut()
            .expect("constructed source-map nodes")
        {
            remap_projected_indexes(&mut mapped["node_id"], &projected_positions)
                .map_err(|()| ceiling.saturating_add(1))?;
        }
        let raw =
            source_frame_wire(&mut projected, source).map_err(|()| ceiling.saturating_add(1))?;
        let (_, frames) = resolve_source_frames(&raw).map_err(|_| ceiling.saturating_add(1))?;
        let wire_bytes = serde_json::to_vec(&projected).map_err(|_| ceiling.saturating_add(1))?;
        let wire: Self =
            serde_json::from_slice(&wire_bytes).map_err(|_| ceiling.saturating_add(1))?;
        let terms = terms
            .into_iter()
            .enumerate()
            .map(|(position, term)| {
                let Some(source_term) = term else {
                    return Ok(None);
                };
                let mut value =
                    serde_json::to_value(&source_term).map_err(|_| ceiling.saturating_add(1))?;
                if !remap_projected_indexes(&mut value, &projected_positions)
                    .map_err(|()| ceiling.saturating_add(1))?
                {
                    return Ok(Some(source_term));
                }
                let mut term = super::decode_admitted_body(
                    node_kinds[position],
                    &wire.semantic_graph.nodes[position].body,
                )
                .map_err(|_| ceiling.saturating_add(1))?;
                restore_operation_views(&source_term, &mut term, &projected_positions)
                    .map_err(|()| ceiling.saturating_add(1))?;
                Ok(Some(term))
            })
            .collect::<Result<Vec<_>, u64>>()?;
        let node_index = wire
            .semantic_graph
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (node.node_id.clone(), position))
            .collect();
        let bytes = canonical(&raw, ceiling)?.into_boxed_slice();
        Ok((
            ContractPackageV1 {
                source_index: result_context::source_index(&wire.source_map.nodes),
                wire,
                frames,
                node_kinds,
                terms,
                node_index,
                unsupported_tags,
                unsupported_source_kinds: Some(unsupported_source_kinds),
            },
            bytes,
        ))
    }
}

/// Intern source-frame IDs and their operand closure without consulting output
/// graph positions. The returned JSON has only source-key references in every
/// source-frame position; `expanded` retains resolved IDs for in-memory views.
fn source_frame_wire(expanded: &mut Value, source: &CheckedPackageV2) -> Result<Value, ()> {
    let paths = source_ref_paths(expanded);
    let direct = paths
        .iter()
        .map(|at| CheckedNodeId::from_value(expanded.pointer(at.as_str()).ok_or(())?).ok_or(()))
        .collect::<Result<Vec<_>, _>>()?;
    let source_positions = source
        .graph()
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (&node.node_id, position))
        .collect::<BTreeMap<_, _>>();
    let mut included = vec![false; source.graph().nodes.len()];
    let mut pending = VecDeque::new();
    for id in &direct {
        include_source_operands(id, &source_positions, &mut included, &mut pending)?;
    }
    while let Some(position) = pending.pop_front() {
        include_source_operands(
            &source.graph().nodes[position].node_id,
            &source_positions,
            &mut included,
            &mut pending,
        )?;
    }
    let mut table_positions = vec![None; included.len()];
    let mut keys = Vec::new();
    for (position, included) in included.into_iter().enumerate() {
        if included {
            table_positions[position] = Some(keys.len());
            keys.push(source.graph().nodes[position].node_id.clone());
        }
    }
    let mut key_positions = keys
        .iter()
        .enumerate()
        .map(|(position, id)| (id.clone(), position))
        .collect::<BTreeMap<_, _>>();
    for id in &direct {
        if !key_positions.contains_key(id) {
            key_positions.insert(id.clone(), keys.len());
            keys.push(id.clone());
        }
    }
    let keys = keys
        .iter()
        .map(|id| remap_checked_id(id, &table_positions).ok_or(()))
        .collect::<Result<Vec<_>, _>>()?;
    expanded["source_keys"] = serde_json::to_value(&keys).map_err(|_| ())?;
    let mut raw = expanded.clone();
    for (at, id) in paths.into_iter().zip(&direct) {
        let position = *key_positions.get(id).ok_or(())?;
        *expanded.pointer_mut(at.as_str()).ok_or(())? =
            serde_json::to_value(&keys[position]).map_err(|_| ())?;
        *raw.pointer_mut(at.as_str()).ok_or(())? = serde_json::json!({ "source_key": position });
    }
    raw["source_keys"] = expanded["source_keys"].clone();
    Ok(raw)
}

fn include_source_operands(
    id: &CheckedNodeId,
    source_positions: &BTreeMap<&CheckedNodeId, usize>,
    included: &mut [bool],
    pending: &mut VecDeque<usize>,
) -> Result<(), ()> {
    if let Some(&position) = source_positions.get(id) {
        if !included[position] {
            included[position] = true;
            pending.push_back(position);
        }
    }
    for segment in &id.path {
        if let CheckedNodePathSegment::Index(index) = segment {
            let position = usize::try_from(*index).map_err(|_| ())?;
            let selected = included.get_mut(position).ok_or(())?;
            if !*selected {
                *selected = true;
                pending.push_back(position);
            }
        }
    }
    Ok(())
}

impl ContractPackageWireV1 {
    /// Reads only a complete canonical FR-330 value; dependencies are admitted inputs.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        dependencies: &ContractDependencies<'_>,
    ) -> ContractPackageReadResult {
        match Self::admit(bytes, limits, dependencies) {
            Ok(value) => ContractPackageReadResult::Admitted(Box::new(value)),
            Err(ValidationFailure::Refused(refusal)) => {
                ContractPackageReadResult::Refused(*refusal)
            }
            Err(ValidationFailure::Incomplete(incomplete)) => {
                ContractPackageReadResult::Incomplete(incomplete)
            }
        }
    }

    /// Reads canonical V1 bytes, then authenticates represented terms against an
    /// already reader-admitted V2 source. Plain reader refusals and limits take
    /// precedence over source correspondence errors.
    pub fn read_with_source(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        dependencies: &ContractDependencies<'_>,
        source: &CheckedPackageV2,
    ) -> Result<ContractPackageReadResult, ContractSourceHandoffError> {
        let mut package = match Self::read(bytes, limits, dependencies) {
            ContractPackageReadResult::Admitted(package) => package,
            other => return Ok(other),
        };
        if package.wire.package.source.name.as_ref() != source.name() {
            return Err(ContractSourceHandoffError::SourceIdentity);
        }
        if package.wire.package.source.lock != *source.lock() {
            return Err(ContractSourceHandoffError::SourceLock);
        }
        let source_nodes = source
            .graph()
            .nodes
            .iter()
            .enumerate()
            .map(|(position, _)| {
                (
                    source
                        .structural_key_at(position)
                        .expect("admitted source graph key"),
                    position,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let projected_keys =
            super::node_index::rebuild_graph_keys(&package.wire.semantic_graph.nodes, |_| {
                unreachable!("admitted V1 graph was already index-checked")
            })
            .expect("admitted V1 graph keys");
        let mut projected_positions = vec![None; source.graph().nodes.len()];
        let mut source_positions = Vec::with_capacity(package.wire.semantic_graph.nodes.len());
        for (position, represented) in package.wire.semantic_graph.nodes.iter().enumerate() {
            let key = &represented.node_id;
            let &source_position = source_nodes
                .get(projected_keys[position].as_ref())
                .ok_or_else(|| ContractSourceHandoffError::MissingNode(key.clone()))?;
            source_positions.push(Some(source_position));
            projected_positions[source_position] = Some(position);
        }
        for (position, represented) in package.wire.semantic_graph.nodes.iter().enumerate() {
            let key = &represented.node_id;
            let source_position = source_positions[position].expect("matched source position");
            let node = &source.graph().nodes[source_position];
            let mut source_value = serde_json::to_value(node.to_raw())
                .map_err(|_| ContractSourceHandoffError::NodeMismatch(key.clone()))?;
            remap_projected_indexes(&mut source_value, &projected_positions)
                .map_err(|()| ContractSourceHandoffError::NodeMismatch(key.clone()))?;
            if source_value
                != serde_json::to_value(represented)
                    .map_err(|_| ContractSourceHandoffError::NodeMismatch(key.clone()))?
            {
                return Err(ContractSourceHandoffError::NodeMismatch(key.clone()));
            }
            let source_term = source
                .term(&node.node_id)
                .ok_or_else(|| ContractSourceHandoffError::MissingNode(key.clone()))?;
            if let Some(Some(projected_term)) = package.terms.get_mut(position) {
                restore_operation_views(source_term, projected_term, &projected_positions)
                    .map_err(|()| ContractSourceHandoffError::NodeMismatch(key.clone()))?;
            }
        }
        let unsupported_source_kinds = package
            .wire
            .feature_accounting
            .iter()
            .map(|record| match record {
                ContractFeatureRecordV1::Unsupported { node_id, .. } => package
                    .frames
                    .source(node_id)
                    .and_then(|key| source_nodes.get(key).copied())
                    .and_then(|position| source.graph().nodes.get(position))
                    .and_then(|node| source.node_kind(&node.node_id))
                    .map(Some)
                    .ok_or_else(|| ContractSourceHandoffError::MissingNode(node_id.clone())),
                _ => Ok(None),
            })
            .collect::<Result<Vec<_>, _>>()?;
        package.unsupported_source_kinds = Some(unsupported_source_kinds);
        Ok(ContractPackageReadResult::Admitted(package))
    }

    fn admit(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        dependencies: &ContractDependencies<'_>,
    ) -> Result<ContractPackageV1, ValidationFailure> {
        if let Err(failure) = read_typed_prefix(bytes, limits) {
            return Err(match failure {
                ValidationFailure::Refused(refusal)
                    if refusal.code == CheckedPackageRefusalCode::DuplicateMember =>
                {
                    malformed(
                        refusal
                            .primary_path()
                            .cloned()
                            .unwrap_or_else(JsonPointer::root),
                        CheckedPackageRefusalCause::DuplicateMember,
                    )
                }
                other => other,
            });
        }
        let value = strict_parse(bytes)?;
        let root = JsonPointer::root();
        let top = object(&value, root.clone())?;
        match top.get("contract_version") {
            Some(Value::String(version))
                if ContractWireTag::from_wire(version) == Some(ContractWireTag::Version) => {}
            Some(Value::String(version)) => {
                return Err(ValidationFailure::unknown_contract_version(version))
            }
            Some(_) => {
                return Err(malformed(
                    root.key("contract_version"),
                    CheckedPackageRefusalCause::WrongValueKind,
                ))
            }
            None => return Err(malformed(root, CheckedPackageRefusalCause::MissingMember)),
        }
        validate_shapes(&value)?;
        let (expanded, frames) = resolve_source_frames(&value)?;
        // CheckedNodeId uses RawValue for iterative nested-path decoding.
        // Deserializing it from an in-memory Value cannot provide that raw
        // token stream, so decode the already shape-checked input bytes.
        // The strict parse above has already enforced Serde's depth limit.
        let expanded_bytes = serde_json::to_vec(&expanded).map_err(|_| {
            ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire)
        })?;
        let mut decoder = serde_json::Deserializer::from_slice(&expanded_bytes);
        let wire: Self = serde_path_to_error::deserialize(&mut decoder).map_err(|error| {
            malformed(
                crate::checked_package::common::decoder_pointer(JsonPointer::root(), error.path()),
                CheckedPackageRefusalCause::WrongValueKind,
            )
        })?;
        let mut work_meter = super::WorkMeter::new(limits.work);
        let (node_kinds, unsupported_tags) = wire.validate_values(limits, &mut work_meter)?;
        wire.validate_correspondence(limits, &frames)?;
        wire.validate_accounting(&frames)?;
        wire.validate_dependencies(dependencies, &mut work_meter)?;
        let terms = wire.validate_references(dependencies, limits, &mut work_meter, &frames)?;
        let node_index = wire
            .semantic_graph
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (node.node_id.clone(), position))
            .collect();
        Ok(ContractPackageV1 {
            source_index: result_context::source_index(&wire.source_map.nodes),
            wire,
            frames,
            node_kinds,
            terms,
            node_index,
            unsupported_tags,
            unsupported_source_kinds: None,
        })
    }
}

/// Validate both document-local index tables and every source reference before
/// expanding the source-frame references for the admitted in-memory views.
fn resolve_source_frames(value: &Value) -> Result<(Value, ContractFrameKeys), ValidationFailure> {
    let graph_at = JsonPointer::root().key("semantic_graph").key("nodes");
    let graph = value["semantic_graph"]["nodes"]
        .as_array()
        .ok_or_else(|| malformed(graph_at.clone(), CheckedPackageRefusalCause::WrongValueKind))?;
    check_output_index_sites(
        &value["semantic_graph"],
        JsonPointer::root().key("semantic_graph"),
        graph.len(),
    )?;
    if let Some(mapped) = value["source_map"]["nodes"].as_array() {
        for (index, entry) in mapped.iter().enumerate() {
            check_output_index_sites(
                &entry["node_id"],
                JsonPointer::root()
                    .key("source_map")
                    .key("nodes")
                    .index(index)
                    .key("node_id"),
                graph.len(),
            )?;
        }
    }
    let output_ids = graph
        .iter()
        .enumerate()
        .map(|(index, node)| {
            CheckedNodeId::from_value(&node["node_id"]).ok_or_else(|| {
                malformed(
                    graph_at.clone().index(index).key("node_id"),
                    CheckedPackageRefusalCause::WrongValueKind,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let output_keys = super::node_index::rebuild_keys(
        &output_ids.iter().collect::<Vec<_>>(),
        |position, slot| {
            graph_at
                .clone()
                .index(position)
                .key("node_id")
                .key("path")
                .index(slot)
                .key("node")
        },
        |at| ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at),
    )?;
    let table_at = JsonPointer::root().key("source_keys");
    let table = value["source_keys"]
        .as_array()
        .ok_or_else(|| malformed(table_at.clone(), CheckedPackageRefusalCause::WrongValueKind))?;
    let source_ids = table
        .iter()
        .enumerate()
        .map(|(index, id)| {
            CheckedNodeId::from_value(id).ok_or_else(|| {
                malformed(
                    table_at.clone().index(index),
                    CheckedPackageRefusalCause::WrongValueKind,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let source_keys = super::node_index::rebuild_keys(
        &source_ids.iter().collect::<Vec<_>>(),
        |position, slot| {
            table_at
                .clone()
                .index(position)
                .key("path")
                .index(slot)
                .key("node")
        },
        |at| ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at),
    )?;
    let paths = source_ref_paths(value);
    for at in &paths {
        let index = value
            .pointer(at.as_str())
            .and_then(|reference| reference["source_key"].as_u64())
            .and_then(|index| usize::try_from(index).ok());
        if index.is_none_or(|index| index >= source_ids.len()) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                at.clone().key("source_key"),
            ));
        }
    }
    // Compare shallow interned descriptors: a long chain of source keys must
    // not compare every deep prefix again during duplicate detection.
    let mut interner = super::node_index::StructuralInterner::new();
    let tokens = interner
        .tokens(&source_keys)
        .ok_or_else(|| malformed(table_at.clone(), CheckedPackageRefusalCause::InvalidValue))?;
    let mut seen = BTreeSet::new();
    for (position, token) in tokens.into_iter().enumerate() {
        if !seen.insert(token) {
            return Err(malformed(
                table_at.clone().index(position),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
    }
    let mut expanded = value.clone();
    for at in paths {
        let index = expanded
            .pointer(at.as_str())
            .and_then(|reference| reference["source_key"].as_u64())
            .and_then(|index| usize::try_from(index).ok())
            .expect("source-key reference checked in bounds");
        if let Some(reference) = expanded.pointer_mut(at.as_str()) {
            *reference = table[index].clone();
        }
    }
    Ok((
        expanded,
        ContractFrameKeys {
            output: output_ids.into_iter().zip(output_keys).collect(),
            source: source_ids.into_iter().zip(source_keys).collect(),
        },
    ))
}

/// Inspect all output-frame ID positions before checking own-path cycles.
/// External `{package,node}` keys are scoped to their selected dependency,
/// never to this ContractPackage's output graph.
fn check_output_index_sites(
    value: &Value,
    at: JsonPointer,
    graph_len: usize,
) -> Result<(), ValidationFailure> {
    let mut pending = vec![(value, at)];
    while let Some((value, at)) = pending.pop() {
        match value {
            Value::Object(members)
                if members.len() == 2
                    && members.contains_key("package")
                    && members.contains_key("node") => {}
            Value::Object(members)
                if members.len() == 3
                    && members.contains_key("package")
                    && members.contains_key("path")
                    && members.contains_key("ordinal") =>
            {
                if let Some(path) = members["path"].as_array() {
                    for (slot, segment) in path.iter().enumerate() {
                        let Some(index) = segment
                            .as_object()
                            .filter(|object| object.len() == 1)
                            .and_then(|object| object.get("node"))
                            .and_then(Value::as_u64)
                        else {
                            continue;
                        };
                        if usize::try_from(index)
                            .ok()
                            .is_none_or(|index| index >= graph_len)
                        {
                            return Err(ValidationFailure::refused(
                                CheckedPackageRefusalCode::MalformedWire,
                                at.clone().key("path").index(slot).key("node"),
                            ));
                        }
                    }
                }
            }
            Value::Object(members) => {
                for (name, child) in members.iter().rev() {
                    pending.push((child, at.clone().key(name)));
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate().rev() {
                    pending.push((child, at.clone().index(index)));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn malformed(path: JsonPointer, cause: CheckedPackageRefusalCause) -> ValidationFailure {
    ValidationFailure::refused_because(CheckedPackageRefusalCode::InvalidPackage, path, cause)
}

fn object(value: &Value, path: JsonPointer) -> Result<&Map<String, Value>, ValidationFailure> {
    value
        .as_object()
        .ok_or_else(|| malformed(path, CheckedPackageRefusalCause::WrongValueKind))
}

fn array(value: &Value, path: JsonPointer) -> Result<&[Value], ValidationFailure> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| malformed(path, CheckedPackageRefusalCause::WrongValueKind))
}

fn string(value: &Value, path: JsonPointer) -> Result<&str, ValidationFailure> {
    value
        .as_str()
        .ok_or_else(|| malformed(path, CheckedPackageRefusalCause::WrongValueKind))
}

fn enum_string<T: DeserializeOwned>(
    value: &Value,
    path: JsonPointer,
) -> Result<(), ValidationFailure> {
    string(value, path.clone())?;
    T::deserialize(value)
        .map(|_| ())
        .map_err(|_| malformed(path, CheckedPackageRefusalCause::InvalidValue))
}

fn occurrence_roles(values: &[Value], at: JsonPointer) -> Result<(), ValidationFailure> {
    for (index, value) in values.iter().enumerate() {
        let entry_at = at.clone().index(index);
        let entry = object(value, entry_at.clone())?;
        enum_string::<CheckedOccurrenceRole>(
            member(entry, "role", &entry_at)?,
            entry_at.key("role"),
        )?;
    }
    Ok(())
}

fn member<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    path: &JsonPointer,
) -> Result<&'a Value, ValidationFailure> {
    object
        .get(name)
        .ok_or_else(|| malformed(path.clone(), CheckedPackageRefusalCause::MissingMember))
}

fn shape<'a>(
    value: &'a Value,
    path: JsonPointer,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>, ValidationFailure> {
    let object = object(value, path.clone())?;
    if required.iter().any(|name| !object.contains_key(*name)) {
        return Err(malformed(path, CheckedPackageRefusalCause::MissingMember));
    }
    if object
        .keys()
        .any(|name| !required.contains(&name.as_str()) && !optional.contains(&name.as_str()))
    {
        return Err(malformed(path, CheckedPackageRefusalCause::UnknownMember));
    }
    Ok(object)
}

#[ix_trace_rs::string_edge]
fn decimal(value: &Value, path: JsonPointer) -> Result<u64, ValidationFailure> {
    let text = string(value, path.clone())?;
    if text != "0"
        && (text.starts_with('0') || text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(malformed(path, CheckedPackageRefusalCause::InvalidValue));
    }
    text.parse::<u64>()
        .map_err(|_| malformed(path, CheckedPackageRefusalCause::InvalidValue))
}

fn id_shape(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = shape(value, at.clone(), &["package", "path", "ordinal"], &[])?;
    if string(member(object, "package", &at)?, at.clone().key("package"))?.is_empty() {
        return Err(malformed(
            at.key("package"),
            CheckedPackageRefusalCause::InvalidValue,
        ));
    }
    let path_at = at.clone().key("path");
    let path = array(member(object, "path", &at)?, path_at.clone())?;
    if path.is_empty() {
        return Err(malformed(path_at, CheckedPackageRefusalCause::InvalidValue));
    }
    let ordinal_at = at.clone().key("ordinal");
    if !object
        .get("ordinal")
        .and_then(Value::as_u64)
        .is_some_and(|ordinal| ordinal <= 9_007_199_254_740_991)
    {
        return Err(malformed(
            ordinal_at,
            CheckedPackageRefusalCause::InvalidValue,
        ));
    }
    for (index, segment) in path.iter().enumerate() {
        let segment_at = path_at.clone().index(index);
        match segment {
            Value::String(_) => {}
            Value::Object(object) if object.len() == 1 && object.contains_key("node") => {
                if !object["node"]
                    .as_u64()
                    .is_some_and(|index| index <= 9_007_199_254_740_991)
                {
                    return Err(malformed(
                        segment_at.key("node"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
            _ => {
                return Err(malformed(
                    segment_at,
                    CheckedPackageRefusalCause::InvalidValue,
                ))
            }
        }
    }
    Ok(())
}

fn source_ref_shape(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = shape(value, at.clone(), &["source_key"], &[])?;
    let key_at = at.clone().key("source_key");
    let key = member(object, "source_key", &at)?;
    match key {
        Value::Number(number) if number.as_u64().is_some_and(|n| n <= 9_007_199_254_740_991) => {
            Ok(())
        }
        Value::Number(number) if number.as_u64().is_some() || number.as_i64().is_some() => {
            Err(malformed(key_at, CheckedPackageRefusalCause::InvalidValue))
        }
        Value::Number(_) => Err(malformed(
            key_at,
            CheckedPackageRefusalCause::WrongValueKind,
        )),
        _ => Err(malformed(
            key_at,
            CheckedPackageRefusalCause::WrongValueKind,
        )),
    }
}

/// The exhaustive FR-330 source-frame positions. A caller runs closed-shape
/// validation before using these pointers, so no unknown object is traversed.
fn source_ref_paths(value: &Value) -> Vec<JsonPointer> {
    let root = JsonPointer::root();
    let mut paths = Vec::new();
    // Traverse these fixed positions in canonical object-member and array
    // order. This also chooses the first out-of-range reference deterministically.
    if let Some(records) = value["feature_accounting"].as_array() {
        for (index, record) in records.iter().enumerate() {
            let at = root.clone().key("feature_accounting").index(index);
            if record.get("body_node_id").is_some() {
                paths.push(at.clone().key("body_node_id"));
            }
            paths.push(at.clone().key("node_id"));
            if record["refusal"].get("locus").is_some() {
                paths.push(at.clone().key("refusal").key("locus"));
            }
            for member in ["unbounded_type", "unsupported_node_id"] {
                if record.get(member).is_some() {
                    paths.push(at.clone().key(member));
                }
            }
        }
    }
    let map = &value["source_map"];
    for collection in ["nodes", "refusals", "requested_items"] {
        if let Some(entries) = map[collection].as_array() {
            for (index, entry) in entries.iter().enumerate() {
                let at = root.clone().key("source_map").key(collection).index(index);
                if let Some(occurrences) = entry["occurrences"].as_array() {
                    for ordinal in 0..occurrences.len() {
                        paths.push(at.clone().key("occurrences").index(ordinal).key("node_id"));
                    }
                }
                if collection != "nodes" {
                    paths.push(at.key("requested_item"));
                }
            }
        }
    }
    paths
}

#[ix_trace_rs::string_edge]
fn optional_typed(
    object: &Map<String, Value>,
    name: &str,
    path: &JsonPointer,
) -> Result<(), ValidationFailure> {
    if let Some(value) = object.get(name) {
        let at = path.clone().key(name);
        let text = string(value, at.clone())?;
        if name != "contract_version" && JsonPointer::parse(text).is_none() {
            return Err(malformed(at, CheckedPackageRefusalCause::InvalidValue));
        }
    }
    Ok(())
}

fn validate_shapes(value: &Value) -> Result<(), ValidationFailure> {
    let root = JsonPointer::root();
    let top = shape(
        value,
        root.clone(),
        &[
            "contract_version",
            "package",
            "semantic_graph",
            "source_keys",
            "source_map",
            "feature_accounting",
        ],
        &[],
    )?;
    let package_at = root.clone().key("package");
    let package = shape(
        member(top, "package", &root)?,
        package_at.clone(),
        &["source", "lowering_profile"],
        &[],
    )?;
    let source_at = package_at.clone().key("source");
    let source = shape(
        member(package, "source", &package_at)?,
        source_at.clone(),
        &["name", "lock"],
        &[],
    )?;
    string(
        member(source, "name", &source_at)?,
        source_at.clone().key("name"),
    )?;
    let lock_at = source_at.clone().key("lock");
    let lock = shape(
        member(source, "lock", &source_at)?,
        lock_at.clone(),
        &[
            "sources",
            "edition",
            "profile_selections",
            "definition_selections",
            "required_features",
            "dependency_selections",
        ],
        &[],
    )?;
    for name in [
        "sources",
        "profile_selections",
        "definition_selections",
        "required_features",
        "dependency_selections",
    ] {
        array(member(lock, name, &lock_at)?, lock_at.clone().key(name))?;
    }
    let edition_at = lock_at.clone().key("edition");
    let edition = object(member(lock, "edition", &lock_at)?, edition_at.clone())?;
    enum_string::<CheckedSelectionRole>(
        member(edition, "role", &edition_at)?,
        edition_at.key("role"),
    )?;
    let selections_at = lock_at.clone().key("profile_selections");
    for (index, value) in array(
        member(lock, "profile_selections", &lock_at)?,
        selections_at.clone(),
    )?
    .iter()
    .enumerate()
    {
        let entry_at = selections_at.clone().index(index);
        let entry = object(value, entry_at.clone())?;
        enum_string::<CheckedSelectionRole>(
            member(entry, "role", &entry_at)?,
            entry_at.key("role"),
        )?;
    }
    let profile_at = package_at.clone().key("lowering_profile");
    let profile = shape(
        member(package, "lowering_profile", &package_at)?,
        profile_at.clone(),
        &["authority", "identity"],
        &[],
    )?;
    for name in ["authority", "identity"] {
        string(
            member(profile, name, &profile_at)?,
            profile_at.clone().key(name),
        )?;
    }
    let graph_at = root.clone().key("semantic_graph");
    let graph = shape(
        member(top, "semantic_graph", &root)?,
        graph_at.clone(),
        &["graph_version", "nodes"],
        &[],
    )?;
    string(
        member(graph, "graph_version", &graph_at)?,
        graph_at.clone().key("graph_version"),
    )?;
    let nodes_at = graph_at.clone().key("nodes");
    for (index, node) in array(member(graph, "nodes", &graph_at)?, nodes_at.clone())?
        .iter()
        .enumerate()
    {
        let at = nodes_at.clone().index(index);
        let node = shape(
            node,
            at.clone(),
            &[
                "node_id",
                "schema_version",
                "node_tag",
                "semantic_form",
                "semantic_type",
                "dependencies",
                "occurrences",
                "body",
            ],
            &["recursion_group", "declaration", "owner"],
        )?;
        id_shape(member(node, "node_id", &at)?, at.clone().key("node_id"))?;
        id_shape(
            member(node, "semantic_type", &at)?,
            at.clone().key("semantic_type"),
        )?;
        for name in ["schema_version", "node_tag", "semantic_form"] {
            string(member(node, name, &at)?, at.clone().key(name))?;
        }
        for name in ["dependencies", "occurrences"] {
            array(member(node, name, &at)?, at.clone().key(name))?;
        }
        let occurrences_at = at.clone().key("occurrences");
        occurrence_roles(
            array(member(node, "occurrences", &at)?, occurrences_at.clone())?,
            occurrences_at,
        )?;
    }
    let source_keys_at = root.clone().key("source_keys");
    for (index, key) in array(member(top, "source_keys", &root)?, source_keys_at.clone())?
        .iter()
        .enumerate()
    {
        id_shape(key, source_keys_at.clone().index(index))?;
    }
    let map_at = root.clone().key("source_map");
    let map = shape(
        member(top, "source_map", &root)?,
        map_at.clone(),
        &["nodes", "requested_items", "refusals"],
        &[],
    )?;
    for (collection, subject, extra) in [
        ("nodes", "node_id", None),
        ("requested_items", "requested_item", None),
        ("refusals", "requested_item", Some("disposition")),
    ] {
        let at = map_at.clone().key(collection);
        for (index, entry) in array(member(map, collection, &map_at)?, at.clone())?
            .iter()
            .enumerate()
        {
            let entry_at = at.clone().index(index);
            let required: &[&str] = if extra.is_some() {
                &[subject, "disposition", "occurrences"]
            } else {
                &[subject, "occurrences"]
            };
            let entry = shape(entry, entry_at.clone(), required, &[])?;
            if collection == "nodes" {
                id_shape(
                    member(entry, "node_id", &entry_at)?,
                    entry_at.clone().key("node_id"),
                )?;
            }
            if extra.is_some() {
                enum_string::<ContractDispositionV1>(
                    member(entry, "disposition", &entry_at)?,
                    entry_at.clone().key("disposition"),
                )?;
            }
            let occurrences_at = entry_at.clone().key("occurrences");
            occurrence_roles(
                array(
                    member(entry, "occurrences", &entry_at)?,
                    occurrences_at.clone(),
                )?,
                occurrences_at,
            )?;
        }
    }
    let accounting_at = root.clone().key("feature_accounting");
    for (index, record) in array(
        member(top, "feature_accounting", &root)?,
        accounting_at.clone(),
    )?
    .iter()
    .enumerate()
    {
        validate_record_shape(record, accounting_at.clone().index(index))?;
    }
    for at in source_ref_paths(value) {
        let Some(reference) = value.pointer(at.as_str()) else {
            return Err(malformed(at, CheckedPackageRefusalCause::MissingMember));
        };
        source_ref_shape(reference, at)?;
    }
    Ok(())
}

fn validate_record_shape(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = object(value, at.clone())?;
    let disposition = string(
        member(object, "disposition", &at)?,
        at.clone().key("disposition"),
    )?;
    let disposition = ContractDispositionV1::from_wire(disposition).ok_or_else(|| {
        malformed(
            at.clone().key("disposition"),
            CheckedPackageRefusalCause::InvalidValue,
        )
    })?;
    let payload: &[&str] = match disposition {
        ContractDispositionV1::Lowered | ContractDispositionV1::InvalidInput => &[],
        ContractDispositionV1::Unsupported => &["unsupported_node_id", "node_tag"],
        ContractDispositionV1::RequiresBound => &["unbounded_type"],
        ContractDispositionV1::Failed => &["limit_kind", "limit", "consumed"],
        ContractDispositionV1::InvalidBody => &["body_node_id", "refusal"],
        ContractDispositionV1::BodyIncomplete => &["body_node_id", "incomplete"],
    };
    let required: Vec<&str> = ["node_id", "disposition"]
        .into_iter()
        .chain(payload.iter().copied())
        .collect();
    let record = shape(value, at.clone(), &required, &[])?;
    if disposition == ContractDispositionV1::Failed {
        for counter in ["limit", "consumed"] {
            decimal(member(record, counter, &at)?, at.clone().key(counter))?;
        }
    }
    if disposition == ContractDispositionV1::InvalidBody {
        validate_body_refusal(member(record, "refusal", &at)?, at.clone().key("refusal"))?;
    }
    if disposition == ContractDispositionV1::BodyIncomplete {
        validate_body_incomplete(member(record, "incomplete", &at)?, at.key("incomplete"))?;
    }
    Ok(())
}

fn validate_body_refusal(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = object(value, at.clone())?;
    let code_at = at.clone().key("code");
    let code_value = member(object, "code", &at)?;
    string(code_value, code_at.clone())?;
    let code: CheckedPackageRefusalCode = serde_json::from_value(code_value.clone())
        .map_err(|_| malformed(code_at, CheckedPackageRefusalCause::InvalidValue))?;
    let required = if code == CheckedPackageRefusalCode::UnknownContractVersion {
        &["code", "contract_version"][..]
    } else {
        &["code"][..]
    };
    let object = shape(
        value,
        at.clone(),
        required,
        &["path", "cause", "locus", "document_pointer"],
    )?;
    for name in ["path", "document_pointer", "contract_version"] {
        optional_typed(object, name, &at)?;
    }
    if let Some(locus) = object.get("locus") {
        let locus_at = at.clone().key("locus");
        source_ref_shape(locus, locus_at)?;
    }
    if let Some(cause_value) = object.get("cause") {
        let cause_at = at.clone().key("cause");
        string(cause_value, cause_at.clone())?;
        let cause: CheckedPackageRefusalCause = serde_json::from_value(cause_value.clone())
            .map_err(|_| malformed(cause_at.clone(), CheckedPackageRefusalCause::InvalidValue))?;
        if !cause_admitted(code, cause) {
            return Err(malformed(
                cause_at,
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
    }
    Ok(())
}

fn cause_admitted(code: CheckedPackageRefusalCode, cause: CheckedPackageRefusalCause) -> bool {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    match cause {
        Cause::MissingTimedModel
        | Cause::TimedNodeReference
        | Cause::DuplicateClockPlace
        | Cause::MalformedClockPlace
        | Cause::ClockWithoutDenseTime
        | Cause::MalformedJson
        | Cause::MissingMember
        | Cause::DuplicateMember
        | Cause::UnknownMember
        | Cause::WrongValueKind
        | Cause::InvalidValue
        | Cause::DefinitionCycle
        | Cause::ConflictingDefinition
        | Cause::IncompatibleDefinition
        | Cause::FeatureSetMismatch
        | Cause::UnknownOperation
        | Cause::OperationClassMismatch
        | Cause::OperationLawMissing
        | Cause::OperationLawMismatch
        | Cause::OperationLawUnselected
        | Cause::OperationModeMismatch
        | Cause::OperationMemberMismatch
        | Cause::OperationModeTypeMismatch
        | Cause::InvalidDerivedShape
        | Cause::DeclarationNominalMismatch => code == Code::InvalidPackage,
        Cause::AmbiguousName => code == Code::AmbiguousDeclaration,
        Cause::InsufficientNextCharge => false,
        Cause::OperatorIneligible
        | Cause::TypeMismatch
        | Cause::NonBooleanRoot
        | Cause::InvalidTimeQuantity => code == Code::IllTyped,
        Cause::MissingName => code == Code::MissingDeclaration,
        Cause::WrongObservation => code == Code::WrongSnapshot,
        Cause::MalformedDeclaration | Cause::WrongExport => code == Code::InvalidModelBinding,
        Cause::DeclarationForm => code == Code::UnsupportedConstruct,
        Cause::MissingSelection => matches!(code, Code::MissingImport | Code::MissingDeclaration),
        Cause::UnsupportedSelection | Cause::WrongSelectionRole => code == Code::UnknownProfile,
        Cause::ExpressionForm
        | Cause::WrongModelSelection
        | Cause::ConflictingBinding
        | Cause::UnpreservedModelMeaning
        | Cause::InexactInteger
        | Cause::InexactNumber => false,
    }
}

#[cfg(test)]
mod refusal_cause_tests {
    use super::{cause_admitted, CheckedPackageRefusalCause, CheckedPackageRefusalCode};

    #[test]
    fn wrong_observation_requires_wrong_snapshot() {
        assert!(cause_admitted(
            CheckedPackageRefusalCode::WrongSnapshot,
            CheckedPackageRefusalCause::WrongObservation,
        ));
        assert!(!cause_admitted(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::WrongObservation,
        ));
    }
}

fn validate_body_incomplete(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = object(value, at.clone())?;
    let kind_at = at.clone().key("limit_kind");
    let kind_value = member(object, "limit_kind", &at)?;
    string(kind_value, kind_at.clone())?;
    let kind: CheckedPackageLimit = serde_json::from_value(kind_value.clone())
        .map_err(|_| malformed(kind_at, CheckedPackageRefusalCause::InvalidValue))?;
    let required = if kind == CheckedPackageLimit::Bytes {
        &["limit_kind", "limit", "consumed"][..]
    } else {
        &["limit_kind", "limit", "consumed", "path"][..]
    };
    let object = shape(value, at.clone(), required, &[])?;
    decimal(member(object, "limit", &at)?, at.clone().key("limit"))?;
    decimal(member(object, "consumed", &at)?, at.clone().key("consumed"))?;
    optional_typed(object, "path", &at)?;
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ContractWireTag {
    Version,
    GraphVersion,
}

impl ContractWireTag {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            CONTRACT_PACKAGE_VERSION => Some(Self::Version),
            "quire.checked-semantic-graph/v2" => Some(Self::GraphVersion),
            _ => None,
        }
    }
}

fn valid_node_id(id: &CheckedNodeId) -> bool {
    id.is_well_formed()
}

impl ContractPackageWireV1 {
    fn validate_values(
        &self,
        limits: CheckedPackageReadLimits,
        work_meter: &mut super::WorkMeter,
    ) -> Result<(Vec<CheckedNodeKind>, Vec<Option<CheckedNodeTag>>), ValidationFailure> {
        let source = &self.package.source;
        if source.name.is_empty() {
            return Err(malformed(
                JsonPointer::root().key("package").key("source").key("name"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        let profile = &self.package.lowering_profile;
        if profile.authority.is_empty() || profile.identity.is_empty() {
            return Err(malformed(
                JsonPointer::root().key("package").key("lowering_profile"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        let lock_at = JsonPointer::root().key("package").key("source").key("lock");
        if source.lock.sources.is_empty() {
            return Err(malformed(
                lock_at.clone().key("sources"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        for (index, selected) in source.lock.sources.iter().enumerate() {
            let at = lock_at.clone().key("sources").index(index);
            for (name, text) in [
                ("authority", selected.authority.as_ref()),
                ("identity", selected.identity.as_ref()),
            ] {
                if text.is_empty() {
                    return Err(malformed(
                        at.clone().key(name),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
        }
        for (at, definition) in std::iter::once((
            lock_at.clone().key("edition").key("definition"),
            &source.lock.edition.definition,
        ))
        .chain(
            source
                .lock
                .profile_selections
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    (
                        lock_at
                            .clone()
                            .key("profile_selections")
                            .index(index)
                            .key("definition"),
                        &row.definition,
                    )
                }),
        )
        .chain(
            source
                .lock
                .definition_selections
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    (
                        lock_at.clone().key("definition_selections").index(index),
                        row,
                    )
                }),
        ) {
            if definition.authority.is_empty() || definition.identity.is_empty() {
                return Err(malformed(at, CheckedPackageRefusalCause::InvalidValue));
            }
        }
        let mut features = BTreeSet::new();
        for (index, feature) in source.lock.required_features.iter().enumerate() {
            if feature.is_empty() || !features.insert(feature) {
                return Err(malformed(
                    lock_at.clone().key("required_features").index(index),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
        }
        let mut previous_identity: Option<&str> = None;
        for (index, selected) in source.lock.dependency_selections.iter().enumerate() {
            let at = JsonPointer::root()
                .key("package")
                .key("source")
                .key("lock")
                .key("dependency_selections")
                .index(index);
            if selected.identity.is_empty()
                || previous_identity.is_some_and(|prior| prior >= selected.identity.as_ref())
            {
                return Err(malformed(
                    at.key("identity"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            previous_identity = Some(&selected.identity);
        }
        if ContractWireTag::from_wire(&self.semantic_graph.graph_version)
            != Some(ContractWireTag::GraphVersion)
        {
            return Err(malformed(
                JsonPointer::root()
                    .key("semantic_graph")
                    .key("graph_version"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        let nodes = &self.semantic_graph.nodes;
        if u64::try_from(nodes.len()).unwrap_or(u64::MAX) > limits.nodes {
            return Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Nodes,
                limits.nodes,
                nodes.len(),
                Some(JsonPointer::root().key("semantic_graph").key("nodes")),
            ));
        }
        let structural_keys = super::node_index::rebuild_graph_keys(nodes, |at| {
            malformed(at, CheckedPackageRefusalCause::InvalidValue)
        })?;
        let mut interner = super::node_index::StructuralInterner::new();
        let structural_tokens = interner.tokens(&structural_keys).ok_or_else(|| {
            malformed(
                JsonPointer::root().key("semantic_graph").key("nodes"),
                CheckedPackageRefusalCause::InvalidValue,
            )
        })?;
        let mut seen = BTreeSet::new();
        let mut edges = 0_u64;
        let mut node_kinds = Vec::with_capacity(nodes.len());
        for (index, node) in nodes.iter().enumerate() {
            let at = JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(index);
            if !valid_node_id(&node.node_id) || !seen.insert(structural_tokens[index]) {
                return Err(malformed(
                    at.key("node_id"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if !valid_node_id(&node.semantic_type)
                || node.dependencies.iter().any(|id| !valid_node_id(id))
            {
                return Err(malformed(
                    at.key("dependencies"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            let kind = super::decode_node_kind(node, |member| at.clone().key(member))?;
            node_kinds.push(kind);
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(index),
                Step::Key("body"),
            ];
            let work =
                super::validate_body(kind, &node.body, &Trail::Base(&steps), &mut |_, _, _| {})?;
            work_meter.charge(work, || at.clone().key("body"))?;
            edges =
                edges.saturating_add(u64::try_from(node.dependencies.len()).unwrap_or(u64::MAX));
            if edges > limits.edges {
                return Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Edges,
                    limits.edges,
                    edges,
                    Some(at.key("dependencies")),
                ));
            }
        }
        let mut unsupported_tags = Vec::with_capacity(self.feature_accounting.len());
        for (index, record) in self.feature_accounting.iter().enumerate() {
            if !valid_node_id(record.node_id()) {
                return Err(malformed(
                    JsonPointer::root()
                        .key("feature_accounting")
                        .index(index)
                        .key("node_id"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            let unsupported_tag = match record {
                ContractFeatureRecordV1::Unsupported { node_tag, .. } => {
                    Some(CheckedNodeTag::from_wire(node_tag).ok_or_else(|| {
                        malformed(
                            JsonPointer::root()
                                .key("feature_accounting")
                                .index(index)
                                .key("node_tag"),
                            CheckedPackageRefusalCause::InvalidValue,
                        )
                    })?)
                }
                _ => None,
            };
            unsupported_tags.push(unsupported_tag);
        }
        Ok((node_kinds, unsupported_tags))
    }

    fn validate_correspondence(
        &self,
        limits: CheckedPackageReadLimits,
        frames: &ContractFrameKeys,
    ) -> Result<(), ValidationFailure> {
        let source_map = &self.source_map;
        let mut count = 0_u64;
        let charge = |count: &mut u64, at: JsonPointer| -> Result<(), ValidationFailure> {
            *count = count.saturating_add(1);
            if *count > limits.occurrences {
                return Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Occurrences,
                    limits.occurrences,
                    *count,
                    Some(at),
                ));
            }
            Ok(())
        };
        for (entries, at) in source_map
            .nodes
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    &entry.occurrences,
                    JsonPointer::root()
                        .key("source_map")
                        .key("nodes")
                        .index(index),
                )
            })
            .chain(
                source_map
                    .requested_items
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| {
                        (
                            &entry.occurrences,
                            JsonPointer::root()
                                .key("source_map")
                                .key("requested_items")
                                .index(index),
                        )
                    }),
            )
            .chain(
                source_map
                    .refusals
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| {
                        (
                            &entry.occurrences,
                            JsonPointer::root()
                                .key("source_map")
                                .key("refusals")
                                .index(index),
                        )
                    }),
            )
        {
            for (index, entry) in entries.iter().enumerate() {
                let occurrence_at = at.clone().key("occurrences").index(index);
                charge(&mut count, occurrence_at.clone())?;
                for region_index in 0..entry.regions.len() {
                    charge(
                        &mut count,
                        occurrence_at.clone().key("regions").index(region_index),
                    )?;
                }
            }
        }
        if source_map.nodes.len() != self.semantic_graph.nodes.len() {
            return Err(malformed(
                JsonPointer::root().key("source_map"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        for (index, (mapped, node)) in source_map
            .nodes
            .iter()
            .zip(&self.semantic_graph.nodes)
            .enumerate()
        {
            let at = JsonPointer::root()
                .key("source_map")
                .key("nodes")
                .index(index);
            if mapped.node_id != node.node_id {
                return Err(malformed(
                    at.key("node_id"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            self.validate_occurrences(&mapped.occurrences, &node.node_id, Some(node), at, frames)?;
        }
        let node_sources: BTreeMap<_, _> = source_map
            .nodes
            .iter()
            .filter_map(|entry| {
                frames
                    .output(&entry.node_id)
                    .map(|key| (key, &entry.occurrences))
            })
            .collect();
        for (index, mapped) in source_map.requested_items.iter().enumerate() {
            self.validate_occurrences(
                &mapped.occurrences,
                &mapped.requested_item,
                None,
                JsonPointer::root()
                    .key("source_map")
                    .key("requested_items")
                    .index(index),
                frames,
            )?;
            if let Some(occurrences) = frames
                .source(&mapped.requested_item)
                .and_then(|key| node_sources.get(key))
            {
                if *occurrences != &mapped.occurrences {
                    return Err(malformed(
                        JsonPointer::root()
                            .key("source_map")
                            .key("requested_items")
                            .index(index)
                            .key("occurrences"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
        }
        for (index, mapped) in source_map.refusals.iter().enumerate() {
            self.validate_occurrences(
                &mapped.occurrences,
                &mapped.requested_item,
                None,
                JsonPointer::root()
                    .key("source_map")
                    .key("refusals")
                    .index(index),
                frames,
            )?;
        }
        Ok(())
    }

    fn validate_occurrences(
        &self,
        entries: &[CheckedSourceMapEntry],
        subject: &CheckedNodeId,
        represented: Option<&CheckedSemanticNodeV2>,
        at: JsonPointer,
        frames: &ContractFrameKeys,
    ) -> Result<(), ValidationFailure> {
        let mut keys = BTreeSet::new();
        for (index, entry) in entries.iter().enumerate() {
            let entry_at = at.clone().key("occurrences").index(index);
            let key = (&entry.role, entry.ordinal);
            let matching_subject = if represented.is_some() {
                frames.source_equals_output(&entry.node_id, subject)
            } else {
                frames.source_equals_source(&entry.node_id, subject)
            };
            if !matching_subject || !keys.insert(key) {
                return Err(malformed(
                    entry_at,
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if entry.regions.is_empty() {
                return Err(malformed(
                    entry_at.clone().key("regions"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if let Some(node) = represented {
                if !node.occurrences.iter().any(|occurrence| {
                    occurrence.role == entry.role && occurrence.ordinal == entry.ordinal
                }) {
                    return Err(malformed(
                        entry_at,
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
            let mut by_source: BTreeMap<&_, Vec<(u64, u64)>> = BTreeMap::new();
            for (region_index, region) in entry.regions.iter().enumerate() {
                if region.start >= region.end {
                    return Err(malformed(
                        entry_at
                            .clone()
                            .key("regions")
                            .index(region_index)
                            .key("start"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
                if !self.package.source.lock.sources.contains(&region.source) {
                    return Err(malformed(
                        entry_at
                            .clone()
                            .key("regions")
                            .index(region_index)
                            .key("source"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
                by_source
                    .entry(&region.source)
                    .or_default()
                    .push((region.start, region.end));
            }
            for ranges in by_source.values_mut() {
                ranges.sort_unstable();
                if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
                    return Err(malformed(
                        entry_at.clone().key("regions"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
        }
        if let Some(node) = represented {
            if keys.len() != node.occurrences.len() {
                return Err(malformed(
                    JsonPointer::root().key("source_map"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
        }
        Ok(())
    }

    fn validate_accounting(&self, frames: &ContractFrameKeys) -> Result<(), ValidationFailure> {
        let accounting = &self.feature_accounting;
        if self.source_map.requested_items.len() != accounting.len() {
            return Err(malformed(
                JsonPointer::root().key("source_map"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        let represented: BTreeSet<&CheckedNodeId> = self
            .semantic_graph
            .nodes
            .iter()
            .filter_map(|node| frames.output(&node.node_id))
            .collect();
        let mut refusal = 0;
        for (record, mapped) in accounting.iter().zip(&self.source_map.requested_items) {
            if !frames.source_equals_source(record.node_id(), &mapped.requested_item) {
                return Err(malformed(
                    JsonPointer::root().key("feature_accounting"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if record.disposition() == ContractDispositionV1::Lowered {
                if frames
                    .source(record.node_id())
                    .is_none_or(|key| !represented.contains(key))
                {
                    return Err(malformed(
                        JsonPointer::root().key("feature_accounting"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            } else {
                let Some(mapped_refusal) = self.source_map.refusals.get(refusal) else {
                    return Err(malformed(
                        JsonPointer::root().key("source_map"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                };
                if !frames.source_equals_source(&mapped_refusal.requested_item, record.node_id())
                    || mapped_refusal.disposition != record.disposition()
                    || mapped_refusal.occurrences != mapped.occurrences
                {
                    return Err(malformed(
                        JsonPointer::root().key("feature_accounting"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
                refusal += 1;
            }
        }
        if refusal != self.source_map.refusals.len() {
            return Err(malformed(
                JsonPointer::root().key("source_map"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        Ok(())
    }

    fn validate_dependencies(
        &self,
        dependencies: &ContractDependencies<'_>,
        meter: &mut super::WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let selected = &self.package.source.lock.dependency_selections;
        let mut packages = Vec::with_capacity(selected.len());
        for (index, selection) in selected.iter().enumerate() {
            let at = JsonPointer::root()
                .key("package")
                .key("source")
                .key("lock")
                .key("dependency_selections")
                .index(index);
            let Some(input) = dependencies.get(selection.identity.as_ref()) else {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::MissingImport,
                    at,
                    CheckedPackageRefusalCause::MissingSelection,
                ));
            };
            if input.identity != selection.identity.as_ref() {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::MissingImport,
                    at,
                    CheckedPackageRefusalCause::MissingSelection,
                ));
            }
            if input.package.name() != selection.identity.as_ref() {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::MissingImport,
                    at,
                    CheckedPackageRefusalCause::MissingSelection,
                ));
            }
            packages.push(input.package);
        }
        check_derived_closure(
            selected,
            &packages,
            JsonPointer::root().key("package").key("source"),
            meter,
        )
    }

    fn validate_references(
        &self,
        dependencies: &ContractDependencies<'_>,
        limits: CheckedPackageReadLimits,
        meter: &mut super::WorkMeter,
        frames: &ContractFrameKeys,
    ) -> Result<Vec<Option<CheckedTerm>>, ValidationFailure> {
        let admitted: BTreeMap<&str, &CheckedPackageV2> = dependencies
            .iter()
            .map(|(identity, input)| (*identity, input.package))
            .collect();
        let supplied = SuppliedDependencies::from_admitted(
            &self.package.source.lock.dependency_selections,
            &admitted,
        )
        .map_err(|index| {
            ValidationFailure::refused_because(
                CheckedPackageRefusalCode::MissingImport,
                JsonPointer::root()
                    .key("package")
                    .key("source")
                    .key("lock")
                    .key("dependency_selections")
                    .index(index),
                CheckedPackageRefusalCause::MissingSelection,
            )
        })?;
        let nodes: BTreeMap<&CheckedNodeId, &CheckedSemanticNodeV2> = self
            .semantic_graph
            .nodes
            .iter()
            .map(|node| (&node.node_id, node))
            .collect();
        let positions: BTreeMap<&CheckedNodeId, usize> = self
            .semantic_graph
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (&node.node_id, index))
            .collect();
        let mut outgoing: BTreeMap<CheckedNodeId, Vec<CheckedNodeId>> = BTreeMap::new();
        let mut adjacency = Vec::with_capacity(self.semantic_graph.nodes.len());
        let mut kinds = Vec::with_capacity(self.semantic_graph.nodes.len());
        let mut terms = Vec::with_capacity(self.semantic_graph.nodes.len());
        for (index, node) in self.semantic_graph.nodes.iter().enumerate() {
            let at = JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(index);
            if !nodes.contains_key(&node.semantic_type)
                || node.dependencies.iter().any(|key| !nodes.contains_key(key))
            {
                return Err(malformed(
                    at.key("dependencies"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            let kind = super::decode_node_kind(node, |member| at.clone().key(member))?;
            kinds.push(kind);
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(index),
                Step::Key("body"),
            ];
            let mut missing = None;
            let mut body_references = Vec::new();
            let mut edges = node.dependencies.clone();
            edges.push(node.semantic_type.clone());
            super::validate_body(
                kind,
                &node.body,
                &Trail::Base(&steps),
                &mut |target, site, pointer| {
                    edges.push(target.clone());
                    body_references.push((target.clone(), site));
                    if missing.is_none() && !nodes.contains_key(target) {
                        missing = Some(pointer.pointer());
                    }
                },
            )?;
            if let Some(pointer) = missing {
                return Err(malformed(pointer, CheckedPackageRefusalCause::InvalidValue));
            }
            terms.push(match super::decode_admitted_body(kind, &node.body) {
                Ok(term) => Some(term),
                Err(_) if super::body_grammar(kind) == super::BodyGrammar::AbstractionRelation => {
                    None
                }
                Err(_) => {
                    return Err(malformed(
                        at.clone().key("body"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            });
            let mut successors = Vec::new();
            let Some(&semantic_type) = positions.get(&node.semantic_type) else {
                return Err(malformed(
                    at.clone().key("semantic_type"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            };
            if semantic_type != index {
                successors.push((semantic_type, super::EdgeSite::SemanticType));
            }
            for (dependency_index, dependency) in node.dependencies.iter().enumerate() {
                let Some(&target) = positions.get(dependency) else {
                    return Err(malformed(
                        at.clone().key("dependencies").index(dependency_index),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                };
                successors.push((target, super::EdgeSite::Dependency(dependency_index)));
            }
            for (reference_index, (target, site)) in body_references.iter().enumerate() {
                let Some(&target) = positions.get(target) else {
                    return Err(malformed(
                        super::body_reference_pointer(node, kind, index, reference_index),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                };
                if super::reference_requires_cycle_edge(index, target, semantic_type, *site) {
                    successors.push((target, super::EdgeSite::Reference(reference_index)));
                }
            }
            adjacency.push(successors);
            outgoing.insert(node.node_id.clone(), edges);
            if let Some(term) = terms.last().and_then(Option::as_ref) {
                DependencyReferences::new(&supplied).walk_admitted(
                    Referrer {
                        node,
                        position: index,
                    },
                    term,
                    &mut *meter,
                )?;
            }
        }
        // FR-322's graph-only checks apply to the retained nodes even though
        // this reader does not have the originating source or model documents.
        // In particular, a reference type cannot acquire a scalar target just
        // because the scalar key exists in this graph.
        for (position, (node, kind)) in self.semantic_graph.nodes.iter().zip(&kinds).enumerate() {
            if *kind == super::CheckedNodeKind::CompositeType(super::CompositeTypeForm::Reference) {
                let target = super::structural::reference_type_target(node)
                    .and_then(|key| positions.get(&key).copied())
                    .and_then(|target| kinds.get(target));
                if !matches!(
                    target,
                    Some(super::CheckedNodeKind::Model(super::ModelForm::ObjectType))
                ) {
                    return Err(malformed(
                        JsonPointer::root()
                            .key("semantic_graph")
                            .key("nodes")
                            .index(position)
                            .key("body"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
        }
        super::structural::validate_structural_nodes(
            &self.semantic_graph.nodes,
            &kinds,
            &terms,
            &positions,
        )?;
        super::identity::validate_nominal_nodes(
            &self.semantic_graph.nodes,
            &kinds,
            &positions,
            &self.package.source.lock,
            &[],
            &supplied,
            &mut *meter,
            limits.bytes,
        )?;
        let reference_pointer = |position: usize, reference: usize| match (
            self.semantic_graph.nodes.get(position),
            kinds.get(position),
        ) {
            (Some(node), Some(kind)) => {
                super::body_reference_pointer(node, *kind, position, reference)
            }
            _ => JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("body"),
        };
        let edge_pointer = |vertex: usize, site: super::EdgeSite| match site {
            super::EdgeSite::SemanticType => JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(vertex)
                .key("semantic_type"),
            super::EdgeSite::Dependency(dependency) => JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(vertex)
                .key("dependencies")
                .index(dependency),
            super::EdgeSite::Reference(reference) => reference_pointer(vertex, reference),
        };
        super::validate_recursion(
            &self.semantic_graph.nodes,
            &adjacency,
            &mut *meter,
            &edge_pointer,
        )?;
        let mut reached = BTreeSet::new();
        let output_by_key = frames
            .output
            .iter()
            .map(|(wire, key)| (key.as_ref(), wire))
            .collect::<BTreeMap<_, _>>();
        let mut queue = VecDeque::new();
        for (index, record) in self.feature_accounting.iter().enumerate() {
            if record.disposition() != ContractDispositionV1::Lowered {
                continue;
            }
            let output = frames
                .source(record.node_id())
                .and_then(|key| output_by_key.get(key))
                .ok_or_else(|| {
                    malformed(
                        JsonPointer::root()
                            .key("feature_accounting")
                            .index(index)
                            .key("node_id"),
                        CheckedPackageRefusalCause::InvalidValue,
                    )
                })?;
            queue.push_back((*output).clone());
        }
        while let Some(key) = queue.pop_front() {
            if reached.insert(key.clone()) {
                if let Some(edges) = outgoing.get(&key) {
                    queue.extend(edges.iter().cloned());
                }
            }
        }
        if reached.len() != nodes.len() {
            return Err(malformed(
                JsonPointer::root().key("semantic_graph").key("nodes"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        Ok(terms)
    }
}
