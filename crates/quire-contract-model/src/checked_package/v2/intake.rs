//! The closed-schema decode of the package document, without native recursion
//! following the nesting of a term.
//!
//! A node `body` and a diagnostic's `details` entry may nest as deep as the
//! strict parser's limit of 128. The outer typed decode retains handles into
//! the shared document and decodes admitted terms after the outer frame has
//! returned. A `NodeId.path` contains fixed-depth string or graph-index
//! segments, so a derived chain does not increase JSON nesting depth.
//!
//! The lossless-decode check (no member defaulted, nulled or dropped) compares
//! document nodes against their typed serialization one at a time, including
//! structural node identities.

use super::NativeJoinBudget;
use super::TermNode;
use super::{
    CheckedDeclaration, CheckedDiagnosticCause, CheckedDiagnosticCode, CheckedDiagnosticStage,
    CheckedDiagnosticV2, CheckedDiagnosticsV2, CheckedJson, CheckedNodeOwner,
    CheckedPackageEnvelope, CheckedPackageLockV2, RawCheckedPackageWireV2, RawSemanticGraphV2,
    RawSemanticNodeV2, ValidationFailure,
};
use crate::checked_package::common::{decode_closed_document, node_pointer};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedOccurrence, CheckedSourceMapEntry,
    CheckedSourceRegion,
};
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use quire_canonical::Document;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

/// Check a present position ordinal before semantic graph validation.
fn key_ordinal_node(value: TermNode<'_>) -> bool {
    const MAX_SAFE: &str = "9007199254740991";
    if value
        .as_u64()
        .is_some_and(|number| number <= 9_007_199_254_740_991)
    {
        return true;
    }
    let Some(text) = value.as_str() else {
        return false;
    };
    !text.is_empty()
        && !text.starts_with('0')
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && (text.len() > MAX_SAFE.len() || (text.len() == MAX_SAFE.len() && text > MAX_SAFE))
}

fn position_member_shape(body: TermNode<'_>) -> bool {
    if body.get("term").and_then(TermNode::as_str) == Some("application") {
        if let Some(member) = body
            .get("operation")
            .and_then(|operation| operation.get("member"))
        {
            if member.get("kind").and_then(TermNode::as_str) == Some("position") {
                if let Some(position) = member.get("position") {
                    if !key_ordinal_node(position) {
                        return false;
                    }
                }
            }
        }
    }
    body.items()
        .is_none_or(|items| items.into_iter().all(position_member_shape))
        && body.members().is_none_or(|members| {
            members
                .into_iter()
                .all(|(_, value)| position_member_shape(value))
        })
}

// The borrowed intake mirrors only the closed containers that contain an
// arbitrary JSON term. Those terms are decoded after the outer typed wire
// frame has returned, keeping their recursion off its call stack. Every other
// member is decoded by the same public type as the admitted wire.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedWire {
    contract_version: Box<str>,
    name: Box<str>,
    lock: CheckedPackageLockV2,
    semantic_graph: BorrowedGraph,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    source_map: Vec<CheckedSourceMapEntry>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    refinement_step_rows: Option<Vec<super::refinement::CheckedRefinementStepRowRegion>>,
    #[serde(default)]
    checked_term_data: Option<super::checked_term_data::CheckedTermDataV2>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    temporal_bindings: Option<Vec<super::temporal_binding::TemporalBinding>>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    temporal_clause_names: Option<Vec<super::TemporalClauseName>>,
    #[serde(default, deserialize_with = "super::document_de::exact_optional_vec")]
    schedulability_claims: Option<Vec<super::CheckedSchedulabilityClaimRow>>,
    #[serde(default, deserialize_with = "super::operation_effects::present_rows")]
    operation_effects: Option<Vec<super::CheckedOperationEffectRow>>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    capability_report: Vec<CheckedCapability>,
    diagnostics: BorrowedDiagnostics,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedGraph {
    graph_version: Box<str>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    nodes: Vec<BorrowedNode>,
    #[serde(deserialize_with = "super::document_de::exact_rows")]
    operand_rows: Vec<Vec<u64>>,
}

// Keep native wire errors at the logical ID boundary. The refinement below
// then locates an invalid ordinal or path segment without exposing an
// implementation-dependent Serde member path. The public ID deserializer
// remains the one owner of the flat ID grammar.
struct WireNodeId(CheckedNodeId);

