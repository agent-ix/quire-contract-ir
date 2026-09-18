//! Strict reader for QSpec I04 `quire.checked-package/v2`, the sole admitted
//! `CheckedPackage` contract.
//!
//! Consumes the public contract merged at quire-specification
//! `56c3e0b40a5eacf35df556c87d5e96d5eae5fe9b` (`proposals/checked-package-v2/`,
//! AD-006). Model selections are `sha256-jcs` domain packages, typed
//! separately from the raw source and definition byte artifacts.

mod identity;
mod lower;
mod natural;

pub use identity::*;
pub use lower::*;

use super::common::{
    canonical_value, count, decode_closed, digest_json, exceeds, is_digest, is_nonempty,
    validate_locked_artifact, validate_source_map_entries, validate_term, Stop, TermGrammar,
    ValidationFailure, NODE_DOMAIN,
};
use super::evidence::{CheckedDomainPackageLocator, CheckedPackageEvidence};
use super::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedOccurrence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CheckedSelection, CheckedSemanticId, CheckedSourceMapEntry,
    CheckedSourceRegion,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The I04 transport version admitted by the V2 reader.
pub const CHECKED_PACKAGE_V2: &str = "quire.checked-package/v2";
/// The V2 semantic package identity domain.
pub const PACKAGE_DOMAIN_V2: &str = "quire.package.semantic/v2";
const IDENTITY_PREIMAGE_V2: &str = "quire.checked-package-id/v2";
const GRAPH_V2: &str = "quire.checked-semantic-graph/v2";
const SOURCE_BYTES: &str = "quire.source.bytes/v1";
const DEFINITION_BYTES: &str = "quire.definition.bytes/v1";
/// The digest domain of a selected domain package.
pub const DOMAIN_PACKAGE_DIGEST: &str = "sha256-jcs";
const SELECTION_ROLES: [&str; 7] = [
    "language",
    "edition",
    "profile",
    "dependency",
    "binding_contract",
    "temporal_profile",
    "protocol_profile",
];
const DISPOSITIONS: [&str; 3] = ["available", "unimplemented", "unsupported"];

/// The closed V2 semantic node family.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CheckedNodeTag {
    /// `scalar_type`.
    ScalarType,
    /// `composite_type`.
    CompositeType,
    /// `bounded_domain`.
    BoundedDomain,
    /// `value`.
    Value,
    /// `expression`.
    Expression,
    /// `function`.
    Function,
    /// `model`.
    Model,
    /// `relation`.
    Relation,
    /// `state`.
    State,
    /// `temporal`.
    Temporal,
    /// `protocol`.
    Protocol,
    /// `claim`.
    Claim,
    /// `correspondence`.
    Correspondence,
}

impl CheckedNodeTag {
    /// Every V2 node family, in schema order.
    pub const ALL: [Self; 13] = [
        Self::ScalarType,
        Self::CompositeType,
        Self::BoundedDomain,
        Self::Value,
        Self::Expression,
        Self::Function,
        Self::Model,
        Self::Relation,
        Self::State,
        Self::Temporal,
        Self::Protocol,
        Self::Claim,
        Self::Correspondence,
    ];

