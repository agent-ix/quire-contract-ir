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
    CheckedSourceMapEntry, CheckedSourceRef, JsonPointer, RefusalFields,
};
use super::terms::{subterms, At, Cursor};
use super::v2::{ApplicationOperator, BodyTerm, LiteralKind, PACKAGE_DOMAIN_V2};
use quire_walk::{walk, Children, Walk};
use serde::de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::ControlFlow;

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
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code,
            path: Some(path),
            cause: None,
            locus: None,
            contract_version: None,
            document_pointer: None,
        }))
    }

    /// `noncanonical_wire` about the number at `document_pointer` inside a
    /// supplied model document, located at the selection row's `path`, with
    /// the `cause` the number earned.
    pub(super) fn refused_number_in_document(
        path: JsonPointer,
        document_pointer: JsonPointer,
        cause: CheckedPackageRefusalCause,
    ) -> Self {
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code: CheckedPackageRefusalCode::NoncanonicalWire,
            path: Some(path),
            cause: Some(cause),
            locus: None,
            contract_version: None,
            document_pointer: Some(document_pointer),
        }))
    }

    /// A refusal about the value at `path` carrying the cause this stage
    /// determined, not located at any graph node.
    pub(super) fn refused_because(
        code: CheckedPackageRefusalCode,
        path: JsonPointer,
        cause: CheckedPackageRefusalCause,
    ) -> Self {
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code,
            path: Some(path),
            cause: Some(cause),
            locus: None,
            contract_version: None,
            document_pointer: None,
        }))
    }

    /// A refusal about the byte stream rather than any value: malformed JSON
    /// or non-canonical bytes.
    pub(super) fn refused_bytes(code: CheckedPackageRefusalCode) -> Self {
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code,
            path: None,
            cause: None,
            locus: None,
            contract_version: None,
            document_pointer: None,
        }))
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
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code,
            path: Some(path),
            cause,
            locus: Some(locus),
            contract_version: None,
            document_pointer: None,
        }))
    }

    /// A stale node key with the different key already derived by this stage.
    pub(super) fn refused_stale_node_key(
        path: JsonPointer,
        locus: CheckedNodeId,
        expected_node_id: CheckedNodeId,
    ) -> Self {
        Self::Refused(CheckedPackageRefusal::stale_node_key(
            path,
            locus,
            expected_node_id,
        ))
    }

    /// `unknown_contract_version` for the version string actually read at
    /// the document's `contract_version` member.
    pub(super) fn unknown_contract_version(version: &str) -> Self {
        Self::Refused(CheckedPackageRefusal::new(RefusalFields {
            code: CheckedPackageRefusalCode::UnknownContractVersion,
            path: Some(JsonPointer::root().key("contract_version")),
            cause: None,
            locus: None,
            contract_version: Some(version.into()),
            document_pointer: None,
        }))
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

    /// The steps from the document root to this position, in order. Walks the
    /// chain iteratively.
    pub(super) fn steps(&self) -> Vec<Step<'a>> {
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
        base.iter().chain(below.iter().rev()).copied().collect()
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
/// validation, canonical bytes. There is no depth limit (FR-038-AC-117).
/// [`strict_shape`] checks syntax and duplicate members without building a
/// value, so a syntax or duplicate-member defect anywhere in the document is
/// refused first, however deep the document is. The document is then parsed
/// under `serde_json`'s own recursion limit of 128, which refuses a deeper
/// document as `malformed_wire` with no pointer before it recurses deeper, so
/// the parse never needs more stack than 128 levels. The closed body grammar
/// ("The flat wire") fixes the depth of every in-grammar package at far less.
pub(super) fn read_value<T>(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    admit: impl FnOnce(Value) -> Result<T, ValidationFailure>,
) -> Result<T, ValidationFailure> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Bytes,
            limits.bytes,
            bytes.len(),
            None,
        ));
    }
    strict_shape(bytes)?;
    let value = strict_parse(bytes)?;
    require_canonical_bytes(bytes, &value, limits.bytes)?;
    admit(value)
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
    // An integer spelling past u64 or i64 parses as a float, whose RFC 8785
    // text is the same digits, so the byte comparison alone admits it: refuse
    // every whole number past 2^53 that is not an `i64` or `u64` here.
    if holds_float_integer_past_2_pow_53(value) {
        return Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::NoncanonicalWire,
        ));
    }
    match quire_canonical::to_vec(value, quire_canonical::Limits::new(ceiling)) {
        Ok(canonical) if canonical.as_slice() == bytes => Ok(()),
        _ => Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::NoncanonicalWire,
        )),
    }
}