impl<'de> Deserialize<'de> for WireNodeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The document deserializer decodes the id whole, so a fault inside
        // it is reported at the id and not at a member of it.
        struct Held;
        impl<'de> serde::de::Visitor<'de> for Held {
            type Value = WireNodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a structural node id")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<WireNodeId, E> {
                super::document_de::take_node_id()
                    .map(WireNodeId)
                    .ok_or_else(|| E::custom("no node id was decoded"))
            }

            fn visit_newtype_struct<D2: Deserializer<'de>>(
                self,
                inner: D2,
            ) -> Result<WireNodeId, D2::Error> {
                CheckedNodeId::deserialize(inner).map(WireNodeId)
            }
        }
        deserializer.deserialize_newtype_struct(super::document_de::NODE_ID_TOKEN, Held)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedNode {
    node_id: WireNodeId,
    schema_version: Box<str>,
    node_tag: Box<str>,
    semantic_form: Box<str>,
    semantic_type: WireNodeId,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    dependencies: Vec<WireNodeId>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    occurrences: Vec<CheckedOccurrence>,
    #[serde(default)]
    recursion_group: Option<Box<str>>,
    #[serde(default, deserialize_with = "present_nullable")]
    declaration: Option<Option<CheckedDeclaration>>,
    #[serde(default)]
    owner: Option<CheckedNodeOwner>,
    #[serde(default)]
    owner_model: Option<WireNodeId>,
    body: CheckedJson,
}