    /// Parses a wire tag; `None` for a tag outside V2.
    pub fn from_wire(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.as_wire() == tag)
    }

    /// The exact wire tag.
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::ScalarType => "scalar_type",
            Self::CompositeType => "composite_type",
            Self::BoundedDomain => "bounded_domain",
            Self::Value => "value",
            Self::Expression => "expression",
            Self::Function => "function",
            Self::Model => "model",
            Self::Relation => "relation",
            Self::State => "state",
            Self::Temporal => "temporal",
            Self::Protocol => "protocol",
            Self::Claim => "claim",
            Self::Correspondence => "correspondence",
        }
    }

    /// The closed semantic forms of this family.
    pub const fn forms(self) -> &'static [&'static str] {
        match self {
            Self::ScalarType => &[
                "boolean",
                "integer",
                "rational",
                "decimal",
                "float32",
                "float64",
                "text",
                "dimension",
                "unit",
                "enum",
            ],
            Self::CompositeType => &[
                "option",
                "sequence",
                "set",
                "bag",
                "ordered_set",
                "record",
                "tuple",
                "alias",
                "reference",
            ],
            Self::BoundedDomain => &[
                "integer_range",
                "rational_range",
                "decimal_range",
                "float_rounding",
                "text_bounds",
                "collection_bounds",
                "model_population",
            ],
            Self::Value => &[
                "literal",
                "enum_value",
                "collection_value",
                "record_value",
                "tuple_value",
                "option_value",
            ],
            Self::Expression => &[
                "reference",
                "call",
                "unary",
                "binary",
                "conditional",
                "let",
                "quantify",
                "conversion",
                "query",
                "pre_read",
                "presence_read",
                "deref",
                "reachability",
            ],
            Self::Function => &["pure_function", "predicate", "recursive_function"],
            Self::Model => &["model_import", "model_type", "model_declaration"],
            Self::Relation => &[
                "relationship",
                "population",
                "membership",
                "causal_relation",
            ],
            Self::State => &[
                "state_clause",
                "frame",
                "transition",
                "operation_anchor",
                "snapshot",
            ],
            Self::Temporal => &[
                "temporal_clause",
                "formula",
                "clock",
                "window",
                "activation",
                "deadline",
            ],
            Self::Protocol => &[
                "protocol_clause",
                "role",
                "channel",
                "queue",
                "control",
                "obligation",
                "compensation",
            ],
            Self::Claim => &[
                "verification_claim",
                "analysis_claim",
                "hyperproperty",
                "synthesis_request",
            ],
            Self::Correspondence => &[
                "source_locus",
                "model_correspondence",
                "binding_role",
                "profile_correspondence",
            ],
        }
    }
}

/// A checked V2 semantic graph node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticNodeV2 {
    /// Stable checked node identity.
    pub node_id: CheckedNodeId,
    /// Must be `quire.checked-semantic-graph/v2`.
    pub schema_version: Box<str>,
    /// Closed semantic family (see [`CheckedNodeTag`]).
    pub node_tag: Box<str>,
    /// Closed semantic form for the family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Exact source occurrence keys.
    pub occurrences: Vec<CheckedOccurrence>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Nominal identity preimage; present exactly for nominal forms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nominal_identity_preimage: Option<NominalIdentityPreimage>,
    /// Typed public semantic term.
    pub body: Value,
}

/// Identity projection of a V2 node, excluding source occurrences.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeProjectionV2 {
    /// Stable node identity.
    pub node_id: CheckedNodeId,
    /// Graph schema version.
    pub schema_version: Box<str>,
    /// Closed node family tag.
    pub node_tag: Box<str>,
    /// Closed form within its node family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Nominal identity preimage when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nominal_identity_preimage: Option<NominalIdentityPreimage>,
    /// Typed semantic term.
    pub body: Value,
}

impl From<&CheckedSemanticNodeV2> for CheckedNodeProjectionV2 {
    fn from(node: &CheckedSemanticNodeV2) -> Self {
        Self {
            node_id: node.node_id.clone(),
            schema_version: node.schema_version.clone(),
            node_tag: node.node_tag.clone(),
            semantic_form: node.semantic_form.clone(),
            semantic_type: node.semantic_type.clone(),
            dependencies: node.dependencies.clone(),
            recursion_group: node.recursion_group.clone(),
            nominal_identity_preimage: node.nominal_identity_preimage.clone(),
            body: node.body.clone(),
        }
    }
}

/// One selected `sha256-jcs` domain package (QSpec `ModelRef`).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDomainPackageRef {
    /// Domain package identity.
    pub identity: Box<str>,
    /// Domain package version.
    pub version: Box<str>,
    /// Must be `sha256-jcs`.
    pub digest_domain: Box<str>,
    /// Lowercase SHA-256 of the package's RFC 8785 canonical bytes.
    pub digest: Box<str>,
}

