//! Strict reader for QSpec I04 `quire.checked-package/v2`, the sole admitted
//! `CheckedPackage` contract.
//!
//! Implements the I04 contract described by AD-006. The wire shape this
//! module admits is stated here, in the types and checks below, rather than
//! by pointing at a revision of another repository. Model selections are
//! `sha256-jcs` domain packages, typed separately from the raw source and
//! definition byte artifacts.

mod identity;
mod lower;
mod natural;
mod operation_catalog;
mod operations;
mod structural;
mod vocabulary;

pub use identity::*;
pub use lower::*;
pub use vocabulary::*;

use operations::{validate_application_keys, validate_operations};
use structural::validate_structural_nodes;

use super::common::{
    canonical_value, count, decode_closed, digest_json, exact_members, exceeds,
    first_difference, is_digest, is_nonempty, node_pointer, validate_locked_artifact,
    validate_source_map_entries, validate_term, visit_reference, ReferenceMember, ReferenceSite,
    ReferenceVisitor, Step, TermGrammar, Trail, ValidationFailure, NODE_DOMAIN,
};
use super::evidence::{CheckedDomainPackageLocator, CheckedPackageEvidence};
use super::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSelection, CheckedSemanticId,
    CheckedSourceMapEntry, CheckedSourceRegion, JsonPointer,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

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
/// A node's declared qualified name (FR-208). Present exactly where the
/// schema's `DeclarationOccurrenceRule` requires it and forbidden where
/// `DeclarationTagRules` forbids it; see [`validate_declaration`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDeclaration {
    /// ASCII identifier segments.
    pub qualified_name: Vec<Box<str>>,
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
    /// Declared qualified name; present exactly per `DeclarationOccurrenceRule`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration: Option<CheckedDeclaration>,
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
    /// Declared qualified name when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration: Option<CheckedDeclaration>,
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
            declaration: node.declaration.clone(),
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
    kinds: Vec<CheckedNodeKind>,
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

    /// Charges `work` for the value at `at`, which an exhausted budget
    /// reports as the value whose charge failed.
    pub(super) fn charge(
        &mut self,
        work: u64,
        at: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
        self.consumed = self.consumed.saturating_add(work);
        if exceeds(self.consumed, self.limit) {
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Work,
                self.limit,
                self.consumed,
                Some(at()),
            ))
        } else {
            Ok(())
        }
    }
}

fn refuse(code: CheckedPackageRefusalCode, path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(code, path)
}

/// A refusal located at a specific graph node, carrying the cause this stage
/// determined (if any) and the node key of the offending entry or node.
fn refuse_at(
    code: CheckedPackageRefusalCode,
    path: JsonPointer,
    cause: Option<CheckedPackageRefusalCause>,
    locus: CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, path, cause, locus)
}

/// The pointer of a fixed member path from the document root.
fn member_pointer(keys: &[&str]) -> JsonPointer {
    keys.iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key))
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
    ) -> Result<Self, ValidationFailure> {
        match value.get("contract_version") {
            Some(Value::String(version)) if version == CHECKED_PACKAGE_V2 => {}
            Some(Value::String(version)) => {
                return Err(ValidationFailure::unknown_contract_version(version))
            }
            Some(_) => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    member_pointer(&["contract_version"]),
                ))
            }
            // Absent, or the document is not an object: the document is the
            // value at fault.
            None => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    JsonPointer::root(),
                ))
            }
        }
        let wire = decode_closed::<CheckedPackageWireV2>(&value)?;
        // A lossless decode: no member was defaulted, nulled or dropped.
        match serde_json::to_value(&wire) {
            Ok(decoded) if decoded == value => {}
            Ok(decoded) => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    first_difference(JsonPointer::root(), &value, &decoded),
                ))
            }
            Err(_) => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    JsonPointer::root(),
                ))
            }
        }
        let kinds = validate(&wire, limits, evidence)?;
        Ok(Self { wire, kinds })
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

    /// Each graph node's decoded family and form, in graph order.
    pub fn node_kinds(&self) -> &[CheckedNodeKind] {
        &self.kinds
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
) -> Result<Vec<CheckedNodeKind>, ValidationFailure> {
    if wire.contract_version.as_ref() != CHECKED_PACKAGE_V2 {
        return Err(ValidationFailure::unknown_contract_version(
            &wire.contract_version,
        ));
    }
    if wire.package_id.domain.as_ref() != PACKAGE_DOMAIN_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            member_pointer(&["package_id", "domain"]),
        ));
    }
    if wire.package_id.algorithm.as_ref() != "sha256" {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["package_id", "algorithm"]),
        ));
    }
    if !is_digest(&wire.package_id.digest) {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["package_id", "digest"]),
        ));
    }
    if wire.identity_preimage.version.as_ref() != IDENTITY_PREIMAGE_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["identity_preimage", "version"]),
        ));
    }
    let unserializable = |_| {
        refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["identity_preimage"]),
        )
    };
    let preimage = serde_json::to_value(&wire.identity_preimage).map_err(unserializable)?;
    let computed = digest_json(&preimage).map_err(unserializable)?;
    if computed != wire.package_id.digest.as_ref() {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            member_pointer(&["package_id", "digest"]),
        ));
    }
    validate_lock(wire, evidence)?;
    let mut meter = WorkMeter::new(limits.work);
    let kinds = validate_graph(wire, limits, &mut meter)?;
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
    Ok(kinds)
}