// The outer option records member presence; the inner option records JSON null.
fn present_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostics {
    catalog: CheckedArtifactRef,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    entries: Vec<BorrowedDiagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BorrowedDiagnostic {
    stage: CheckedDiagnosticStage,
    code: CheckedDiagnosticCode,
    cause_tag: CheckedDiagnosticCause,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    details: Vec<CheckedJson>,
    #[serde(deserialize_with = "super::document_de::exact_vec")]
    loci: Vec<CheckedSourceRegion>,
}

#[ix_trace_rs::string_edge]
pub(super) fn decode_borrowed_wire(
    document: &Arc<Document>,
    budget: &mut NativeJoinBudget,
) -> Result<(RawCheckedPackageWireV2, Vec<CheckedJson>), ValidationFailure> {
    let wire: BorrowedWire = decode_closed_document(document, budget).map_err(|mut failure| {
        if let ValidationFailure::Refused(ref mut refusal) = failure {
            if refusal.code == CheckedPackageRefusalCode::UnknownMember {
                if let Some(index) = refusal
                    .primary_path()
                    .and_then(|at| at.as_str().strip_prefix("/temporal_clause_names/"))
                    .and_then(|suffix| suffix.split('/').next())
                    .and_then(|index| index.parse::<usize>().ok())
                {
                    refusal.code = CheckedPackageRefusalCode::MalformedWire;
                    refusal.set_primary_path(Some(
                        JsonPointer::root()
                            .key("temporal_clause_names")
                            .index(index),
                    ));
                }
            }
            if refusal.code == CheckedPackageRefusalCode::UnknownMember {
                if let Some(index) = refusal
                    .primary_path()
                    .and_then(|at| at.as_str().strip_prefix("/schedulability_claims/"))
                    .and_then(|suffix| suffix.split('/').next())
                    .and_then(|index| index.parse::<usize>().ok())
                {
                    refusal.code = CheckedPackageRefusalCode::MalformedWire;
                    refusal.set_primary_path(Some(
                        JsonPointer::root()
                            .key("schedulability_claims")
                            .index(index),
                    ));
                }
            }
            if refusal.code == CheckedPackageRefusalCode::UnknownMember
                && refusal
                    .primary_path()
                    .is_some_and(|path| path.as_str().starts_with("/checked_term_data"))
            {
                if let Some(row) = refusal
                    .primary_path()
                    .and_then(probabilistic_row_unknown_member_path)
                {
                    refusal.set_primary_path(Some(row));
                }
                refusal.code = CheckedPackageRefusalCode::MalformedWire;
            }
            if refusal.code == CheckedPackageRefusalCode::UnknownMember
                && refusal.primary_path().is_some_and(|path| {
                    path.as_str() == "/operation_effects"
                        || path.as_str().starts_with("/operation_effects/")
                })
            {
                refusal.code = CheckedPackageRefusalCode::MalformedWire;
            }
            if refusal.code == CheckedPackageRefusalCode::MalformedWire {
                if let Some(at) = refusal.primary_path().cloned() {
                    if let Some(member) = invalid_node_id_field(document, &at) {
                        refusal.set_primary_path(Some(member));
                    } else if let Some(member) = invalid_clock_field(document, &at) {
                        refusal.set_primary_path(Some(member));
                    } else if let Some(member) = missing_probabilistic_occurrence(document, &at) {
                        refusal.set_primary_path(Some(member));
                    } else if let Some(parent) = operation_effect_schema_parent_path(&at) {
                        refusal.set_primary_path(Some(parent));
                    }
                }
            }
        }
        failure
    })?;
    if wire
        .schedulability_claims
        .as_ref()
        .is_some_and(Vec::is_empty)
    {
        return Err(malformed(
            super::stage_storage::Path::root().key("schedulability_claims"),
            &mut super::WorkMeter::over(budget),
        ));
    }
    let graph = wire.semantic_graph;
    let diagnostics = wire.diagnostics;
    let mut nodes = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "semantic_graph.nodes",
        ),
        graph.nodes.len(),
    )?;
    // The node bodies as handles into the retained document; validators read
    // these directly through TermNode.
    let mut bodies = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "semantic_graph.nodes",
        ),
        graph.nodes.len(),
    )?;
    for (position, node) in graph.nodes.into_iter().enumerate() {
        if node.node_tag.as_ref() == "scalar_type"
            && node.semantic_form.as_ref() == "dimension_equivalence"
            && node.declaration.is_some()
        {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        bodies.push(node.body.clone());
        let body = node.body;
        if !position_member_shape(TermNode::of(&body)) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                node_pointer(position),
            ));
        }
        let mut dependencies = super::read_charge::with_capacity(
            budget,
            super::read_charge::Site::new(
                super::PackageReadStage::WireDecode,
                position,
                "dependencies",
            ),
            node.dependencies.len(),
        )?;
        dependencies.extend(node.dependencies.into_iter().map(|id| id.0));
        nodes.push(RawSemanticNodeV2 {
            node_id: node.node_id.0,
            schema_version: node.schema_version,
            node_tag: node.node_tag,
            semantic_form: node.semantic_form,
            semantic_type: node.semantic_type.0,
            dependencies,
            occurrences: node.occurrences,
            recursion_group: node.recursion_group,
            declaration: node.declaration.flatten(),
            owner: node.owner,
            owner_model: node.owner_model.map(|id| id.0),
            body,
        });
    }
    let mut diagnostic_entries = super::read_charge::with_capacity(
        budget,
        super::read_charge::Site::new(
            super::PackageReadStage::WireDecode,
            0,
            "diagnostics.entries",
        ),
        diagnostics.entries.len(),
    )?;
    for entry in diagnostics.entries {
        diagnostic_entries.push(CheckedDiagnosticV2 {
            stage: entry.stage,
            code: entry.code,
            cause_tag: entry.cause_tag,
            details: entry.details,
            loci: entry.loci,
        });
    }
    let envelope = CheckedPackageEnvelope {
        contract_version: wire.contract_version,
        name: wire.name,
        lock: wire.lock,
        semantic_graph: RawSemanticGraphV2 {
            graph_version: graph.graph_version,
            nodes,
            operand_rows: graph.operand_rows,
        },
        source_map: wire.source_map,
        refinement_step_rows: wire.refinement_step_rows,
        checked_term_data: wire.checked_term_data,
        temporal_bindings: wire.temporal_bindings,
        temporal_clause_names: wire.temporal_clause_names,
        schedulability_claims: wire.schedulability_claims,
        operation_effects: wire.operation_effects,
        capability_report: wire.capability_report,
        diagnostics: CheckedDiagnosticsV2 {
            catalog: diagnostics.catalog,
            entries: diagnostic_entries,
        },
    };
    Ok((envelope, bodies))
}