/// The V2 intake checks canonical source text without building a document
/// value or asking the canonical writer to buffer the enclosing object. That
/// writer buffers every open object's members, including the entire package;
/// each scalar below is instead checked independently by the same encoder.
pub(super) fn read_typed_prefix(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
) -> Result<(), ValidationFailure> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Bytes,
            limits.bytes,
            bytes.len(),
            None,
        ));
    }
    // Keep serde_json's 128-level refusal before canonicality. The strict
    // scan completes first, so a later syntax or duplicate defect still wins.
    if strict_shape_depth(bytes)? >= 128 {
        return Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::MalformedWire,
        ));
    }
    require_canonical_tokens(bytes)
}

/// A sink comparing one scalar's canonical bytes with its original token.
struct TokenSink<'a> {
    original: &'a [u8],
    at: usize,
    agrees: bool,
}

impl quire_canonical::Sink for TokenSink<'_> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
        let end = self.at.checked_add(bytes.len());
        if end.and_then(|end| self.original.get(self.at..end)) != Some(bytes) {
            self.agrees = false;
        }
        self.at = end.unwrap_or(usize::MAX);
        Ok(())
    }
}

fn canonical_token(value: &Value, original: &[u8]) -> bool {
    let mut sink = TokenSink {
        original,
        at: 0,
        agrees: true,
    };
    quire_canonical::encode(
        &mut sink,
        value,
        quire_canonical::Limits::new(u64::try_from(original.len()).unwrap_or(u64::MAX)),
    )
    .is_ok()
        && sink.agrees
        && sink.at == original.len()
}

enum CanonicalOpen {
    Array,
    Object { previous: String },
}

/// The strict scan has already proved syntax and duplicate-member validity.
/// This second, allocation-bounded scan checks the ordering of object members
/// and compares each scalar spelling with `quire-canonical`'s own spelling.
fn require_canonical_tokens(input: &[u8]) -> Result<(), ValidationFailure> {
    let noncanonical =
        || ValidationFailure::refused_bytes(CheckedPackageRefusalCode::NoncanonicalWire);
    let mut scan = Scan {
        input,
        at: 0,
        open: Vec::new(),
    };
    let mut open = Vec::new();
    loop {
        match input.get(scan.at).copied().ok_or_else(noncanonical)? {
            b'[' => {
                scan.at += 1;
                if input.get(scan.at) != Some(&b']') {
                    open.push(CanonicalOpen::Array);
                    continue;
                }
                scan.at += 1;
            }
            b'{' => {
                scan.at += 1;
                if input.get(scan.at) != Some(&b'}') {
                    let previous = canonical_member(&mut scan)?;
                    // With arbitrary-precision serde_json, this private key
                    // as the first member decodes as a number, not an object.
                    if previous == SERDE_JSON_NUMBER_TOKEN {
                        return Err(noncanonical());
                    }
                    open.push(CanonicalOpen::Object { previous });
                    continue;
                }
                scan.at += 1;
            }
            _ => {
                let start = scan.at;
                let mut scalar = scan.token::<Value>().map_err(|_| noncanonical())?;
                if matches!(input[start], b'-' | b'0'..=b'9') {
                    let text =
                        std::str::from_utf8(&input[start..scan.at]).map_err(|_| noncanonical())?;
                    scalar =
                        number_from_token::<serde_json::Error>(text).map_err(|_| noncanonical())?;
                }
                if holds_float_integer_past_2_pow_53(&scalar)
                    || !canonical_token(&scalar, &input[start..scan.at])
                {
                    return Err(noncanonical());
                }
            }
        }
        loop {
            match open.last_mut() {
                None => {
                    return if scan.at == input.len() {
                        Ok(())
                    } else {
                        Err(noncanonical())
                    };
                }
                Some(CanonicalOpen::Array) => match input.get(scan.at) {
                    Some(b']') => {
                        scan.at += 1;
                        open.pop();
                    }
                    Some(b',') => {
                        scan.at += 1;
                        break;
                    }
                    _ => return Err(noncanonical()),
                },
                Some(CanonicalOpen::Object { previous }) => match input.get(scan.at) {
                    Some(b'}') => {
                        scan.at += 1;
                        open.pop();
                    }
                    Some(b',') => {
                        scan.at += 1;
                        let next = canonical_member(&mut scan)?;
                        if previous.encode_utf16().cmp(next.encode_utf16()).is_ge() {
                            return Err(noncanonical());
                        }
                        *previous = next;
                        break;
                    }
                    _ => return Err(noncanonical()),
                },
            }
        }
    }
}

