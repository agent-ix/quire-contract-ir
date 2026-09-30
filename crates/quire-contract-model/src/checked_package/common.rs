//! Version-neutral I04 wire front end and helpers shared by the strict V2
//! decoder and its lowering pipeline.
//!
//! Every function here measures bytes, nesting, duplicate members and
//! canonical form identically regardless of which closed schema is decoded
//! from the resulting value.

use super::shared::{push_index, push_key};
use super::shared::{
    CheckedArtifactLocator, CheckedArtifactRef, CheckedNodeId, CheckedOccurrence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSemanticId,
    CheckedSourceMapEntry, JsonPointer,
};
use super::v2::{ApplicationOperator, BodyTerm, LiteralKind, PACKAGE_DOMAIN_V2};
use serde::de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Identity domain shared by every checked semantic node key.
pub(super) const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";

/// A terminal front-end or validation outcome, before it is wrapped in a
/// version-specific read result. It carries the public refusal or incomplete
/// record directly, so every stage builds the exact value the caller sees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ValidationFailure {
    Refused(CheckedPackageRefusal),
    Incomplete(CheckedPackageIncomplete),
}

impl ValidationFailure {
    /// A refusal about the value at `path`.
    pub(super) fn refused(code: CheckedPackageRefusalCode, path: JsonPointer) -> Self {
        Self::Refused(CheckedPackageRefusal {
            code,
            path: Some(path),
            cause: None,
            locus: None,
            contract_version: None,
        })
    }

    /// A refusal about the value at `path` carrying the cause this stage
    /// determined, not located at any graph node.
    pub(super) fn refused_because(
        code: CheckedPackageRefusalCode,
        path: JsonPointer,
        cause: CheckedPackageRefusalCause,
    ) -> Self {
        Self::Refused(CheckedPackageRefusal {
            code,
            path: Some(path),
            cause: Some(cause),
            locus: None,
            contract_version: None,
        })
    }

    /// A refusal about the byte stream rather than any value: malformed JSON
    /// or non-canonical bytes.
    pub(super) fn refused_bytes(code: CheckedPackageRefusalCode) -> Self {
        Self::Refused(CheckedPackageRefusal {
            code,
            path: None,
            cause: None,
            locus: None,
            contract_version: None,
        })
    }

    /// A refusal located at a specific graph node, carrying the cause tag
    /// this reader determined (when the stage determines one) and the node
    /// key of the offending entry or node.
    pub(super) fn refused_at(
        code: CheckedPackageRefusalCode,
        path: JsonPointer,
        cause: Option<CheckedPackageRefusalCause>,
        locus: CheckedNodeId,
    ) -> Self {
        Self::Refused(CheckedPackageRefusal {
            code,
            path: Some(path),
            cause,
            locus: Some(locus),
            contract_version: None,
        })
    }

    /// `unknown_contract_version` for the version string actually read at
    /// the document's `contract_version` member.
    pub(super) fn unknown_contract_version(version: &str) -> Self {
        Self::Refused(CheckedPackageRefusal {
            code: CheckedPackageRefusalCode::UnknownContractVersion,
            path: Some(JsonPointer::root().key("contract_version")),
            cause: None,
            locus: None,
            contract_version: Some(version.into()),
        })
    }

    /// The first exhausted limit, charged at the value `path` names (absent
    /// only for the byte limit, charged before any value exists).
    pub(super) fn incomplete(
        limit_kind: CheckedPackageLimit,
        limit: u64,
        consumed: impl TryInto<u64>,
        path: Option<JsonPointer>,
    ) -> Self {
        Self::Incomplete(CheckedPackageIncomplete {
            limit_kind,
            limit,
            consumed: consumed.try_into().unwrap_or(u64::MAX),
            path,
        })
    }

    /// Wraps the outcome in a version-specific closed read result.
    pub(super) fn into_result<R>(
        self,
        refused: fn(CheckedPackageRefusal) -> R,
        incomplete: fn(CheckedPackageIncomplete) -> R,
    ) -> R {
        match self {
            Self::Refused(refusal) => refused(refusal),
            Self::Incomplete(record) => incomplete(record),
        }
    }
}

/// One RFC 6901 reference token, borrowed from the reader's own traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Step<'a> {
    /// An object member name.
    Key(&'a str),
    /// An array index.
    Index(usize),
}

/// The reader's current position in the document, as a chain of borrowed
/// steps on the walking code's own stack. Building it allocates nothing; a
/// [`JsonPointer`] is materialized only when an outcome needs one.
#[derive(Clone, Copy, Debug)]
pub(super) enum Trail<'a> {
    /// A fixed prefix of steps from the document root.
    Base(&'a [Step<'a>]),
    /// One step below a parent position.
    Child(&'a Trail<'a>, Step<'a>),
}

impl<'a> Trail<'a> {
    /// The position one object member below this one.
    pub(super) fn key<'b>(&'b self, key: &'b str) -> Trail<'b>
    where
        'a: 'b,
    {
        Trail::Child(self, Step::Key(key))
    }

    /// The position one array element below this one.
    pub(super) fn index<'b>(&'b self, index: usize) -> Trail<'b>
    where
        'a: 'b,
    {
        Trail::Child(self, Step::Index(index))
    }

    /// The RFC 6901 pointer of this position. Walks the chain iteratively.
    pub(super) fn pointer(&self) -> JsonPointer {
        let mut below = Vec::new();
        let mut current = self;
        let base = loop {
            match current {
                Trail::Base(steps) => break *steps,
                Trail::Child(parent, step) => {
                    below.push(*step);
                    current = parent;
                }
            }
        };
        pointer_from_steps(base.iter().chain(below.iter().rev()).copied())
    }
}