#[ix_trace_rs::string_edge]
fn operation_effect_schema_parent_path(at: &JsonPointer) -> Option<JsonPointer> {
    let path = at.as_str().strip_prefix("/operation_effects/")?;
    if let Some((index, member)) = path.split_once('/') {
        if !member.contains('/')
            && !matches!(
                member,
                "document" | "declaring" | "operation" | "effect" | "provenance"
            )
        {
            let index: usize = index.parse().ok()?;
            return Some(JsonPointer::root().key("operation_effects").index(index));
        }
    }
    let (index, path) = path.split_once("/effect/")?;
    let index: usize = index.parse().ok()?;
    if let Some(entry) = path
        .strip_prefix("modifies/")
        .and_then(|entry| entry.strip_suffix("/kind"))
    {
        let entry: usize = entry.parse().ok()?;
        return Some(
            JsonPointer::root()
                .key("operation_effects")
                .index(index)
                .key("effect")
                .key("modifies")
                .index(entry),
        );
    }
    if path.is_empty()
        || path.contains('/')
        || matches!(path, "modifies" | "creates" | "deletes" | "deletes_scoped")
    {
        return None;
    }
    Some(
        JsonPointer::root()
            .key("operation_effects")
            .index(index)
            .key("effect"),
    )
}

#[ix_trace_rs::string_edge]
fn probabilistic_row_unknown_member_path(at: &JsonPointer) -> Option<JsonPointer> {
    let remainder = at
        .as_str()
        .strip_prefix("/checked_term_data/probabilistic_claims/")?;
    let (index, member) = remainder.split_once('/')?;
    if member.is_empty() || member.contains('/') {
        return None;
    }
    let index: usize = index.parse().ok()?;
    Some(
        JsonPointer::root()
            .key("checked_term_data")
            .key("probabilistic_claims")
            .index(index),
    )
}

fn missing_probabilistic_occurrence(document: &Document, at: &JsonPointer) -> Option<JsonPointer> {
    let prefix = "/checked_term_data/probabilistic_claims/";
    let index = at.as_str().strip_prefix(prefix)?;
    if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let row = term_at_pointer(document, at)?;
    (row.get("declaration").is_some() && row.get("occurrence").is_none())
        .then(|| at.clone().key("occurrence"))
}

#[ix_trace_rs::string_edge]
fn invalid_clock_field(document: &Document, at: &JsonPointer) -> Option<JsonPointer> {
    if !at.as_str().starts_with("/temporal_bindings/") || !at.as_str().ends_with("/clock") {
        return None;
    }
    let clock = term_at_pointer(document, at)?;
    let field = match clock.get("kind")?.as_str()? {
        "event_position" => "sequence_authority",
        "timestamped_event" => "timestamp_unit",
        "fixed_sample" => "unit",
        _ => return None,
    };
    clock
        .get(field)
        .filter(|value| value.as_str().is_none())
        .map(|_| at.clone().key(field))
}

/// Refine a buffered node-ID decode error to an ordinal or path segment.
/// V2 path objects are closed graph-local indices; nested IDs are refused at
/// the segment itself, before semantic-graph admission.
#[ix_trace_rs::string_edge]
fn invalid_node_id_field(document: &Document, at: &JsonPointer) -> Option<JsonPointer> {
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    let id = term_at_pointer(document, at)?;
    if id.get("package").is_none() || id.get("path").is_none() {
        return None;
    }
    if let Some(ordinal) = id.get("ordinal") {
        if ordinal.as_u64().is_none_or(|value| value > MAX_SAFE) {
            return Some(at.clone().key("ordinal"));
        }
    }
    let segments = id.get("path")?.items()?;
    for (index, segment) in segments.enumerate() {
        if segment.as_str().is_some() {
            continue;
        }
        if segment.exact_members(&["node"])
            && segment
                .get("node")
                .and_then(TermNode::as_u64)
                .is_some_and(|value| value <= MAX_SAFE)
        {
            continue;
        }
        return Some(at.clone().key("path").index(index));
    }
    None
}

fn term_at_pointer<'d>(document: &'d Document, pointer: &JsonPointer) -> Option<TermNode<'d>> {
    let mut current = TermNode::from_ref(document.root());
    let path = pointer.as_str();
    if path.is_empty() {
        return Some(current);
    }
    for token in path.strip_prefix('/')?.split('/') {
        current = if current.is_object() {
            current
                .members()?
                .find(|(name, _)| pointer_token_matches(name, token))?
                .1
        } else {
            current.items()?.nth(token.parse::<usize>().ok()?)?
        };
    }
    Some(current)
}