fn canonical_member(scan: &mut Scan<'_>) -> Result<String, ValidationFailure> {
    let noncanonical =
        || ValidationFailure::refused_bytes(CheckedPackageRefusalCode::NoncanonicalWire);
    let start = scan.at;
    let member = scan.member_name().map_err(|_| noncanonical())?;
    // `member_name` consumed the colon; the key token ends one byte before it.
    let key_end = scan.at.checked_sub(1).ok_or_else(noncanonical)?;
    if scan.input.get(key_end) != Some(&b':')
        || !canonical_token(&Value::String(member.clone()), &scan.input[start..key_end])
    {
        return Err(noncanonical());
    }
    Ok(member)
}

/// Whether `value` holds, at any depth, a number that is no `i64` or `u64` but
/// is whole with a magnitude above 2^53: an integer spelled past the 64-bit
/// range. Walks with an explicit stack.
fn holds_float_integer_past_2_pow_53(value: &Value) -> bool {
    const MAXIMUM_INTEGER: f64 = 9_007_199_254_740_992.0;
    let mut pending = vec![value];
    while let Some(next) = pending.pop() {
        match next {
            Value::Number(number) => {
                if number.as_i64().is_none()
                    && number.as_u64().is_none()
                    && number
                        .as_f64()
                        .is_some_and(|float| float.fract() == 0.0 && float.abs() > MAXIMUM_INTEGER)
                {
                    return true;
                }
            }
            Value::Array(items) => pending.extend(items),
            Value::Object(members) => pending.extend(members.values()),
            Value::Null | Value::Bool(_) | Value::String(_) => {}
        }
    }
    false
}

/// [`read_value`] handing back the value itself, for tests.
#[cfg(test)]
pub(super) fn canonical_value(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
) -> Result<Value, ValidationFailure> {
    read_value(bytes, limits, Ok)
}

/// Decodes a closed wire value, classifying closed-schema member violations
/// and locating each at the position the decoder had reached: an unknown
/// member at that member, a missing member at the object lacking it, and a
/// wrongly typed value at that value.
pub(super) fn decode_closed<T: DeserializeOwned>(value: &Value) -> Result<T, ValidationFailure> {
    serde_path_to_error::deserialize::<_, T>(value).map_err(classify_closed_error)
}

/// Decode a closed package straight from its validated bytes. This uses the
/// same error classification and pointer mapping as [`decode_closed`], without
/// first materializing the complete package as a `Value`.
pub(super) fn decode_closed_bytes<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, ValidationFailure> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    serde_path_to_error::deserialize::<_, T>(&mut deserializer).map_err(classify_closed_error)
}

