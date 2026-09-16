//! Version-neutral I04 wire front end and helpers shared by the V1 and V2
//! strict decoders.
//!
//! Every function here preserves the exact behavior the frozen V1 reader had
//! before the version split; V2 reuses it so both versions measure bytes,
//! nesting, duplicate members and canonical form identically.

use super::v1::{
    CheckedArtifactLocator, CheckedArtifactRef, CheckedNodeId, CheckedOccurrence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageReadResult, CheckedPackageRefusal, CheckedPackageRefusalCode,
    CheckedSourceMapEntry,
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

impl From<Stop> for CheckedPackageReadResult {
    fn from(stop: Stop) -> Self {
        stop.into_result(Self::Refused, Self::Incomplete)
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
/// bytes. This is the V1 front end, unchanged.
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

/// Validates one public semantic term, returning its work and reporting every
/// reference target to `visit`.
pub(super) fn validate_term(
    value: &Value,
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
            if exact_members(object, &["term", "value_kind", "value"])
                && object
                    .get("value_kind")
                    .and_then(Value::as_str)
                    .is_some_and(is_literal_kind)
                && object.get("value").is_some_and(is_literal_value) =>
        {
            Ok(1)
        }
        "reference" if exact_members(object, &["term", "target"]) => {
            let target = object
                .get("target")
                .cloned()
                .ok_or(ValidationFailure::Refused(
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
        "application"
            if exact_members(object, &["term", "operator", "arguments"])
                && object
                    .get("operator")
                    .and_then(Value::as_str)
                    .is_some_and(is_operator) =>
        {
            visit_terms(object.get("arguments"), visit)
        }
        "aggregate" if exact_members(object, &["term", "members"]) => {
            visit_terms(object.get("members"), visit)
        }
        "binding"
            if exact_members(object, &["term", "name", "value"])
                && object
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(is_nonempty) =>
        {
            validate_term(object.get("value").unwrap_or(&Value::Null), visit)
                .map(|work| work.saturating_add(1))
        }
        _ => Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        )),
    }
}

fn exact_members(object: &Map<String, Value>, expected: &[&str]) -> bool {
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

fn is_literal_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Bool(_) | Value::String(_) | Value::Number(_) | Value::Null
    )
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
    visit: &mut dyn FnMut(&CheckedNodeId),
) -> Result<u64, ValidationFailure> {
    let Some(Value::Array(values)) = value else {
        return Err(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        ));
    };
    values.iter().try_fold(1_u64, |work, term| {
        validate_term(term, visit).map(|child| work.saturating_add(child))
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
