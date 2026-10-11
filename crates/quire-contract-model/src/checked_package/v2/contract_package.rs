//! FR-035 and QSpec FR-330: the single target-neutral ContractPackage wire.

use super::dependency_references::{DependencyReferences, Referrer, SuppliedDependencies};
use super::encode::{elements, encoded, member as encode_member};
use super::lower::{
    CompleteContractNodeV2, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    ContractPackageDependencyV2, LoweringProfileRef, NodeSourceMaps, CONTRACT_IR_SEMANTIC_DOMAIN,
    CONTRACT_PACKAGE_VERSION,
};
use super::{CheckedNodeTag, CheckedPackageLockV2, CheckedPackageV2, CheckedSemanticNodeV2};
use crate::checked_package::common::{
    read_typed_prefix, strict_parse, Step, Trail, ValidationFailure, NODE_DOMAIN,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedSemanticId, CheckedSourceMapEntry, JsonPointer,
};
use quire_canonical::{Encode, Error as CanonicalError, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The source package's exact admitted identity and lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractSourceV1 {
    /// Originating admitted checked-package identity.
    pub package_id: CheckedSemanticId,
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

/// One key-ordered collection of exact admitted semantic nodes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSemanticGraphV1 {
    /// The FR-322 graph version.
    pub graph_version: Box<str>,
    /// Nodes represented by at least one lowered request, including their closure.
    pub nodes: Vec<CheckedSemanticNodeV2>,
}

/// One represented graph node's exact source occurrences.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractNodeSourcesV1 {
    /// Represented semantic node key.
    pub node_id: CheckedNodeId,
    /// Exact FR-322 source-map entries for the node.
    pub occurrences: Vec<CheckedSourceMapEntry>,
}

/// One requested item's source occurrences, including an absent request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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

/// A refused request's exact available sources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct ContractSourceMapV1 {
    /// One entry per represented node, in node-key order.
    pub nodes: Vec<ContractNodeSourcesV1>,
    /// One entry per request, in request order.
    pub requested_items: Vec<ContractRequestSourcesV1>,
    /// One entry per non-lowered request, in request order.
    pub refusals: Vec<ContractRefusalSourcesV1>,
}

/// Closed projection of a defensive FR-322 refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
            path: value.path.clone(),
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractPackageWireV1 {
    /// Exactly `quire.contract-package/v1`.
    pub contract_version: Box<str>,
    /// Exact source provenance and explicit lowering profile.
    pub package: ContractProvenanceV1,
    /// One key-ordered semantic node collection.
    pub semantic_graph: ContractSemanticGraphV1,
    /// Total source correspondence.
    pub source_map: ContractSourceMapV1,
    /// One result per request, in request order.
    pub feature_accounting: Vec<ContractFeatureRecordV1>,
    /// SHA-256 of the four-member canonical semantic projection.
    pub identity: CheckedSemanticId,
}

/// A ContractPackage produced from an admitted source or admitted by the sole
/// strict reader. Consumers take this type so a raw wire value cannot bypass
/// admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPackageV1 {
    wire: ContractPackageWireV1,
}

impl ContractPackageV1 {
    /// The exact six-field wire, available for read-only inspection.
    pub fn wire(&self) -> &ContractPackageWireV1 {
        &self.wire
    }

    /// The canonical semantic identity retained in the admitted value.
    pub fn identity(&self) -> &CheckedSemanticId {
        &self.wire.identity
    }
}

/// One typed FR-330 read outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractPackageReadResult {
    /// Fully admitted package.
    Admitted(Box<ContractPackageV1>),
    /// Located refusal.
    Refused(CheckedPackageRefusal),
    /// A caller limit stopped admission.
    Incomplete(CheckedPackageIncomplete),
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

struct SemanticProjection<'a> {
    contract_version: &'a str,
    package: &'a ContractProvenanceV1,
    semantic_graph: &'a ContractSemanticGraphV1,
    feature_accounting: &'a [ContractFeatureRecordV1],
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

impl Encode for SemanticProjection<'_> {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encode_member(writer, "contract_version", &self.contract_version)?;
        encode_member(writer, "package", self.package)?;
        encoded(writer, "semantic_graph", self.semantic_graph)?;
        encode_member(writer, "feature_accounting", &self.feature_accounting)?;
        writer.end_object()
    }
}

