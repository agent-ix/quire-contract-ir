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
use super::v2::{encode, ApplicationOperator, BodyTerm, LiteralKind, PACKAGE_DOMAIN_V2};
use serde::de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
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

/// Measures, parses and canonicalizes untrusted bytes, then hands the value
/// to `admit`.
///
/// Order, as FR-322 states it: byte limit, strict syntax and member
/// validation, depth limit, canonical bytes. [`strict_shape`] checks syntax
/// and duplicate members and measures depth without building a value, so a
/// syntax or duplicate-member defect anywhere in the document is refused
/// before depth is charged, however deep the document is. Only a document
/// within the caller's depth limit is then parsed.
///
/// The value is parsed, re-encoded, admitted and dropped inside `admit`'s
/// closure on a stack sized for the measured depth, so the recursion each of
/// those stages makes over the value cannot exhaust the caller's stack.
/// `admit` is given the value and that depth. The effective depth limit is the
/// caller's, up to [`CheckedPackageReadLimits::MAXIMUM_DEPTH`], which bounds
/// the stack reserved.
pub(super) fn read_value<T>(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    admit: impl FnOnce(Value, u64) -> Result<T, ValidationFailure>,
) -> Result<T, ValidationFailure> {
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
    on_stack_for(shape.depth, || {
        let value = strict_parse(bytes)?;
        require_canonical_bytes(bytes, &value, limits.bytes)?;
        admit(value, shape.depth)
    })
}

/// Requires `bytes` to be `quire-canonical`'s RFC 8785 bytes for `value`, the
/// document they hold, under a byte ceiling of `ceiling`. The value is written
/// through `quire-canonical`'s writer from an explicit heap stack, so the
/// check does not recurse over it. A document whose bytes differ, and one
/// `quire-canonical` refuses to encode (an integer past 2^53, a canonical text
/// past the ceiling) refuse `noncanonical_wire`, with no pointer.
fn require_canonical_bytes(
    bytes: &[u8],
    value: &Value,
    ceiling: u64,
) -> Result<(), ValidationFailure> {
    match encode::value_to_vec(value, ceiling) {
        Ok(canonical) if canonical.as_slice() == bytes => Ok(()),
        _ => Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::NoncanonicalWire,
        )),
    }
}

/// Runs `operation` on a stack with room for the recursion it makes over a
/// value `depth` levels deep, growing onto the heap when the current stack
/// lacks it. `depth` is at most [`CheckedPackageReadLimits::MAXIMUM_DEPTH`]
/// for any value the reader admitted, so the reservation is bounded.
pub(super) fn on_stack_for<T>(depth: u64, operation: impl FnOnce() -> T) -> T {
    let stack = STACK_BASE.saturating_add(
        usize::try_from(depth.min(CheckedPackageReadLimits::MAXIMUM_DEPTH))
            .unwrap_or(usize::MAX)
            .saturating_mul(STACK_PER_LEVEL),
    );
    stacker::maybe_grow(stack, stack, operation)
}

/// [`read_value`] handing back the value itself, for tests whose documents are
/// shallow enough to drop on the test's own stack.
#[cfg(test)]
pub(super) fn canonical_value(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
) -> Result<Value, ValidationFailure> {
    read_value(bytes, limits, |value, _| Ok(value))
}

/// Empties `value`, dropping its descendants iteratively. A deep value's
/// default drop recurses once per level, so a package holding one must take
/// it apart with this before it is dropped on an ordinary stack.
pub(super) fn dismantle(value: &mut Value) {
    let mut pending = vec![std::mem::take(value)];
    while let Some(mut next) = pending.pop() {
        match &mut next {
            Value::Array(elements) => pending.append(elements),
            Value::Object(members) => {
                pending.extend(
                    std::mem::take(members)
                        .into_iter()
                        .map(|(_, member)| member),
                );
            }
            _ => {}
        }
    }
}

/// Stack reserved for any read, before its depth is counted.
const STACK_BASE: usize = 256 * 1024;

/// Stack reserved for each level of nesting: an upper bound on the frames the
/// stages that recurse over a parsed value (encoding, decoding, comparison,
/// term validation, drop) use per level.
const STACK_PER_LEVEL: usize = 4 * 1024;

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