impl CheckedDomainPackageRef {
    /// The evidence locator of this selection.
    pub fn locator(&self) -> CheckedDomainPackageLocator {
        CheckedDomainPackageLocator {
            identity: self.identity.clone(),
            version: self.version.clone(),
        }
    }
}

/// The exact immutable V2 package lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageLockV2 {
    /// Locked raw source documents.
    pub sources: Vec<CheckedArtifactRef>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Selected domain packages.
    pub model_selections: Vec<CheckedDomainPackageRef>,
    /// Required features.
    pub required_features: Vec<Box<str>>,
    /// Dependency selections.
    pub dependency_selections: Vec<CheckedSelection>,
}

/// The non-circular V2 package identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageIdentityPreimageV2 {
    /// Must be `quire.checked-package-id/v2`.
    pub version: Box<str>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Selected domain packages.
    pub model_selections: Vec<CheckedDomainPackageRef>,
    /// Required features.
    pub required_features: Vec<Box<str>>,
    /// Dependency selections.
    pub dependency_selections: Vec<CheckedSelection>,
    /// Ordered graph identity projection.
    pub identity_projection: Vec<CheckedNodeProjectionV2>,
}

/// A checked V2 graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticGraphV2 {
    /// Must be `quire.checked-semantic-graph/v2`.
    pub graph_version: Box<str>,
    /// Complete closed graph.
    pub nodes: Vec<CheckedSemanticNodeV2>,
}

/// Pipeline stage of a typed V2 diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedDiagnosticStage {
    /// `source_recognition`.
    SourceRecognition,
    /// `resolution`.
    Resolution,
    /// `type_checking`.
    TypeChecking,
    /// `package_read`.
    PackageRead,
    /// `lowering`.
    Lowering,
}

/// Closed V2 diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedDiagnosticCode {
    /// `invalid_syntax`.
    InvalidSyntax,
    /// `unsupported_construct`.
    UnsupportedConstruct,
    /// `unknown_language`.
    UnknownLanguage,
    /// `unknown_edition`.
    UnknownEdition,
    /// `unknown_profile`.
    UnknownProfile,
    /// `unknown_wire`.
    UnknownWire,
    /// `unknown_required_feature`.
    UnknownRequiredFeature,
    /// `missing_import`.
    MissingImport,
    /// `missing_declaration`.
    MissingDeclaration,
    /// `stale_dependency`.
    StaleDependency,
    /// `source_digest_mismatch`.
    SourceDigestMismatch,
    /// `ambiguous_declaration`.
    AmbiguousDeclaration,
    /// `invalid_package`.
    InvalidPackage,
    /// `invalid_model_binding`.
    InvalidModelBinding,
    /// `ill_typed`.
    IllTyped,
    /// `undefined_expression`.
    UndefinedExpression,
    /// `wrong_snapshot`.
    WrongSnapshot,
    /// `resource_exhausted`.
    ResourceExhausted,
    /// `unimplemented_capability`.
    UnimplementedCapability,
}

/// Closed V2 diagnostic cause tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedDiagnosticCause {
    /// `malformed-json`.
    MalformedJson,
    /// `missing-member`.
    MissingMember,
    /// `duplicate-member`.
    DuplicateMember,
    /// `unknown-member`.
    UnknownMember,
    /// `wrong-value-kind`.
    WrongValueKind,
    /// `invalid-value`.
    InvalidValue,
    /// `unsupported-wire`.
    UnsupportedWire,
    /// `unsupported-feature`.
    UnsupportedFeature,
    /// `revision-mismatch`.
    RevisionMismatch,
    /// `byte-digest-mismatch`.
    ByteDigestMismatch,
    /// `digest-domain-mismatch`.
    DigestDomainMismatch,
    /// `definition-cycle`.
    DefinitionCycle,
    /// `conflicting-definition`.
    ConflictingDefinition,
    /// `incompatible-definition`.
    IncompatibleDefinition,
    /// `feature-set-mismatch`.
    FeatureSetMismatch,
    /// `insufficient-next-charge`.
    InsufficientNextCharge,
}