fn classify_closed_error<E: fmt::Display>(
    error: serde_path_to_error::Error<E>,
) -> ValidationFailure {
    let code = if error.inner().to_string().contains("unknown field") {
        CheckedPackageRefusalCode::UnknownMember
    } else {
        CheckedPackageRefusalCode::MalformedWire
    };
    ValidationFailure::refused(code, decoder_pointer(JsonPointer::root(), error.path()))
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

pub(super) fn artifact_locator(value: &CheckedSourceRef) -> CheckedArtifactLocator {
    CheckedArtifactLocator {
        authority: value.authority.clone(),
        identity: value.identity.clone(),
        domain: value.digest_domain.clone(),
    }
}

/// Checks one locked source row's digest domain and shape. `at` names the
/// source reference; each refusal points at the member it is about. The digest
/// domain is checked first, so a row with another domain and also an empty
/// member or a non-hex digest refuses for the domain.
// Intake check of a locked source's digest domain and digest text.
pub(super) fn validate_locked_artifact(
    artifact: &CheckedSourceRef,
    expected_domain: &str,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let refuse = |code, member: &str| ValidationFailure::refused(code, at().key(member));
    if artifact.digest_domain.as_ref() != expected_domain {
        return Err(refuse(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "digest_domain",
        ));
    }
    let members: [(&str, &str); 2] = [
        (&artifact.authority, "authority"),
        (&artifact.identity, "identity"),
    ];
    if let Some((_, member)) = members.iter().find(|(value, _)| !is_nonempty(value)) {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, member));
    }
    if !is_digest(&artifact.digest) {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, "digest"));
    }
    Ok(())
}

/// Checks one definition reference's `authority` and `identity` are nonempty,
/// the only check a definition reference takes: it carries no digest domain or
/// digest. `at` names the reference; each refusal points at the empty member.
pub(super) fn validate_definition_ref(
    definition: &CheckedArtifactRef,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let members: [(&str, &str); 2] = [
        (&definition.authority, "authority"),
        (&definition.identity, "identity"),
    ];
    match members.iter().find(|(value, _)| !is_nonempty(value)) {
        Some((_, member)) => Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::MalformedWire,
            at().key(member),
        )),
        None => Ok(()),
    }
}

/// Requires the source map to be exactly the graph's occurrence set, with
/// nonempty, locked, current, non-overlapping half-open regions.
pub(super) fn validate_source_map_entries<'a>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a [CheckedOccurrence])>,
    source_map: &[CheckedSourceMapEntry],
    locked_sources: &[CheckedSourceRef],
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
/// term below the first — an `aggregate` member, a `binding` value, an
/// `application` argument — is entered with `is_body_root: false`: only the
/// term handed to this call can be the body root. The walk enters the terms in
/// document pre-order on `quire-walk`'s heap stack, so it never recurses on the
/// call stack. It does not decide where an application may stand: the flat-wire
/// check (`v2::flat_wire`) refuses a nested application ahead of it. A refusal
/// points at the term that matches no closed shape, or at the member it is
/// about.
pub(super) fn validate_term(
    value: &Value,
    grammar: TermGrammar,
    is_body_root: bool,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let cursor = Cursor::at(at);
    let root = cursor.root(value, is_body_root);
    let mut terms = TermWalk {
        grammar,
        cursor,
        visit,
        work: 0,
    };
    match walk(&mut terms, root) {
        ControlFlow::Continue(()) => Ok(terms.work),
        ControlFlow::Break(failure) => Err(failure),
    }
}

/// [`validate_term`]'s walk: each entered term is checked and charged on its
/// own, and its subterms are entered after it.
struct TermWalk<'a, 'v, 'w> {
    grammar: TermGrammar,
    cursor: Cursor<'a>,
    visit: &'v mut ReferenceVisitor<'w>,
    work: u64,
}