/// Checks one locked artifact's digest domain and shape. `at` names the
/// artifact reference; each refusal points at the member it is about.
// Intake check of a locked artifact's digest domain and digest text.
pub(super) fn validate_locked_artifact(
    artifact: &CheckedArtifactRef,
    expected_domain: &str,
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
    Ok(())
}

/// Requires the source map to be exactly the graph's occurrence set, with
/// nonempty, locked, current, non-overlapping half-open regions.
pub(super) fn validate_source_map_entries<'a>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a [CheckedOccurrence])>,
    source_map: &[CheckedSourceMapEntry],
    locked_sources: &[CheckedArtifactRef],
    limits: CheckedPackageReadLimits,
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
            validate_locked_artifact(&region.source, "quire.source.bytes/v1", &source_pointer)?;
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
/// by the depth limit `read_value` already enforced. A refusal points at
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
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let parsed = StrictSeed::root(&duplicate).deserialize(&mut deserializer);
    finish(parsed, duplicate)
}

/// Parses `input` into a value without serde_json's nesting cap, on a stack
/// that grows with the nesting. The caller must already have charged the
/// document's depth.
fn strict_parse(input: &[u8]) -> Result<Value, ValidationFailure> {
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let parsed = StrictSeed::root(&duplicate)
        .deserialize(serde_stacker::Deserializer::new(&mut deserializer));
    finish(parsed, duplicate)
}

/// Maps a finished parse to the reader's outcome. Trailing bytes are left for
/// the canonical-bytes comparison to refuse.
fn finish<T>(
    parsed: Result<T, serde_json::Error>,
    duplicate: RefCell<Option<JsonPointer>>,
) -> Result<T, ValidationFailure> {
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

/// What the first pass learns about a document.
struct Shape {
    /// The document's nesting depth: a container is one level and a scalar
    /// one level below the container holding it, so `1`, `[]` and `{}` are
    /// depth 1, and `[1]` and `{"a":1}` are depth 2.
    /// [`CheckedPackageReadLimits::depth`] is charged in this unit.
    depth: u64,
    /// The position of the first value, in document order, nested deeper than
    /// the depth limit the pass was given.
    first_past_limit: Option<JsonPointer>,
}

/// One container the scan is inside.
enum Open {
    Array {
        /// The element being read.
        index: usize,
    },
    Object {
        /// The member being read.
        key: String,
        /// Every member name met so far, `key` included.
        names: BTreeSet<String>,
        /// Whether `key` repeats an earlier member.
        repeated: bool,
    },
}

impl Open {
    fn step(&self) -> Step<'_> {
        match self {
            Self::Array { index } => Step::Index(*index),
            Self::Object { key, .. } => Step::Key(key),
        }
    }
}

/// The first pass: the strict grammar [`StrictSeed`] parses — syntax, finite
/// numbers, duplicate members — and the document's depth, read iteratively
/// with an explicit stack of open containers, so cost is linear in the input
/// and independent of nesting. Scalars and member names are read by
/// serde_json itself, so their grammar is exactly the parser's. No value is
/// built. Trailing bytes are left for the canonical-bytes comparison to
/// refuse.
fn strict_shape(input: &[u8], depth_limit: u64) -> Result<Shape, ValidationFailure> {
    let malformed = || ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire);
    let mut scan = Scan {
        input,
        at: 0,
        open: Vec::new(),
    };
    let mut depth = 0_u64;
    let mut first_past_limit = None;
    loop {
        // A value starts here.
        let level = count(scan.open.len()).saturating_add(1);
        depth = depth.max(level);
        if level > depth_limit && first_past_limit.is_none() {
            first_past_limit = Some(scan.pointer());
        }
        let opens = match scan.peek().ok_or_else(malformed)? {
            b'[' => {
                scan.at += 1;
                if scan.peek() == Some(b']') {
                    scan.at += 1;
                    false
                } else {
                    scan.open.push(Open::Array { index: 0 });
                    true
                }
            }
            b'{' => {
                scan.at += 1;
                if scan.peek() == Some(b'}') {
                    scan.at += 1;
                    false
                } else {
                    let key = scan.member_name()?;
                    if key == SERDE_JSON_NUMBER_TOKEN {
                        scan.number_token()?;
                        false
                    } else {
                        scan.open.push(Open::Object {
                            names: BTreeSet::from([key.clone()]),
                            key,
                            repeated: false,
                        });
                        true
                    }
                }
            }
            _ => {
                scan.scalar()?;
                false
            }
        };
        if opens {
            continue;
        }
        // The value just read is complete: close every container it ends,
        // then read the next sibling, if any.
        loop {
            let Some(top) = scan.open.last() else {
                return Ok(Shape {
                    depth,
                    first_past_limit,
                });
            };
            let in_array = matches!(top, Open::Array { .. });
            if matches!(top, Open::Object { repeated: true, .. }) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::DuplicateMember,
                    scan.pointer(),
                ));
            }
            let byte = scan.peek().ok_or_else(malformed)?;
            scan.at += 1;
            match (in_array, byte) {
                (true, b']') | (false, b'}') => {
                    scan.open.pop();
                    continue;
                }
                (_, b',') => {}
                _ => return Err(malformed()),
            }
            if in_array {
                if let Some(Open::Array { index }) = scan.open.last_mut() {
                    *index += 1;
                }
            } else {
                let next = scan.member_name()?;
                if let Some(Open::Object {
                    key,
                    names,
                    repeated,
                }) = scan.open.last_mut()
                {
                    *repeated = !names.insert(next.clone());
                    *key = next;
                }
            }
            break;
        }
    }
}