impl Encode for ContractPackageWireV1 {
    fn encode_into<S: Sink + ?Sized>(
        &self,
        writer: &mut Writer<'_, S>,
    ) -> Result<(), CanonicalError> {
        writer.begin_object()?;
        encode_member(writer, "contract_version", &self.contract_version)?;
        encode_member(writer, "package", &self.package)?;
        encoded(writer, "semantic_graph", &self.semantic_graph)?;
        encode_member(writer, "source_map", &self.source_map)?;
        encode_member(writer, "feature_accounting", &self.feature_accounting)?;
        encode_member(writer, "identity", &self.identity)?;
        writer.end_object()
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
        let mut nodes = BTreeMap::new();
        for represented in lowered {
            nodes.insert(represented.node.node_id.clone(), represented.node.clone());
        }
        for represented in dependencies {
            nodes.insert(represented.node.node_id.clone(), represented.node.clone());
        }
        let node_sources = nodes
            .keys()
            .map(|node_id| ContractNodeSourcesV1 {
                node_id: node_id.clone(),
                occurrences: source_maps.of(node_id),
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
                package_id: source.package_id().clone(),
                lock: source.lock().clone(),
            },
            lowering_profile: profile.reference.clone(),
        };
        let semantic_graph = ContractSemanticGraphV1 {
            graph_version: source.graph().graph_version.clone(),
            nodes: nodes.into_values().collect(),
        };
        let source_map = ContractSourceMapV1 {
            nodes: node_sources,
            requested_items,
            refusals,
        };
        let projection = SemanticProjection {
            contract_version: CONTRACT_PACKAGE_VERSION,
            package: &package,
            semantic_graph: &semantic_graph,
            feature_accounting: &accounting,
        };
        // The package ceiling charges the emitted six-field value. Hashing its
        // smaller semantic projection must not report that projection's length
        // as the package's failed byte charge.
        let preimage = canonical(&projection, u64::MAX)?;
        let digest = format!("{:x}", Sha256::digest(&preimage));
        let wire = Self {
            contract_version: CONTRACT_PACKAGE_VERSION.into(),
            package,
            semantic_graph,
            source_map,
            feature_accounting: accounting,
            identity: CheckedSemanticId {
                domain: CONTRACT_IR_SEMANTIC_DOMAIN.into(),
                algorithm: "sha256".into(),
                digest: digest.into_boxed_str(),
            },
        };
        let bytes = canonical(&wire, ceiling)?.into_boxed_slice();
        Ok((ContractPackageV1 { wire }, bytes))
    }
}