/// The pointer of the first lock value that differs from its mirror in the
/// identity preimage, or `None` when every mirrored member is equal.
fn non_graph_lock_difference(
    preimage: &CheckedPackageIdentityPreimageV2,
    lock: &CheckedPackageLockV2,
) -> Option<JsonPointer> {
    fn differing<T: PartialEq + Serialize>(
        member: &str,
        lock: &T,
        preimage: &T,
    ) -> Option<JsonPointer> {
        if lock == preimage {
            return None;
        }
        let at = member_pointer(&["lock", member]);
        Some(
            match (serde_json::to_value(lock), serde_json::to_value(preimage)) {
                (Ok(lock), Ok(preimage)) => first_difference(at, &lock, &preimage),
                _ => at,
            },
        )
    }
    differing("edition", &lock.edition, &preimage.edition)
        .or_else(|| {
            differing(
                "profile_selections",
                &lock.profile_selections,
                &preimage.profile_selections,
            )
        })
        .or_else(|| {
            differing(
                "definition_selections",
                &lock.definition_selections,
                &preimage.definition_selections,
            )
        })
        .or_else(|| {
            differing(
                "model_selections",
                &lock.model_selections,
                &preimage.model_selections,
            )
        })
        .or_else(|| {
            differing(
                "required_features",
                &lock.required_features,
                &preimage.required_features,
            )
        })
        .or_else(|| {
            differing(
                "dependency_selections",
                &lock.dependency_selections,
                &preimage.dependency_selections,
            )
        })
}

fn validate_lock(
    wire: &CheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    let lock = &wire.lock;
    if lock.sources.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            member_pointer(&["lock", "sources"]),
        ));
    }
    if let Some(path) = non_graph_lock_difference(&wire.identity_preimage, lock) {
        return Err(refuse(CheckedPackageRefusalCode::StaleDependency, path));
    }
    for (index, source) in lock.sources.iter().enumerate() {
        validate_unexported(source, SOURCE_BYTES, evidence, &|| {
            member_pointer(&["lock", "sources"]).index(index)
        })?;
    }
    validate_unexported(&lock.edition.definition, DEFINITION_BYTES, evidence, &|| {
        member_pointer(&["lock", "edition", "definition"])
    })?;
    for (member, selections) in [
        ("profile_selections", &lock.profile_selections),
        ("dependency_selections", &lock.dependency_selections),
    ] {
        for (index, selection) in selections.iter().enumerate() {
            validate_unexported(&selection.definition, DEFINITION_BYTES, evidence, &|| {
                member_pointer(&["lock", member])
                    .index(index)
                    .key("definition")
            })?;
        }
    }
    for (index, definition) in lock.definition_selections.iter().enumerate() {
        validate_unexported(definition, DEFINITION_BYTES, evidence, &|| {
            member_pointer(&["lock", "definition_selections"]).index(index)
        })?;
    }
    // Whole-array uniqueness is checked before any entry's digest is
    // evaluated against evidence: a `model_selections` array that repeats an
    // entry (identity, version, digest_domain and digest all equal —
    // whole-item equality, matching the schema's `uniqueItems`; two entries
    // pinning the same package to different digests are distinct items) is a
    // defect in the shape of the wire, and evidence for an already-malformed
    // array is not meaningful to check. This order — not the reverse, and not
    // interleaved per entry — is what keeps the outcome for an array carrying
    // both defects independent of which one comes first in the array.
    // Class 1 (repeated entry) of the five classes FR-038 orders this array's
    // refusal code by; class 2 (same identity, different version) is swept
    // immediately below, and the remaining three, in their own stated order,
    // by `validate_domain_packages` below — so the code is decided by defect
    // class throughout and never by array position. Only the lock's copy is
    // checked: the `same_non_graph_lock` equality above already requires
    // `identity_preimage.model_selections` to equal `lock.model_selections`
    // element-for-element, so a duplicate-free lock guarantees the mirrored
    // preimage is too.
    let models_pointer = || member_pointer(&["lock", "model_selections"]);
    let mut models = BTreeSet::new();
    if let Some(repeat) = lock
        .model_selections
        .iter()
        .position(|model| !models.insert(model))
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            models_pointer().index(repeat),
        ));
    }
    // Class 2: two selections naming the same identity at different
    // versions. The nominal `Model` owner (`identity::validate_owner`) joins
    // `lock.model_selections` by identity alone, so the lock must guarantee
    // at most one selection per identity for that join to stay unambiguous
    // (FR-038 carries the full rationale, including its scope: this rule
    // guarantees at most one selection per identity within one lock, not a
    // version-independent node key across packages, which is a separate,
    // tracked concern). A same-identity, same-version pair differing only in
    // digest is not this class — it shares one locator, so class 5 below
    // already refuses it deterministically as `stale_dependency` per
    // FR-038-AC-10, and this check does not widen to reach it. Swept over the
    // whole array before any per-entry check, so the outcome does not depend
    // on array position (FR-038-AC-11), and before `validate_domain_packages`
    // below, so this class outranks classes 3 and 5 (FR-038-AC-20).
    let mut model_versions: BTreeMap<&str, &str> = BTreeMap::new();
    for (index, model) in lock.model_selections.iter().enumerate() {
        match model_versions.entry(model.identity.as_ref()) {
            Entry::Occupied(entry) if *entry.get() != model.version.as_ref() => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    models_pointer().index(index).key("version"),
                ));
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(model.version.as_ref());
            }
        }
    }
    validate_domain_packages(&lock.model_selections, evidence)?;
    let mut features = BTreeSet::new();
    if let Some(index) = lock
        .required_features
        .iter()
        .position(|feature| !(is_nonempty(feature) && features.insert(feature)))
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "required_features"]).index(index),
        ));
    }
    validate_unexported(&wire.diagnostics.catalog, DEFINITION_BYTES, evidence, &|| {
        member_pointer(&["diagnostics", "catalog"])
    })
}

/// Checks one locked raw artifact at `at` and that it carries no `export`.
fn validate_unexported(
    artifact: &CheckedArtifactRef,
    domain: &str,
    evidence: &CheckedPackageEvidence,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    validate_locked_artifact(artifact, domain, evidence, at)?;
    if artifact.export.is_some() {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            at().key("export"),
        ));
    }
    Ok(())
}