/// The scan's position in the input and in the document.
struct Scan<'i> {
    input: &'i [u8],
    at: usize,
    open: Vec<Open>,
}

impl Scan<'_> {
    /// The next byte that is not JSON whitespace, left unconsumed.
    fn peek(&mut self) -> Option<u8> {
        while let Some(&byte) = self.input.get(self.at) {
            if !matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                return Some(byte);
            }
            self.at += 1;
        }
        None
    }

    /// The pointer of the value being read.
    fn pointer(&self) -> JsonPointer {
        pointer_from_steps(self.open.iter().map(Open::step))
    }

    /// Reads one JSON token with serde_json, as `T`, and advances past it.
    fn token<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, ValidationFailure> {
        let mut tokens =
            serde_json::Deserializer::from_slice(&self.input[self.at..]).into_iter::<T>();
        let token = tokens.next().and_then(Result::ok).ok_or_else(|| {
            ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire)
        })?;
        self.at += tokens.byte_offset();
        Ok(token)
    }

    /// Reads a scalar value as the strict grammar admits it.
    fn scalar(&mut self) -> Result<(), ValidationFailure> {
        self.token::<StrictScalar>().map(|_| ())
    }

    /// Reads an object member name and the `:` after it.
    fn member_name(&mut self) -> Result<String, ValidationFailure> {
        if self.peek() != Some(b'"') {
            return Err(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::MalformedWire,
            ));
        }
        let name = self.token::<String>()?;
        if self.peek() != Some(b':') {
            return Err(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::MalformedWire,
            ));
        }
        self.at += 1;
        Ok(name)
    }

    /// Reads the rest of an object whose first member is the number token:
    /// its string value, which must spell a finite number, and the closing
    /// brace. The object reads as that number.
    fn number_token(&mut self) -> Result<(), ValidationFailure> {
        let malformed =
            || ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire);
        if self.peek() != Some(b'"') {
            return Err(malformed());
        }
        let digits = self.token::<String>()?;
        number_from_token::<serde_json::Error>(&digits).map_err(|_| malformed())?;
        if self.peek() != Some(b'}') {
            return Err(malformed());
        }
        self.at += 1;
        Ok(())
    }
}

/// A scalar read under the strict grammar, for [`Scan::scalar`].
struct StrictScalar;

impl<'de> Deserialize<'de> for StrictScalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let duplicate = RefCell::new(None);
        StrictSeed::root(&duplicate)
            .deserialize(deserializer)
            .map(|_| StrictScalar)
    }
}

/// Parses one JSON value at position `at`, recording the pointer of the first
/// repeated member it meets in `duplicate`.
#[derive(Clone, Copy)]
struct StrictSeed<'s> {
    at: &'s Trail<'s>,
    duplicate: &'s RefCell<Option<JsonPointer>>,
}