/// One typed V2 diagnostic; never classified from message text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnosticV2 {
    /// Pipeline stage.
    pub stage: CheckedDiagnosticStage,
    /// Closed diagnostic code.
    pub code: CheckedDiagnosticCode,
    /// Closed cause tag.
    pub cause_tag: CheckedDiagnosticCause,
    /// Typed semantic-term details.
    pub details: Vec<Value>,
    /// Source loci under locked sources.
    pub loci: Vec<CheckedSourceRegion>,
}

/// Typed V2 package diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnosticsV2 {
    /// Diagnostic catalog definition.
    pub catalog: CheckedArtifactRef,
    /// Typed diagnostic records.
    pub entries: Vec<CheckedDiagnosticV2>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedPackageWireV2 {
    contract_version: Box<str>,
    identity_preimage: CheckedPackageIdentityPreimageV2,
    package_id: CheckedSemanticId,
    lock: CheckedPackageLockV2,
    semantic_graph: CheckedSemanticGraphV2,
    source_map: Vec<CheckedSourceMapEntry>,
    capability_report: Vec<CheckedCapability>,
    diagnostics: CheckedDiagnosticsV2,
}

/// The closed result of reading untrusted bytes as V2.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageV2ReadResult {
    /// A complete V2 package is safe for lowering.
    Admitted(Box<CheckedPackageV2>),
    /// The input is invalid for V2.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a caller limit.
    Incomplete(CheckedPackageIncomplete),
}

/// Immutable, admitted `quire.checked-package/v2` data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageV2 {
    wire: CheckedPackageWireV2,
    tags: Vec<CheckedNodeTag>,
}

/// Cumulative validation work against one caller limit.
#[derive(Clone, Copy, Debug)]
pub(super) struct WorkMeter {
    consumed: u64,
    limit: u64,
}

impl WorkMeter {
    pub(super) const fn new(limit: u64) -> Self {
        Self { consumed: 0, limit }
    }

    #[cfg(test)]
    pub(super) const fn consumed(&self) -> u64 {
        self.consumed
    }

    pub(super) fn charge(&mut self, work: u64) -> Result<(), ValidationFailure> {
        self.consumed = self.consumed.saturating_add(work);
        if exceeds(self.consumed, self.limit) {
            Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Work,
                self.limit,
                self.consumed,
            ))
        } else {
            Ok(())
        }
    }
}

fn refuse(code: CheckedPackageRefusalCode, path: &'static str) -> ValidationFailure {
    ValidationFailure::Refused(code, path)
}

