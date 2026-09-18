//! Version-neutral I04 wire front end and helpers shared by the strict V2
//! decoder and its lowering pipeline.
//!
//! Every function here measures bytes, nesting, duplicate members and
//! canonical form identically regardless of which closed schema is decoded
//! from the resulting value.

use super::shared::{
    CheckedArtifactLocator, CheckedArtifactRef, CheckedNodeId, CheckedOccurrence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CheckedSourceMapEntry,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Identity domain shared by every checked semantic node key.
pub(super) const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";

/// A terminal front-end or validation stop, before it is wrapped in a
/// version-specific read result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Stop {
    Refused(CheckedPackageRefusalCode, Box<str>),
    Incomplete(CheckedPackageLimit, u64, u64),
}

impl Stop {
    pub(super) fn refused(code: CheckedPackageRefusalCode, path: impl Into<Box<str>>) -> Self {
        Self::Refused(code, path.into())
    }

    pub(super) fn incomplete(
        kind: CheckedPackageLimit,
        limit: u64,
        consumed: impl TryInto<u64>,
    ) -> Self {
        Self::Incomplete(kind, limit, consumed.try_into().unwrap_or(u64::MAX))
    }

    /// Wraps the stop in a version-specific closed read result.
    pub(super) fn into_result<R>(
        self,
        refused: fn(CheckedPackageRefusal) -> R,
        incomplete: fn(CheckedPackageIncomplete) -> R,
    ) -> R {
        match self {
            Self::Refused(code, path) => refused(CheckedPackageRefusal { code, path }),
            Self::Incomplete(limit_kind, limit, consumed) => incomplete(CheckedPackageIncomplete {
                limit_kind,
                limit,
                consumed,
            }),
        }
    }
}

/// Validation failure carrying a static structural path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ValidationFailure {
    Refused(CheckedPackageRefusalCode, &'static str),
    Incomplete(CheckedPackageLimit, u64, u64),
}

impl From<ValidationFailure> for Stop {
    fn from(failure: ValidationFailure) -> Self {
        match failure {
            ValidationFailure::Refused(code, path) => Self::Refused(code, path.into()),
            ValidationFailure::Incomplete(kind, limit, consumed) => {
                Self::Incomplete(kind, limit, consumed)
            }
        }
    }
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
) -> Result<Value, Stop> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(Stop::incomplete(
            CheckedPackageLimit::Bytes,
            limits.bytes,
            bytes.len(),
        ));
    }
    let value = match strict_json_value(bytes) {
        Ok(value) => value,
        Err(StrictJsonError::Duplicate(path)) => {
            return Err(Stop::Refused(
                CheckedPackageRefusalCode::DuplicateMember,
                path,
            ))
        }
        Err(StrictJsonError::Syntax) => {
            return Err(Stop::refused(
                CheckedPackageRefusalCode::MalformedWire,
                "document",
            ))
        }
    };
    let depth = json_depth(&value);
    if exceeds(depth, limits.depth) {
        return Err(Stop::incomplete(
            CheckedPackageLimit::Depth,
            limits.depth,
            depth,
        ));
    }
    let canonical = serde_json::to_vec(&value)
        .map_err(|_| Stop::refused(CheckedPackageRefusalCode::MalformedWire, "document"))?;
    if canonical.as_slice() != bytes {
        return Err(Stop::refused(
            CheckedPackageRefusalCode::NoncanonicalWire,
            "document",
        ));
    }
    Ok(value)
}