impl<'s> StrictSeed<'s> {
    fn root(duplicate: &'s RefCell<Option<JsonPointer>>) -> Self {
        Self {
            at: &Trail::Base(&[]),
            duplicate,
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
    // Canonical-JSON front end: reads serde_json's number token.
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
        for (bytes, integer, canonical) in [
            (&br#"{"v":1.5}"#[..], false, true),
            // Not canonical RFC 8785 bytes: `2.0` is spelled `2`, and an
            // integer past 2^53 has no encoding (FR-038-AC-79).
            (br#"{"v":2.0}"#, false, false),
            (br#"{"v":-7}"#, true, true),
            (br#"{"v":18446744073709551615}"#, true, false),
        ] {
            let text = String::from_utf8_lossy(bytes);
            assert_eq!(canonical_value(bytes, limits).is_ok(), canonical, "{text}");
            let value: Value = serde_json::from_slice(bytes).expect("number document parses");
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
    use super::{canonical_value, read_value, strict_json_value, strict_shape, ValidationFailure};
    use crate::checked_package::shared::{
        CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCode, JsonPointer,
    };
    use std::time::Instant;

    fn nested(depth: usize) -> String {
        format!("{}{}", "[".repeat(depth), "]".repeat(depth))
    }

    fn limits(depth: u64) -> CheckedPackageReadLimits {
        CheckedPackageReadLimits {
            bytes: 1 << 24,
            depth,
            ..CheckedPackageReadLimits::bounded()
        }
    }

    fn malformed() -> ValidationFailure {
        ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire)
    }

    /// The first pass measures depth in the documented unit.
    ///
    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_strict_shape_measures_depth_per_container_and_scalar() {
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
            (" [ 1 , { \"a\" : [ ] } ] ", 3),
        ] {
            let shape = strict_shape(text.as_bytes(), u64::MAX).expect("strict JSON");
            assert_eq!(shape.depth, depth, "{text}");
            assert_eq!(shape.first_past_limit, None, "{text}");
        }
    }

    /// The iterative first pass accepts and refuses exactly what the strict
    /// parser does, with the same refusal.
    ///
    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_strict_shape_agrees_with_the_strict_parser() {
        for text in [
            "",
            " ",
            "null",
            "true",
            "tru",
            "nul1",
            "01",
            "-",
            "1.",
            "1e",
            "1e999",
            "-1.5E+3",
            "18446744073709551616",
            "\"a\\u0041\\n\"",
            "\"a\\ud800\"",
            "\"a\\x\"",
            "\"a\nb\"",
            "\"unterminated",
            "[",
            "]",
            "[1,]",
            "[,1]",
            "[1 2]",
            "[1,2]",
            "{}",
            "{,}",
            "{\"a\"}",
            "{\"a\":}",
            "{a:1}",
            "{\"a\":1,}",
            "{\"a\":1 \"b\":2}",
            "{\"a\":1,\"b\":{\"c\":[1,{\"d\":2,\"d\":3}]}}",
            "{\"a\":1,\"a\":2,\"b\":x}",
            "{\"a\":1,\"a\":[}",
            "{\"a\":{\"b\":1,\"b\":2},\"a\":3}",
            "[{\"k\":1},{\"k\":2,\"k\":3}]",
            "{\"$serde_json::private::Number\":\"12\"}",
            "{\"$serde_json::private::Number\":\"1e999\"}",
            "{\"$serde_json::private::Number\":\"x\"}",
            "{\"$serde_json::private::Number\":1}",
            "{\"$serde_json::private::Number\":\"1\",\"b\":2}",
            "{\"b\":1,\"$serde_json::private::Number\":\"1\"}",
            "[1] trailing",
            "[1]]",
        ] {
            let shape = strict_shape(text.as_bytes(), u64::MAX).map(|shape| shape.depth);
            let parsed = strict_json_value(text.as_bytes());
            assert_eq!(shape.is_ok(), parsed.is_ok(), "{text:?}");
            if let (Err(shape), Err(parsed)) = (&shape, &parsed) {
                assert_eq!(shape, parsed, "{text:?}");
            }
        }
    }

    /// A document at the caller's depth limit is admitted; one level deeper is
    /// incomplete, reporting the caller's limit and the measured depth, at
    /// the first value past the limit.
    ///
    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_a_document_at_the_callers_limit_is_admitted() {
        for limit in [3_u64, 128, 129, 200, 1_000] {
            let depth = usize::try_from(limit).expect("small");
            assert!(
                canonical_value(nested(depth).as_bytes(), limits(limit)).is_ok(),
                "{limit}"
            );
            assert_eq!(
                canonical_value(nested(depth + 1).as_bytes(), limits(limit)),
                Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Depth,
                    limit,
                    limit + 1,
                    JsonPointer::parse(&"/0".repeat(depth)),
                )),
                "{limit}"
            );
        }
    }

    /// The reader's ceiling is exactly 16,384 levels: that many is admitted
    /// under any caller limit, one more is incomplete at 16,384.
    ///
    /// Tracing: TC-048
    #[test]
    fn tc_048_the_reader_ceiling_is_sixteen_thousand_three_hundred_eighty_four_levels() {
        assert_eq!(CheckedPackageReadLimits::MAXIMUM_DEPTH, 16_384);
        assert!(read_value(nested(16_384).as_bytes(), limits(u64::MAX), |_, _| Ok(())).is_ok());
        assert_eq!(
            read_value(nested(16_385).as_bytes(), limits(u64::MAX), |_, _| Ok(())),
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Depth,
                16_384,
                16_385_u64,
                JsonPointer::parse(&"/0".repeat(16_384)),
            ))
        );
    }

    /// Nesting far past serde_json's own cap is decided by the caller's limit
    /// (at or below the reader's ceiling) without exhausting the stack, and a
    /// duplicate member or syntax error anywhere is still found first.
    ///
    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_deep_nesting_is_decided_by_the_callers_limit() {
        let ceiling = CheckedPackageReadLimits::MAXIMUM_DEPTH;
        let ceiling_len = usize::try_from(ceiling).expect("small");
        let at_ceiling = nested(ceiling_len);
        assert!(read_value(at_ceiling.as_bytes(), limits(u64::MAX), |_, _| Ok(())).is_ok());
        assert_eq!(
            read_value(at_ceiling.as_bytes(), limits(ceiling - 1), |_, _| Ok(())),
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Depth,
                ceiling - 1,
                ceiling,
                JsonPointer::parse(&"/0".repeat(ceiling_len - 1)),
            ))
        );
        let depth = 2_000_000;
        let deep = nested(depth);
        let over = strict_shape(deep.as_bytes(), 128).expect("strict");
        assert_eq!(over.depth, 2_000_000);
        assert_eq!(over.first_past_limit, JsonPointer::parse(&"/0".repeat(128)));
        let duplicate = format!("{{\"a\":{deep},\"a\":1}}");
        assert_eq!(
            read_value(duplicate.as_bytes(), limits(128), |_, _| Ok(())),
            Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::DuplicateMember,
                JsonPointer::root().key("a"),
            ))
        );
        let broken = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(
            read_value(broken.as_bytes(), limits(128), |_, _| Ok(())),
            Err(malformed())
        );
    }

    /// Past the reader's ceiling the ceiling is what is charged and reported,
    /// for any caller limit, and the document is not parsed or given a stack
    /// of its own depth.
    ///
    /// Tracing: TC-048
    #[test]
    fn tc_048_nesting_past_the_ceiling_is_charged_at_the_ceiling() {
        let ceiling = CheckedPackageReadLimits::MAXIMUM_DEPTH;
        let deep = nested(2_000_000);
        assert_eq!(
            read_value(deep.as_bytes(), limits(u64::MAX), |_, _| Ok(())),
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Depth,
                ceiling,
                2_000_000_u64,
                JsonPointer::parse(&"/0".repeat(usize::try_from(ceiling).expect("small"))),
            ))
        );
    }

    /// The first pass costs time linear in the input: a syntax error at the
    /// bottom of a document half a million levels deep, which a recursive
    /// parser reports once per enclosing level, is refused in time that grows
    /// with the depth, not with its square. Quadrupling the depth takes
    /// about four times as long; the bound of eight separates that from
    /// sixteen, on the fastest of three runs so a busy machine does not move it.
    ///
    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_a_deep_syntax_error_is_refused_in_linear_time() {
        let fastest = |depth: usize| {
            let broken = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
            (0..3)
                .map(|_| {
                    let started = Instant::now();
                    let result = read_value(broken.as_bytes(), limits(u64::MAX), |_, _| Ok(()));
                    let elapsed = started.elapsed();
                    assert_eq!(result, Err(malformed()));
                    elapsed
                })
                .min()
                .expect("three runs")
        };
        let shallow = fastest(131_000);
        let deep = fastest(524_000);
        assert!(
            deep.as_secs_f64() < shallow.as_secs_f64() * 8.0,
            "131000 levels took {shallow:?}, 524000 took {deep:?}"
        );
    }
}