impl CheckedPackageV2 {
    /// Reads one canonical V2 value without exposing a partial package.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        match canonical_value(bytes, limits)
            .and_then(|value| Self::admit_value(value, limits, evidence))
        {
            Ok(package) => CheckedPackageV2ReadResult::Admitted(Box::new(package)),
            Err(stop) => stop.into_result(
                CheckedPackageV2ReadResult::Refused,
                CheckedPackageV2ReadResult::Incomplete,
            ),
        }
    }

    /// Selects V2 exactly, decodes the closed wire, and validates it.
    pub(in crate::checked_package) fn admit_value(
        value: Value,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> Result<Self, Stop> {
        match value.get("contract_version") {
            Some(Value::String(version)) if version == CHECKED_PACKAGE_V2 => {}
            Some(Value::String(_)) => {
                return Err(Stop::refused(
                    CheckedPackageRefusalCode::UnknownContractVersion,
                    "contract_version",
                ))
            }
            _ => {
                return Err(Stop::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    "contract_version",
                ))
            }
        }
        let wire = decode_closed::<CheckedPackageWireV2>(value.clone())?;
        // A lossless decode: no member was defaulted, nulled or dropped.
        if serde_json::to_value(&wire).ok().as_ref() != Some(&value) {
            return Err(Stop::refused(
                CheckedPackageRefusalCode::MalformedWire,
                "document",
            ));
        }
        let tags = validate(&wire, limits, evidence)?;
        Ok(Self { wire, tags })
    }

    /// The versioned semantic package identity.
    pub fn package_id(&self) -> &CheckedSemanticId {
        &self.wire.package_id
    }

    /// The exact identity preimage.
    pub fn identity_preimage(&self) -> &CheckedPackageIdentityPreimageV2 {
        &self.wire.identity_preimage
    }

    /// The exact immutable lock.
    pub fn lock(&self) -> &CheckedPackageLockV2 {
        &self.wire.lock
    }

    /// The complete closed checked graph.
    pub fn graph(&self) -> &CheckedSemanticGraphV2 {
        &self.wire.semantic_graph
    }

    /// The parsed family of each graph node, in graph order.
    pub fn node_tags(&self) -> &[CheckedNodeTag] {
        &self.tags
    }

    /// The exact source map.
    pub fn source_map(&self) -> &[CheckedSourceMapEntry] {
        &self.wire.source_map
    }

    /// The reported capabilities.
    pub fn capability_report(&self) -> &[CheckedCapability] {
        &self.wire.capability_report
    }

    /// The typed diagnostics.
    pub fn diagnostics(&self) -> &CheckedDiagnosticsV2 {
        &self.wire.diagnostics
    }
}

fn validate(
    wire: &CheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> Result<Vec<CheckedNodeTag>, ValidationFailure> {
    if wire.contract_version.as_ref() != CHECKED_PACKAGE_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version",
        ));
    }
    if wire.package_id.domain.as_ref() != PACKAGE_DOMAIN_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "package_id.domain",
        ));
    }
    if wire.package_id.algorithm.as_ref() != "sha256" || !is_digest(&wire.package_id.digest) {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            "package_id",
        ));
    }
    if wire.identity_preimage.version.as_ref() != IDENTITY_PREIMAGE_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            "identity_preimage.version",
        ));
    }
    let preimage = serde_json::to_value(&wire.identity_preimage).map_err(|_| {
        refuse(
            CheckedPackageRefusalCode::MalformedWire,
            "identity_preimage",
        )
    })?;
    let computed = digest_json(&preimage).map_err(|_| {
        refuse(
            CheckedPackageRefusalCode::MalformedWire,
            "identity_preimage",
        )
    })?;
    if computed != wire.package_id.digest.as_ref() {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            "package_id.digest",
        ));
    }
    validate_lock(wire, evidence)?;
    let mut meter = WorkMeter::new(limits.work);
    let tags = validate_graph(wire, limits, &mut meter)?;
    validate_source_map_entries(
        wire.semantic_graph
            .nodes
            .iter()
            .map(|node| (&node.node_id, node.occurrences.as_slice())),
        &wire.source_map,
        &wire.lock.sources,
        limits,
        evidence,
    )?;
    validate_capabilities(wire, evidence)?;
    validate_diagnostics(wire, limits, &mut meter)?;
    Ok(tags)
}

fn same_non_graph_lock(
    preimage: &CheckedPackageIdentityPreimageV2,
    lock: &CheckedPackageLockV2,
) -> bool {
    preimage.edition == lock.edition
        && preimage.profile_selections == lock.profile_selections
        && preimage.definition_selections == lock.definition_selections
        && preimage.model_selections == lock.model_selections
        && preimage.required_features == lock.required_features
        && preimage.dependency_selections == lock.dependency_selections
}

