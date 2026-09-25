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
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
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

/// Measures, parses and canonicalizes untrusted bytes exactly once.
///
/// Order: byte limit, strict JSON (duplicate members), depth limit, canonical
/// bytes.
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
    let value = strict_json_value(bytes)?;
    let depth = json_depth(&value);
    if exceeds(depth, limits.depth) {
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Depth,
            limits.depth,
            depth,
            first_value_at_level(&value, limits.depth.saturating_add(1)),
        ));
    }
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
    let term = match object.get("term") {
        Some(Value::String(term)) => term,
        Some(_) => return Err(invalid(&at.key("term"))),
        None => return Err(invalid(at)),
    };
    match term.as_str() {
        "literal"
            if exact_members(object, &["term", "type", "value_kind", "value"])
                && object
                    .get("value_kind")
                    .and_then(Value::as_str)
                    .is_some_and(is_literal_kind)
                && object
                    .get("value")
                    .is_some_and(|value| is_literal_value(value, grammar)) =>
        {
            visit_member(
                object,
                "type",
                ReferenceMember::Type,
                is_body_root,
                at,
                visit,
            )
        }
        "reference" if exact_members(object, &["term", "target"]) => visit_member(
            object,
            "target",
            ReferenceMember::Target,
            is_body_root,
            at,
            visit,
        ),
        "application"
            if exact_members(
                object,
                &["term", "operator", "operation", "result_type", "arguments"],
            ) && object
                .get("operator")
                .and_then(Value::as_str)
                .is_some_and(is_operator) =>
        {
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
        "aggregate" if exact_members(object, &["term", "members"]) => {
            visit_terms(object, "members", grammar, at, visit)
        }
        "binding"
            if exact_members(object, &["term", "name", "value"])
                && object
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(is_nonempty) =>
        {
            let value = object.get("value").unwrap_or(&Value::Null);
            validate_term(value, grammar, false, &at.key("value"), visit)
                .map(|work| work.saturating_add(1))
        }
        _ => Err(invalid(at)),
    }
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

fn is_literal_kind(value: &str) -> bool {
    matches!(
        value,
        "boolean"
            | "integer"
            | "rational"
            | "decimal"
            | "float32_bits"
            | "float64_bits"
            | "text"
            | "enum"
            | "none"
    )
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

fn is_operator(value: &str) -> bool {
    matches!(
        value,
        "call"
            | "unary"
            | "binary"
            | "conditional"
            | "let"
            | "quantify"
            | "collection"
            | "query"
            | "convert"
            | "pre"
            | "present"
            | "value"
            | "deref"
            | "reaches"
            | "temporal"
            | "protocol_control"
            | "state_transition"
            | "claim"
    )
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

/// Parses strict JSON: a repeated object member refuses as
/// `duplicate_member` at that member, any other syntax error as
/// `malformed_wire` with no pointer. Nesting is bounded by serde_json's own
/// recursion limit.
fn strict_json_value(input: &[u8]) -> Result<Value, ValidationFailure> {
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let seed = StrictSeed {
        at: &Trail::Base(&[]),
        duplicate: &duplicate,
    };
    // Trailing bytes are left for the canonical-bytes comparison to refuse.
    let parsed = seed.deserialize(&mut deserializer);
    match (parsed, duplicate.into_inner()) {
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

/// Parses one JSON value at position `at`, recording the pointer of the first
/// repeated member it meets in `duplicate`.
#[derive(Clone, Copy)]
struct StrictSeed<'s> {
    at: &'s Trail<'s>,
    duplicate: &'s RefCell<Option<JsonPointer>>,
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

impl<'de> DeserializeSeed<'de> for StrictSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StrictSeed<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("strict JSON value")
    }
    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }
    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }
    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A>(self, mut access: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        loop {
            let at = self.at.index(values.len());
            let seed = StrictSeed {
                at: &at,
                duplicate: self.duplicate,
            };
            match access.next_element_seed(seed)? {
                Some(value) => values.push(value),
                None => return Ok(Value::Array(values)),
            }
        }
    }
    fn visit_map<A>(self, mut access: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut map = Map::new();
        let mut first = true;
        while let Some(key) = access.next_key::<String>()? {
            if first && key == SERDE_JSON_NUMBER_TOKEN {
                let digits = access.next_value::<String>()?;
                return number_from_token(&digits);
            }
            first = false;
            let at = self.at.key(&key);
            let value = access.next_value_seed(StrictSeed {
                at: &at,
                duplicate: self.duplicate,
            })?;
            if map.contains_key(&key) {
                self.duplicate.replace(Some(at.pointer()));
                return Err(serde::de::Error::custom("duplicate JSON member"));
            }
            map.insert(key, value);
        }
        Ok(Value::Object(map))
    }
}

pub(super) fn json_depth(value: &Value) -> u64 {
    match value {
        Value::Array(values) => values
            .iter()
            .map(json_depth)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
        Value::Object(values) => values
            .values()
            .map(json_depth)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
        _ => 1,
    }
}

/// The children of one container, yielded with the step that reaches each.
enum Children<'a> {
    Array(std::iter::Enumerate<std::slice::Iter<'a, Value>>),
    Object(serde_json::map::Iter<'a>),
    None,
}

impl<'a> Children<'a> {
    fn of(value: &'a Value) -> Self {
        match value {
            Value::Array(values) => Self::Array(values.iter().enumerate()),
            Value::Object(members) => Self::Object(members.iter()),
            _ => Self::None,
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = (Step<'a>, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Array(values) => values
                .next()
                .map(|(index, value)| (Step::Index(index), value)),
            Self::Object(members) => members
                .next()
                .map(|(key, value)| (Step::Key(key.as_str()), value)),
            Self::None => None,
        }
    }
}

/// The pointer of the first value, in document order, at nesting `level`
/// (the document itself is level 1), found with an explicit stack. This is
/// where a depth charge one past the limit failed.
fn first_value_at_level(root: &Value, level: u64) -> Option<JsonPointer> {
    if level <= 1 {
        return Some(JsonPointer::root());
    }
    let mut steps: Vec<Step<'_>> = Vec::new();
    let mut stack = vec![Children::of(root)];
    while let Some(children) = stack.last_mut() {
        match children.next() {
            Some((step, child)) => {
                steps.push(step);
                // `stack` holds the child's ancestors, so its length is the
                // child's level minus one.
                if count(stack.len()).saturating_add(1) >= level {
                    return Some(pointer_from_steps(steps));
                }
                stack.push(Children::of(child));
            }
            None => {
                stack.pop();
                steps.pop();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_value, first_difference, first_value_at_level, is_literal_value, Step,
        TermGrammar, Trail, ValidationFailure,
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
    /// level deeper than the limit; the scan uses its own stack.
    ///
    /// Tracing: TC-048, FR-038-AC-26
    #[test]
    fn tc_048_depth_is_charged_at_the_first_value_past_the_limit() {
        let value = json!({"a": [1, {"b": 2}], "c": {"d": {"e": 3}}});
        let at = |level| first_value_at_level(&value, level).map(|p| p.as_str().to_owned());
        assert_eq!(at(1).as_deref(), Some(""));
        assert_eq!(at(2).as_deref(), Some("/a"));
        assert_eq!(at(3).as_deref(), Some("/a/0"));
        assert_eq!(at(4).as_deref(), Some("/a/1/b"));
        assert_eq!(at(5), None);
        let limits = CheckedPackageReadLimits {
            depth: 3,
            ..CheckedPackageReadLimits::bounded()
        };
        let bytes = serde_json::to_vec(&value).expect("bytes");
        assert_eq!(
            canonical_value(&bytes, limits),
            Err(ValidationFailure::incomplete(
                crate::checked_package::shared::CheckedPackageLimit::Depth,
                3,
                4_u64,
                JsonPointer::parse("/a/1/b"),
            ))
        );
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