impl<'a> Walk for TermWalk<'a, '_, '_> {
    /// A term, and whether it is the body root.
    type Node = At<'a, bool>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        node: At<'a, bool>,
        children: &mut Children<'_, At<'a, bool>>,
    ) -> ControlFlow<ValidationFailure> {
        self.cursor.enter(&node);
        match validate_one(
            node.value,
            self.grammar,
            node.extra,
            &self.cursor.trail(),
            &mut *self.visit,
        ) {
            Ok(work) => self.work = self.work.saturating_add(work),
            Err(failure) => return ControlFlow::Break(failure),
        }
        children.extend(subterms(node.value).map(|subterm| self.cursor.subterm(subterm, false)));
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

/// Validates the one term `value` at `at`, without its subterms, and returns
/// the work it is charged: the checks of its own closed shape, in the order the
/// term's members are read, and each reference it carries reported to `visit`.
/// An `application` or `aggregate` whose `arguments` or `members` is no array is
/// refused here, at that member, before any subterm is entered.
fn validate_one(
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
            let arguments = require_array(object, "arguments", at)?;
            Ok(result_type_work.saturating_add(arguments))
        }
        BodyTerm::Aggregate => {
            if exact_members(object, &["term", "members"]) {
                require_array(object, "members", at)
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
                Ok(1)
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
        // A frame and an abstraction relation are node bodies of their own,
        // never a nested term and never the body of a node of another form.
        BodyTerm::Frame | BodyTerm::AbstractionRelation => Err(invalid(at)),
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

/// Requires `object[key]`, the `aggregate.members` or `application.arguments`
/// array of the term at `at`, to be an array, else refuses at that member, and
/// charges the one unit of work the array itself costs. Its elements are
/// entered, and charged, by the walk.
fn require_array(
    object: &Map<String, Value>,
    key: &str,
    at: &Trail<'_>,
) -> Result<u64, ValidationFailure> {
    match object.get(key) {
        Some(Value::Array(_)) => Ok(1),
        _ => Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.key(key).pointer(),
        )),
    }
}

/// Parses strict JSON with serde_json's own nesting cap: a repeated object
/// member refuses as `duplicate_member` at that member, any other syntax
/// error as `malformed_wire` with no pointer. For input whose depth no caller
/// limit has charged. Test-only: the reader parses a package through
/// `strict_parse`, and a selected model document is read once by
/// `quire-canonical`.
#[cfg(test)]
pub(super) fn strict_json_value(input: &[u8]) -> Result<Value, ValidationFailure> {
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let parsed = StrictSeed::root(&duplicate).deserialize(&mut deserializer);
    finish(parsed, duplicate)
}

/// Parses `input` into a value under serde_json's own recursion limit of 128,
/// which stops the parse with an error before it recurses deeper: a document
/// nested past it refuses `malformed_wire` with no pointer, as malformed JSON
/// does (FR-038-AC-117). [`strict_shape`] has already refused every syntax and
/// duplicate-member defect, so the recursion limit is the one error this parse
/// can return.
fn strict_parse(input: &[u8]) -> Result<Value, ValidationFailure> {
    let duplicate = RefCell::new(None);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let parsed = StrictSeed::root(&duplicate).deserialize(&mut deserializer);
    finish(parsed, duplicate)
}

/// Decode one arbitrary term with the same shallow-stack visitor used by the
/// strict package parse. The preceding strict scan has already located any
/// duplicate member; this preserves its number semantics while a typed wire
/// is read directly from the caller's bytes.
pub(super) fn deserialize_strict_value<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let duplicate = RefCell::new(None);
    StrictSeed::root(&duplicate).deserialize(deserializer)
}