/// Assembles an RFC 6901 pointer from reference tokens, escaping each key.
pub(super) fn pointer_from_steps<'a>(steps: impl IntoIterator<Item = Step<'a>>) -> JsonPointer {
    let mut text = String::new();
    for step in steps {
        match step {
            Step::Key(key) => push_key(&mut text, key),
            Step::Index(index) => push_index(&mut text, index),
        }
    }
    JsonPointer::from_escaped(text)
}

/// The pointer of `/semantic_graph/nodes/{position}`.
pub(super) fn node_pointer(position: usize) -> JsonPointer {
    JsonPointer::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}

/// Authoritative raw-artifact digests used to prove lock entries are current.
pub(super) trait ArtifactDigests {
    /// Returns the lowercase SHA-256 digest for the exact locator, if known.
    fn artifact_digest(&self, locator: &CheckedArtifactLocator) -> Option<Cow<'_, str>>;
}

/// Measures, parses and canonicalizes untrusted bytes.
///
/// Order, as FR-322 states it: byte limit, strict syntax and member
/// validation, depth limit, canonical bytes. The first parse pass,
/// [`strict_json_shape`], checks syntax and duplicate members and measures
/// depth without building a value, on a stack that grows onto the heap, so a
/// syntax or duplicate-member defect anywhere in the document is refused
/// before depth is charged, however deep the document is. Only a document
/// within the depth limit is then parsed into a value, so nothing downstream
/// ever holds a value deeper than [`CheckedPackageReadLimits::MAXIMUM_DEPTH`].
pub(super) fn canonical_value(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
) -> Result<Value, ValidationFailure> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Bytes,
            limits.bytes,
            bytes.len(),
            None,
        ));
    }
    let depth_limit = limits.depth.min(CheckedPackageReadLimits::MAXIMUM_DEPTH);
    let shape = strict_shape(bytes, depth_limit)?;
    if exceeds(shape.depth, depth_limit) {
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Depth,
            depth_limit,
            shape.depth,
            shape.first_past_limit,
        ));
    }
    let value = strict_parse(bytes, true)?;
    let canonical = serde_json::to_vec(&value)
        .map_err(|_| ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire))?;
    if canonical.as_slice() != bytes {
        return Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::NoncanonicalWire,
        ));
    }
    Ok(value)
}

/// Decodes a closed wire value, classifying closed-schema member violations
/// and locating each at the position the decoder had reached: an unknown
/// member at that member, a missing member at the object lacking it, and a
/// wrongly typed value at that value.
pub(super) fn decode_closed<T: DeserializeOwned>(value: &Value) -> Result<T, ValidationFailure> {
    serde_path_to_error::deserialize::<_, T>(value).map_err(|error| {
        let code = if error.inner().to_string().contains("unknown field") {
            CheckedPackageRefusalCode::UnknownMember
        } else {
            CheckedPackageRefusalCode::MalformedWire
        };
        ValidationFailure::refused(code, decoder_pointer(JsonPointer::root(), error.path()))
    })
}

/// The decoder's position below `base` (the value it decoded) as a pointer.
/// A position the decoder reached only through buffered content (an
/// internally tagged enum's members) is not tracked below that enum, so the
/// pointer stops at the deepest position it names exactly.
pub(super) fn decoder_pointer(base: JsonPointer, path: &serde_path_to_error::Path) -> JsonPointer {
    use serde_path_to_error::Segment;
    let mut pointer = base;
    for segment in path.iter() {
        pointer = match segment {
            Segment::Seq { index } => pointer.index(*index),
            Segment::Map { key } => pointer.key(key),
            Segment::Enum { .. } | Segment::Unknown => break,
        };
    }
    pointer
}

/// The pointer of the first position, in document order, at which `other`
/// differs from `original`, where `original` sits at `base`: the member or
/// element whose value differs or is missing from `other`, or the container
/// whose shape differs. Descends iteratively, one differing child at a time.
pub(super) fn first_difference(base: JsonPointer, original: &Value, other: &Value) -> JsonPointer {
    let mut steps = Vec::new();
    let (mut left, mut right) = (original, other);
    loop {
        let next = match (left, right) {
            (Value::Object(left_members), Value::Object(right_members)) => left_members
                .iter()
                .find_map(|(key, left_value)| match right_members.get(key) {
                    Some(right_value) if right_value == left_value => None,
                    Some(right_value) => {
                        Some((Step::Key(key.as_str()), left_value, Some(right_value)))
                    }
                    None => Some((Step::Key(key.as_str()), left_value, None)),
                }),
            (Value::Array(left_items), Value::Array(right_items))
                if left_items.len() == right_items.len() =>
            {
                left_items
                    .iter()
                    .zip(right_items)
                    .enumerate()
                    .find(|(_, (left_item, right_item))| left_item != right_item)
                    .map(|(index, (left_item, right_item))| {
                        (Step::Index(index), left_item, Some(right_item))
                    })
            }
            _ => None,
        };
        match next {
            Some((step, left_value, Some(right_value))) => {
                steps.push(step);
                left = left_value;
                right = right_value;
            }
            Some((step, _, None)) => {
                steps.push(step);
                break;
            }
            None => break,
        }
    }
    steps.into_iter().fold(base, |pointer, step| match step {
        Step::Key(key) => pointer.key(key),
        Step::Index(index) => pointer.index(index),
    })
}