/// Checks every domain package selection's domain, shape and `sha256-jcs`
/// digest against the domain package evidence. Raw artifact evidence is never
/// consulted, so equal digest bytes in another domain cannot satisfy it.
///
/// Checks classes 3 through 5 of FR-038's five-class `model_selections`
/// order. Each check sweeps the whole array before the next one begins, so
/// the refusal an array carrying two different defects draws is decided by
/// defect class and not by which defective entry the reader reaches first:
/// declared-domain mismatch (`digest_domain_mismatch`, class 3) outranks a
/// shape defect (`malformed_wire`, class 4), which outranks a digest the
/// evidence does not attest (`stale_dependency`, class 5). Classes 1
/// (repeated entry) and 2 (same identity, different version) are checked by
/// the caller before any of these. Array position never decides the code;
/// within the refusing class, the pointer names the first entry of that class
/// in array order, at the member at fault.
fn validate_domain_packages(
    models: &[CheckedDomainPackageRef],
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    let at = |index: usize, member: &str| {
        member_pointer(&["lock", "model_selections"])
            .index(index)
            .key(member)
    };
    if let Some(index) = models
        .iter()
        .position(|model| model.digest_domain.as_ref() != DOMAIN_PACKAGE_DIGEST)
    {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            at(index, "digest_domain"),
        ));
    }
    let malformed = models.iter().enumerate().find_map(|(index, model)| {
        if !is_nonempty(&model.identity) {
            Some(at(index, "identity"))
        } else if !is_nonempty(&model.version) {
            Some(at(index, "version"))
        } else if !is_digest(&model.digest) {
            Some(at(index, "digest"))
        } else {
            None
        }
    });
    if let Some(path) = malformed {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, path));
    }
    if let Some(index) = models.iter().position(|model| {
        evidence.domain_package_digest(&model.locator()) != Some(model.digest.as_ref())
    }) {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            at(index, "digest"),
        ));
    }
    Ok(())
}

/// Requires a node key in the node domain with a lowercase digest; `at`
/// names the key.
fn validate_node_id(
    id: &CheckedNodeId,
    at: impl FnOnce() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let member = if id.domain.as_ref() != NODE_DOMAIN {
        "domain"
    } else if !is_digest(&id.digest) {
        "digest"
    } else {
        return Ok(());
    };
    Err(refuse(
        CheckedPackageRefusalCode::DigestDomainMismatch,
        at().key(member),
    ))
}

/// Enforces the schema's `DeclarationTagRules` and `DeclarationOccurrenceRule`
/// (FR-208): `declaration` is forbidden on `expression`, `relation`, `state`,
/// `temporal` and `correspondence` nodes and on `value`/`enum_value` nodes;
/// for every other family it is present exactly when the node carries a
/// `declaration`-role occurrence.
fn validate_declaration(
    kind: CheckedNodeKind,
    occurrences: &[CheckedOccurrence],
    declaration: Option<&CheckedDeclaration>,
    position: usize,
) -> Result<(), ValidationFailure> {
    let forced_absent = declaration_forbidden(kind);
    let required = !forced_absent
        && occurrences
            .iter()
            .any(|occurrence| occurrence.role == CheckedOccurrenceRole::Declaration);
    match (required, declaration) {
        (true, None) => Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position),
        )),
        (false, Some(_)) => Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position).key("declaration"),
        )),
        (true, Some(declaration)) => {
            identity::validate_qualified_name(&declaration.qualified_name, &|| {
                node_pointer(position)
                    .key("declaration")
                    .key("qualified_name")
            })
        }
        (false, None) => Ok(()),
    }
}

/// `DeclarationTagRules`: whether a node of this kind may never carry a
/// `declaration`. The rule is stated per family, except `value`/`enum_value`,
/// but every form is listed so a new form is a compile error here until its
/// rule is decided.
fn declaration_forbidden(kind: CheckedNodeKind) -> bool {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum,
        ) => false,
        // QSL FR-094: a compound unit is anonymous and never declares.
        K::ScalarType(ScalarTypeForm::CompoundUnit) => true,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => false,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => false,
        // QSL FR-092: a parameter names its binder's value; it declares no
        // name of its own.
        K::Value(ValueForm::EnumValue | ValueForm::Parameter) => true,
        K::Value(
            ValueForm::Literal
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::OptionValue,
        ) => false,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => true,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => false,
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ObjectType
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::FieldDeclaration
            | ModelForm::OperationDeclaration
            | ModelForm::ClauseMemberDeclaration
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => false,
        K::Relation(
            RelationForm::Relationship
            | RelationForm::Population
            | RelationForm::Membership
            | RelationForm::CausalRelation,
        ) => true,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => true,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline,
        ) => true,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => false,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => false,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence,
        ) => true,
    }
}

/// Minimal structural admission for one node body. Every node validates as
/// the closed `SemanticTerm` grammar, except a `state`/`frame` node, whose
/// `BodyBindingRules`-selected shape is the closed reference triple
/// `{term: "frame", modifies, creates, deletes}`, each member a `uniqueItems`
/// array per the wire schema. This parses and rejects a repeated entry within
/// one member, but reports no reference targets to the caller: a frame body's
/// declaration-level meaning comes from `dependencies`, not from independent
/// successor edges, so its entries never resolve through the generic
/// reference mechanism `validate_graph`'s Loop 2 uses for every other body
/// (which would refuse an entry that fails to resolve as
/// `invalid_semantic_graph`, the wrong code for FR-340's `missing_declaration`).
/// Entry eligibility, canonical member order and refusal precedence across a
/// frame's own violations are [`validate_frame_semantics`]'s job, run once
/// the whole graph's identity and dependency edges are known (FR-340).
/// `at` is the body's own position, `/semantic_graph/nodes/{n}/body`.
fn validate_body(
    kind: CheckedNodeKind,
    body: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    if is_frame(kind) {
        validate_frame_body(body, at)
    } else {
        // `body` is the node's own top-level term, never a nested one, so
        // this is the one call in the module that reports `is_body_root: true`.
        validate_term(body, TermGrammar::V2, true, at, visit)
    }
}