#[derive(Deserialize)]
struct StrictValue(#[serde(deserialize_with = "deserialize_strict_value")] Value);

/// Decode a diagnostic's terms with the same visitor, one at a time.
pub(super) fn deserialize_strict_values<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<StrictValue>::deserialize(deserializer)
        .map(|items| items.into_iter().map(|item| item.0).collect())
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
/// numbers, duplicate members — read iteratively with an explicit stack of
/// open containers, so cost is linear in the input and independent of nesting.
/// Scalars and member names are read by serde_json itself, so their grammar is
/// exactly the parser's. No value is built. Trailing bytes are left for the
/// canonical-bytes comparison to refuse.
fn strict_shape(input: &[u8]) -> Result<(), ValidationFailure> {
    strict_shape_depth(input).map(|_| ())
}

/// The strict shape and deepest container, counted without native recursion.
fn strict_shape_depth(input: &[u8]) -> Result<usize, ValidationFailure> {
    let malformed = || ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire);
    let mut scan = Scan {
        input,
        at: 0,
        open: Vec::new(),
    };
    let mut deepest = 0;
    loop {
        let opens = match scan.peek().ok_or_else(malformed)? {
            b'[' => {
                deepest = deepest.max(scan.open.len().saturating_add(1));
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
                deepest = deepest.max(scan.open.len().saturating_add(1));
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
                return Ok(deepest);
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
        canonical_value, first_difference, is_literal_value, validate_term, Step, TermGrammar,
        Trail, ValidationFailure, NODE_DOMAIN,
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

    /// A document within serde_json's recursion limit is read whatever its
    /// shape, and no limit of the read names a depth (FR-038-AC-117).
    ///
    /// Tracing: TC-048, FR-038-AC-117
    #[test]
    fn tc_048_a_document_within_the_parse_recursion_limit_is_read() {
        let value = json!({"a": [1, {"b": 2}], "c": {"d": {"e": 3}}});
        let bytes = serde_json::to_vec(&value).expect("bytes");
        assert_eq!(
            canonical_value(&bytes, CheckedPackageReadLimits::bounded()),
            Ok(value)
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
        for (bytes, integer, canonical) in [
            (&br#"{"v":1.5}"#[..], false, true),
            // Not canonical RFC 8785 bytes: `2.0` is spelled `2`, and an
            // integer past 2^53 has no encoding (FR-038-AC-79).
            (br#"{"v":2.0}"#, false, false),
            (br#"{"v":-7}"#, true, true),
            (br#"{"v":18446744073709551615}"#, true, false),
        ] {
            let text = String::from_utf8_lossy(bytes);
            let expected = if canonical {
                None
            } else {
                Some(ValidationFailure::refused_bytes(
                    CheckedPackageRefusalCode::NoncanonicalWire,
                ))
            };
            assert_eq!(canonical_value(bytes, limits).err(), expected, "{text}");
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

    /// The term walk's native stack use does not follow the depth of the term
    /// and its work is each term's own charge: a term of 200000 nested
    /// aggregates over a literal is validated whole on a thread whose stack is
    /// 256 KiB, charged one unit per aggregate and one for the literal, and
    /// reports the one reference the literal carries.
    ///
    /// Tracing: TC-048, FR-038-AC-117
    #[test]
    fn tc_048_the_term_walk_validates_a_deep_term_on_a_small_stack() {
        const LEVELS: usize = 200_000;
        let outcome = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let digest = "a".repeat(64);
                let mut term = json!({
                    "term": "literal",
                    "type": {"domain": NODE_DOMAIN, "digest": digest},
                    "value_kind": "integer",
                    "value": 1,
                });
                // Built by moving each level into the next: `json!` would copy
                // the term below it, recursively.
                for _ in 0..LEVELS {
                    let mut aggregate = serde_json::Map::new();
                    aggregate.insert("term".to_owned(), Value::String("aggregate".to_owned()));
                    aggregate.insert("members".to_owned(), Value::Array(vec![term]));
                    term = Value::Object(aggregate);
                }
                let mut reported = Vec::new();
                let work = validate_term(
                    &term,
                    TermGrammar::V2,
                    true,
                    &Trail::Base(&[]),
                    &mut |target, site, _| {
                        reported.push((target.digest.to_string(), site.is_body_root));
                    },
                );
                quire_canonical::drop_value(term);
                (work, reported)
            })
            .expect("spawns")
            .join()
            .expect("the walk ran to completion");
        assert_eq!(
            outcome,
            (Ok(200_001), vec![("a".repeat(64), false)]),
            "the literal is nested, so it is no body root"
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
    use super::{read_value, strict_json_value, strict_shape, ValidationFailure};
    use crate::checked_package::shared::{
        CheckedPackageReadLimits, CheckedPackageRefusalCode, JsonPointer,
    };
    use std::time::Instant;

    fn nested(depth: usize) -> String {
        format!("{}{}", "[".repeat(depth), "]".repeat(depth))
    }

    fn limits() -> CheckedPackageReadLimits {
        CheckedPackageReadLimits {
            bytes: 1 << 24,
            ..CheckedPackageReadLimits::bounded()
        }
    }

    fn malformed() -> ValidationFailure {
        ValidationFailure::refused_bytes(CheckedPackageRefusalCode::MalformedWire)
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
            let shape = strict_shape(text.as_bytes());
            let parsed = strict_json_value(text.as_bytes());
            assert_eq!(shape.is_ok(), parsed.is_ok(), "{text:?}");
            if let (Err(shape), Err(parsed)) = (&shape, &parsed) {
                assert_eq!(shape, parsed, "{text:?}");
            }
        }
    }

    /// Runs `operation` on a thread whose stack is 256 KiB.
    fn on_small_stack<T: Send + 'static>(operation: impl FnOnce() -> T + Send + 'static) -> T {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(operation)
            .expect("spawns")
            .join()
            .expect("the read overflowed its stack or panicked")
    }

    /// serde_json's recursion limit of 128 is the one bound on nesting: a
    /// document nested past it refuses `malformed_wire` with no pointer, never
    /// `incomplete`, and a document at or within it is read, with every outcome
    /// reached on a 256 KiB stack.
    ///
    /// Tracing: TC-048, FR-038-AC-117
    #[test]
    fn tc_048_the_parse_recursion_limit_refuses_malformed_wire_with_no_pointer() {
        on_small_stack(|| {
            // One level fewer than serde_json's 128 is read; 128 is the first
            // refused.
            assert!(read_value(nested(127).as_bytes(), limits(), Ok).is_ok());
            assert_eq!(
                read_value(nested(128).as_bytes(), limits(), Ok),
                Err(malformed())
            );
            for depth in [129, 300, 100_000, 2_000_000] {
                assert_eq!(
                    read_value(nested(depth).as_bytes(), limits(), Ok),
                    Err(malformed()),
                    "{depth}"
                );
            }
        });
    }

    /// A syntax error or a duplicate member anywhere in a document is refused
    /// first, even in one nested past the recursion limit, so the recursion
    /// limit never hides it.
    ///
    /// Tracing: TC-048, FR-038-AC-3, FR-038-AC-117
    #[test]
    fn tc_048_a_syntax_or_duplicate_defect_is_found_before_the_recursion_limit() {
        on_small_stack(|| {
            let depth = 2_000_000;
            let deep = nested(depth);
            let duplicate = format!("{{\"a\":{deep},\"a\":1}}");
            assert_eq!(
                read_value(duplicate.as_bytes(), limits(), Ok),
                Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::DuplicateMember,
                    JsonPointer::root().key("a"),
                ))
            );
            let broken = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
            assert_eq!(
                read_value(broken.as_bytes(), limits(), Ok),
                Err(malformed())
            );
        });
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
                    let result = read_value(broken.as_bytes(), limits(), Ok);
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