pub(super) fn exceeds(consumed: impl TryInto<u64>, limit: u64) -> bool {
    consumed.try_into().map_or(true, |value| value > limit)
}

pub(super) fn count(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

pub(super) fn is_nonempty(value: &str) -> bool {
    !value.is_empty()
}

pub(super) fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn digest_json(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_vec(value).map(|bytes| digest_bytes(&bytes))
}

pub(super) fn artifact_locator(value: &CheckedArtifactRef) -> CheckedArtifactLocator {
    CheckedArtifactLocator {
        authority: value.authority.clone(),
        identity: value.identity.clone(),
        revision_namespace: value.revision.namespace.clone(),
        revision_value: value.revision.value.clone(),
        domain: value.digest_domain.clone(),
    }
}

/// Checks one locked artifact's domain, shape and digest against evidence.
/// `at` names the artifact reference; each refusal points at the member it
/// is about, or at the reference itself when its locator is unattested.
// Intake check of a locked artifact's digest domain and digest text.
pub(super) fn validate_locked_artifact(
    artifact: &CheckedArtifactRef,
    expected_domain: &str,
    context: &dyn ArtifactDigests,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let refuse = |code, member: &[&str]| {
        let path = member.iter().fold(at(), |path, key| path.key(key));
        ValidationFailure::refused(code, path)
    };
    if artifact.digest_domain.as_ref() != expected_domain {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            &["digest_domain"],
        ));
    }
    let members: [(&str, &[&str]); 4] = [
        (&artifact.authority, &["authority"]),
        (&artifact.identity, &["identity"]),
        (&artifact.revision.namespace, &["revision", "namespace"]),
        (&artifact.revision.value, &["revision", "value"]),
    ];
    if let Some((_, member)) = members.iter().find(|(value, _)| !is_nonempty(value)) {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, member));
    }
    if !is_digest(&artifact.digest) {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            &["digest"],
        ));
    }
    let locator = artifact_locator(artifact);
    let Some(digest) = context.artifact_digest(&locator) else {
        return Err(refuse(CheckedPackageRefusalCode::StaleDependency, &[]));
    };
    if digest.as_ref() != artifact.digest.as_ref() {
        return Err(refuse(
            CheckedPackageRefusalCode::StaleDependency,
            &["digest"],
        ));
    }
    Ok(())
}

/// Requires the source map to be exactly the graph's occurrence set, with
/// nonempty, locked, current, non-overlapping half-open regions.
pub(super) fn validate_source_map_entries<'a>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a [CheckedOccurrence])>,
    source_map: &[CheckedSourceMapEntry],
    locked_sources: &[CheckedArtifactRef],
    limits: CheckedPackageReadLimits,
    context: &dyn ArtifactDigests,
) -> Result<(), ValidationFailure> {
    let entry_pointer = |entry: usize| JsonPointer::root().key("source_map").index(entry);
    let region_pointer =
        |entry: usize, region: usize| entry_pointer(entry).key("regions").index(region);
    let mut expected = BTreeSet::new();
    for (node_id, occurrences) in nodes {
        for occurrence in occurrences {
            expected.insert((node_id.clone(), occurrence.role.clone(), occurrence.ordinal));
        }
    }
    let mut actual = BTreeSet::new();
    let locked_sources = locked_sources.iter().collect::<BTreeSet<_>>();
    let mut count = 0_u64;
    for (entry_index, entry) in source_map.iter().enumerate() {
        count = count.saturating_add(1);
        if exceeds(count, limits.occurrences) {
            return Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
                Some(entry_pointer(entry_index)),
            ));
        }
        if entry.regions.is_empty() {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                entry_pointer(entry_index).key("regions"),
            ));
        }
        if !actual.insert((entry.node_id.clone(), entry.role.clone(), entry.ordinal)) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                entry_pointer(entry_index),
            ));
        }
        let before_regions = count;
        count = count.saturating_add(u64::try_from(entry.regions.len()).unwrap_or(u64::MAX));
        if exceeds(count, limits.occurrences) {
            // `before_regions` was admitted, so the first region whose
            // charge failed is the one that took the counter one past the
            // limit.
            let first_over = limits.occurrences.saturating_sub(before_regions);
            return Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
                Some(region_pointer(
                    entry_index,
                    usize::try_from(first_over).unwrap_or(usize::MAX),
                )),
            ));
        }
        let mut ranges = BTreeMap::<CheckedArtifactLocator, Vec<(u64, u64, usize)>>::new();
        for (region_index, region) in entry.regions.iter().enumerate() {
            let source_pointer = || region_pointer(entry_index, region_index).key("source");
            if !locked_sources.contains(&region.source) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    source_pointer(),
                ));
            }
            validate_locked_artifact(
                &region.source,
                "quire.source.bytes/v1",
                context,
                &source_pointer,
            )?;
            if region.start >= region.end {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    region_pointer(entry_index, region_index),
                ));
            }
            ranges
                .entry(artifact_locator(&region.source))
                .or_default()
                .push((region.start, region.end, region_index));
        }
        for ranges in ranges.values_mut() {
            ranges.sort_unstable();
            if let Some(pair) = ranges.windows(2).find(|pair| pair[0].1 > pair[1].0) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    region_pointer(entry_index, pair[1].2),
                ));
            }
        }
    }
    if actual != expected {
        // An entry naming no graph occurrence is the value at fault; an
        // occurrence no entry maps is a defect of the array as a whole.
        let stray = source_map.iter().position(|entry| {
            !expected.contains(&(entry.node_id.clone(), entry.role.clone(), entry.ordinal))
        });
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSourceMap,
            stray.map_or_else(|| JsonPointer::root().key("source_map"), entry_pointer),
        ));
    }
    Ok(())
}