/// `BodyBindingRules`: only a `state`/`frame` node's body is the frame
/// reference triple rather than a `SemanticTerm`.
fn is_frame(kind: CheckedNodeKind) -> bool {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum
            | ScalarTypeForm::CompoundUnit,
        ) => false,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => false,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => false,
        K::Value(
            ValueForm::Literal
            | ValueForm::EnumValue
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => false,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => false,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => false,
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ObjectType
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::FieldDeclaration
            | ModelForm::OperationDeclaration
            | ModelForm::ClauseMemberDeclaration
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => false,
        K::Relation(
            RelationForm::Relationship
            | RelationForm::Population
            | RelationForm::Membership
            | RelationForm::CausalRelation,
        ) => false,
        K::State(StateForm::Frame) => true,
        K::State(
            StateForm::StateClause
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => false,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline,
        ) => false,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => false,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => false,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence,
        ) => false,
    }
}

fn validate_frame_body(body: &Value, at: &Trail<'_>) -> Result<u64, ValidationFailure> {
    let Value::Object(object) = body else {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        ));
    };
    if !exact_members(object, &["term", "modifies", "creates", "deletes"]) {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        ));
    }
    if object.get("term").and_then(Value::as_str) != Some("frame") {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.key("term").pointer(),
        ));
    }
    let mut work = 1_u64;
    for member in FrameMember::ALL {
        let key = member.wire_key();
        work = work.saturating_add(visit_node_refs(object.get(key), &at.key(key))?);
    }
    Ok(work)
}

/// Validates one frame reference array at `at` (`modifies`, `creates` or
/// `deletes`): every entry a well-formed node key and none repeated. A
/// repeated entry refuses at its second occurrence. Every entry's shape is
/// checked before any repeat, so a shape defect anywhere outranks a repeat.
/// A frame body reports no reference targets to its caller: its
/// declaration-level meaning comes from `dependencies` (FR-340).
fn visit_node_refs(value: Option<&Value>, at: &Trail<'_>) -> Result<u64, ValidationFailure> {
    let Some(Value::Array(values)) = value else {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        ));
    };
    let mut targets = Vec::with_capacity(values.len());
    let work = values
        .iter()
        .enumerate()
        .try_fold(0_u64, |work, (index, entry)| {
            visit_reference(
                entry,
                ReferenceMember::FrameEntry,
                false,
                &at.index(index),
                &mut |target, _site, _at| targets.push(target.clone()),
            )
            .map(|charged| work.saturating_add(charged))
        })?;
    let mut seen = BTreeSet::new();
    if let Some(repeat) = targets.into_iter().position(|target| !seen.insert(target)) {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.index(repeat).pointer(),
        ));
    }
    Ok(work)
}

/// The three frame body members, in the body's own member order — also the
/// tie-break order FR-340 uses when meaning-join defects tie across members
/// (`modifies` first, then `creates`, then `deletes`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameMember {
    Modifies,
    Creates,
    Deletes,
}

impl FrameMember {
    const ALL: [Self; 3] = [Self::Modifies, Self::Creates, Self::Deletes];

    const fn wire_key(self) -> &'static str {
        match self {
            Self::Modifies => "modifies",
            Self::Creates => "creates",
            Self::Deletes => "deletes",
        }
    }

    /// FR-340's closed eligibility table (FR-340-AC-1 through AC-4 upstream):
    /// the single source of truth for which (member, node kind) pairs a frame
    /// entry may name. [`frame_eligibility`] decides every form of every
    /// family with no wildcard, so a new form fails to compile until its
    /// eligibility is decided.
    fn admits(self, kind: CheckedNodeKind) -> bool {
        let eligibility = frame_eligibility(kind);
        match self {
            Self::Modifies => eligibility.modifies,
            Self::Creates | Self::Deletes => eligibility.creates_or_deletes,
        }
    }
}

/// Which frame members may name a node of one kind.
struct FrameEligibility {
    modifies: bool,
    creates_or_deletes: bool,
}

/// FR-340's eligibility for every form: `modifies` takes a relation's
/// `relationship` or a model's `field_declaration`; `creates` and `deletes`
/// take a model's `object_type` or `process`. Nothing else is eligible.
fn frame_eligibility(kind: CheckedNodeKind) -> FrameEligibility {
    use CheckedNodeKind as K;
    const NONE: FrameEligibility = FrameEligibility {
        modifies: false,
        creates_or_deletes: false,
    };
    const MODIFIES: FrameEligibility = FrameEligibility {
        modifies: true,
        creates_or_deletes: false,
    };
    const CREATES_OR_DELETES: FrameEligibility = FrameEligibility {
        modifies: false,
        creates_or_deletes: true,
    };
    match kind {
        K::Relation(RelationForm::Relationship) => MODIFIES,
        K::Relation(
            RelationForm::Population | RelationForm::Membership | RelationForm::CausalRelation,
        ) => NONE,
        K::Model(ModelForm::FieldDeclaration) => MODIFIES,
        K::Model(ModelForm::ObjectType | ModelForm::Process) => CREATES_OR_DELETES,
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::OperationDeclaration
            | ModelForm::ClauseMemberDeclaration
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => NONE,
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum
            | ScalarTypeForm::CompoundUnit,
        ) => NONE,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => NONE,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => NONE,
        K::Value(
            ValueForm::Literal
            | ValueForm::EnumValue
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => NONE,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => NONE,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => NONE,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => NONE,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline,
        ) => NONE,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => NONE,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => NONE,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence,
        ) => NONE,
    }
}