/// Decodes a closed wire value, classifying closed-schema member violations.
pub(super) fn decode_closed<T: DeserializeOwned>(value: Value) -> Result<T, Stop> {
    serde_json::from_value::<T>(value).map_err(|error| {
        let code = if error.to_string().contains("unknown field") {
            CheckedPackageRefusalCode::UnknownMember
        } else {
            CheckedPackageRefusalCode::MalformedWire
        };
        Stop::refused(code, "document")
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
pub(super) fn validate_locked_artifact(
    artifact: &CheckedArtifactRef,
    expected_domain: &str,
    context: &dyn ArtifactDigests,
    path: &'static str,
) -> Result<(), ValidationFailure> {
    if artifact.digest_domain.as_ref() != expected_domain {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            path,
        ));
    }
    if !is_nonempty(&artifact.authority)
        || !is_nonempty(&artifact.identity)
        || !is_nonempty(&artifact.revision.namespace)
        || !is_nonempty(&artifact.revision.value)
        || !is_digest(&artifact.digest)
    {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::MalformedWire,
            path,
        ));
    }
    let locator = artifact_locator(artifact);
    let Some(digest) = context.artifact_digest(&locator) else {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::StaleDependency,
            path,
        ));
    };
    if digest.as_ref() != artifact.digest.as_ref() {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::StaleDependency,
            path,
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
    let mut expected = BTreeSet::new();
    for (node_id, occurrences) in nodes {
        for occurrence in occurrences {
            expected.insert((node_id.clone(), occurrence.role.clone(), occurrence.ordinal));
        }
    }
    let mut actual = BTreeSet::new();
    let locked_sources = locked_sources.iter().collect::<BTreeSet<_>>();
    let mut count = 0_u64;
    for entry in source_map {
        count = count.saturating_add(1);
        if exceeds(count, limits.occurrences) {
            return Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
            ));
        }
        if entry.regions.is_empty()
            || !actual.insert((entry.node_id.clone(), entry.role.clone(), entry.ordinal))
        {
            return Err(ValidationFailure::Refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                "source_map",
            ));
        }
        count = count.saturating_add(u64::try_from(entry.regions.len()).unwrap_or(u64::MAX));
        if exceeds(count, limits.occurrences) {
            return Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Occurrences,
                limits.occurrences,
                count,
            ));
        }
        let mut ranges = BTreeMap::<CheckedArtifactLocator, Vec<(u64, u64)>>::new();
        for region in &entry.regions {
            if !locked_sources.contains(&region.source) {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    "source_map.regions.source",
                ));
            }
            validate_locked_artifact(
                &region.source,
                "quire.source.bytes/v1",
                context,
                "source_map.regions.source",
            )?;
            if region.start >= region.end {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    "source_map.regions",
                ));
            }
            ranges
                .entry(artifact_locator(&region.source))
                .or_default()
                .push((region.start, region.end));
        }
        for ranges in ranges.values_mut() {
            ranges.sort_unstable();
            if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
                return Err(ValidationFailure::Refused(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    "source_map.regions",
                ));
            }
        }
    }
    if actual != expected {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSourceMap,
            "source_map",
        ));
    }
    Ok(())
}

/// The literal-value grammar a semantic term is validated against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TermGrammar {
    /// V2 `SemanticTerm`: a numeric literal value must be an integer token
    /// within the signed or unsigned 64-bit range. The vendored schema admits
    /// only `integer` numbers; a fraction, exponent or out-of-range integer is
    /// refused rather than rounded.
    V2,
}

/// Validates one public semantic term, returning its work and reporting every
/// reference target to `visit`.
pub(super) fn validate_term(
    value: &Value,
    grammar: TermGrammar,
    visit: &mut dyn FnMut(&CheckedNodeId),
) -> Result<u64, ValidationFailure> {
    let Value::Object(object) = value else {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        ));
    };
    let Some(Value::String(term)) = object.get("term") else {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body.term",
        ));
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
            visit_reference(object.get("type"), visit)
        }
        "reference" if exact_members(object, &["term", "target"]) => {
            visit_reference(object.get("target"), visit)
        }
        "application"
            if exact_members(
                object,
                &["term", "operator", "operation", "result_type", "arguments"],
            ) && object
                .get("operator")
                .and_then(Value::as_str)
                .is_some_and(is_operator) =>
        {
            // The `operation` member's own closed shape (`OperationIdentity`,
            // laws, mode, member and leaves) is admitted opaquely here: this
            // reader accepts the member is present and validates only
            // `result_type` as a reference. Deep operation-law validation is
            // not implemented by this reader.
            let result_type_work = visit_reference(object.get("result_type"), visit)?;
            let arguments_work = visit_terms(object.get("arguments"), grammar, visit)?;
            Ok(result_type_work.saturating_add(arguments_work))
        }
        "aggregate" if exact_members(object, &["term", "members"]) => {
            visit_terms(object.get("members"), grammar, visit)
        }
        "binding"
            if exact_members(object, &["term", "name", "value"])
                && object
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(is_nonempty) =>
        {
            validate_term(object.get("value").unwrap_or(&Value::Null), grammar, visit)
                .map(|work| work.saturating_add(1))
        }
        _ => Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        )),
    }
}