/// Which member carried a reported reference target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReferenceMember {
    /// A `reference` term's `target`.
    Target,
    /// A `literal` term's `type`.
    Type,
    /// An `application` term's `result_type`.
    ResultType,
    /// One entry of a frame body's `modifies`, `creates` or `deletes`.
    FrameEntry,
}

/// Where a resolved reference target was found: the member that carried it,
/// and whether it was reported by the node body's own top-level term rather
/// than a term nested inside it (an `aggregate` member, a `binding` value, or
/// an `application` argument). Only a target reported with `is_body_root:
/// true` can be the node's own self-typed `literal.type`; the same member
/// reported from a nested term names a different, non-exempt occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReferenceSite {
    pub(super) member: ReferenceMember,
    pub(super) is_body_root: bool,
}

/// Receives each reference target a walk finds, the site that carried it and
/// the walk's position at that target.
pub(super) type ReferenceVisitor<'v> = dyn FnMut(&CheckedNodeId, ReferenceSite, &Trail<'_>) + 'v;

/// The literal-value grammar a semantic term is validated against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TermGrammar {
    /// V2 `SemanticTerm`: a numeric literal value must be an integer token
    /// within the signed or unsigned 64-bit range. The upstream schema admits
    /// only `integer` numbers; a fraction, exponent or out-of-range integer is
    /// refused rather than rounded.
    V2,
}

/// Validates one public semantic term at position `at`, returning its work
/// and reporting every reference target to `visit` with a [`ReferenceSite`]
/// naming the member that carried it and whether `value` itself is the node
/// body's own top-level term (`is_body_root`), so a caller can tell a target
/// found in the body root from the same member found in a nested term. Every
/// recursive descent — an `aggregate` member, a `binding` value, an
/// `application` argument — passes `is_body_root: false`: only the term
/// handed to the outermost call can be the body root. The descent is bounded
/// by the depth limit `canonical_value` already enforced. A refusal points at
/// the term that matches no closed shape, or at the member it is about.
pub(super) fn validate_term(
    value: &Value,
    grammar: TermGrammar,
    is_body_root: bool,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let invalid = |position: &Trail<'_>| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            position.pointer(),
        )
    };
    let Value::Object(object) = value else {
        return Err(invalid(at));
    };
    match object.get("term") {
        Some(Value::String(_)) => {}
        Some(_) => return Err(invalid(&at.key("term"))),
        None => return Err(invalid(at)),
    }
    let Some(term) = body_term(value) else {
        return Err(invalid(at));
    };
    match term {
        BodyTerm::Literal => {
            if exact_members(object, &["term", "type", "value_kind", "value"])
                && literal_kind(value).is_some()
                && object
                    .get("value")
                    .is_some_and(|value| is_literal_value(value, grammar))
            {
                visit_member(
                    object,
                    "type",
                    ReferenceMember::Type,
                    is_body_root,
                    at,
                    visit,
                )
            } else {
                Err(invalid(at))
            }
        }
        BodyTerm::Reference => {
            if exact_members(object, &["term", "target"]) {
                visit_member(
                    object,
                    "target",
                    ReferenceMember::Target,
                    is_body_root,
                    at,
                    visit,
                )
            } else {
                Err(invalid(at))
            }
        }
        BodyTerm::Application => {
            if !(exact_members(
                object,
                &["term", "operator", "operation", "result_type", "arguments"],
            ) && application_operator(value).is_some())
            {
                return Err(invalid(at));
            }
            // The `operation` member's presence is checked here; its own
            // closed shape and catalog-law validation is
            // `v2::operations::validate_operations`'s job, run once the
            // whole graph's identity and dependency edges are known — the
            // same reason `validate_frame_semantics` runs separately from
            // this per-term walk rather than inline here.
            let result_type_work = visit_member(
                object,
                "result_type",
                ReferenceMember::ResultType,
                is_body_root,
                at,
                visit,
            )?;
            let arguments_work = visit_terms(object, "arguments", grammar, at, visit)?;
            Ok(result_type_work.saturating_add(arguments_work))
        }
        BodyTerm::Aggregate => {
            if exact_members(object, &["term", "members"]) {
                visit_terms(object, "members", grammar, at, visit)
            } else {
                Err(invalid(at))
            }
        }
        BodyTerm::Binding => {
            if exact_members(object, &["term", "name", "value"])
                && object
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(is_nonempty)
            {
                let value = object.get("value").unwrap_or(&Value::Null);
                validate_term(value, grammar, false, &at.key("value"), visit)
                    .map(|work| work.saturating_add(1))
            } else {
                Err(invalid(at))
            }
        }
        // Implements: FR-322. `{term, package, node}`. The target names a node of a
        // dependency package, so it is never a reference of this graph and
        // is not reported to `visit`. Which positions admit it is decided by
        // the operation stage (step 7), not by this shape check.
        BodyTerm::DependencyReference => {
            if !exact_members(object, &["term", "package", "node"]) {
                return Err(invalid(at));
            }
            if dependency_reference_package(value).is_none() {
                return Err(invalid(&at.key("package")));
            }
            if dependency_reference_node(value).is_none() {
                return Err(invalid(&at.key("node")));
            }
            Ok(1)
        }
        // A frame is a node body of its own, never a nested term.
        BodyTerm::Frame => Err(invalid(at)),
    }
}