fn validate_lock(
    wire: &CheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    let lock = &wire.lock;
    if lock.sources.is_empty() || !same_non_graph_lock(&wire.identity_preimage, lock) {
        return Err(refuse(CheckedPackageRefusalCode::StaleDependency, "lock"));
    }
    for source in &lock.sources {
        validate_unexported(source, SOURCE_BYTES, evidence, "lock.sources")?;
    }
    let selections = std::iter::once(&lock.edition)
        .chain(&lock.profile_selections)
        .chain(&lock.dependency_selections);
    for selection in selections {
        if !SELECTION_ROLES.contains(&selection.role.as_ref()) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                "lock.selection.role",
            ));
        }
        validate_unexported(
            &selection.definition,
            DEFINITION_BYTES,
            evidence,
            "lock.selection",
        )?;
    }
    for definition in &lock.definition_selections {
        validate_unexported(
            definition,
            DEFINITION_BYTES,
            evidence,
            "lock.definition_selections",
        )?;
    }
    // Schema `uniqueItems: true` on `model_selections` is whole-item
    // equality (identity, version, digest_domain and digest all equal), not
    // identity/version locator equality, so two entries pinning the same
    // package to different digests are distinct items here. Only the lock's
    // copy is checked: the `same_non_graph_lock` equality above already
    // requires `identity_preimage.model_selections` to equal
    // `lock.model_selections` element-for-element, so a lock free of
    // duplicates guarantees the mirrored preimage is too.
    let mut models = BTreeSet::new();
    for model in &lock.model_selections {
        validate_domain_package(model, evidence)?;
        if !models.insert(model) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                "lock.model_selections",
            ));
        }
    }
    let mut features = BTreeSet::new();
    if !lock
        .required_features
        .iter()
        .all(|feature| is_nonempty(feature) && features.insert(feature))
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            "lock.required_features",
        ));
    }
    validate_unexported(
        &wire.diagnostics.catalog,
        DEFINITION_BYTES,
        evidence,
        "diagnostics.catalog",
    )
}

fn validate_unexported(
    artifact: &CheckedArtifactRef,
    domain: &str,
    evidence: &CheckedPackageEvidence,
    path: &'static str,
) -> Result<(), ValidationFailure> {
    validate_locked_artifact(artifact, domain, evidence, path)?;
    if artifact.export.is_some() {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, path));
    }
    Ok(())
}

/// Checks one domain package selection's domain, shape and `sha256-jcs`
/// digest against the domain package evidence. Raw artifact evidence is never
/// consulted, so equal digest bytes in another domain cannot satisfy it.
fn validate_domain_package(
    model: &CheckedDomainPackageRef,
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    const PATH: &str = "lock.model_selections";
    if model.digest_domain.as_ref() != DOMAIN_PACKAGE_DIGEST {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            PATH,
        ));
    }
    if !is_nonempty(&model.identity) || !is_nonempty(&model.version) || !is_digest(&model.digest) {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, PATH));
    }
    if evidence.domain_package_digest(&model.locator()) != Some(model.digest.as_ref()) {
        return Err(refuse(CheckedPackageRefusalCode::StaleDependency, PATH));
    }
    Ok(())
}

fn validate_node_id(id: &CheckedNodeId, path: &'static str) -> Result<(), ValidationFailure> {
    if id.domain.as_ref() == NODE_DOMAIN && is_digest(&id.digest) {
        Ok(())
    } else {
        Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            path,
        ))
    }
}