/// Parses one node-reference member, reports its target to `visit`, and
/// charges one unit of work. Shared by `reference.target`, `literal.type`,
/// `application.result_type`, and the V2 `frame` body's reference arrays.
pub(super) fn visit_reference(
    value: Option<&Value>,
    visit: &mut dyn FnMut(&CheckedNodeId),
) -> Result<u64, ValidationFailure> {
    let target = value.cloned().ok_or(ValidationFailure::Refused(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "semantic_graph.nodes.body.target",
    ))?;
    let target = serde_json::from_value::<CheckedNodeId>(target).map_err(|_| {
        ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body.target",
        )
    })?;
    if target.domain.as_ref() == NODE_DOMAIN && is_digest(&target.digest) {
        visit(&target);
        Ok(1)
    } else {
        Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "semantic_graph.nodes.body.target",
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

fn visit_terms(
    value: Option<&Value>,
    grammar: TermGrammar,
    visit: &mut dyn FnMut(&CheckedNodeId),
) -> Result<u64, ValidationFailure> {
    let Some(Value::Array(values)) = value else {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        ));
    };
    values.iter().try_fold(1_u64, |work, term| {
        validate_term(term, grammar, visit).map(|child| work.saturating_add(child))
    })
}

#[derive(Debug)]
enum StrictJsonError {
    Duplicate(Box<str>),
    Syntax,
}

fn strict_json_value(input: &[u8]) -> Result<Value, StrictJsonError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    StrictValue::deserialize(&mut deserializer)
        .map(|value| value.0)
        .map_err(|error| {
            error
                .to_string()
                .strip_prefix("duplicate JSON member at ")
                .map_or(StrictJsonError::Syntax, |path| {
                    StrictJsonError::Duplicate(path.into())
                })
        })
}

struct StrictValue(Value);

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

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StrictValue;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("strict JSON value")
            }
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StrictValue(Value::String(value.to_owned())))
            }
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = access.next_element::<StrictValue>()? {
                    values.push(value.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = Map::new();
                let Some(first) = access.next_key::<String>()? else {
                    return Ok(StrictValue(Value::Object(map)));
                };
                if first == SERDE_JSON_NUMBER_TOKEN {
                    let digits = access.next_value::<String>()?;
                    return number_from_token(&digits).map(StrictValue);
                }
                let value = access.next_value::<StrictValue>()?;
                map.insert(first, value.0);
                while let Some((key, value)) = access.next_entry::<String, StrictValue>()? {
                    if map.insert(key.clone(), value.0).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON member at {key}"
                        )));
                    }
                }
                Ok(StrictValue(Value::Object(map)))
            }
        }
        deserializer.deserialize_any(Visitor)
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

#[cfg(test)]
mod tests {
    use super::{canonical_value, is_literal_value, Stop, TermGrammar};
    use crate::checked_package::shared::{CheckedPackageReadLimits, CheckedPackageRefusalCode};
    use serde_json::{json, Number, Value};

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
            Err(Stop::refused(
                CheckedPackageRefusalCode::NoncanonicalWire,
                "document"
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