/// One frame body member's entries in wire order (neither deduplicated nor
/// reordered), so [`frame_defect`] can check FR-340's canonical ascending-
/// digest order. `validate_frame_body` (Loop 1, earlier in `validate_graph`)
/// already required this member to be a `uniqueItems` array of well-formed
/// `NodeRef` objects before the frame stage is ever reached, so this reparse
/// of the already-admitted body cannot fail.
fn frame_entries(body: &Value, key: &str) -> Vec<CheckedNodeId> {
    body.get(key)
        .and_then(Value::as_array)
        .expect("frame body member shape already validated by validate_frame_body")
        .iter()
        .map(|entry| {
            serde_json::from_value(entry.clone())
                .expect("frame body entry shape already validated by validate_frame_body")
        })
        .collect()
}

/// One meaning-join defect found while scanning a frame's body: an entry
/// naming no declared dependency of the frame (including one declared but
/// resolving to no real node), or a declared dependency of a meaning its
/// member does not admit. `member_index` and `digest` together are exactly
/// FR-340's precedence sort key — member group first, then ascending entry
/// digest — kept as their own fields (not derived from `locus`/`path`) so the
/// sort in [`frame_defect`] cannot silently drift from the fields it reports.
/// `member` and `entry_index` locate the entry in the wire body.
struct MeaningDefect {
    member_index: u8,
    digest: Box<str>,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    locus: CheckedNodeId,
    member: FrameMember,
    entry_index: usize,
}

/// The single refusal FR-340 selects for one frame node's body, or `None`
/// when the body is admitted. `frame_id` and `frame` are the same node;
/// `frame_id` is threaded separately because it is the locus of a canonical-
/// order defect, while a meaning-join defect's locus is the offending entry.
///
/// Collects every meaning-join defect (an entry naming no declared
/// dependency of the frame, or a declared dependency of a meaning its member
/// does not admit) across all three members, and separately whether any
/// member's wire order is not strictly ascending by entry digest. Meaning-
/// join defects always outrank a canonical-order defect; among meaning-join
/// defects, `(member order, ascending entry digest)` — exactly the sort key
/// [`MeaningDefect`] carries — selects the one FR-340 reports, so the outcome
/// never depends on which member an author wrote a defect into, where in its
/// array an entry sits, or the order this function happens to collect
/// defects in.
fn frame_defect(
    frame_id: &CheckedNodeId,
    frame: &CheckedSemanticNodeV2,
    frame_position: usize,
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<ValidationFailure> {
    let declared: BTreeSet<&CheckedNodeId> = frame.dependencies.iter().collect();
    let mut order_defect = false;
    let mut meaning_defects: Vec<MeaningDefect> = Vec::new();
    for (member_index, member) in FrameMember::ALL.into_iter().enumerate() {
        let entries = frame_entries(&frame.body, member.wire_key());
        if entries
            .windows(2)
            .any(|pair| pair[0].digest >= pair[1].digest)
        {
            order_defect = true;
        }
        for (entry_index, entry) in entries.into_iter().enumerate() {
            // `FrameMember::ALL` has exactly 3 members, so `enumerate()`
            // never reaches a value `u8` cannot hold.
            let member_index = u8::try_from(member_index).expect("FrameMember::ALL has 3 members");
            if !declared.contains(&entry) {
                meaning_defects.push(MeaningDefect {
                    member_index,
                    digest: entry.digest.clone(),
                    code: CheckedPackageRefusalCode::MissingDeclaration,
                    cause: CheckedPackageRefusalCause::MissingName,
                    locus: entry,
                    member,
                    entry_index,
                });
                continue;
            }
            // FR-340 resolves each declared entry itself rather than relying
            // on the graph's generic dependency-edge resolution (which runs
            // later, see `validate_graph`): a frame dependency naming no
            // real node is `missing_declaration` exactly like an entry the
            // frame never declared at all, not the generic
            // `invalid_semantic_graph` an unresolved edge would otherwise be.
            let Some(&position) = index.get(&entry) else {
                meaning_defects.push(MeaningDefect {
                    member_index,
                    digest: entry.digest.clone(),
                    code: CheckedPackageRefusalCode::MissingDeclaration,
                    cause: CheckedPackageRefusalCause::MissingName,
                    locus: entry,
                    member,
                    entry_index,
                });
                continue;
            };
            if !member.admits(kinds[position]) {
                meaning_defects.push(MeaningDefect {
                    member_index,
                    digest: entry.digest.clone(),
                    code: CheckedPackageRefusalCode::InvalidModelBinding,
                    cause: CheckedPackageRefusalCause::MalformedDeclaration,
                    locus: entry,
                    member,
                    entry_index,
                });
            }
        }
    }
    if !meaning_defects.is_empty() {
        meaning_defects.sort_by(|left, right| {
            (left.member_index, &left.digest).cmp(&(right.member_index, &right.digest))
        });
        let winner = meaning_defects
            .into_iter()
            .next()
            .expect("checked nonempty above");
        return Some(refuse_at(
            winner.code,
            node_pointer(frame_position)
                .key("body")
                .key(winner.member.wire_key())
                .index(winner.entry_index),
            Some(winner.cause),
            winner.locus,
        ));
    }
    if order_defect {
        return Some(refuse_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(frame_position).key("body"),
            None,
            frame_id.clone(),
        ));
    }
    None
}