/// The `term` tag of a semantic term or frame body, decoded; `None` when the
/// member is absent, not a string, or outside the vocabulary.
// Reads the JSON `term` member of a body and decodes it once.
pub(super) fn body_term(value: &Value) -> Option<BodyTerm> {
    BodyTerm::from_wire(value.get("term")?.as_str()?)
}

/// A `dependency_reference` term's `package`: a `quire.package.semantic/v2`
/// SHA-256 [`CheckedSemanticId`]; `None` for any other shape, a bare digest
/// or another digest domain included.
// Intake check of a dependency reference's package domain and algorithm.
pub(super) fn dependency_reference_package(term: &Value) -> Option<CheckedSemanticId> {
    let package = CheckedSemanticId::deserialize(term.get("package")?).ok()?;
    (package.domain.as_ref() == PACKAGE_DOMAIN_V2
        && package.algorithm.as_ref() == "sha256"
        && is_digest(&package.digest))
    .then_some(package)
}

/// A `dependency_reference` term's `node`: a node key in the node domain;
/// `None` for any other shape.
// Intake check of a dependency reference's node domain.
pub(super) fn dependency_reference_node(term: &Value) -> Option<CheckedNodeId> {
    let node = CheckedNodeId::deserialize(term.get("node")?).ok()?;
    (node.domain.as_ref() == NODE_DOMAIN && is_digest(&node.digest)).then_some(node)
}

/// A `literal` term's decoded `value_kind`.
// Reads the JSON `value_kind` member of a literal and decodes it.
pub(super) fn literal_kind(value: &Value) -> Option<LiteralKind> {
    LiteralKind::from_wire(value.get("value_kind")?.as_str()?)
}

/// An `application` term's decoded `operator` class.
// Reads the JSON `operator` member of an application and decodes it.
pub(super) fn application_operator(value: &Value) -> Option<ApplicationOperator> {
    ApplicationOperator::from_wire(value.get("operator")?.as_str()?)
}

/// Reports the node reference held in `object[key]` (the term at `at`).
fn visit_member(
    object: &Map<String, Value>,
    key: &str,
    member: ReferenceMember,
    is_body_root: bool,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let Some(value) = object.get(key) else {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        ));
    };
    visit_reference(value, member, is_body_root, &at.key(key), visit)
}

/// Parses one node reference at position `at`, reports its target and a
/// [`ReferenceSite`] naming `member` and `is_body_root` to `visit`, and
/// charges one unit of work. Shared by `reference.target`, `literal.type`,
/// `application.result_type`, and the V2 `frame` body's reference arrays.
// Intake check of a node reference's domain and digest text.
pub(super) fn visit_reference(
    value: &Value,
    member: ReferenceMember,
    is_body_root: bool,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let target = CheckedNodeId::deserialize(value).map_err(|_| {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    })?;
    if target.domain.as_ref() == NODE_DOMAIN && is_digest(&target.digest) {
        visit(
            &target,
            ReferenceSite {
                member,
                is_body_root,
            },
            at,
        );
        Ok(1)
    } else {
        Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            at.pointer(),
        ))
    }
}

pub(super) fn exact_members(object: &Map<String, Value>, expected: &[&str]) -> bool {
    object.len() == expected.len() && expected.iter().all(|member| object.contains_key(*member))
}

fn is_literal_value(value: &Value, grammar: TermGrammar) -> bool {
    match value {
        Value::Bool(_) | Value::String(_) | Value::Null => true,
        Value::Number(number) => match grammar {
            TermGrammar::V2 => number.is_i64() || number.is_u64(),
        },
        Value::Array(_) | Value::Object(_) => false,
    }
}

/// Validates each term in the `aggregate.members` or `application.arguments`
/// array held in `object[key]` (the term at `at`). Every element is nested one
/// level below the term that holds this array, so each is validated with
/// `is_body_root: false` regardless of whether that enclosing term was itself
/// the body root.
fn visit_terms(
    object: &Map<String, Value>,
    key: &str,
    grammar: TermGrammar,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let array_at = at.key(key);
    let Some(Value::Array(values)) = object.get(key) else {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            array_at.pointer(),
        ));
    };
    values
        .iter()
        .enumerate()
        .try_fold(1_u64, |work, (index, term)| {
            validate_term(term, grammar, false, &array_at.index(index), visit)
                .map(|child| work.saturating_add(child))
        })
}

/// Parses strict JSON with serde_json's own nesting cap: a repeated object
/// member refuses as `duplicate_member` at that member, any other syntax
/// error as `malformed_wire` with no pointer. For input whose depth no caller
/// limit has charged.
pub(super) fn strict_json_value(input: &[u8]) -> Result<Value, ValidationFailure> {
    strict_parse(input, false)
}

/// Parses `input` into a value. With `lift_nesting_cap`, serde_json's own
/// nesting cap is lifted, so the caller must already have bounded the
/// document's depth.
fn strict_parse(input: &[u8], lift_nesting_cap: bool) -> Result<Value, ValidationFailure> {
    let run = Run::new(u64::MAX);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    if lift_nesting_cap {
        deserializer.disable_recursion_limit();
    }
    let parsed = Strict::<Value>::root(&run).deserialize(&mut deserializer);
    run.finish(parsed)
}

/// What the first parse pass learns about a document.
struct Shape {
    /// The document's nesting depth: a container is one level and a scalar one
    /// level below the container holding it, so `1`, `[]` and `{}` are depth
    /// 1, and `[1]` and `{"a":1}` are depth 2.
    /// [`CheckedPackageReadLimits::depth`] is charged in this unit.
    depth: u64,
    /// The position of the first value, in document order, nested deeper than
    /// the limit the pass was given.
    first_past_limit: Option<JsonPointer>,
}

