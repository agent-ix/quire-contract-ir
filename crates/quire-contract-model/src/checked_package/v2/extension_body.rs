// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-348 extension form body grammar, read from a `serde_json::Value` node
//! body. The graph still holds node bodies as values; this reader is the one
//! place an extension body is parsed from one, apart from the charged joins of
//! [`super::extension`].

use super::identity::is_identifier;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedPackageRefusalCode as Code, JsonPointer,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExtensionBody {
    #[serde(rename = "term")]
    _term: ExtensionTerm,
    definition: CheckedArtifactRef,
    node_kind: Box<str>,
    clauses: Vec<ExtensionClause>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionClause {
    #[serde(rename = "term")]
    _term: BindingTerm,
    name: Box<str>,
    value: ExtensionReference,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionReference {
    #[serde(rename = "term")]
    _term: ReferenceTerm,
    target: CheckedNodeId,
}

#[derive(Deserialize)]
enum ExtensionTerm {
    #[serde(rename = "extension_form")]
    ExtensionForm,
}

#[derive(Deserialize)]
enum BindingTerm {
    #[serde(rename = "binding")]
    Binding,
}

#[derive(Deserialize)]
enum ReferenceTerm {
    #[serde(rename = "reference")]
    Reference,
}

/// The body has its own closed grammar, checked before key authentication.
#[ix_trace_rs::string_edge]
pub(super) fn read_body(body: &Value, at: JsonPointer) -> Result<ExtensionBody, ValidationFailure> {
    let parsed: ExtensionBody = serde_json::from_value(body.clone())
        .map_err(|_| ValidationFailure::refused(Code::MalformedWire, at.clone()))?;
    if parsed.definition.authority.is_empty()
        || parsed.definition.identity.is_empty()
        || !is_identifier(&parsed.node_kind)
        || parsed
            .clauses
            .iter()
            .any(|clause| !is_identifier(&clause.name) || !clause.value.target.is_well_formed())
    {
        return Err(ValidationFailure::refused(Code::MalformedWire, at));
    }
    Ok(parsed)
}

pub(super) fn targets(body: &ExtensionBody) -> impl Iterator<Item = &CheckedNodeId> {
    body.clauses.iter().map(|clause| &clause.value.target)
}

pub(super) fn body_work(body: &ExtensionBody) -> u64 {
    u64::try_from(body.clauses.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2)
        .saturating_add(1)
}