fn validate_graph(
    wire: &CheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<Vec<CheckedNodeTag>, ValidationFailure> {
    let graph = &wire.semantic_graph;
    if graph.graph_version.as_ref() != GRAPH_V2 || graph.nodes.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph",
        ));
    }
    let node_count = count(graph.nodes.len());
    if exceeds(node_count, limits.nodes) {
        return Err(ValidationFailure::Incomplete(
            CheckedPackageLimit::Nodes,
            limits.nodes,
            node_count,
        ));
    }
    let mut index = BTreeMap::new();
    let mut tags = Vec::with_capacity(graph.nodes.len());
    let mut references = Vec::with_capacity(graph.nodes.len());
    let mut edges = 0_u64;
    for (position, node) in graph.nodes.iter().enumerate() {
        validate_node_id(&node.node_id, "semantic_graph.nodes.node_id")?;
        if index.insert(&node.node_id, position).is_some() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.node_id",
            ));
        }
        if node.schema_version.as_ref() != GRAPH_V2 {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.schema_version",
            ));
        }
        let Some(tag) = CheckedNodeTag::from_wire(&node.node_tag) else {
            return Err(refuse(
                CheckedPackageRefusalCode::UnsupportedNodeTag,
                "semantic_graph.nodes.node_tag",
            ));
        };
        if !tag.forms().contains(&node.semantic_form.as_ref()) {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.semantic_form",
            ));
        }
        tags.push(tag);
        validate_node_id(&node.semantic_type, "semantic_graph.nodes.semantic_type")?;
        for dependency in &node.dependencies {
            validate_node_id(dependency, "semantic_graph.nodes.dependencies")?;
        }
        if node.recursion_group.as_deref().is_some_and(str::is_empty) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                "semantic_graph.nodes.recursion_group",
            ));
        }
        edges = edges.saturating_add(count(node.dependencies.len()));
        if exceeds(edges, limits.edges) {
            return Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Edges,
                limits.edges,
                edges,
            ));
        }
        let mut targets = Vec::new();
        let work = validate_term(&node.body, TermGrammar::V2, &mut |target| {
            targets.push(target.clone())
        })?;
        meter.charge(work)?;
        references.push(targets);
        let mut occurrences = BTreeSet::new();
        if node.occurrences.is_empty()
            || !node
                .occurrences
                .iter()
                .all(|occurrence| occurrences.insert(occurrence))
        {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                "semantic_graph.nodes.occurrences",
            ));
        }
    }
    let mut adjacency = Vec::with_capacity(graph.nodes.len());
    for (position, (node, targets)) in graph.nodes.iter().zip(&references).enumerate() {
        let resolve = |id: &CheckedNodeId, path| {
            index.get(id).copied().ok_or(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                path,
            ))
        };
        let semantic_type = resolve(&node.semantic_type, "semantic_graph.nodes.semantic_type")?;
        let mut successors = Vec::new();
        if semantic_type != position {
            successors.push(semantic_type);
        }
        for dependency in &node.dependencies {
            successors.push(resolve(dependency, "semantic_graph.nodes.dependencies")?);
        }
        for target in targets {
            successors.push(resolve(target, "semantic_graph.nodes.body.target")?);
        }
        adjacency.push(successors);
    }
    validate_nominal_nodes(&graph.nodes, &tags, &index, &wire.lock, meter)?;
    validate_recursion(&graph.nodes, &adjacency, meter)?;
    let projection = graph
        .nodes
        .iter()
        .map(CheckedNodeProjectionV2::from)
        .collect::<Vec<_>>();
    if projection != wire.identity_preimage.identity_projection {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            "identity_preimage.identity_projection",
        ));
    }
    Ok(tags)
}