impl ContractPackageWireV1 {
    /// Reads only a complete canonical FR-330 value; dependencies are admitted inputs.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        dependencies: &ContractDependencies<'_>,
    ) -> ContractPackageReadResult {
        match Self::admit(bytes, limits, dependencies) {
            Ok(value) => {
                ContractPackageReadResult::Admitted(Box::new(ContractPackageV1 { wire: value }))
            }
            Err(ValidationFailure::Refused(refusal)) => ContractPackageReadResult::Refused(refusal),
            Err(ValidationFailure::Incomplete(incomplete)) => {
                ContractPackageReadResult::Incomplete(incomplete)
            }
        }
    }

    fn admit(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        dependencies: &ContractDependencies<'_>,
    ) -> Result<Self, ValidationFailure> {
        if let Err(failure) = read_typed_prefix(bytes, limits) {
            return Err(match failure {
                ValidationFailure::Refused(refusal)
                    if refusal.code == CheckedPackageRefusalCode::DuplicateMember =>
                {
                    malformed(
                        refusal.path.unwrap_or_else(JsonPointer::root),
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
            Some(Value::String(version)) if version == CONTRACT_PACKAGE_VERSION => {}
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
        let wire: Self = serde_path_to_error::deserialize(&value).map_err(|error| {
            malformed(
                crate::checked_package::common::decoder_pointer(JsonPointer::root(), error.path()),
                CheckedPackageRefusalCause::WrongValueKind,
            )
        })?;
        wire.validate_values(limits)?;
        wire.validate_correspondence(limits)?;
        wire.validate_accounting()?;
        wire.validate_identity(limits.bytes)?;
        wire.validate_dependencies(dependencies)?;
        wire.validate_references(dependencies, limits)?;
        Ok(wire)
    }
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

fn id_shape(value: &Value, at: JsonPointer, package: bool) -> Result<(), ValidationFailure> {
    let required = if package {
        &["domain", "algorithm", "digest"][..]
    } else {
        &["domain", "digest"][..]
    };
    let object = shape(value, at.clone(), required, &[])?;
    for name in required {
        string(member(object, name, &at)?, at.clone().key(name))?;
    }
    Ok(())
}

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
            "source_map",
            "feature_accounting",
            "identity",
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
        &["package_id", "lock"],
        &[],
    )?;
    id_shape(
        member(source, "package_id", &source_at)?,
        source_at.clone().key("package_id"),
        true,
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
            "model_selections",
            "required_features",
            "dependency_selections",
        ],
        &[],
    )?;
    for name in [
        "sources",
        "profile_selections",
        "definition_selections",
        "model_selections",
        "required_features",
        "dependency_selections",
    ] {
        array(member(lock, name, &lock_at)?, lock_at.clone().key(name))?;
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
            &[
                "recursion_group",
                "nominal_identity_preimage",
                "declaration",
                "owner",
            ],
        )?;
        id_shape(
            member(node, "node_id", &at)?,
            at.clone().key("node_id"),
            false,
        )?;
        id_shape(
            member(node, "semantic_type", &at)?,
            at.clone().key("semantic_type"),
            false,
        )?;
        for name in ["schema_version", "node_tag", "semantic_form"] {
            string(member(node, name, &at)?, at.clone().key(name))?;
        }
        for name in ["dependencies", "occurrences"] {
            array(member(node, name, &at)?, at.clone().key(name))?;
        }
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
            array(
                member(entry, "occurrences", &entry_at)?,
                entry_at.key("occurrences"),
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
    id_shape(member(top, "identity", &root)?, root.key("identity"), true)?;
    Ok(())
}

fn validate_record_shape(value: &Value, at: JsonPointer) -> Result<(), ValidationFailure> {
    let object = object(value, at.clone())?;
    let disposition = string(
        member(object, "disposition", &at)?,
        at.clone().key("disposition"),
    )?;
    let payload: &[&str] = match disposition {
        "lowered" | "invalid_input" => &[],
        "unsupported" => &["unsupported_node_id", "node_tag"],
        "requires_bound" => &["unbounded_type"],
        "failed" => &["limit_kind", "limit", "consumed"],
        "invalid_body" => &["body_node_id", "refusal"],
        "body_incomplete" => &["body_node_id", "incomplete"],
        _ => {
            return Err(malformed(
                at.key("disposition"),
                CheckedPackageRefusalCause::InvalidValue,
            ))
        }
    };
    let required: Vec<&str> = ["node_id", "disposition"]
        .into_iter()
        .chain(payload.iter().copied())
        .collect();
    let record = shape(value, at.clone(), &required, &[])?;
    id_shape(
        member(record, "node_id", &at)?,
        at.clone().key("node_id"),
        false,
    )?;
    for name in ["unsupported_node_id", "unbounded_type", "body_node_id"] {
        if let Some(value) = record.get(name) {
            id_shape(value, at.clone().key(name), false)?;
        }
    }
    if disposition == "failed" {
        for counter in ["limit", "consumed"] {
            decimal(member(record, counter, &at)?, at.clone().key(counter))?;
        }
    }
    if disposition == "invalid_body" {
        validate_body_refusal(member(record, "refusal", &at)?, at.clone().key("refusal"))?;
    }
    if disposition == "body_incomplete" {
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
        shape(locus, locus_at.clone(), &["domain", "digest"], &[])?;
        let id: CheckedNodeId = serde_json::from_value(locus.clone())
            .map_err(|_| malformed(locus_at.clone(), CheckedPackageRefusalCause::WrongValueKind))?;
        if !valid_node_id(&id) {
            return Err(malformed(
                locus_at,
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
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
        Cause::MalformedJson
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
        | Cause::StaleNodeKey
        | Cause::DeclarationNominalMismatch => code == Code::InvalidPackage,
        Cause::AmbiguousName => code == Code::AmbiguousDeclaration,
        Cause::UnsupportedWire => code == Code::UnknownContractVersion,
        Cause::UnsupportedFeature => code == Code::UnknownRequiredCapability,
        Cause::ByteDigestMismatch | Cause::DigestDomainMismatch | Cause::ContentMismatch => {
            code == Code::StaleDependency
        }
        Cause::InsufficientNextCharge => false,
        Cause::OperatorIneligible | Cause::TypeMismatch => code == Code::IllTyped,
        Cause::MissingName => code == Code::MissingDeclaration,
        Cause::MalformedDeclaration => code == Code::InvalidModelBinding,
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

fn valid_digest(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_node_id(id: &CheckedNodeId) -> bool {
    id.domain.as_ref() == NODE_DOMAIN && valid_digest(&id.digest)
}

impl ContractPackageWireV1 {
    fn validate_values(&self, limits: CheckedPackageReadLimits) -> Result<(), ValidationFailure> {
        let source = &self.package.source;
        let package_id = &source.package_id;
        if package_id.domain.as_ref() != super::PACKAGE_DOMAIN_V2
            || package_id.algorithm.as_ref() != "sha256"
            || !valid_digest(&package_id.digest)
        {
            return Err(malformed(
                JsonPointer::root()
                    .key("package")
                    .key("source")
                    .key("package_id"),
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
            if selected.digest_domain.as_ref() != super::SOURCE_BYTES {
                return Err(malformed(
                    at.clone().key("digest_domain"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if !valid_digest(&selected.digest) {
                return Err(malformed(
                    at.key("digest"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
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
        let mut models = BTreeSet::new();
        let mut model_identities = BTreeSet::new();
        for (index, selected) in source.lock.model_selections.iter().enumerate() {
            if selected.identity.is_empty()
                || selected.digest_domain.as_ref() != super::DOMAIN_PACKAGE_DIGEST
                || !valid_digest(&selected.digest)
                || !models.insert(selected)
                || !model_identities.insert(selected.identity.as_ref())
            {
                return Err(malformed(
                    lock_at.clone().key("model_selections").index(index),
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
            if selected.package_id.domain.as_ref() != super::PACKAGE_DOMAIN_V2
                || selected.package_id.algorithm.as_ref() != "sha256"
                || !valid_digest(&selected.package_id.digest)
            {
                return Err(malformed(
                    at.key("package_id"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
        }
        if self.semantic_graph.graph_version.as_ref() != "quire.checked-semantic-graph/v2" {
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
        let mut previous: Option<&CheckedNodeId> = None;
        let mut edges = 0_u64;
        let mut work = 0_u64;
        for (index, node) in nodes.iter().enumerate() {
            let at = JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(index);
            if !valid_node_id(&node.node_id) || previous.is_some_and(|key| key >= &node.node_id) {
                return Err(malformed(
                    at.key("node_id"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            previous = Some(&node.node_id);
            if !valid_node_id(&node.semantic_type)
                || node.dependencies.iter().any(|id| !valid_node_id(id))
            {
                return Err(malformed(
                    at.key("dependencies"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            let kind = super::decode_node_kind(node, |member| at.clone().key(member))?;
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(index),
                Step::Key("body"),
            ];
            work = work.saturating_add(super::validate_body(
                kind,
                &node.body,
                &Trail::Base(&steps),
                &mut |_, _, _| {},
            )?);
            if work > limits.work {
                return Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Work,
                    limits.work,
                    work,
                    Some(at.clone().key("body")),
                ));
            }
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
            if let ContractFeatureRecordV1::Unsupported { node_tag, .. } = record {
                if CheckedNodeTag::from_wire(node_tag).is_none() {
                    return Err(malformed(
                        JsonPointer::root()
                            .key("feature_accounting")
                            .index(index)
                            .key("node_tag"),
                        CheckedPackageRefusalCause::InvalidValue,
                    ));
                }
            }
        }
        if self.identity.domain.as_ref() != CONTRACT_IR_SEMANTIC_DOMAIN {
            return Err(malformed(
                JsonPointer::root().key("identity").key("domain"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        if self.identity.algorithm.as_ref() != "sha256" || !valid_digest(&self.identity.digest) {
            return Err(malformed(
                JsonPointer::root().key("identity").key("digest"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        Ok(())
    }

    fn validate_correspondence(
        &self,
        limits: CheckedPackageReadLimits,
    ) -> Result<(), ValidationFailure> {
        let source_map = &self.source_map;
        let mut count = 0_u64;
        for entries in source_map
            .nodes
            .iter()
            .map(|entry| &entry.occurrences)
            .chain(
                source_map
                    .requested_items
                    .iter()
                    .map(|entry| &entry.occurrences),
            )
            .chain(source_map.refusals.iter().map(|entry| &entry.occurrences))
        {
            count = count.saturating_add(u64::try_from(entries.len()).unwrap_or(u64::MAX));
            for entry in entries {
                count =
                    count.saturating_add(u64::try_from(entry.regions.len()).unwrap_or(u64::MAX));
            }
            if count > limits.occurrences {
                return Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Occurrences,
                    limits.occurrences,
                    count,
                    Some(JsonPointer::root().key("source_map")),
                ));
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
            self.validate_occurrences(&mapped.occurrences, &node.node_id, Some(node), at)?;
        }
        let node_sources: BTreeMap<_, _> = source_map
            .nodes
            .iter()
            .map(|entry| (&entry.node_id, &entry.occurrences))
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
            )?;
            if let Some(occurrences) = node_sources.get(&mapped.requested_item) {
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
    ) -> Result<(), ValidationFailure> {
        let mut keys = BTreeSet::new();
        for (index, entry) in entries.iter().enumerate() {
            let entry_at = at.clone().key("occurrences").index(index);
            let key = (&entry.role, entry.ordinal);
            if &entry.node_id != subject || !keys.insert(key) {
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

    fn validate_accounting(&self) -> Result<(), ValidationFailure> {
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
            .map(|node| &node.node_id)
            .collect();
        let mut refusal = 0;
        for (record, mapped) in accounting.iter().zip(&self.source_map.requested_items) {
            if record.node_id() != &mapped.requested_item {
                return Err(malformed(
                    JsonPointer::root().key("feature_accounting"),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
            if record.disposition() == ContractDispositionV1::Lowered {
                if !represented.contains(record.node_id()) {
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
                if mapped_refusal.requested_item != *record.node_id()
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

    fn validate_identity(&self, ceiling: u64) -> Result<(), ValidationFailure> {
        let projection = SemanticProjection {
            contract_version: &self.contract_version,
            package: &self.package,
            semantic_graph: &self.semantic_graph,
            feature_accounting: &self.feature_accounting,
        };
        let preimage = canonical(&projection, ceiling).map_err(|_| {
            malformed(
                JsonPointer::root().key("identity"),
                CheckedPackageRefusalCause::InvalidValue,
            )
        })?;
        if format!("{:x}", Sha256::digest(&preimage)) != self.identity.digest.as_ref() {
            return Err(malformed(
                JsonPointer::root().key("identity").key("digest"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
        Ok(())
    }

    fn validate_dependencies(
        &self,
        dependencies: &ContractDependencies<'_>,
    ) -> Result<(), ValidationFailure> {
        let selected = &self.package.source.lock.dependency_selections;
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
            if input.package.package_id() != &selection.package_id {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusal::dependency_content_mismatch(
                        at,
                        selection.package_id.clone(),
                        input.package.package_id().clone(),
                    ),
                ));
            }
        }
        Ok(())
    }

    fn validate_references(
        &self,
        dependencies: &ContractDependencies<'_>,
        limits: CheckedPackageReadLimits,
    ) -> Result<(), ValidationFailure> {
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
        let mut meter = super::WorkMeter::new(limits.work);
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
            DependencyReferences::new(&supplied).walk_body(
                Referrer {
                    node,
                    position: index,
                },
                &mut meter,
            )?;
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
            &positions,
        )?;
        super::operations::validate_application_keys(
            &self.semantic_graph.nodes,
            &kinds,
            &positions,
            &mut meter,
            limits.bytes,
        )?;
        super::derived_keys::validate_derived_keys(
            &self.semantic_graph.nodes,
            &kinds,
            &positions,
            &[],
            &mut meter,
            limits.bytes,
        )?;
        super::identity::validate_nominal_nodes(
            &self.semantic_graph.nodes,
            &kinds,
            &positions,
            &self.package.source.lock,
            &mut meter,
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
            &mut meter,
            &edge_pointer,
        )?;
        let mut reached = BTreeSet::new();
        let mut queue = self
            .feature_accounting
            .iter()
            .filter_map(|record| {
                (record.disposition() == ContractDispositionV1::Lowered)
                    .then(|| record.node_id().clone())
            })
            .collect::<VecDeque<_>>();
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
        Ok(())
    }
}