/// FR-322's declaration-name refusals, after the nominal checks and before
/// frame and operation checks. First, a node carrying both a `declaration` and
/// a nominal preimage that fixes a name must declare exactly that name, else
/// `invalid_package` / `declaration-nominal-mismatch`. Then no two nodes may
/// declare the same `qualified_name`, else `ambiguous_declaration` /
/// `ambiguous-name`. Each sweep reports at the first offending node in
/// ascending node-id digest order, the iteration order of `index`; for an
/// ambiguous name that is the lowest-digest node sharing it. FR-322 orders
/// both before any operation refusal but not against each other: this reader
/// checks the nominal join first. Takes no work meter: every name compared
/// here was already charged when `validate_graph`'s per-node loop admitted it.
fn validate_declaration_names(
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let name_pointer = |position: usize| {
        node_pointer(position)
            .key("declaration")
            .key("qualified_name")
    };
    for (&node_id, &position) in index {
        let node = &nodes[position];
        let (Some(declaration), Some(preimage)) =
            (&node.declaration, &node.nominal_identity_preimage)
        else {
            continue;
        };
        if preimage
            .qualified_declaration()
            .is_some_and(|name| name != declaration.qualified_name.as_slice())
        {
            return Err(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                name_pointer(position),
                Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
                node_id.clone(),
            ));
        }
    }
    let mut holders: BTreeMap<&[Box<str>], Vec<(&CheckedNodeId, usize)>> = BTreeMap::new();
    for (&node_id, &position) in index {
        if let Some(declaration) = &nodes[position].declaration {
            holders
                .entry(declaration.qualified_name.as_slice())
                .or_default()
                .push((node_id, position));
        }
    }
    if let Some((first, position)) = holders
        .values()
        .filter(|ids| ids.len() > 1)
        .filter_map(|ids| ids.first())
        .min()
    {
        return Err(refuse_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            name_pointer(*position),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            (*first).clone(),
        ));
    }
    Ok(())
}

/// FR-340 frame-body semantics: entry eligibility, canonical member order and
/// cross-defect refusal precedence. Runs once every node's own identity is
/// known (`index`/`tags`, built by `validate_graph`'s per-node loop),
/// immediately after declaration checks (`validate_nominal_nodes`) and before
/// the graph's dependency/body-reference edges are resolved — the
/// "graph-shape, ..., declaration, frame, operation" reader order the
/// upstream contract description states normatively (`validate_application_keys` runs the
/// stale-application-key stage just before this one, ahead of declaration;
/// `validate_operations` runs the operation stage just after).
/// Running before edge resolution matters: `frame_defect` resolves each
/// declared entry itself, so a frame `dependencies` entry naming no real node
/// is reported as FR-340's own `missing_declaration`, not the generic
/// unresolved-edge `invalid_semantic_graph` the later resolution pass would
/// otherwise raise for the same node first. Visits `state`/`frame` nodes in
/// ascending `node_id` digest order — the iteration order of `index`, a
/// `BTreeMap` — and reports the first one carrying a defect, so a package
/// with several defective frames refuses at the least such frame
/// (FR-340-AC-9). Takes no `&mut WorkMeter`: every frame entry it walks
/// (via [`frame_entries`]) was already parsed, shape-validated and charged
/// once by `validate_frame_body` in the per-node loop above, so this stage's
/// cost is already bounded by that earlier charge (see FR-038's "Frame
/// bodies" section) rather than uncharged and unbounded.
fn validate_frame_semantics(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    for (&node_id, &position) in index {
        let node = &nodes[position];
        if !is_frame(kinds[position]) {
            continue;
        }
        if let Some(failure) = frame_defect(node_id, node, position, kinds, index) {
            return Err(failure);
        }
    }
    Ok(())
}

/// Which member of a node supplied one graph edge, so a charge on that edge
/// can name the value that carried it.
#[derive(Clone, Copy, Debug)]
enum EdgeSite {
    SemanticType,
    Dependency(usize),
    /// The body reference at this index of the node's recorded references.
    Reference(usize),
}

/// One body reference target, the site that carried it, and its pointer.
type BodyReference = (CheckedNodeId, ReferenceSite, JsonPointer);