fn pointer_token_matches(name: &str, token: &str) -> bool {
    let mut name = name.chars();
    let mut token = token.chars();
    while let Some(character) = token.next() {
        let character = if character == '~' {
            match token.next() {
                Some('0') => '~',
                Some('1') => '/',
                _ => return false,
            }
        } else {
            character
        };
        if name.next() != Some(character) {
            return false;
        }
    }
    name.next().is_none()
}

/// `malformed_wire` at `at`.
fn malformed(
    at: super::stage_storage::Path<'static>,
    meter: &mut super::WorkMeter,
) -> ValidationFailure {
    super::stage_storage::refuse(
        meter,
        &super::stage_storage::Site::new(
            super::PackageReadStage::WireDecode,
            at.steps()
                .iter()
                .filter_map(|step| match step {
                    crate::checked_package::common::Step::Index(index) => Some(*index),
                    _ => None,
                })
                .last()
                .unwrap_or(0),
            "wire",
            at,
        ),
        super::stage_storage::Refusal::new(CheckedPackageRefusalCode::MalformedWire, at),
    )
}

/// Check the lossless closed decode against the retained source document.
/// The comparator follows document nodes as the typed serializer visits them;
/// it does not reparse a source slice or build a second JSON tree.
pub(super) fn check_lossless_source(
    document: &Document,
    wire: &RawCheckedPackageWireV2,
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let mut meter = super::WorkMeter::over(budget);
    let meter = &mut meter;
    let root = TermNode::from_ref(document.root());
    let root_at = super::stage_storage::Path::root();
    compare_root(
        root,
        "capability_report",
        &wire.capability_report,
        &[],
        meter,
    )?;
    compare_root(root, "contract_version", &wire.contract_version, &[], meter)?;
    compare_root(root, "diagnostics", &wire.diagnostics, &["entries"], meter)?;
    compare_root(root, "lock", &wire.lock, &[], meter)?;
    compare_root(root, "name", &wire.name, &[], meter)?;
    compare_root(
        root,
        "semantic_graph",
        &wire.semantic_graph,
        &["nodes"],
        meter,
    )?;

    let nodes_at = root_at.key("semantic_graph").key("nodes");
    let nodes = root
        .get("semantic_graph")
        .and_then(|graph| graph.get("nodes"))
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(nodes_at, meter))?;
    if nodes.len() != wire.semantic_graph.nodes.len() {
        return Err(malformed(nodes_at, meter));
    }
    for (index, (original, node)) in nodes.zip(&wire.semantic_graph.nodes).enumerate() {
        let at = root_at.key("semantic_graph").key("nodes").index(index);
        super::lossless::compare(
            original,
            node,
            at,
            &[
                "body",
                "node_id",
                "semantic_type",
                "dependencies",
                "owner_model",
            ],
            meter,
        )?;
        for key in ["node_id", "semantic_type"] {
            let locus = at.key(key);
            let source = original.get(key).ok_or_else(|| malformed(locus, meter))?;
            let typed = if key == "node_id" {
                &node.node_id
            } else {
                &node.semantic_type
            };
            super::lossless::compare(source, typed, locus, &[], meter)?;
        }
        let dependencies_at = at.key("dependencies");
        let dependencies = original
            .get("dependencies")
            .and_then(TermNode::items)
            .ok_or_else(|| malformed(dependencies_at, meter))?;
        if dependencies.len() != node.dependencies.len() {
            return Err(malformed(dependencies_at, meter));
        }
        for (index, (source, id)) in dependencies.zip(&node.dependencies).enumerate() {
            super::lossless::compare(source, id, at.key("dependencies").index(index), &[], meter)?;
        }
        match (&node.owner_model, original.get("owner_model")) {
            (Some(id), Some(source)) => {
                super::lossless::compare(source, id, at.key("owner_model"), &[], meter)?
            }
            (None, None) => {}
            _ => return Err(malformed(at.key("owner_model"), meter)),
        }
    }

    compare_row_collection(root, "source_map", &wire.source_map, &[], meter)?;
    if let Some(rows) = &wire.refinement_step_rows {
        compare_row_collection(root, "refinement_step_rows", rows, &[], meter)?;
    } else if root.get("refinement_step_rows").is_some() {
        return Err(malformed(root_at.key("refinement_step_rows"), meter));
    }
    if let Some(data) = &wire.checked_term_data {
        compare_root(root, "checked_term_data", data, &[], meter)?;
    } else if root.get("checked_term_data").is_some() {
        return Err(malformed(root_at.key("checked_term_data"), meter));
    }
    compare_optional_rows(
        root,
        "temporal_bindings",
        wire.temporal_bindings.as_deref(),
        meter,
    )?;
    compare_optional_rows(
        root,
        "temporal_clause_names",
        wire.temporal_clause_names.as_deref(),
        meter,
    )?;
    compare_optional_rows(
        root,
        "schedulability_claims",
        wire.schedulability_claims.as_deref(),
        meter,
    )?;
    let entries = root
        .get("diagnostics")
        .and_then(|value| value.get("entries"))
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(root_at.key("diagnostics").key("entries"), meter))?;
    compare_terms(
        root_at.key("diagnostics").key("entries"),
        entries,
        &wire.diagnostics.entries,
        &["details"],
        meter,
    )
}