/// Every strongly connected component that forms a cycle must share one
/// explicit `recursion_group`. Iterative Tarjan; each edge costs one work.
fn validate_recursion(
    nodes: &[CheckedSemanticNodeV2],
    adjacency: &[Vec<usize>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    const UNVISITED: usize = usize::MAX;
    let size = adjacency.len();
    let mut order = vec![UNVISITED; size];
    let mut low = vec![0_usize; size];
    let mut on_stack = vec![false; size];
    let mut stack = Vec::new();
    let mut calls: Vec<(usize, usize)> = Vec::new();
    let mut next = 0_usize;
    for root in 0..size {
        if order.get(root) != Some(&UNVISITED) {
            continue;
        }
        enter(
            root,
            &mut next,
            &mut order,
            &mut low,
            &mut on_stack,
            &mut stack,
        );
        calls.push((root, 0));
        while let Some((vertex, edge)) = calls.last().copied() {
            let successor = adjacency
                .get(vertex)
                .and_then(|edges| edges.get(edge))
                .copied();
            if let Some(successor) = successor {
                meter.charge(1)?;
                if let Some(frame) = calls.last_mut() {
                    frame.1 = edge.saturating_add(1);
                }
                if order.get(successor) == Some(&UNVISITED) {
                    enter(
                        successor,
                        &mut next,
                        &mut order,
                        &mut low,
                        &mut on_stack,
                        &mut stack,
                    );
                    calls.push((successor, 0));
                } else if on_stack.get(successor) == Some(&true) {
                    let reached = order.get(successor).copied().unwrap_or(UNVISITED);
                    if let Some(current) = low.get_mut(vertex) {
                        *current = (*current).min(reached);
                    }
                }
                continue;
            }
            calls.pop();
            let vertex_low = low.get(vertex).copied().unwrap_or(0);
            if let Some((parent, _)) = calls.last() {
                if let Some(parent_low) = low.get_mut(*parent) {
                    *parent_low = (*parent_low).min(vertex_low);
                }
            }
            if order.get(vertex) != Some(&vertex_low) {
                continue;
            }
            let mut component = Vec::new();
            while let Some(member) = stack.pop() {
                if let Some(flag) = on_stack.get_mut(member) {
                    *flag = false;
                }
                component.push(member);
                if member == vertex {
                    break;
                }
            }
            let cyclic = component.len() > 1
                || adjacency
                    .get(vertex)
                    .is_some_and(|edges| edges.contains(&vertex));
            if cyclic {
                let mut groups = component.iter().map(|member| {
                    nodes
                        .get(*member)
                        .and_then(|node| node.recursion_group.as_ref())
                });
                let first = groups.next().flatten();
                if first.is_none() || !groups.all(|group| group == first) {
                    return Err(refuse(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        "semantic_graph.nodes.recursion_group",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn enter(
    vertex: usize,
    next: &mut usize,
    order: &mut [usize],
    low: &mut [usize],
    on_stack: &mut [bool],
    stack: &mut Vec<usize>,
) {
    if let (Some(slot), Some(low_slot), Some(flag)) = (
        order.get_mut(vertex),
        low.get_mut(vertex),
        on_stack.get_mut(vertex),
    ) {
        *slot = *next;
        *low_slot = *next;
        *flag = true;
    }
    *next = next.saturating_add(1);
    stack.push(vertex);
}

fn validate_capabilities(
    wire: &CheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    let mut reported = BTreeMap::new();
    for capability in &wire.capability_report {
        if !DISPOSITIONS.contains(&capability.disposition.as_ref()) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                "capability_report.disposition",
            ));
        }
        if !is_nonempty(&capability.feature)
            || reported
                .insert(capability.feature.as_ref(), capability.disposition.as_ref())
                .is_some()
        {
            return Err(refuse(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "capability_report",
            ));
        }
    }
    for feature in &wire.lock.required_features {
        if reported.get(feature.as_ref()) != Some(&"available") || !evidence.supports(feature) {
            return Err(refuse(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "capability_report",
            ));
        }
    }
    Ok(())
}

fn validate_diagnostics(
    wire: &CheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let diagnostics = count(wire.diagnostics.entries.len());
    if exceeds(diagnostics, limits.diagnostics) {
        return Err(ValidationFailure::Incomplete(
            CheckedPackageLimit::Diagnostics,
            limits.diagnostics,
            diagnostics,
        ));
    }
    let nodes = wire
        .semantic_graph
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<BTreeSet<_>>();
    for entry in &wire.diagnostics.entries {
        for detail in &entry.details {
            let mut resolved = true;
            let work = validate_term(detail, TermGrammar::V2, &mut |target| {
                resolved &= nodes.contains(target);
            })?;
            meter.charge(work)?;
            if !resolved {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    "diagnostics.entries.details",
                ));
            }
        }
        for region in &entry.loci {
            if !wire.lock.sources.contains(&region.source) || region.start >= region.end {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    "diagnostics.entries.loci",
                ));
            }
        }
    }
    Ok(())
}