fn validate_graph(
    wire: &CheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<Vec<CheckedNodeKind>, ValidationFailure> {
    let graph = &wire.semantic_graph;
    if graph.graph_version.as_ref() != GRAPH_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            member_pointer(&["semantic_graph", "graph_version"]),
        ));
    }
    if graph.nodes.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            member_pointer(&["semantic_graph", "nodes"]),
        ));
    }
    let node_count = count(graph.nodes.len());
    if exceeds(node_count, limits.nodes) {
        // The first node past the ceiling is the one whose charge failed.
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Nodes,
            limits.nodes,
            node_count,
            Some(node_pointer(
                usize::try_from(limits.nodes).unwrap_or(usize::MAX),
            )),
        ));
    }
    let mut index = BTreeMap::new();
    let mut kinds = Vec::with_capacity(graph.nodes.len());
    let mut references: Vec<Vec<BodyReference>> = Vec::with_capacity(graph.nodes.len());
    let mut edges = 0_u64;
    for (position, node) in graph.nodes.iter().enumerate() {
        let at = |member: &str| node_pointer(position).key(member);
        validate_node_id(&node.node_id, || at("node_id"))?;
        if index.insert(&node.node_id, position).is_some() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("node_id"),
            ));
        }
        if node.schema_version.as_ref() != GRAPH_V2 {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("schema_version"),
            ));
        }
        let Some(tag) = CheckedNodeTag::from_wire(&node.node_tag) else {
            return Err(refuse(
                CheckedPackageRefusalCode::UnsupportedNodeTag,
                at("node_tag"),
            ));
        };
        let Some(kind) = CheckedNodeKind::decode(tag, &node.semantic_form) else {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("semantic_form"),
            ));
        };
        kinds.push(kind);
        validate_declaration(kind, &node.occurrences, node.declaration.as_ref(), position)?;
        validate_node_id(&node.semantic_type, || at("semantic_type"))?;
        for (dependency_index, dependency) in node.dependencies.iter().enumerate() {
            validate_node_id(dependency, || at("dependencies").index(dependency_index))?;
        }
        if node.recursion_group.as_deref().is_some_and(str::is_empty) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                at("recursion_group"),
            ));
        }
        let edges_before = edges;
        edges = edges.saturating_add(count(node.dependencies.len()));
        if exceeds(edges, limits.edges) {
            // `edges_before` was admitted, so the dependency that took the
            // counter one past the ceiling is the one whose charge failed.
            let first_over = limits.edges.saturating_sub(edges_before);
            return Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Edges,
                limits.edges,
                edges,
                Some(at("dependencies").index(usize::try_from(first_over).unwrap_or(usize::MAX))),
            ));
        }
        let mut targets = Vec::new();
        let body_steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(position),
            Step::Key("body"),
        ];
        let work = validate_body(
            kind,
            &node.body,
            &Trail::Base(&body_steps),
            &mut |target, site, target_at| {
                targets.push((target.clone(), site, target_at.pointer()));
            },
        )?;
        meter.charge(work, || at("body"))?;
        references.push(targets);
        let mut occurrences = BTreeSet::new();
        if node.occurrences.is_empty() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at("occurrences"),
            ));
        }
        if let Some(repeat) = node
            .occurrences
            .iter()
            .position(|occurrence| !occurrences.insert(occurrence))
        {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at("occurrences").index(repeat),
            ));
        }
    }
    // Declaration and frame checks (FR-322/FR-340) run here, against `index`
    // and `kinds` alone, before the dependency/body-target edges below are
    // resolved against the graph: a frame `dependencies` entry naming no
    // real node is FR-340's own `missing_declaration` refusal (`frame_defect`
    // resolves each entry itself), not the generic unresolved-reference
    // `invalid_semantic_graph` the edge-resolution loop below would raise for
    // the same node first if it ran first.
    // Graph-shape body and dependency rules FR-322 orders before the
    // stale-key stage: the structural forms' bodies and the
    // application-node dependency join.
    validate_structural_nodes(&graph.nodes, &kinds, &index)?;
    validate_application_keys(&graph.nodes, &index, meter)?;
    validate_nominal_nodes(&graph.nodes, &kinds, &index, &wire.lock, meter)?;
    validate_declaration_names(&graph.nodes, &index)?;
    validate_frame_semantics(&graph.nodes, &kinds, &index)?;
    validate_operations(&graph.nodes, &kinds, &index, &wire.lock, meter)?;
    let mut adjacency = Vec::with_capacity(graph.nodes.len());
    for (position, (node, targets)) in graph.nodes.iter().zip(&references).enumerate() {
        let resolve = |id: &CheckedNodeId, path: &dyn Fn() -> JsonPointer| {
            index.get(id).copied().ok_or_else(|| {
                refuse(CheckedPackageRefusalCode::InvalidSemanticGraph, path())
            })
        };
        let at = |member: &str| node_pointer(position).key(member);
        let semantic_type = resolve(&node.semantic_type, &|| at("semantic_type"))?;
        let mut successors = Vec::new();
        if semantic_type != position {
            successors.push((semantic_type, EdgeSite::SemanticType));
        }
        for (dependency_index, dependency) in node.dependencies.iter().enumerate() {
            let target = resolve(dependency, &|| at("dependencies").index(dependency_index))?;
            successors.push((target, EdgeSite::Dependency(dependency_index)));
        }
        for (reference_index, (target, site, target_at)) in targets.iter().enumerate() {
            let target = resolve(target, &|| target_at.clone())?;
            // Only a self-typed node's own top-level `literal.type` — the
            // literal that *is* the node body, e.g. a self-typed scalar's
            // `literal.type` — may name itself from its body without that
            // counting as a reference cycle requiring `recursion_group`:
            // FR-322's foundational nominal axiom is typed by itself and can
            // state that fact only through its own `literal.type`. The
            // carve-out is keyed on that member *and* on `is_body_root`, not
            // on node identity alone: a `literal.type` nested inside an
            // `aggregate` member, a `binding` value or an `application`
            // argument reports the same `ReferenceMember::Type` but with
            // `is_body_root: false`, and does not qualify — nor does a
            // `reference` body or an `application.result_type`
            // self-referencing a self-typed node. Both still resolve through
            // `recursion_group` or refuse, exactly like every other 1-node
            // cycle reached by this loop.
            let is_self_typed_literal_type = site.is_body_root
                && site.member == ReferenceMember::Type
                && semantic_type == position;
            if target != position || !is_self_typed_literal_type {
                successors.push((target, EdgeSite::Reference(reference_index)));
            }
        }
        adjacency.push(successors);
    }
    let edge_pointer = |vertex: usize, site: EdgeSite| match site {
        EdgeSite::SemanticType => node_pointer(vertex).key("semantic_type"),
        EdgeSite::Dependency(dependency) => node_pointer(vertex)
            .key("dependencies")
            .index(dependency),
        EdgeSite::Reference(reference) => references
            .get(vertex)
            .and_then(|targets| targets.get(reference))
            .map_or_else(|| node_pointer(vertex).key("body"), |(_, _, at)| at.clone()),
    };
    validate_recursion(&graph.nodes, &adjacency, meter, &edge_pointer)?;
    let projection = graph
        .nodes
        .iter()
        .map(CheckedNodeProjectionV2::from)
        .collect::<Vec<_>>();
    if projection != wire.identity_preimage.identity_projection {
        let at = member_pointer(&["identity_preimage", "identity_projection"]);
        let path = match (
            serde_json::to_value(&wire.identity_preimage.identity_projection),
            serde_json::to_value(&projection),
        ) {
            (Ok(retained), Ok(computed)) => first_difference(at, &retained, &computed),
            _ => at,
        };
        return Err(refuse(CheckedPackageRefusalCode::StaleDependency, path));
    }
    Ok(kinds)
}