/// The first parse pass: strict syntax and duplicate-member validation, and
/// the document's depth, building no value. Nesting recurses on a stack that
/// `stacker` and `serde_stacker` grow onto the heap, the crate's idiom for
/// untrusted nesting, so depth cannot exhaust the caller's stack and
/// serde_json's own nesting cap never decides the outcome. Its memory grows
/// with the document, so the byte limit bounds it. Trailing bytes are left
/// for the canonical-bytes comparison to refuse.
fn strict_shape(input: &[u8], depth_limit: u64) -> Result<Shape, ValidationFailure> {
    let run = Run::new(depth_limit);
    let parsed = stacker::maybe_grow(16 * 1024 * 1024, 16 * 1024 * 1024, || {
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        deserializer.disable_recursion_limit();
        Strict::<Depth>::root(&run).deserialize(serde_stacker::Deserializer::new(&mut deserializer))
    });
    let Depth(depth) = run.finish(parsed)?;
    Ok(Shape {
        depth,
        first_past_limit: run.past_limit.into_inner(),
    })
}

/// State shared by every position of one strict parse.
struct Run {
    /// Where the first repeated object member was met.
    duplicate: RefCell<Option<JsonPointer>>,
    /// Values nested deeper than this are recorded in `past_limit`.
    depth_limit: u64,
    /// The first value, in document order, nested deeper than `depth_limit`.
    past_limit: RefCell<Option<JsonPointer>>,
}

impl Run {
    fn new(depth_limit: u64) -> Self {
        Self {
            duplicate: RefCell::new(None),
            depth_limit,
            past_limit: RefCell::new(None),
        }
    }

    /// Maps a finished parse to the reader's outcome.
    fn finish<T>(&self, parsed: Result<T, serde_json::Error>) -> Result<T, ValidationFailure> {
        match (parsed, self.duplicate.take()) {
            (Ok(value), _) => Ok(value),
            (Err(_), Some(pointer)) => Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::DuplicateMember,
                pointer,
            )),
            (Err(_), None) => Err(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::MalformedWire,
            )),
        }
    }
}

/// What one strict JSON parse produces from each construct. The grammar
/// itself — scalars, finite numbers, the number token, duplicate members —
/// is written once, in [`Strict`]; a sink only decides what to keep.
trait StrictSink: Sized {
    /// A scalar value.
    fn scalar(value: Value) -> Self;
    /// An array of already-admitted elements.
    fn array(elements: Vec<Self>) -> Self;
    /// An object of already-admitted members, with unique names.
    fn object(members: Vec<(String, Self)>) -> Self;
}

impl StrictSink for Value {
    fn scalar(value: Value) -> Self {
        value
    }
    fn array(elements: Vec<Self>) -> Self {
        Value::Array(elements)
    }
    fn object(members: Vec<(String, Self)>) -> Self {
        Value::Object(members.into_iter().collect())
    }
}

/// A strict value reduced to its nesting depth, in the unit of [`Shape::depth`].
struct Depth(u64);

impl StrictSink for Depth {
    fn scalar(_: Value) -> Self {
        Depth(1)
    }
    fn array(elements: Vec<Self>) -> Self {
        Depth(
            elements
                .iter()
                .map(|depth| depth.0)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        )
    }
    fn object(members: Vec<(String, Self)>) -> Self {
        Depth(
            members
                .iter()
                .map(|(_, depth)| depth.0)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        )
    }
}

/// Parses one JSON value at position `at`, `level` levels deep (the document
/// is level 1), collecting it with sink `S`.
struct Strict<'s, S> {
    at: &'s Trail<'s>,
    level: u64,
    run: &'s Run,
    sink: std::marker::PhantomData<fn() -> S>,
}

impl<'s, S> Strict<'s, S> {
    fn root(run: &'s Run) -> Strict<'s, S> {
        Strict {
            at: &Trail::Base(&[]),
            level: 1,
            run,
            sink: std::marker::PhantomData,
        }
    }

    fn below(&self, at: &'s Trail<'s>) -> Strict<'s, S> {
        Strict {
            at,
            level: self.level.saturating_add(1),
            run: self.run,
            sink: std::marker::PhantomData,
        }
    }
}

/// The single member name serde_json uses to hand a number's source text to a
/// visitor when its `arbitrary_precision` feature is on. Feature unification
/// can switch that on for this crate without this crate asking for it.
const SERDE_JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";

/// Converts a number's source text exactly as the default serde_json build
/// would have visited it: `u64`, then `i64`, then a finite `f64`.
fn number_from_token<E: serde::de::Error>(text: &str) -> Result<Value, E> {
    if let Ok(unsigned) = text.parse::<u64>() {
        return Ok(Value::Number(unsigned.into()));
    }
    if let Ok(signed) = text.parse::<i64>() {
        return Ok(Value::Number(signed.into()));
    }
    text.parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .map(Value::Number)
        .ok_or_else(|| E::custom("non-finite JSON number"))
}