fn compare_row_collection<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: &[T],
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let at = super::stage_storage::Path::root().key(key);
    let rows = root
        .get(key)
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(at, meter))?;
    compare_terms(at, rows, typed, skipped, meter)
}

fn compare_root<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: &T,
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let at = super::stage_storage::Path::root().key(key);
    let raw = root.get(key).ok_or_else(|| malformed(at, meter))?;
    super::lossless::compare(raw, typed, at, skipped, meter)
}

fn compare_optional_rows<T: serde::Serialize>(
    root: TermNode<'_>,
    key: &'static str,
    typed: Option<&[T]>,
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(typed) = typed else {
        return if root.get(key).is_some() {
            Err(malformed(
                super::stage_storage::Path::root().key(key),
                meter,
            ))
        } else {
            Ok(())
        };
    };
    if typed.is_empty() {
        return Err(malformed(
            super::stage_storage::Path::root().key(key),
            meter,
        ));
    }
    compare_row_collection(root, key, typed, &[], meter)
}

fn compare_terms<T: serde::Serialize>(
    at: super::stage_storage::Path<'static>,
    terms: super::TermItems<'_>,
    typed: &[T],
    skipped: &'static [&'static str],
    meter: &mut super::WorkMeter,
) -> Result<(), ValidationFailure> {
    if terms.len() != typed.len() {
        return Err(malformed(at, meter));
    }
    for (index, (source, value)) in terms.zip(typed).enumerate() {
        super::lossless::compare(source, value, at.index(index), skipped, meter)?;
    }
    Ok(())
}

#[cfg(test)]
mod ordinal_locus_tests {
    use super::*;

    #[test]
    fn source_lookup_skips_deep_sibling_without_deserializing_it() {
        let deep = format!("{}0{}", "[".repeat(120), "]".repeat(120));
        let bytes = format!(
            "{{\"semantic_graph\":{{\"nodes\":[{{\"body\":{deep},\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":0}}}}]}}}}"
        );
        let document = quire_canonical::read(bytes.as_bytes(), u64::MAX).expect("document");
        let pointer = JsonPointer::parse("/semantic_graph/nodes/0/node_id/ordinal").unwrap();
        assert_eq!(
            term_at_pointer(&document, &pointer).and_then(TermNode::as_u64),
            Some(0)
        );
    }

    #[test]
    fn source_lookup_handles_escaped_member_and_string_contents() {
        let bytes = br#"{"a/b":{"~name":["quoted \" { [ }",{"node":3}]}}"#;
        let document = quire_canonical::read(bytes, u64::MAX).expect("document");
        let pointer = JsonPointer::parse("/a~1b/~0name/1/node").unwrap();
        assert_eq!(
            term_at_pointer(&document, &pointer).and_then(TermNode::as_u64),
            Some(3)
        );
    }

    #[test]
    fn malformed_node_id_members_keep_their_exact_pointer() {
        let node = JsonPointer::parse("/semantic_graph/nodes/1/node_id").unwrap();
        for ordinal in ["9007199254740992", "1.5", "-1", "\"one\""] {
            let bytes = format!(
                "{{\"semantic_graph\":{{\"nodes\":[null,{{\"node_id\":{{\"package\":\"p\",\"path\":[\"x\"],\"ordinal\":{ordinal}}}}}]}}}}"
            );
            let document = quire_canonical::read(bytes.as_bytes(), u64::MAX).expect("document");
            assert_eq!(
                invalid_node_id_field(&document, &node),
                Some(node.clone().key("ordinal"))
            );
        }

        let nested = br#"{"semantic_graph":{"nodes":[null,{"node_id":{"package":"p","path":[{"package":"p","path":["x"],"ordinal":1.5}],"ordinal":0}}]}}"#;
        let document = quire_canonical::read(nested, u64::MAX).expect("document");
        assert_eq!(
            invalid_node_id_field(&document, &node),
            Some(node.key("path").index(0))
        );

        let dependency = br#"{"semantic_graph":{"nodes":[{"dependencies":[{"package":"p","path":["x"],"ordinal":"one"}]}]}}"#;
        let reference = JsonPointer::parse("/semantic_graph/nodes/0/dependencies/0").unwrap();
        let document = quire_canonical::read(dependency, u64::MAX).expect("document");
        assert_eq!(
            invalid_node_id_field(&document, &reference),
            Some(reference.key("ordinal"))
        );
    }
}