/// Every strongly connected component that forms a cycle must share one
/// explicit `recursion_group`. Iterative Tarjan; each edge costs one work,
/// charged at the member that carried the edge (`edge_pointer`).
fn validate_recursion(
    nodes: &[CheckedSemanticNodeV2],
    adjacency: &[Vec<(usize, EdgeSite)>],
    meter: &mut WorkMeter,
    edge_pointer: &dyn Fn(usize, EdgeSite) -> JsonPointer,
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
            if let Some((successor, site)) = successor {
                meter.charge(1, || edge_pointer(vertex, site))?;
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
                    .is_some_and(|edges| edges.iter().any(|(target, _)| *target == vertex));
            if cyclic {
                let group_of = |member: usize| {
                    nodes
                        .get(member)
                        .and_then(|node| node.recursion_group.as_ref())
                };
                let first = component.first().and_then(|member| group_of(*member));
                // The first member (in component order) lacking the shared
                // group: its `recursion_group` when it carries a different
                // one, else the node object that lacks the member.
                let offender = if first.is_none() {
                    component.first().copied()
                } else {
                    component
                        .iter()
                        .copied()
                        .find(|member| group_of(*member) != first)
                };
                if let Some(member) = offender {
                    let path = if group_of(member).is_some() {
                        node_pointer(member).key("recursion_group")
                    } else {
                        node_pointer(member)
                    };
                    return Err(refuse(CheckedPackageRefusalCode::InvalidSemanticGraph, path));
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
    let report = || member_pointer(&["capability_report"]);
    let mut reported = BTreeMap::new();
    for (index, capability) in wire.capability_report.iter().enumerate() {
        if !is_nonempty(&capability.feature)
            || reported
                .insert(capability.feature.as_ref(), (capability.disposition, index))
                .is_some()
        {
            return Err(refuse(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                report().index(index).key("feature"),
            ));
        }
    }
    for (index, feature) in wire.lock.required_features.iter().enumerate() {
        let path = match reported.get(feature.as_ref()) {
            // Unreported: the report lacks the capability.
            None => report(),
            Some((disposition, reported_at))
                if *disposition != CheckedCapabilityDisposition::Available =>
            {
                report().index(*reported_at).key("disposition")
            }
            // Reported available, but this reader does not support it.
            Some(_) if !evidence.supports(feature) => {
                member_pointer(&["lock", "required_features"]).index(index)
            }
            Some(_) => continue,
        };
        return Err(refuse(
            CheckedPackageRefusalCode::UnknownRequiredCapability,
            path,
        ));
    }
    Ok(())
}

fn validate_diagnostics(
    wire: &CheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let entries = || member_pointer(&["diagnostics", "entries"]);
    let diagnostics = count(wire.diagnostics.entries.len());
    if exceeds(diagnostics, limits.diagnostics) {
        // The first entry past the ceiling is the one whose charge failed.
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Diagnostics,
            limits.diagnostics,
            diagnostics,
            Some(entries().index(usize::try_from(limits.diagnostics).unwrap_or(usize::MAX))),
        ));
    }
    let nodes = wire
        .semantic_graph
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<BTreeSet<_>>();
    for (entry_index, entry) in wire.diagnostics.entries.iter().enumerate() {
        for (detail_index, detail) in entry.details.iter().enumerate() {
            let detail_steps = [
                Step::Key("diagnostics"),
                Step::Key("entries"),
                Step::Index(entry_index),
                Step::Key("details"),
                Step::Index(detail_index),
            ];
            let detail_at = Trail::Base(&detail_steps);
            let mut unresolved = None;
            let work = validate_term(
                detail,
                TermGrammar::V2,
                false,
                &detail_at,
                &mut |target, _site, target_at| {
                    if unresolved.is_none() && !nodes.contains(target) {
                        unresolved = Some(target_at.pointer());
                    }
                },
            )?;
            meter.charge(work, || detail_at.pointer())?;
            if let Some(path) = unresolved {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    path,
                ));
            }
        }
        for (locus_index, region) in entry.loci.iter().enumerate() {
            let locus = || entries().index(entry_index).key("loci").index(locus_index);
            if !wire.lock.sources.contains(&region.source) {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    locus().key("source"),
                ));
            }
            if region.start >= region.end {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    locus(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_frame, CheckedNodeKind, FrameMember, ModelForm, RelationForm, StateForm};

    /// `BodyBindingRules`: exactly one kind, `state`/`frame`, has the frame
    /// reference-triple body; every other kind's body is a `SemanticTerm`.
    #[test]
    fn exactly_one_kind_has_a_frame_body() {
        let frames = CheckedNodeKind::all()
            .into_iter()
            .filter(|kind| is_frame(*kind))
            .collect::<Vec<_>>();
        assert_eq!(frames, [CheckedNodeKind::State(StateForm::Frame)]);
    }

    /// FR-340 admits exactly six `(member, kind)` pairs: `modifies` takes a
    /// relation's `relationship` or a model's `field_declaration`; `creates`
    /// and `deletes` each take a model's `object_type` or `process`. This
    /// walks every member against every kind the closed vocabularies can
    /// produce, so a `frame_eligibility` edit that widens or narrows any pair
    /// fails here. The kind count is pinned too: a new form must also pass
    /// through `frame_eligibility`'s exhaustive match, and this test states
    /// that the population it decided over is the one it walked.
    ///
    /// Tracing: TC-053, FR-038-AC-12
    #[test]
    fn tc_053_frame_member_admits_exactly_the_closed_eligible_pairs() {
        let kinds = CheckedNodeKind::all();
        assert_eq!(
            kinds.len(),
            100,
            "the closed form set changed; decide its frame eligibility"
        );
        let admitted = FrameMember::ALL
            .into_iter()
            .flat_map(|member| kinds.iter().map(move |kind| (member, *kind)))
            .filter(|(member, kind)| member.admits(*kind))
            .collect::<Vec<_>>();
        assert_eq!(
            admitted,
            vec![
                (
                    FrameMember::Modifies,
                    CheckedNodeKind::Model(ModelForm::FieldDeclaration)
                ),
                (
                    FrameMember::Modifies,
                    CheckedNodeKind::Relation(RelationForm::Relationship)
                ),
                (
                    FrameMember::Creates,
                    CheckedNodeKind::Model(ModelForm::ObjectType)
                ),
                (
                    FrameMember::Creates,
                    CheckedNodeKind::Model(ModelForm::Process)
                ),
                (
                    FrameMember::Deletes,
                    CheckedNodeKind::Model(ModelForm::ObjectType)
                ),
                (
                    FrameMember::Deletes,
                    CheckedNodeKind::Model(ModelForm::Process)
                ),
            ]
        );
    }
}