impl<'de, S: StrictSink> DeserializeSeed<'de> for Strict<'_, S> {
    type Value = S;

    fn deserialize<D>(self, deserializer: D) -> Result<S, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if self.level > self.run.depth_limit {
            let mut first = self.run.past_limit.borrow_mut();
            if first.is_none() {
                *first = Some(self.at.pointer());
            }
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de, S: StrictSink> Visitor<'de> for Strict<'_, S> {
    type Value = S;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("strict JSON value")
    }
    fn visit_bool<E>(self, value: bool) -> Result<S, E> {
        Ok(S::scalar(Value::Bool(value)))
    }
    fn visit_i64<E>(self, value: i64) -> Result<S, E> {
        Ok(S::scalar(Value::Number(value.into())))
    }
    fn visit_u64<E>(self, value: u64) -> Result<S, E> {
        Ok(S::scalar(Value::Number(value.into())))
    }
    fn visit_f64<E>(self, value: f64) -> Result<S, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(|number| S::scalar(Value::Number(number)))
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<S, E> {
        Ok(S::scalar(Value::String(value.to_owned())))
    }
    fn visit_string<E>(self, value: String) -> Result<S, E> {
        Ok(S::scalar(Value::String(value)))
    }
    fn visit_none<E>(self) -> Result<S, E> {
        Ok(S::scalar(Value::Null))
    }
    fn visit_unit<E>(self) -> Result<S, E> {
        Ok(S::scalar(Value::Null))
    }
    fn visit_seq<A>(self, mut access: A) -> Result<S, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements = Vec::new();
        loop {
            let at = self.at.index(elements.len());
            match access.next_element_seed(self.below(&at))? {
                Some(element) => elements.push(element),
                None => return Ok(S::array(elements)),
            }
        }
    }
    // Canonical-JSON front end: reads serde_json's number token.
    fn visit_map<A>(self, mut access: A) -> Result<S, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut names = BTreeSet::new();
        let mut members = Vec::new();
        while let Some(key) = access.next_key::<String>()? {
            if members.is_empty() && key == SERDE_JSON_NUMBER_TOKEN {
                let digits = access.next_value::<String>()?;
                return number_from_token(&digits).map(S::scalar);
            }
            let at = self.at.key(&key);
            let value = access.next_value_seed(self.below(&at))?;
            if !names.insert(key.clone()) {
                self.run.duplicate.replace(Some(at.pointer()));
                return Err(serde::de::Error::custom("duplicate JSON member"));
            }
            members.push((key, value));
        }
        Ok(S::object(members))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_value, first_difference, is_literal_value, Step, TermGrammar, Trail,
        ValidationFailure,
    };
    use crate::checked_package::shared::{
        CheckedPackageReadLimits, CheckedPackageRefusalCode, JsonPointer,
    };
    use serde_json::{json, Number, Value};

    /// A trail materializes as an RFC 6901 pointer: `~` and `/` escaped in
    /// every member name, array elements by index, the empty pointer for the
    /// document itself; and the text round-trips through `JsonPointer::parse`.
    ///
    /// Tracing: TC-048, FR-038-AC-24
    #[test]
    fn tc_048_trails_materialize_as_escaped_rfc_6901_pointers() {
        let base = [Step::Key("semantic_graph"), Step::Key("nodes")];
        let root = Trail::Base(&base);
        let node = root.index(7);
        let member = node.key("a/b~c");
        let deeper = member.index(0);
        assert_eq!(
            deeper.pointer().as_str(),
            "/semantic_graph/nodes/7/a~1b~0c/0"
        );
        assert_eq!(Trail::Base(&[]).pointer(), JsonPointer::root());
        assert_eq!(JsonPointer::root().as_str(), "");
        assert_eq!(
            JsonPointer::root().key("~1").index(2).as_str(),
            "/~01/2",
            "an escape sequence spelled in a key is itself escaped"
        );
        assert_eq!(
            JsonPointer::parse("/semantic_graph/nodes/7/a~1b~0c/0"),
            Some(deeper.pointer())
        );
        for invalid in ["semantic_graph", "/a~", "/a~2", "/~x"] {
            assert_eq!(JsonPointer::parse(invalid), None, "{invalid}");
        }
    }

    /// The depth charge that fails is the first value, in document order, one
    /// level deeper than the limit.
    ///
    /// Tracing: TC-048, FR-038-AC-26
    #[test]
    fn tc_048_depth_is_charged_at_the_first_value_past_the_limit() {
        let value = json!({"a": [1, {"b": 2}], "c": {"d": {"e": 3}}});
        let bytes = serde_json::to_vec(&value).expect("bytes");
        let charged_at = |depth| {
            let limits = CheckedPackageReadLimits {
                depth,
                ..CheckedPackageReadLimits::bounded()
            };
            match canonical_value(&bytes, limits) {
                Err(ValidationFailure::Incomplete(incomplete)) => {
                    assert_eq!(incomplete.consumed, 4, "depth {depth}");
                    incomplete.path.map(|at| at.as_str().to_owned())
                }
                other => panic!("depth {depth}: {other:?}"),
            }
        };
        assert_eq!(charged_at(0).as_deref(), Some(""));
        assert_eq!(charged_at(1).as_deref(), Some("/a"));
        assert_eq!(charged_at(2).as_deref(), Some("/a/0"));
        assert_eq!(charged_at(3).as_deref(), Some("/a/1/b"));
        assert!(canonical_value(&bytes, CheckedPackageReadLimits::bounded()).is_ok());
    }

    /// A lossy decode is located at the first member it changed or dropped.
    ///
    /// Tracing: TC-048, FR-038-AC-24
    #[test]
    fn tc_048_first_difference_names_the_changed_member() {
        let original = json!({"a": {"b": [1, {"c": null}], "d": 1}});
        let decoded = json!({"a": {"b": [1, {}], "d": 1}});
        assert_eq!(
            first_difference(JsonPointer::root(), &original, &decoded).as_str(),
            "/a/b/1/c"
        );
        let shorter = json!({"a": {"b": [1], "d": 1}});
        assert_eq!(
            first_difference(JsonPointer::root().key("x"), &original, &shorter).as_str(),
            "/x/a/b",
            "an array whose length changed is itself the value at fault"
        );
    }

    /// Tracing: TC-048, FR-038-AC-2
    #[test]
    fn tc_048_number_tokens_decode_identically_with_or_without_arbitrary_precision() {
        let limits = CheckedPackageReadLimits::bounded();
        for (bytes, integer) in [
            (&br#"{"v":1.5}"#[..], false),
            (br#"{"v":2.0}"#, false),
            (br#"{"v":-7}"#, true),
            (br#"{"v":18446744073709551615}"#, true),
        ] {
            let value = canonical_value(bytes, limits).expect("canonical number admits");
            let number = &value["v"];
            assert!(number.is_number(), "{number}");
            assert_eq!(
                is_literal_value(number, TermGrammar::V2),
                integer,
                "{}",
                String::from_utf8_lossy(bytes)
            );
        }
        // The feature's private member name spelled as data decodes as the
        // number it names in every build, so it never re-serializes to the
        // bytes it arrived as and is refused identically.
        assert_eq!(
            canonical_value(br#"{"$serde_json::private::Number":"1"}"#, limits),
            Err(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::NoncanonicalWire
            ))
        );
    }

    /// Tracing: TC-048, FR-038-AC-2
    #[test]
    fn tc_048_v2_literal_numbers_are_integers() {
        let fractional = Value::Number(Number::from_f64(1.5).expect("finite"));
        let whole_float = Value::Number(Number::from_f64(2.0).expect("finite"));
        for (value, is_literal) in [
            (json!(7), true),
            (json!(-7), true),
            (json!(u64::MAX), true),
            (json!(true), true),
            (json!("7"), true),
            (Value::Null, true),
            (fractional, false),
            (whole_float, false),
            (json!([]), false),
            (json!({}), false),
        ] {
            assert_eq!(
                is_literal_value(&value, TermGrammar::V2),
                is_literal,
                "{value}"
            );
        }
    }
}

#[cfg(test)]
mod depth_tests {
    use super::{canonical_value, strict_shape, ValidationFailure};
    use crate::checked_package::shared::{
        CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCode, JsonPointer,
    };

    fn nested(depth: usize) -> String {
        format!("{}{}", "[".repeat(depth), "]".repeat(depth))
    }

    /// The first pass measures depth in the documented unit.
    #[test]
    fn strict_shape_measures_depth_per_container_and_scalar() {
        for (text, depth) in [
            ("1", 1),
            ("\"x\"", 1),
            ("[]", 1),
            ("{}", 1),
            ("[1]", 2),
            ("{\"a\":1}", 2),
            ("{\"a\":[{\"b\":\"x\"}]}", 4),
            ("[[],[[1]],{\"k\":{}}]", 4),
            ("{\"a\":\"[[[{{\\\"\",\"b\":[true,null,-1.5e3]}", 3),
        ] {
            let shape = strict_shape(text.as_bytes(), u64::MAX).expect("strict JSON");
            assert_eq!(shape.depth, depth, "{text}");
            assert_eq!(shape.first_past_limit, None, "{text}");
        }
    }

    /// A document of exactly the maximum depth is admitted; one level deeper
    /// is incomplete at its own depth.
    #[test]
    fn a_document_at_the_depth_limit_is_admitted() {
        let limits = CheckedPackageReadLimits::bounded();
        assert!(canonical_value(nested(128).as_bytes(), limits).is_ok());
        assert_eq!(
            canonical_value(nested(129).as_bytes(), limits),
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Depth,
                128,
                129_u64,
                JsonPointer::parse(&"/0".repeat(128)),
            ))
        );
    }

    /// Depth far past serde_json's own nesting cap is measured without
    /// exhausting the stack or building a value, and a duplicate member or
    /// syntax error anywhere is still found first.
    #[test]
    fn strict_shape_measures_deep_nesting_and_finds_later_defects() {
        let depth = 100_000;
        let deep = nested(depth);
        let shape = strict_shape(deep.as_bytes(), 128).expect("strict");
        assert_eq!(shape.depth, 100_000);
        assert_eq!(
            shape.first_past_limit,
            JsonPointer::parse(&"/0".repeat(128))
        );
        let duplicate = format!("{{\"a\":{deep},\"a\":1}}");
        assert_eq!(
            strict_shape(duplicate.as_bytes(), 128).err(),
            Some(ValidationFailure::refused(
                CheckedPackageRefusalCode::DuplicateMember,
                JsonPointer::root().key("a"),
            ))
        );
        let malformed = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(
            strict_shape(malformed.as_bytes(), 128).err(),
            Some(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::MalformedWire
            ))
        );
    }

    /// A caller limit above the maximum reads as the maximum, so a document
    /// nested far past it is incomplete at the maximum rather than parsed.
    #[test]
    fn a_caller_limit_above_the_maximum_reads_as_the_maximum() {
        let limits = CheckedPackageReadLimits {
            bytes: 1 << 24,
            depth: u64::MAX,
            ..CheckedPackageReadLimits::bounded()
        };
        let result = canonical_value(nested(100_000).as_bytes(), limits);
        assert_eq!(
            result,
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Depth,
                CheckedPackageReadLimits::MAXIMUM_DEPTH,
                100_000_u64,
                JsonPointer::parse(&"/0".repeat(128)),
            ))
        );
    }
}