#[cfg(test)]
mod lossless_budget_tests {
    use super::*;
    use crate::checked_package::v2::monitor_plan::model_document_test_fixture as fixture;

    /// The actual intake comparison borrows healthy paths, and pays its sole
    /// malformed-field diagnostic before requesting any owned pointer bytes.
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn lossless_intake_borrows_healthy_paths_and_prepays_first_fault() {
        let value = fixture::v2_all_families();
        let bytes = fixture::canonical(&value);
        let document = Arc::new(quire_canonical::read(&bytes, u64::MAX).unwrap());
        let mut decode_budget =
            NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let (mut wire, _bodies) = decode_borrowed_wire(&document, &mut decode_budget).unwrap();
        let run = |wire: &RawCheckedPackageWireV2, limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(check_lossless_source(&document, wire, &mut budget));
            });
            (
                result.unwrap(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (healthy, allocated, charged) = run(&wire, 0);
        healthy.expect("healthy original document agrees with its typed wire");
        assert_eq!((allocated, charged), (0, 0));
        wire.name = format!("{}-different", wire.name).into_boxed_str();
        let (failure, allocated, required) = run(&wire, u64::MAX);
        let ValidationFailure::Refused(refusal) = failure.unwrap_err() else {
            panic!("the first changed name is malformed wire");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(refusal.primary_path().unwrap().as_str(), "/name");
        assert_eq!(allocated, required);
        assert_eq!(
            required as usize,
            std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusal>()
                + std::mem::size_of::<crate::checked_package::shared::CheckedPackageRefusalLocation>(
                )
                + "/name".len()
        );
        let (exact, allocated, charged) = run(&wire, required);
        assert!(matches!(exact, Err(ValidationFailure::Refused(_))));
        assert_eq!((allocated, charged), (required, required));
        for limit in [0, required - 1] {
            let (short, allocated, charged) = run(&wire, limit);
            assert!(matches!(short, Err(ValidationFailure::Budget(_))));
            assert_eq!((allocated, charged), (0, 0));
        }
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn lossless_public_reader_preserves_dropped_null_first_fault() {
        use crate::checked_package::shared::CheckedPackageReadLimits;
        use crate::checked_package::v2::{CheckedPackageV2, CheckedPackageV2BudgetedReadResult};
        let mut value = fixture::v2_all_families();
        let evidence = fixture::evidence_for(&value);
        let nodes = value["semantic_graph"]["nodes"].as_array_mut().unwrap();
        let (index, node) = nodes
            .iter_mut()
            .enumerate()
            .find(|(_, node)| node.get("owner").is_none() && node.get("recursion_group").is_none())
            .expect("the complete fixture includes an unowned node");
        node["owner"] = serde_json::Value::Null;
        node["recursion_group"] = serde_json::Value::Null;
        let bytes = fixture::canonical(&value);
        let limits = CheckedPackageReadLimits {
            bytes: 1 << 30,
            nodes: 1 << 24,
            edges: 1 << 26,
            occurrences: 1 << 26,
            diagnostics: 1 << 24,
            work: u64::MAX,
        };
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let result = CheckedPackageV2::read_with_budget(&bytes, limits, &evidence, &mut budget);
        let CheckedPackageV2BudgetedReadResult::Refused(refusal) = result else {
            panic!("the original null owner is not silently dropped: {result:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refusal.primary_path().unwrap().as_str(),
            format!("/semantic_graph/nodes/{index}/owner")
        );
    }
}
